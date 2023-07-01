use core::fmt::Write;

use agb_fixnum::{Rect, Vector2D};
use alloc::{collections::VecDeque, vec::Vec};

use crate::display::{object::font::preprocess::Word, Font};

use self::{
    preprocess::{Line, Preprocessed, PreprocessedElement},
    renderer::{Configuration, WordRender},
};

use super::{OamIterator, ObjectUnmanaged, PaletteVram, Size, SpriteVram};

mod preprocess;
mod renderer;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub(crate) enum WhiteSpace {
    NewLine,
    Space,
}

impl WhiteSpace {
    pub(crate) fn from_char(c: char) -> Self {
        match c {
            ' ' => WhiteSpace::Space,
            '\n' => WhiteSpace::NewLine,
            _ => panic!("char not supported whitespace"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct LetterGroup {
    sprite: SpriteVram,
    // the width of the letter group
    width: u16,
    left: i16,
}

pub struct BufferedRender<'font> {
    char_render: WordRender,
    preprocessor: Preprocessed,
    buffered_chars: VecDeque<char>,
    letters: Letters,
    font: &'font Font,
}

#[derive(Debug, Default)]
struct Letters {
    letters: VecDeque<LetterGroup>,
    number_of_groups: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum TextAlignment {
    #[default]
    Left,
    Right,
    Center,
}

struct TextAlignmentSettings {
    space_width: i32,
    start_x: i32,
}

impl TextAlignment {
    fn settings(
        self,
        line: &Line,
        minimum_space_width: i32,
        size: Vector2D<i32>,
    ) -> TextAlignmentSettings {
        match self {
            TextAlignment::Left => TextAlignmentSettings {
                space_width: minimum_space_width,
                start_x: 0,
            },
            TextAlignment::Right => TextAlignmentSettings {
                space_width: minimum_space_width,
                start_x: size.x - line.width(),
            },
            TextAlignment::Center => TextAlignmentSettings {
                space_width: minimum_space_width,
                start_x: (size.x - line.width()) / 2,
            },
        }
    }
}

impl<'font> BufferedRender<'font> {
    #[must_use]
    fn new(font: &'font Font, sprite_size: Size, palette: PaletteVram) -> Self {
        let config = Configuration::new(sprite_size, palette);
        BufferedRender {
            char_render: WordRender::new(config),
            preprocessor: Preprocessed::new(),
            buffered_chars: VecDeque::new(),
            letters: Default::default(),
            font,
        }
    }
}

fn is_private_use(c: char) -> bool {
    ('\u{E000}'..'\u{F8FF}').contains(&c)
}

impl BufferedRender<'_> {
    fn input_character(&mut self, character: char) {
        if !is_private_use(character) {
            self.preprocessor
                .add_character(self.font, character, self.char_render.sprite_width());
        }
        self.buffered_chars.push_back(character);
    }

    fn process(&mut self) {
        let Some(c) = self.buffered_chars.pop_front() else { return };
        match c {
            ' ' | '\n' => {
                if let Some(group) = self.char_render.finalise_letter() {
                    self.letters.letters.push_back(group);
                    self.letters.number_of_groups += 1;
                }

                self.letters.number_of_groups += 1;
            }
            letter => {
                if let Some(group) = self.char_render.render_char(self.font, letter) {
                    self.letters.letters.push_back(group);
                    self.letters.number_of_groups += 1;
                }
            }
        }
    }
}

pub struct ObjectTextRender<'font> {
    buffer: BufferedRender<'font>,
    layout: LayoutCache,
    settings: LayoutSettings,
}

impl<'font> ObjectTextRender<'font> {
    #[must_use]
    pub fn new(font: &'font Font, sprite_size: Size, palette: PaletteVram) -> Self {
        Self {
            buffer: BufferedRender::new(font, sprite_size, palette),
            layout: LayoutCache::new(),
            settings: Default::default(),
        }
    }
}

impl Write for ObjectTextRender<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.buffer.input_character(c);
        }

        Ok(())
    }
}

