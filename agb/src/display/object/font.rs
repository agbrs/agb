use alignment::{AlignmentIterator, AlignmentIteratorLeft};
use alloc::borrow::Cow;
use render::LetterRender;

use crate::display::Font;

use super::{PaletteVram, Size, SpriteVram};

mod alignment;
mod char_iterator;
mod configuration;
mod render;

pub struct TextBlock<T> {
    text: T,
    render: LetterRender,
    alignment_iterator: AlignmentIterator,
    config: TextConfig,
}

struct TextConfig {
    font: &'static Font,
    line_width: u32,
    sprite_size: Size,
    palette: PaletteVram,
}

struct Letter {
    sprite: SpriteVram,
}

struct LetterPosition {
    x: i32,
    line: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
    /// None aligned text does not consider alignment at all, it appears left
    /// aligned but will never line break. This could be useful for UI elements
    /// that you know are on a single line.
    None,
}

impl<T> TextBlock<T>
where
    T: AsRef<str>,
{
    #[must_use]
    pub fn new(
        font: &'static Font,
        text: T,
        palette: PaletteVram,
        alignment: Alignment,
        line_width: u32,
        sprite_size: Size,
    ) -> Self {
        let config = TextConfig {
            font,
            line_width,
            sprite_size,
            palette,
        };
        Self {
            text,
            render: LetterRender::new(&config),
            config,
            alignment_iterator: AlignmentIterator::Left(AlignmentIteratorLeft::new()),
        }
    }

    pub fn do_work(&mut self, max_buffered_work: usize) {
        self.alignment_iterator
            .do_work(self.text.as_ref(), &self.config, max_buffered_work);
        self.render
            .do_work(self.text.as_ref(), &self.config, max_buffered_work);
    }
}

pub struct LetterGroup {
    pub letter: SpriteVram,
    pub x: i32,
    pub line: i32,
}

impl<T> Iterator for TextBlock<T>
where
    T: AsRef<str>,
{
    type Item = LetterGroup;

    fn next(&mut self) -> Option<Self::Item> {
        match (
            self.alignment_iterator
                .next(self.text.as_ref(), &self.config),
            self.render.next(self.text.as_ref(), &self.config),
        ) {
            (Some(position), Some(letter)) => Some(LetterGroup {
                letter: letter.sprite,
                x: position.x,
                line: position.line,
            }),
            (None, None) => None,
            (None, Some(_)) => panic!("Alignment finished but renderer did not"),
            (Some(_), None) => panic!("Renderer finished but alignment did not"),
        }
    }
}
