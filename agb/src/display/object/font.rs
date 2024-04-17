use core::{fmt::Display, num::NonZeroU32};

use alloc::{borrow::Cow, collections::VecDeque, vec::Vec};

use crate::display::Font;

use self::renderer::Configuration;

use super::{PaletteVram, Size, SpriteVram};

mod renderer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChangeColour(u8);

impl Display for ChangeColour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;
        f.write_char(self.to_char())
    }
}

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

fn is_private_use(c: char) -> bool {
    ('\u{E000}'..'\u{F8FF}').contains(&c)
}

struct RenderConfig<'string> {
    string: Cow<'string, str>,
    font: &'static Font,
}

struct RenderedSpriteInternal {
    start: usize,
    end: usize,
    width: i32,
    sprite: SpriteVram,
}

struct RenderedSprite<'text_render> {
    string: &'text_render str,
    width: i32,
    sprite: &'text_render SpriteVram,
}

impl RenderedSprite<'_> {
    fn text(&self) -> &str {
        self.string
    }

    fn width(&self) -> i32 {
        self.width
    }

    fn sprite(&self) -> &SpriteVram {
        &self.sprite
    }
}

pub struct SimpleTextRender<'string> {
    config: RenderConfig<'string>,
    render_index: usize,
    inner_renderer: renderer::WordRender,
    rendered_sprite_window: VecDeque<RenderedSpriteInternal>,
    word_lengths: VecDeque<WordLength>,
}

#[derive(Debug, Copy, Clone, Default)]
struct WordLength {
    letter_groups: usize,
    pixels: i32,
}

struct SimpleLayoutItem<'text_render> {
    string: &'text_render str,
    sprite: &'text_render SpriteVram,
    x: i32,
}

impl<'text_render> SimpleLayoutItem<'text_render> {
    fn displayed_string(&self) -> &str {
        &self.string
    }

    fn sprite(&self) -> &SpriteVram {
        &self.sprite
    }

    fn x_offset(&self) -> i32 {
        self.x
    }
}

struct SimpleLayoutIterator<'text_render> {
    string: &'text_render str,
    vec_iter: alloc::collections::vec_deque::Iter<'text_render, RenderedSpriteInternal>,
    word_lengths_iter: alloc::collections::vec_deque::Iter<'text_render, WordLength>,
    space_width: i32,
    current_word_length: usize,
    x_offset: i32,
}

impl<'text_render> Iterator for SimpleLayoutIterator<'text_render> {
    type Item = SimpleLayoutItem<'text_render>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_word_length == 0 {
            self.x_offset += self.space_width;
            self.current_word_length = self.word_lengths_iter.next()?.letter_groups;
        }

        let rendered = self.vec_iter.next()?;
        let my_x_offset = self.x_offset;
        self.x_offset += rendered.width;

        self.current_word_length -= 1;

        Some(SimpleLayoutItem {
            string: &self.string[rendered.start..rendered.end],
            sprite: &rendered.sprite,
            x: my_x_offset,
        })
    }
}