impl ObjectTextRender<'_> {
    /// Remove a line from the render and shift everything up one line.
    /// A full complete line must be rendered for this to do anything, incomplete lines won't be popped. Returns whether a line could be popped.
    pub fn pop_line(&mut self) -> bool {
        let width = self.layout.settings.area.x;
        let space = self.buffer.font.letter(' ').advance_width as i32;
        let Some(line) = self.buffer.preprocessor.lines(width, space).next() else {
            return false;
        };

        let number_of_elements = line.number_of_letter_groups();
        if self.layout.state.line_depth >= 1 && self.layout.objects.len() >= number_of_elements {
            for _ in 0..number_of_elements {
                // self.buffer.letters.letters.pop_front();
                self.layout.objects.pop_front();
            }
            self.buffer.preprocessor.pop(&line);

            self.layout.state.head_offset.y -= self.buffer.font.line_height();
            for obj in self.layout.objects.iter_mut() {
                obj.offset.y -= self.buffer.font.line_height() as i16;
                let object_offset = obj.offset.change_base();
                obj.object
                    .set_position(self.layout.position + object_offset);
            }

            self.layout.state.line_depth -= 1;

            true
        } else {
            false
        }
    }

    /// On next update, the next unit of letters will be rendered. Returns whether the next element could be added.
    /// Can only be called once per layout.
    pub fn next_letter_group(&mut self) -> bool {
        self.layout.next_letter_group(&self.buffer)
    }
    /// Commits work already done to screen. You can commit to multiple places in the same frame.
    pub fn commit(&mut self, oam: &mut OamIterator, position: Vector2D<i32>) {
        self.layout.commit(oam, position);
    }
    /// Updates the internal state based on the chosen render settings. Best
    /// effort is made to reuse previous layouts, but a full rerender may be
    /// required if certain settings are changed.
    pub fn layout(&mut self) {
        self.layout.update(
            &mut self.buffer,
            self.settings.area,
            self.settings.alignment,
            self.settings.paragraph_spacing,
        );
    }
    /// Causes a change to the area that text is rendered. This will cause a relayout.
    pub fn set_size(&mut self, size: Vector2D<i32>) {
        self.settings.area = size;
    }
    /// Causes a change to the text alignment. This will cause a relayout.
    pub fn set_alignment(&mut self, alignment: TextAlignment) {
        self.settings.alignment = alignment;
    }
    /// Sets the paragraph spacing. This will cause a relayout.
    pub fn set_paragraph_spacing(&mut self, paragraph_spacing: i32) {
        self.settings.paragraph_spacing = paragraph_spacing;
    }
}

struct LayoutObject {
    object: ObjectUnmanaged,
    offset: Vector2D<i16>,
}

struct LayoutCache {
    objects: VecDeque<LayoutObject>,
    state: LayoutCacheState,
    settings: LayoutSettings,
    desired_number_of_groups: usize,
    position: Vector2D<i32>,
}

impl LayoutCache {
    fn next_letter_group(&mut self, buffer: &BufferedRender) -> bool {
        let width = self.settings.area.x;
        let space = buffer.font.letter(' ').advance_width as i32;
        let line_height = buffer.font.line_height();

        if self.state.head_offset.y + line_height > self.settings.area.y {
            return false;
        }

        if let Some((_line, mut line_elements)) = buffer
            .preprocessor
            .lines_element(width, space)
            .nth(self.state.line_depth)
        {
            match line_elements.nth(self.state.line_element_depth) {
                Some(PreprocessedElement::Word(_)) => {
                    self.desired_number_of_groups += 1;
                }
                Some(PreprocessedElement::WhiteSpace(WhiteSpace::Space)) => {
                    self.desired_number_of_groups += 1;
                }
                Some(PreprocessedElement::WhiteSpace(WhiteSpace::NewLine)) => {
                    self.desired_number_of_groups += 1;
                }
                None => {
                    if self.state.head_offset.y + line_height * 2 > self.settings.area.y {
                        return false;
                    }
                    self.desired_number_of_groups += 1;
                }
            }
        }

        true
    }

