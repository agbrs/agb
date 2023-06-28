use core::fmt::Write;

use agb_fixnum::{Rect, Vector2D};
use alloc::{collections::VecDeque, vec::Vec};

use crate::display::Font;

use self::{
    preprocess::{Line, Preprocessed, PreprocessedElement},
    renderer::{Configuration, WordRender},
};

use super::{DynamicSprite, OamIterator, ObjectUnmanaged, PaletteVram, Size, SpriteVram};

mod preprocess;
mod renderer;

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug)]
struct Word {
    index: usize,
    length: usize,
}

#[derive(Debug, Default)]
struct Letters {
    letters: Vec<LetterGroup>,
    number_of_groups: usize,
}

impl Write for BufferedRender<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.input_character(c);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextAlignment {
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
    pub fn new(font: &'font Font, sprite_size: Size, palette: PaletteVram) -> Self {
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

impl BufferedRender<'_> {
    fn input_character(&mut self, character: char) {
        self.preprocessor
            .add_character(self.font, character, self.char_render.sprite_width());
        self.buffered_chars.push_back(character);
    }

    pub fn process(&mut self) {
        let Some(c) = self.buffered_chars.pop_front() else { return };
        match c {
            ' ' | '\n' => {
                if let Some(group) = self.char_render.finalise_letter() {
                    self.letters.letters.push(group);
                    self.letters.number_of_groups += 1;
                }

                self.letters.number_of_groups += 1;
            }
            letter => {
                if let Some(group) = self.char_render.render_char(self.font, letter) {
                    self.letters.letters.push(group);
                    self.letters.number_of_groups += 1;
                }
            }
        }
    }
}

pub struct LayoutCache {
    objects: Vec<ObjectUnmanaged>,
    state: LayoutCacheState,
    settings: LayoutSettings,
}

impl LayoutCache {
    fn update_cache(&mut self, number_of_groups: usize, render: &BufferedRender) {
        let minimum_space_width = render.font.letter(' ').advance_width as i32;

        let lines = render
            .preprocessor
            .lines_element(self.settings.area.size.x, minimum_space_width);

        'outer: for (line, line_elements) in lines.skip(self.state.line_depth) {
            let settings = self.settings.alignment.settings(
                &line,
                minimum_space_width,
                self.settings.area.size,
            );

            if self.state.line_element_depth == 0 {
                self.state.head_position.x += settings.start_x;
            }

            for element in line_elements.iter().skip(self.state.line_element_depth) {
                match element {
                    PreprocessedElement::Word(word) => {
                        for letter in (word.sprite_index()
                            ..(word.sprite_index() + word.number_of_sprites()))
                            .skip(self.state.word_depth)
                            .map(|x| &render.letters.letters[x])
                        {
                            let mut object = ObjectUnmanaged::new(letter.sprite.clone());
                            self.state.head_position.x += letter.left as i32;
                            object.set_position(self.state.head_position);
                            self.state.head_position.x += letter.width as i32;
                            object.show();
                            self.objects.push(object);
                            self.state.rendered_groups += 1;
                            self.state.word_depth += 1;
                            if self.state.rendered_groups >= number_of_groups {
                                break 'outer;
                            }
                        }

                        self.state.word_depth = 0;
                        self.state.line_element_depth += 1;
                    }
                    PreprocessedElement::WhiteSpace(space_type) => {
                        if *space_type == WhiteSpace::NewLine {
                            self.state.head_position.y += self.settings.paragraph_spacing;
                        }
                        self.state.head_position.x += settings.space_width;
                        self.state.rendered_groups += 1;
                        self.state.line_element_depth += 1;
                        if self.state.rendered_groups >= number_of_groups {
                            break 'outer;
                        }
                    }
                }
            }

            self.state.head_position.y += render.font.line_height();
            self.state.head_position.x = self.settings.area.position.x;

            self.state.line_element_depth = 0;
            self.state.line_depth += 1;
        }
    }

    pub fn update(
        &mut self,
        r: &mut BufferedRender<'_>,
        area: Rect<i32>,
        alignment: TextAlignment,
        paragraph_spacing: i32,
        number_of_groups: usize,
    ) {
        while !r.buffered_chars.is_empty() && r.letters.number_of_groups <= number_of_groups {
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

        self.update_cache(number_of_groups, r);
    }

    pub fn commit(&self, oam: &mut OamIterator) {
        for (object, slot) in self.objects.iter().zip(oam) {
            slot.set(object);
        }
    }

    #[must_use]
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            state: Default::default(),
            settings: LayoutSettings {
                area: Rect::new((0, 0).into(), (0, 0).into()),
                alignment: TextAlignment::Right,
                paragraph_spacing: -100,
            },
        }
    }

    fn reset(&mut self, settings: LayoutSettings) {
        self.objects.clear();
        self.state = LayoutCacheState {
            head_position: settings.area.position,
            processed_depth: 0,
            group_depth: 0,
            word_depth: 0,
            rendered_groups: 0,
            line_depth: 0,
            line_element_depth: 0,
        };
        self.settings = settings;
    }
}

#[derive(PartialEq, Eq)]
struct LayoutSettings {
    area: Rect<i32>,
    alignment: TextAlignment,
    paragraph_spacing: i32,
}

#[derive(Default)]
struct LayoutCacheState {
    head_position: Vector2D<i32>,
    processed_depth: usize,
    group_depth: usize,
    word_depth: usize,
    rendered_groups: usize,
    line_depth: usize,
    line_element_depth: usize,
}