impl<'string> SimpleTextRender<'string> {
    /// Lays out text in one line with a space between each word, note that
    /// newlines are just treated as word breaks.
    ///
    /// If you want to treat layout fully use one of the layouts
    /// [`LeftAlignLayout`], [`RightAlignLayout`], [`CenterAlignLayout`], or
    /// [`JustifyAlignLayout`].
    pub fn simple_layout(&self) -> SimpleLayoutIterator<'_> {
        SimpleLayoutIterator {
            string: &self.config.string,
            word_lengths_iter: self.word_lengths.iter(),
            vec_iter: self.rendered_sprite_window.iter(),
            space_width: self.config.font.letter(' ').advance_width as i32,
            current_word_length: 0,
            x_offset: 0,
        }
    }

    fn words(&self) -> impl Iterator<Item = (Option<i32>, impl Iterator<Item = RenderedSprite>)> {
        let mut start = 0;
        self.word_lengths
            .iter()
            .copied()
            .enumerate()
            .map(move |(idx, length)| {
                let potentially_incomplete = self.word_lengths.len() == idx + 1;
                let definitely_complete = !potentially_incomplete;

                let end = start + length.letter_groups;
                let this_start = start;
                start = end;

                (
                    definitely_complete.then_some(length.pixels),
                    self.rendered_sprite_window
                        .range(this_start..end)
                        .map(|x| RenderedSprite {
                            string: &self.config.string[x.start..x.end],
                            width: x.width,
                            sprite: &x.sprite,
                        }),
                )
            })
    }

    fn next_character(&mut self) -> Option<(usize, char)> {
        let next = self
            .config
            .string
            .get(self.render_index..)?
            .chars()
            .next()?;
        let idx = self.render_index;

        self.render_index += next.len_utf8();
        Some((idx, next))
    }

    pub fn is_done(&self) -> bool {
        self.string().len() == self.render_index
    }

    pub fn number_of_letter_groups(&self) -> usize {
        self.rendered_sprite_window.len()
    }

    pub fn pop_words(&mut self, words: usize) -> usize {
        assert!(self.word_lengths.len() > words);

        let mut total_letters_to_pop = 0;
        for _ in 0..words {
            let number_of_letters_to_pop = self.word_lengths.pop_front().unwrap();
            total_letters_to_pop += number_of_letters_to_pop.letter_groups;
        }

        for _ in 0..total_letters_to_pop {
            self.rendered_sprite_window.pop_front();
        }

        total_letters_to_pop
    }

    pub fn update(&mut self) {
        let Some((idx, c)) = self.next_character() else {
            return;
        };
        match c {
            ' ' | '\n' => {
                let length = self
                    .word_lengths
                    .back_mut()
                    .expect("There should always be at least one word length");
                if let Some((start_index, group, width)) = self.inner_renderer.finalise_letter(idx)
                {
                    self.rendered_sprite_window
                        .push_back(RenderedSpriteInternal {
                            start: start_index,
                            end: idx,
                            sprite: group,
                            width,
                        });

                    length.letter_groups += 1;
                    length.pixels += width;
                }

                self.word_lengths.push_back(WordLength::default());
            }
            letter => {
                if let Some((start_index, group, width)) =
                    self.inner_renderer
                        .render_char(self.config.font, letter, idx)
                {
                    self.rendered_sprite_window
                        .push_back(RenderedSpriteInternal {
                            start: start_index,
                            end: idx,
                            sprite: group,
                            width,
                        });
                    let length = self
                        .word_lengths
                        .back_mut()
                        .expect("There should always be at least one word length");
                    length.letter_groups += 1;
                    length.pixels += width;
                }
            }
        }
    }

    pub fn new(
        string: Cow<'string, str>,
        font: &'static Font,
        palette: PaletteVram,
        sprite_size: Size,
        explicit_break_on: Option<fn(char) -> bool>,
    ) -> Self {
        let mut word_lengths = VecDeque::new();
        word_lengths.push_back(WordLength::default());
        Self {
            config: RenderConfig { string, font },
            rendered_sprite_window: VecDeque::new(),
            word_lengths,
            render_index: 0,
            inner_renderer: renderer::WordRender::new(
                Configuration::new(sprite_size, palette),
                explicit_break_on,
            ),
        }
    }

    fn string(&self) -> &str {
        &self.config.string
    }
}

pub struct LeftAlignLayout<'string> {
    simple: SimpleTextRender<'string>,
    data: LeftAlignLayoutData,
}

struct LeftAlignLayoutData {
    width: Option<NonZeroU32>,
    string_index: usize,
    words_per_line: VecDeque<usize>,
    current_line_width: i32,
}

struct PreparedLetterGroupPosition {
    x: i32,
    line: i32,
}

fn length_of_next_word(current_index: &mut usize, s: &str, font: &Font) -> Option<(bool, i32)> {
    let s = &s[*current_index..];
    if s.is_empty() {
        return None;
    }

    let mut width = 0;
    let mut previous_character = None;
    for (idx, chr) in s.char_indices() {
        match chr {
            '\n' | ' ' => {
                *current_index += idx + 1;
                return Some((chr == '\n', width));
            }
            _ if is_private_use(chr) => {}
            letter => {
                let letter = font.letter(letter);
                if let Some(previous_character) = previous_character {
                    width += letter.kerning_amount(previous_character);
                }

                // width += letter.xmin as i32;
                width += letter.advance_width as i32;
            }
        }
        previous_character = Some(chr);
    }
    *current_index += s.len();
    Some((false, width))
}

