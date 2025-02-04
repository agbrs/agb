use core::fmt::{Display, Write};

use agb_fixnum::{Num, Vector2D};
use alloc::{collections::VecDeque, vec::Vec};

use crate::display::Font;

use self::{
    preprocess::{Line, Preprocessed, PreprocessedElement},
    renderer::{Configuration, WordRender},
};

use super::{sprites::PaletteVramSingle, OamFrame, Object, Size, SpriteVram};

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

struct BufferedRender<'font> {
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
/// The text alignment of the layout
pub enum TextAlignment {
    #[default]
    /// Left aligned, the left edge of the text lines up
    Left,
    /// Right aligned, the right edge of the text lines up
    Right,
    /// Center aligned, the center of the text lines up
    Center,
    /// Justified, both the left and right edges line up with space width adapted to make it so.
    Justify,
}

struct TextAlignmentSettings {
    space_width: Num<i32, 10>,
    start_x: i32,
}

impl TextAlignment {
    fn settings(self, line: &Line, minimum_space_width: i32, width: i32) -> TextAlignmentSettings {
        match self {
            TextAlignment::Left => TextAlignmentSettings {
                space_width: minimum_space_width.into(),
                start_x: 0,
            },
            TextAlignment::Right => TextAlignmentSettings {
                space_width: minimum_space_width.into(),
                start_x: width - line.width(),
            },
            TextAlignment::Center => TextAlignmentSettings {
                space_width: minimum_space_width.into(),
                start_x: (width - line.width()) / 2,
            },
            TextAlignment::Justify => {
                let space_width = if line.number_of_spaces() != 0 {
                    Num::new(
                        width - line.width() + line.number_of_spaces() as i32 * minimum_space_width,
                    ) / line.number_of_spaces() as i32
                } else {
                    minimum_space_width.into()
                };
                TextAlignmentSettings {
                    space_width,
                    start_x: 0,
                }
            }
        }
    }
}

