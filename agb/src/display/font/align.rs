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
    kind: AlignmentKind,
}

pub struct Line {
    pub left: i32,
    pub finish_index: usize,
    pub space_width: i32,
}

impl Align {
    pub fn new(alignment: AlignmentKind) -> Self {
        Self {
            processed: 0,
            line_length: 0,
            word_length: 0,
            space_count: 0,
            kind: alignment,
        }
    }

    pub fn next(&mut self, text: &str, font: &Font, max_line_length: i32) -> Option<Line> {
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

            if c != ' ' || c != '\n' {
                self.word_length += letter.advance_width as i32;
            }

            if self.line_length + self.word_length >= max_line_length || c == '\n' {
                let space_count = self.space_count;
                let line_length = self.line_length;
                self.line_length = 0;
                self.space_count = 0;

                return match self.kind {
                    AlignmentKind::Left => Some(Line {
                        left: 0,
                        finish_index: idx,
                        space_width,
                    }),
                    AlignmentKind::Right => Some(Line {
                        left: max_line_length - line_length,
                        finish_index: idx,
                        space_width,
                    }),
                    AlignmentKind::Justify => Some(Line {
                        left: 0,
                        finish_index: idx,
                        space_width: (max_line_length - line_length) / space_count,
                    }),
                    AlignmentKind::None => unreachable!("handled elsewhere"),
                };
            }
        }

        let line_length = self.line_length;
        self.line_length = 0;
        self.space_count = 0;
        let idx = self.processed;

        match self.kind {
            AlignmentKind::Left => Some(Line {
                left: 0,
                finish_index: idx,
                space_width,
            }),
            AlignmentKind::Right => Some(Line {
                left: max_line_length - line_length,
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
