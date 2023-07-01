use core::fmt::{Display, Write};

use agb_fixnum::Vector2D;
use alloc::{collections::VecDeque, vec::Vec};

use crate::display::Font;

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

pub struct BufferedRender<'font> {
    char_render: WordRender,
    preprocessor: Preprocessed,
    buffered_chars: VecDeque<char>,
    letters: Letters,
    font: &'font Font,
}

#[derive(Debug, Default)]
struct Letters {
    letters: VecDeque<SpriteVram>,
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
    fn settings(self, line: &Line, minimum_space_width: i32, width: i32) -> TextAlignmentSettings {
        match self {
            TextAlignment::Left => TextAlignmentSettings {
                space_width: minimum_space_width,
                start_x: 0,
            },
            TextAlignment::Right => TextAlignmentSettings {
                space_width: minimum_space_width,
                start_x: width - line.width(),
            },
            TextAlignment::Center => TextAlignmentSettings {
                space_width: minimum_space_width,
                start_x: (width - line.width()) / 2,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChangeColour(u8);

impl ChangeColour {
    #[must_use]
    pub fn new(colour: u32) -> Self {
        assert!(colour < 16, "paletted colour must be valid (0..=15)");

        Self(colour as u8)
    }

    fn try_from_char(c: char) -> Option<Self> {
        let c = c as u32;
        if c >= 0xE000 && c < 0xE000 + 16 {
            Some(ChangeColour::new(c - 0xE000))
        } else {
            None
        }
    }

    fn to_char(self) -> char {
        char::from_u32(self.0 as u32 + 0xE000).unwrap()
    }
}

impl Display for ChangeColour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_char(self.to_char())
    }
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
    number_of_objects: usize,
}

impl<'font> ObjectTextRender<'font> {
    #[must_use]
    pub fn new(font: &'font Font, sprite_size: Size, palette: PaletteVram) -> Self {
        Self {
            buffer: BufferedRender::new(font, sprite_size, palette),
            number_of_objects: 0,
            layout: LayoutCache {
                positions: VecDeque::new(),
                line_capacity: VecDeque::new(),
                objects: Vec::new(),
                objects_are_at_origin: (0, 0).into(),
                area: (0, 0).into(),
            },
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
    /// Commits work already done to screen. You can commit to multiple places in the same frame.
    pub fn commit(&mut self, oam: &mut OamIterator) {
        for (object, slot) in self.layout.objects.iter().zip(oam) {
            slot.set(object);
        }
    }

    /// Force a relayout, must be called after writing.
    pub fn layout(
        &mut self,
        area: Vector2D<i32>,
        alignment: TextAlignment,
        paragraph_spacing: i32,
    ) {
        self.layout.create_positions(
            self.buffer.font,
            &self.buffer.preprocessor,
            &LayoutSettings {
                area,
                alignment,
                paragraph_spacing,
            },
        );
    }

    /// Removes one complete line.
    pub fn pop_line(&mut self) -> bool {
        let width = self.layout.area.x;
        let space = self.buffer.font.letter(' ').advance_width as i32;
        let line_height = self.buffer.font.line_height();
        if let Some(line) = self.buffer.preprocessor.lines(width, space).next() {
            // there is a line
            if self.layout.objects.len() >= line.number_of_letter_groups() {
                // we have enough rendered letter groups to count
                self.number_of_objects -= line.number_of_letter_groups();
                for _ in 0..line.number_of_letter_groups() {
                    self.buffer.letters.letters.pop_front();
                    self.layout.positions.pop_front();
                }
                self.layout.line_capacity.pop_front();
                self.layout.objects.clear();
                self.buffer.preprocessor.pop(&line);
                for position in self.layout.positions.iter_mut() {
                    position.y -= line_height as i16;
                }
                return true;
            }
        }
        false
    }

    pub fn update(&mut self, position: Vector2D<i32>) {
        if !self.buffer.buffered_chars.is_empty()
            && self.buffer.letters.letters.len() <= self.number_of_objects + 5
        {
            self.buffer.process();
        }

        self.layout.update_objects_to_display_at_position(
            position,
            self.buffer.letters.letters.iter(),
            self.number_of_objects,
        );
    }

    pub fn next_letter_group(&mut self) -> bool {
        if !self.can_render_another_element() {
            return false;
        }
        self.number_of_objects += 1;
        self.at_least_n_letter_groups(self.number_of_objects);

        true
    }

    fn can_render_another_element(&self) -> bool {
        let max_number_of_lines = (self.layout.area.y / self.buffer.font.line_height()) as usize;

        let max_number_of_objects = self
            .layout
            .line_capacity
            .iter()
            .take(max_number_of_lines)
            .sum::<usize>();

        max_number_of_objects > self.number_of_objects
    }

    pub fn next_line(&mut self) -> bool {
        let max_number_of_lines = (self.layout.area.y / self.buffer.font.line_height()) as usize;

        // find current line

        for (start, end) in self
            .layout
            .line_capacity
            .iter()
            .scan(0, |count, line_size| {
                let start = *count;
                *count += line_size;
                Some((start, *count))
            })
            .take(max_number_of_lines)
        {
            if self.number_of_objects >= start && self.number_of_objects < end {
                self.number_of_objects = end;
                self.at_least_n_letter_groups(end);
                return true;
            }
        }

        false
    }

    fn at_least_n_letter_groups(&mut self, n: usize) {
        while !self.buffer.buffered_chars.is_empty() && self.buffer.letters.letters.len() <= n {
            self.buffer.process();
        }
    }
}

struct LayoutCache {
    positions: VecDeque<Vector2D<i16>>,
    line_capacity: VecDeque<usize>,
    objects: Vec<ObjectUnmanaged>,
    objects_are_at_origin: Vector2D<i32>,
    area: Vector2D<i32>,
}

impl LayoutCache {
    fn update_objects_to_display_at_position<'a>(
        &mut self,
        position: Vector2D<i32>,
        letters: impl Iterator<Item = &'a SpriteVram>,
        number_of_objects: usize,
    ) {
        let already_done = if position == self.objects_are_at_origin {
            self.objects.len()
        } else {
            self.objects.clear();
            0
        };
        self.objects.extend(
            self.positions
                .iter()
                .zip(letters)
                .take(number_of_objects)
                .skip(already_done)
                .map(|(offset, letter)| {
                    let position = offset.change_base() + position;
                    let mut object = ObjectUnmanaged::new(letter.clone());
                    object.show().set_position(position);
                    object
                }),
        );
        self.objects.truncate(number_of_objects);
        self.objects_are_at_origin = position;
    }

    fn create_positions(
        &mut self,
        font: &Font,
        preprocessed: &Preprocessed,
        settings: &LayoutSettings,
    ) {
        self.area = settings.area;
        self.line_capacity.clear();
        self.positions.clear();
        for (line, line_positions) in Self::create_layout(font, preprocessed, settings) {
            self.line_capacity.push_back(line.number_of_letter_groups());
            self.positions
                .extend(line_positions.map(|x| Vector2D::new(x.x as i16, x.y as i16)));
        }
    }

    fn create_layout<'a>(
        font: &Font,
        preprocessed: &'a Preprocessed,
        settings: &'a LayoutSettings,
    ) -> impl Iterator<Item = (Line, impl Iterator<Item = Vector2D<i32>> + 'a)> + 'a {
        let minimum_space_width = font.letter(' ').advance_width as i32;
        let width = settings.area.x;
        let line_height = font.line_height();

        let mut head_position: Vector2D<i32> = (0, -line_height).into();

        preprocessed
            .lines_element(width, minimum_space_width)
            .map(move |(line, line_elements)| {
                let line_settings = settings
                    .alignment
                    .settings(&line, minimum_space_width, width);

                head_position.y += line_height;
                head_position.x = line_settings.start_x;

                (
                    line,
                    line_elements.filter_map(move |element| match element.decode() {
                        PreprocessedElement::LetterGroup { width } => {
                            let this_position = head_position;
                            head_position.x += width as i32;
                            Some(this_position)
                        }
                        PreprocessedElement::WhiteSpace(space) => {
                            match space {
                                WhiteSpace::NewLine => {
                                    head_position.y += settings.paragraph_spacing;
                                }
                                WhiteSpace::Space => head_position.x += line_settings.space_width,
                            }
                            None
                        }
                    }),
                )
            })
    }
}

#[derive(PartialEq, Eq, Default)]
struct LayoutSettings {
    area: Vector2D<i32>,
    alignment: TextAlignment,
    paragraph_spacing: i32,
}