impl<'font> BufferedRender<'font> {
    #[must_use]
    fn new(font: &'font Font, sprite_size: Size, palette: PaletteVramSingle) -> Self {
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
/// Changes the palette to use to draw characters.
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// use agb::display::object::{ObjectTextRender, PaletteVramSingle, ChangeColour, Size};
/// use agb::display::palette16::Palette16;
/// use agb::display::Font;
///
/// use core::fmt::Write;
///
/// static EXAMPLE_FONT: Font = agb::include_font!("examples/font/yoster.ttf", 12);
///
/// # fn foo() {
/// let mut palette = [0x0; 16];
/// palette[1] = 0xFF_FF;
/// palette[2] = 0x00_FF;
/// let palette = Palette16::new(palette);
/// let palette = PaletteVramSingle::new(&palette).unwrap();
/// let mut writer = ObjectTextRender::new(&EXAMPLE_FONT, Size::S16x16, palette);
///
/// let _ = writeln!(writer, "Hello, {}World{}!", ChangeColour::new(2), ChangeColour::new(1));
/// # }
/// ```
pub struct ChangeColour(u8);

impl ChangeColour {
    #[must_use]
    /// Creates the colour changer. Colour is a palette index and must be in the range 0..16.
    pub fn new(colour: usize) -> Self {
        assert!(colour < 16, "paletted colour must be valid (0..=15)");

        Self(colour as u8)
    }

    fn try_from_char(c: char) -> Option<Self> {
        let c = c as u32 as usize;
        if (0xE000..0xE000 + 16).contains(&c) {
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
        let Some(c) = self.buffered_chars.pop_front() else {
            return;
        };
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

/// The object text renderer. Uses objects to render and layout text. It's use is non trivial.
/// Changes the palette to use to draw characters.
/// ```rust,no_run
/// #![no_std]
/// #![no_main]
/// use agb::display::object::{ObjectTextRender, PaletteVramSingle, TextAlignment, Size};
/// use agb::display::palette16::Palette16;
/// use agb::display::{Font, WIDTH};
///
/// use core::fmt::Write;
///
/// static EXAMPLE_FONT: Font = agb::include_font!("examples/font/yoster.ttf", 12);
///
/// #[agb::entry]
/// fn main(gba: &mut agb::Gba) -> ! {
///     let mut oam = gba.display.object.get();
///     let vblank = agb::interrupt::VBlank::get();
///
///     let mut palette = [0x0; 16];
///     palette[1] = 0xFF_FF;
///     let palette = Palette16::new(palette);
///     let palette = PaletteVramSingle::new(&palette).unwrap();
///
///     let mut writer = ObjectTextRender::new(&EXAMPLE_FONT, Size::S16x16, palette);
///
///     let _ = writeln!(writer, "Hello, World!");
///     writer.layout((WIDTH, 40), TextAlignment::Left, 2);
///
///     loop {
///         writer.next_letter_group();
///         writer.update((0, 0));
///         let mut frame = oam.frame();
///         writer.commit(&mut frame);
///         vblank.wait_for_vblank();
///         frame.commit();
///     }
/// }
/// ```
pub struct ObjectTextRender<'font> {
    buffer: BufferedRender<'font>,
    layout: LayoutCache,
    number_of_objects: usize,
}

impl<'font> ObjectTextRender<'font> {
    #[must_use]
    /// Creates a new text renderer with a given font, sprite size, and palette.
    /// You must ensure that the sprite size can accomodate the letters from the
    /// font otherwise it will panic at render time.
    pub fn new(font: &'font Font, sprite_size: Size, palette: PaletteVramSingle) -> Self {
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
    pub fn commit(&mut self, oam: &mut OamFrame) {
        for object in self.layout.objects.iter() {
            object.show(oam);
        }
    }

    /// Force a relayout, must be called after writing.
    pub fn layout(
        &mut self,
        area: impl Into<Vector2D<i32>>,
        alignment: TextAlignment,
        paragraph_spacing: i32,
    ) {
        self.layout.create_positions(
            self.buffer.font,
            &self.buffer.preprocessor,
            &LayoutSettings {
                area: area.into(),
                alignment,
                paragraph_spacing,
            },
        );
    }

    /// Removes one complete line. Returns whether a line could be removed. You must call [`update`][ObjectTextRender::update] after this
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

    /// Updates the internal state of the number of letters to write and popped
    /// line. Should be called in the same frame as and after
    /// [`next_letter_group`][ObjectTextRender::next_letter_group], [`next_line`][ObjectTextRender::next_line], and [`pop_line`][ObjectTextRender::pop_line].
    pub fn update(&mut self, position: impl Into<Vector2D<i32>>) {
        if !self.buffer.buffered_chars.is_empty()
            && self.buffer.letters.letters.len() <= self.number_of_objects + 5
        {
            self.buffer.process();
        }

        self.layout.update_objects_to_display_at_position(
            position.into(),
            self.buffer.letters.letters.iter(),
            self.number_of_objects,
        );
    }

    /// Causes the next letter group to be shown on the next update. Returns
    /// whether another letter could be added in the space given.
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

    /// Causes the next line to be shown on the next update. Returns
    /// whether another line could be added in the space given.
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
    objects: Vec<Object>,
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
                    let mut object = Object::new(letter.clone());
                    object.set_position(position);
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

        let mut head_position: Vector2D<Num<i32, 10>> = (0, -line_height).into();

        preprocessed
            .lines_element(width, minimum_space_width)
            .map(move |(line, line_elements)| {
                let line_settings = settings
                    .alignment
                    .settings(&line, minimum_space_width, width);

                head_position.y += line_height;
                head_position.x = line_settings.start_x.into();

                (
                    line,
                    line_elements.filter_map(move |element| match element.decode() {
                        PreprocessedElement::LetterGroup { width } => {
                            let this_position = head_position;
                            head_position.x += width as i32;
                            Some(this_position.floor())
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
