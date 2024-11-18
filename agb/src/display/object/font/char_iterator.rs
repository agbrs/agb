use crate::display::FontLetter;

use crate::display::Font;

use super::configuration::CharConfigurator;

pub(crate) struct KerningCharIterator {
    iterator: CharIterator,
    previous_letter: Option<char>,
}

impl KerningCharIterator {
    pub(crate) fn new() -> Self {
        Self {
            iterator: CharIterator::new(),
            previous_letter: None,
        }
    }

    pub(crate) fn next<T: CharConfigurator>(
        &mut self,
        text: &str,
        font: &Font,
        configuration: &mut T,
    ) -> Option<(&'static FontLetter, i32)> {
        let letter_char = self.iterator.next(text, configuration)?;

        let letter = font.letter(letter_char);
        let kern = if let Some(previous) = self.previous_letter {
            letter.kerning_amount(previous)
        } else {
            0
        };
        self.previous_letter = Some(letter_char);

        Some((letter, kern))
    }
}

/// You provide the same string and this iterates over the characters in a non
/// horrific way (ie. not O(n^2)).
struct CharIterator {
    index: usize,
}

impl CharIterator {
    fn next<T: CharConfigurator>(&mut self, text: &str, _configuration: &mut T) -> Option<char> {
        let letter = text[self.index..].chars().next()?;
        self.index += letter.len_utf8();

        Some(letter)
    }

    fn new() -> Self {
        Self { index: 0 }
    }
}
