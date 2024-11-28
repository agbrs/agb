use alignment::{
    AlignmentIterator, AlignmentIteratorCenter, AlignmentIteratorLeft, AlignmentIteratorRight,
};
use alloc::{boxed::Box, collections::vec_deque::VecDeque};
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

#[derive(Debug)]
struct LetterPosition {
    x: i32,
    line: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
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
            alignment_iterator: match alignment {
                Alignment::Left => AlignmentIterator::Left(AlignmentIteratorLeft::new()),
                Alignment::Center => AlignmentIterator::Center(AlignmentIteratorCenter::new()),
                Alignment::Right => AlignmentIterator::Right(AlignmentIteratorRight::new()),
            },
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

struct InnerMultiLineTextDisplay<T> {
    block: TextBlock<T>,
    peeked: Option<Option<LetterGroup>>,
    letters: VecDeque<LetterGroup>,
    max_number_of_lines: i32,
    current_line: i32,
}

pub struct MultiLineTextDisplay<T>(Box<InnerMultiLineTextDisplay<T>>);

impl<T> MultiLineTextDisplay<T>
where
    T: AsRef<str>,
{
    pub fn new(text: TextBlock<T>, max_number_of_lines: i32) -> Self {
        Self(Box::new(InnerMultiLineTextDisplay {
            block: text,
            peeked: None,
            letters: VecDeque::new(),
            max_number_of_lines,
            current_line: 0,
        }))
    }

    fn peek(&'_ mut self) -> Option<&LetterGroup> {
        self.0
            .peeked
            .get_or_insert_with(|| self.0.block.next())
            .as_ref()
    }

    fn next(&mut self) -> Option<LetterGroup> {
        match self.0.peeked.take() {
            Some(v) => v,
            None => self.0.block.next(),
        }
    }

    pub fn do_work(&mut self) {
        self.0.block.do_work(16);
    }

    pub fn is_done(&mut self) -> bool {
        self.peek().is_none()
    }

    pub fn is_showing_all_available_lines(&mut self) -> bool {
        let Some(next_letter) = self.peek() else {
            return false;
        };
        let line = next_letter.line;

        self.0.current_line + self.0.max_number_of_lines <= line
    }

    pub fn increase_letters(&mut self) {
        let max_line = self.0.current_line + self.0.max_number_of_lines;
        let Some(next_letter) = self.peek() else {
            return;
        };

        if max_line > next_letter.line {
            let next = self.next().unwrap();
            self.0.letters.push_back(next);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = LetterGroup> + use<'_, T> {
        self.0.letters.iter().map(|x| LetterGroup {
            letter: x.letter.clone(),
            x: x.x,
            line: x.line - self.0.current_line,
        })
    }

    pub fn pop_line(&mut self) {
        while let Some(letter) = self.0.letters.front() {
            if letter.line == self.0.current_line {
                self.0.letters.pop_front();
            } else {
                break;
            }
        }

        self.0.current_line += 1;
    }
}