pub struct LaidOutLetter<'text_render> {
    line: usize,
    x: i32,
    sprite: &'text_render SpriteVram,
    string: &'text_render str,
}

impl LaidOutLetter<'_> {
    pub fn line(&self) -> usize {
        self.line
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn sprite(&self) -> &SpriteVram {
        self.sprite
    }

    pub fn string(&self) -> &str {
        self.string
    }
}

impl<'string> LeftAlignLayout<'string> {
    pub fn new(simple: SimpleTextRender<'string>, width: Option<NonZeroU32>) -> Self {
        let mut words_per_line = VecDeque::new();
        words_per_line.push_back(0);

        Self {
            simple,
            data: LeftAlignLayoutData {
                string_index: 0,
                words_per_line,
                current_line_width: 0,
                width,
            },
        }
    }

    pub fn pop_line(&mut self) -> usize {
        assert!(self.data.words_per_line.len() > 1, "line not complete");
        let words = self.data.words_per_line.pop_front().unwrap();
        self.simple.pop_words(words)
    }

    pub fn at_least_n_letter_groups(&mut self, desired: usize) {
        while self.simple.number_of_letter_groups() < desired && !self.simple.is_done() {
            self.simple.update();
        }
    }

    pub fn layout(&mut self) -> impl Iterator<Item = LaidOutLetter> {
        self.data.layout(
            self.simple.string(),
            self.simple.config.font,
            self.simple.words(),
        )
    }
}

impl LeftAlignLayoutData {
    fn length_of_next_word(&mut self, string: &str, font: &Font) -> Option<(bool, i32)> {
        length_of_next_word(&mut self.string_index, string, font)
    }

    fn try_extend_line(&mut self, string: &str, font: &Font, space_width: i32) -> bool {
        let (force_new_line, length_of_next_word) = self
            .length_of_next_word(string, font)
            .expect("Should have more in the line to extend into");

        if self.current_line_width + length_of_next_word
            > self.width.map_or(i32::MAX, |x| x.get() as i32)
        {
            self.current_line_width = length_of_next_word + space_width;
            self.words_per_line.push_back(1);
            true
        } else {
            let current_line = self
                .words_per_line
                .back_mut()
                .expect("should always have a line");
            self.current_line_width += length_of_next_word + space_width;

            *current_line += 1;
            if force_new_line {
                self.current_line_width = 0;
                self.words_per_line.push_back(0);
            }
            false
        }
    }

    fn layout<'a, 'text_render>(
        &'a mut self,
        string: &'a str,
        font: &'static Font,
        simple: impl Iterator<
                Item = (
                    Option<i32>,
                    impl Iterator<Item = RenderedSprite<'text_render>> + 'a,
                ),
            > + 'a,
    ) -> impl Iterator<Item = LaidOutLetter<'text_render>> + 'a {
        let mut words_in_current_line = 0;
        let mut current_line = 0;
        let mut current_line_x_offset = 0;
        let space_width = font.letter(' ').advance_width as i32;

        simple.flat_map(move |(pixels, letters)| {
            let this_line_is_the_last_processed = current_line + 1 == self.words_per_line.len();
            words_in_current_line += 1;

            if words_in_current_line > self.words_per_line[current_line]
                && (!this_line_is_the_last_processed
                    || self.try_extend_line(string, font, space_width))
            {
                current_line += 1;
                current_line_x_offset = 0;
                words_in_current_line = 1;
            }

            let current_line = current_line;
            let mut letter_x_offset = current_line_x_offset;
            current_line_x_offset += pixels.unwrap_or(0);
            current_line_x_offset += space_width;

            letters.map(move |x| {
                let my_offset = letter_x_offset;
                letter_x_offset += x.width;
                LaidOutLetter {
                    line: current_line,
                    x: my_offset,
                    sprite: x.sprite,
                    string: x.string,
                }
            })
        })
    }
}

struct RightAlignLayout {}
struct CenterAlignLayout {}
struct JustifyAlignLayout {}
