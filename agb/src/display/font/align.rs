use super::Font;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AlignmentKind {
    Left,
    Right,
    Justify,
    None,
}

pub struct Align {
    processed: usize,
    kind: AlignmentKind,
    max_line_length: i32,

    default_space_width: i32,
}

#[derive(Debug)]
pub struct Line {
    pub left: i32,
    pub finish_index: usize,
    pub start_index: usize,
    pub space_width: i32,
}

impl Align {
    pub fn new(alignment: AlignmentKind, max_line_length: i32, font: &Font) -> Self {
        let default_space_width = font.letter(' ').advance_width as i32;

        Self {
            processed: 0,
            kind: alignment,
            max_line_length,
            default_space_width,
        }
    }

    pub fn next(&mut self, text: &str, font: &Font) -> Option<Line> {
        if self.processed + 1 >= text.len() {
            return None;
        }

        if self.kind == AlignmentKind::None {
            self.processed = text.len() - 1;
            return Some(Line {
                left: 0,
                start_index: 0,
                finish_index: self.processed,
                space_width: self.default_space_width,
            });
        }

        // skip leading spaces
        let Some(start_index) = text[self.processed..]
            .char_indices()
            .find_map(|(idx, char)| {
                if char != ' ' {
                    Some(idx + self.processed)
                } else {
                    None
                }
            })
        else {
            // Only spaces remain, so nothing to do
            self.processed = text.len() - 1;
            return None;
        };

        let mut previous_char: Option<char> = None;
        let mut current_width_of_words_in_line = 0;
        let mut current_word_start_index = start_index;
        let mut current_width_of_word = 0;
        let mut spaces_in_line = 0;

        for (char_index, c) in text[start_index..].char_indices() {
            let char_index = char_index + start_index;

            let letter = font.letter(c);

            // new line, so finish this line here
            if c == '\n' {
                self.processed = char_index + c.len_utf8();

                let left = if matches!(self.kind, AlignmentKind::Right) {
                    current_width_of_words_in_line + spaces_in_line * self.default_space_width
                } else {
                    0
                };

                return Some(Line {
                    left,
                    start_index,
                    finish_index: self.processed,
                    space_width: self.default_space_width,
                });
            } else if c == ' ' {
                spaces_in_line += 1;

                current_width_of_words_in_line += current_width_of_word;
                current_width_of_word = 0;
                current_word_start_index = char_index + ' '.len_utf8();

                previous_char = None;
            } else {
                let kerning =
                    previous_char.map_or(0, |previous_char| letter.kerning_amount(previous_char));
                current_width_of_word += i32::from(letter.advance_width) + kerning;
            }

            if self.max_line_length
                < current_width_of_words_in_line
                    + current_width_of_word
                    + spaces_in_line * self.default_space_width
            {
                // We've done a complete line now, and should break before the start of the current word. However, if
                // the current word is the first word we started laying out on this line, then we should break anyway in
                // the middle of that word
                let line_width = if spaces_in_line == 0 {
                    self.processed = char_index;
                    current_width_of_word
                } else {
                    self.processed = current_word_start_index;
                    current_width_of_words_in_line + (spaces_in_line - 1) * self.default_space_width
                };

                return Some(match self.kind {
                    AlignmentKind::Left => Line {
                        left: 0,
                        start_index,
                        finish_index: self.processed,
                        space_width: self.default_space_width,
                    },
                    AlignmentKind::Right => Line {
                        left: self.max_line_length - line_width,
                        start_index,
                        finish_index: self.processed,
                        space_width: self.default_space_width,
                    },
                    AlignmentKind::Justify => Line {
                        left: 0,
                        start_index,
                        finish_index: self.processed,
                        space_width: (self.max_line_length - current_width_of_words_in_line)
                            .checked_div(spaces_in_line)
                            .unwrap_or(0),
                    },
                    AlignmentKind::None => unreachable!("Handled above"),
                });
            }
        }

        self.processed = text.len();
        let left = if matches!(self.kind, AlignmentKind::Right) {
            self.max_line_length
                - (current_width_of_words_in_line
                    + current_width_of_word
                    + spaces_in_line * self.default_space_width)
        } else {
            0
        };

        Some(Line {
            left,
            start_index,
            finish_index: self.processed,
            space_width: self.default_space_width,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::display::font::Font;

    use super::*;

    static FONT: Font = include_font!("fnt/ark-pixel-10px-proportional-latin.ttf", 10);

    #[test_case]
    fn align_benchmark_short(_: &mut crate::Gba) {
        let mut align = Align::new(AlignmentKind::Left, 200, &FONT);
        let text = "Hello, world!";
        while let Some(line) = align.next(text, &FONT) {
            core::hint::black_box(line);
        }
    }

    #[test_case]
    fn align_benchmark_long(_: &mut crate::Gba) {
        let mut align = Align::new(AlignmentKind::Left, 200, &FONT);
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        if let Some(line) = align.next(text, &FONT) {
            core::hint::black_box(line);
        }
    }

    #[test_case]
    fn benchmark_text_format(_: &mut crate::Gba) {
        let x = alloc::format!(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, {} sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, {} quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. {} Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, {} sunt in culpa qui officia deserunt mollit anim id est laborum.",
            core::hint::black_box(128),
            "abc",
            "233",
            3540
        );
        core::hint::black_box(&x);
    }

    #[test_case]
    fn benchmark_japanese_text(_: &mut crate::Gba) {
        let text = "現代社会において、情報技術の進化は目覚ましい。それは、私たちの生活様式だけでなく、思考様式にも大きな影響を与えている。例えば、スマートフォンやタブレット端末の普及により、いつでもどこでも情報にアクセスできるようになった。これにより、知識の共有やコミュニケーションが容易になり、新しい文化や価値観が生まれている。しかし、一方で、情報過多やプライバシーの問題など、新たな課題も浮上している。私たちは、これらの課題にどのように向き合い、情報技術をどのように活用していくべきだろうか。それは、私たち一人ひとりが真剣に考えるべき重要なテーマである。";

        let mut align = Align::new(AlignmentKind::Left, 200, &FONT);
        while let Some(line) = align.next(text, &FONT) {
            core::hint::black_box(line);
        }
    }
}
