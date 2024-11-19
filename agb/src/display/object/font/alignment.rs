use alloc::{collections::vec_deque::VecDeque, vec::Vec};

use super::{
    char_iterator::KerningCharIterator, configuration::NullCharConfigurator, LetterPosition,
    TextConfig,
};

/// What we want to get out of it is a set of LetterPosition s
pub enum AlignmentIterator {
    Left(AlignmentIteratorLeft),
}

impl AlignmentIterator {
    pub fn next(&mut self, text: &str, config: &TextConfig) -> Option<LetterPosition> {
        match self {
            AlignmentIterator::Left(alignment_iterator_left) => {
                alignment_iterator_left.next(text, config)
            }
        }
    }

    pub fn do_work(&mut self, text: &str, config: &TextConfig, max_buffered_work: usize) {
        match self {
            AlignmentIterator::Left(alignment_iterator_left) => {
                alignment_iterator_left.do_work(text, config, max_buffered_work);
            }
        }
    }
}

pub struct AlignmentIteratorLeft {
    // stores the letters in the current word
    word_queue: Vec<i32>,

    // stores the width of the line WITHOUT the current word
    current_line_width: i32,
    current_word_width: i32,

    current_number_of_letters_in_current_letter_group: i32,
    current_letter_width: i32,

    current_line_count: i32,

    iterator: KerningCharIterator,

    output_queue: VecDeque<LetterPosition>,
}

impl AlignmentIteratorLeft {
    pub fn new() -> Self {
        Self {
            word_queue: Vec::new(),
            iterator: KerningCharIterator::new(),
            current_line_width: 0,
            current_word_width: 0,
            current_letter_width: 0,
            current_number_of_letters_in_current_letter_group: 0,
            current_line_count: 0,
            output_queue: VecDeque::new(),
        }
    }

    fn flush_letter(&mut self) {
        if self.current_number_of_letters_in_current_letter_group == 0 {
            return;
        }

        self.current_word_width += self.current_letter_width;
        self.word_queue.push(self.current_word_width);
        self.current_letter_width = 0;
        self.current_number_of_letters_in_current_letter_group = 0;
    }

    fn force_new_line(&mut self) {
        self.current_line_count += 1;
        self.current_line_width = 0;
    }

    fn complete_word(&mut self, config: &TextConfig, extra_width: i32) {
        self.flush_letter();

        if self.current_line_width + self.current_word_width > config.line_width as i32 {
            // this word will not fit, increase the line count
            self.current_line_count += 1;
            self.current_line_width = 0;
        }
        if !self.word_queue.is_empty() {
            self.word_queue.pop();
            self.output_queue
                .extend(
                    core::iter::once(0)
                        .chain(self.word_queue.drain(..))
                        .map(|letter_width| LetterPosition {
                            x: self.current_line_width + letter_width,
                            line: self.current_line_count,
                        }),
                );
        }
        self.current_line_width += self.current_word_width + extra_width;
        self.current_word_width = 0;
    }

    fn do_work_with_work_done(&mut self, text: &str, config: &TextConfig) -> bool {
        let Some((character, letter, kern)) =
            self.iterator
                .next(text, config.font, &mut NullCharConfigurator)
        else {
            self.complete_word(config, 0);
            return false;
        };

        // only support ascii whitespace for word breaks
        if character.is_ascii_whitespace() {
            // stop the current word
            self.complete_word(config, letter.advance_width as i32);
            if character == '\n' {
                self.force_new_line();
            }
        } else {
            // continue the current word
            if self.current_number_of_letters_in_current_letter_group == 0 {
                self.current_word_width += kern + letter.xmin as i32;
            } else {
                self.current_letter_width += kern + letter.xmin as i32;
            }

            if self.current_letter_width + letter.width as i32
                > config.sprite_size.to_width_height().0 as i32
            {
                self.flush_letter();
            }

            self.current_number_of_letters_in_current_letter_group += 1;

            self.current_letter_width += letter.advance_width as i32;
        }

        true
    }

    pub fn do_work(&mut self, text: &str, config: &TextConfig, max_buffered_work: usize) {
        if self.output_queue.len() < max_buffered_work {
            self.do_work_with_work_done(text, config);
        }
    }

    pub fn next(&mut self, text: &str, config: &TextConfig) -> Option<LetterPosition> {
        while self.output_queue.is_empty() {
            if !self.do_work_with_work_done(text, config) {
                break;
            }
        }

        self.output_queue.pop_front()
    }
}