    fn update_cache(&mut self, render: &BufferedRender) {
        if self.state.rendered_groups >= self.desired_number_of_groups {
            return;
        }

        let minimum_space_width = render.font.letter(' ').advance_width as i32;

        let lines = render
            .preprocessor
            .lines_element(self.settings.area.x, minimum_space_width);

        'outer: for (line, line_elements) in lines.skip(self.state.line_depth) {
            let settings =
                self.settings
                    .alignment
                    .settings(&line, minimum_space_width, self.settings.area);

            if self.state.line_element_depth == 0 {
                self.state.head_offset.x += settings.start_x;
            }

            for element in line_elements.skip(self.state.line_element_depth) {
                match element {
                    PreprocessedElement::Word(word) => {
                        for letter in (word.sprite_index()
                            ..(word.sprite_index() + word.number_of_sprites()))
                            .skip(self.state.word_depth)
                            .map(|x| &render.letters.letters[x])
                        {
                            let mut object = ObjectUnmanaged::new(letter.sprite.clone());
                            self.state.head_offset.x += letter.left as i32;
                            object
                                .set_position(self.state.head_offset + self.position)
                                .show();

                            let layout_object = LayoutObject {
                                object,
                                offset: (
                                    self.state.head_offset.x as i16,
                                    self.state.head_offset.y as i16,
                                )
                                    .into(),
                            };
                            self.state.head_offset.x += letter.width as i32;
                            self.objects.push_back(layout_object);
                            self.state.rendered_groups += 1;
                            self.state.word_depth += 1;
                            if self.state.rendered_groups >= self.desired_number_of_groups {
                                break 'outer;
                            }
                        }

                        self.state.word_depth = 0;
                        self.state.line_element_depth += 1;
                    }
                    PreprocessedElement::WhiteSpace(space_type) => {
                        if space_type == WhiteSpace::NewLine {
                            self.state.head_offset.y += self.settings.paragraph_spacing;
                        }
                        self.state.head_offset.x += settings.space_width;
                        self.state.rendered_groups += 1;
                        self.state.line_element_depth += 1;
                        if self.state.rendered_groups >= self.desired_number_of_groups {
                            break 'outer;
                        }
                    }
                }
            }

            self.state.head_offset.y += render.font.line_height();
            self.state.head_offset.x = 0;

            self.state.line_element_depth = 0;
            self.state.line_depth += 1;
        }
    }

    fn update(
        &mut self,
        r: &mut BufferedRender<'_>,
        area: Vector2D<i32>,
        alignment: TextAlignment,
        paragraph_spacing: i32,
    ) {
        r.process();

        while !r.buffered_chars.is_empty()
            && r.letters.number_of_groups <= self.desired_number_of_groups
        {
            r.process();
        }

        let settings = LayoutSettings {
            area,
            alignment,
            paragraph_spacing,
        };
        if settings != self.settings {
            self.reset(settings);
        }

        self.update_cache(r);
    }

    fn commit(&mut self, oam: &mut OamIterator, position: Vector2D<i32>) {
        if self.position != position {
            for (object, slot) in self.objects.iter_mut().zip(oam) {
                let object_offset = object.offset.change_base();
                object.object.set_position(position + object_offset);
                slot.set(&object.object);
            }
            self.position = position;
        } else {
            for (object, slot) in self.objects.iter().zip(oam) {
                slot.set(&object.object);
            }
        }
    }

    #[must_use]
    fn new() -> Self {
        Self {
            objects: VecDeque::new(),
            state: Default::default(),
            settings: LayoutSettings {
                area: (0, 0).into(),
                alignment: TextAlignment::Right,
                paragraph_spacing: -100,
            },
            desired_number_of_groups: 0,
            position: (0, 0).into(),
        }
    }

    fn reset(&mut self, settings: LayoutSettings) {
        self.objects.clear();
        self.state = LayoutCacheState {
            head_offset: (0, 0).into(),
            word_depth: 0,
            rendered_groups: 0,
            line_depth: 0,
            line_element_depth: 0,
        };
        self.settings = settings;
    }
}

#[derive(PartialEq, Eq, Default)]
struct LayoutSettings {
    area: Vector2D<i32>,
    alignment: TextAlignment,
    paragraph_spacing: i32,
}

#[derive(Default)]
struct LayoutCacheState {
    head_offset: Vector2D<i32>,
    word_depth: usize,
    rendered_groups: usize,
    line_depth: usize,
    line_element_depth: usize,
}
