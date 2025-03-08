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
    line_length: i32,
    word_length: i32,
    space_count: i32,
    word_start_index: usize,
    previous_char: Option<char>,
    kind: AlignmentKind,
    max_line_length: i32,
}

#[derive(Debug)]
pub struct Line {
    pub left: i32,
    pub finish_index: usize,
    pub space_width: i32,
}

impl Align {
    pub fn new(alignment: AlignmentKind, max_line_length: i32) -> Self {
        Self {
            processed: 0,
            word_start_index: 0,
            line_length: 0,
            word_length: 0,
            space_count: 0,
            kind: alignment,
            previous_char: None,
            max_line_length,
        }
    }

    pub fn next(&mut self, text: &str, font: &Font) -> Option<Line> {
        if self.processed >= text.len() - 1 {
            return None;
        }

        let space_width = font.letter(' ').advance_width as i32;

        if self.kind == AlignmentKind::None {
            self.processed = text.len() - 1;
            return Some(Line {
                left: 0,
                finish_index: self.processed,
                space_width,
            });
        }

        let start_idx = self.processed;

        for (idx, c) in text[self.processed..]
            .char_indices()
            .map(|(idx, c)| (idx + start_idx, c))
        {
            self.processed = idx;

            let letter = font.letter(c);

            if c == ' ' {
                self.line_length += self.word_length + letter.advance_width as i32;
                self.word_length = 0;
                self.space_count += 1;
            }

            let kern_amount = if let Some(previous_char) = self.previous_char {
                letter.kerning_amount(previous_char)
            } else {
                0
            };

            self.previous_char = Some(c);

            if c != ' ' || c != '\n' {
                self.word_length += letter.advance_width as i32 + kern_amount;
            }

            if c == ' ' || c == '\n' {
                self.word_start_index = idx;
            }

            if self.line_length + self.word_length >= self.max_line_length || c == '\n' {
                if self.line_length == 0 {
                    // force break word
                    self.line_length = self.word_length;
                    self.word_length = 0;
                    self.previous_char = None;
                    self.word_start_index = idx;
                }

                let space_count = self.space_count;
                let line_length = self.line_length;
                self.line_length = 0;
                self.space_count = 0;

                return match self.kind {
                    AlignmentKind::Left => Some(Line {
                        left: 0,
                        finish_index: self.word_start_index,
                        space_width,
                    }),
                    AlignmentKind::Right => Some(Line {
                        left: self.max_line_length - line_length,
                        finish_index: self.word_start_index,
                        space_width,
                    }),
                    AlignmentKind::Justify => Some(Line {
                        left: 0,
                        finish_index: self.word_start_index,
                        space_width: (self.max_line_length - line_length)
                            .checked_div(space_count)
                            .unwrap_or_default(),
                    }),
                    AlignmentKind::None => unreachable!("handled elsewhere"),
                };
            }
        }

        let line_length = self.line_length;
        self.line_length = 0;
        self.space_count = 0;
        let idx = text.len();
        self.processed = idx;

        match self.kind {
            AlignmentKind::Left => Some(Line {
                left: 0,
                finish_index: idx,
                space_width,
            }),
            AlignmentKind::Right => Some(Line {
                left: self.max_line_length - line_length,
                finish_index: idx,
                space_width,
            }),
            AlignmentKind::Justify => Some(Line {
                left: 0,
                finish_index: idx,
                space_width,
            }),
            AlignmentKind::None => unreachable!("handled elsewhere"),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::display::Font;

    use super::*;

    static FONT: Font = include_font!("fnt/ark-pixel-10px-proportional-latin.ttf", 10);

    #[test_case]
    fn align_benchmark_short(_: &mut crate::Gba) {
        let mut align = Align::new(AlignmentKind::Left, 200);
        let text = "Hello, world!";
        while let Some(line) = align.next(text, &FONT) {
            core::hint::black_box(line);
        }
    }

    #[test_case]
    fn align_benchmark_long(_: &mut crate::Gba) {
        let mut align = Align::new(AlignmentKind::Left, 200);
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

        let mut align = Align::new(AlignmentKind::Left, 200);
        while let Some(line) = align.next(text, &FONT) {
            core::hint::black_box(line);
        }
    }
}
