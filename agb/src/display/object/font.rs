use core::{cell::Cell, fmt::Write, num::NonZeroUsize};

use agb_fixnum::{Rect, Vector2D};
use alloc::{collections::VecDeque, vec::Vec};

use crate::display::{object::ObjectUnmanaged, Font};

use super::{DynamicSprite, OamIterator, PaletteVram, Size, SpriteVram};

struct LetterGroup {
    sprite: SpriteVram,
    // the width of the letter group
    width: u16,
    left: i16,
}

enum WhiteSpace {
    Space,
    NewLine,
}

enum TextElementReference<'text> {
    Word(Word<'text>),
    WhiteSpace(WhiteSpace),
}

struct Word<'letters> {
    letters: &'letters [LetterGroup],
    width: Cell<Option<NonZeroUsize>>,
}

impl<'letters> Word<'letters> {
    fn new(letters: &'letters [LetterGroup]) -> Self {
        Self {
            letters,
            width: Cell::new(None),
        }
    }
}

impl Word<'_> {
    fn width(&self) -> usize {
        match self.width.get() {
            Some(width) => width.get(),
            None => {
                let width = self.letters.iter().fold(0, |acc, letter| {
                    acc + (letter.width as i32 + letter.left as i32)
                });
                let width = width as usize;

                self.width.set(NonZeroUsize::new(width));
                width
            }
        }
    }
}

#[derive(Clone, Copy)]
struct WordLength(u8);

#[derive(Clone, Copy)]
enum Element {
    Word(u8),
    NewLine,
    Space,
}
const NEW_LINE: u8 = 0xFF;
const SPACE: u8 = 0xFE;
impl WordLength {
    fn parse(self) -> Element {
        if self.0 == NEW_LINE {
            Element::NewLine
        } else if self.0 == SPACE {
            Element::Space
        } else {
            Element::Word(self.0)
        }
    }

    fn from_element(e: Element) -> Self {
        WordLength(match e {
            Element::Word(len) => len,
            Element::NewLine => NEW_LINE,
            Element::Space => SPACE,
        })
    }
}

struct Letters(Vec<LetterGroup>);
struct Words {
    letters: Letters,
    word_lengths: Vec<WordLength>,
}

struct WordRenderCache {
    objects: Vec<ObjectUnmanaged>,
    state: WordRenderCacheState,
    poison_condition: WordRenderPoisonCondition,
}

struct WordRenderPoisonCondition {
    area: Rect<i32>,
}

struct WordRenderCacheState {
    depth_in_word_iterator: usize,
    depth_in_word: usize,
    depth_in_elements: usize,
    head_position: Vector2D<i32>,
}

impl WordRenderCache {
    fn new() -> Self {
        WordRenderCache {
            objects: Vec::new(),
            state: WordRenderCacheState {
                depth_in_word_iterator: 0,
                depth_in_word: 0,
                depth_in_elements: 0,
                head_position: (0, 0).into(),
            },
            poison_condition: WordRenderPoisonCondition {
                area: Rect::new((0, 0).into(), (0, 0).into()),
            },
        }
    }

    fn reset_state(&mut self, position: Rect<i32>) {
        self.state = WordRenderCacheState {
            depth_in_elements: 0,
            depth_in_word: 0,
            depth_in_word_iterator: 0,
            head_position: position.position,
        };

        self.poison_condition = WordRenderPoisonCondition { area: position };
    }

    fn generate_cache(&mut self, words: &Words, desired_element_count: usize) {
        let position = self.poison_condition.area;
        if self.state.depth_in_elements >= desired_element_count {
            return;
        }

        'outer: for elem in words.iter_words().skip(self.state.depth_in_word_iterator) {
            match elem {
                TextElementReference::Word(word) => {
                    let prospective_x = self.state.head_position.x + word.width() as i32;

                    if self.state.depth_in_word == 0
                        && prospective_x > position.position.x + position.size.x
                    {
                        self.state.head_position.x = position.position.x;
                        self.state.head_position.y += 15;
                    }

                    for letter in word.letters.iter().skip(self.state.depth_in_word) {
                        self.state.head_position.x += letter.left as i32;
                        let mut object = ObjectUnmanaged::new(letter.sprite.clone());
                        object.show();
                        object.set_position(self.state.head_position);

                        self.objects.push(object);

                        self.state.head_position.x += letter.width as i32;

                        self.state.depth_in_elements += 1;
                        self.state.depth_in_word += 1;
                        if self.state.depth_in_elements >= desired_element_count {
                            break 'outer;
                        }
                    }

                    self.state.depth_in_word = 0;
                }
                TextElementReference::WhiteSpace(space) => {
                    match space {
                        WhiteSpace::Space => self.state.head_position.x += 10,
                        WhiteSpace::NewLine => {
                            self.state.head_position.x = position.position.x;
                            self.state.head_position.y += 15;
                        }
                    }
                    self.state.depth_in_elements += 1;
                }
            }
            self.state.depth_in_word_iterator += 1;
            if self.state.depth_in_elements >= desired_element_count {
                break 'outer;
            }
        }
    }

    fn update(&mut self, words: &Words, desired_element_count: usize, position: Rect<i32>) {
        if self.poison_condition.area != position {
            self.reset_state(position);
        }

        self.generate_cache(words, desired_element_count);
    }

    fn commit(&self, oam: &mut OamIterator) {
        for (object, slot) in self.objects.iter().zip(oam) {
            slot.set(object);
        }
    }
}

impl Words {
    fn iter_words(&self) -> impl Iterator<Item = TextElementReference> {
        let mut letters_idx: usize = 0;

        self.word_lengths.iter().map(move |x| match x.parse() {
            Element::Word(length) => {
                let idx = letters_idx;
                let end_idx = idx + length as usize;
                letters_idx = end_idx;

                TextElementReference::Word(Word::new(&self.letters.0[idx..end_idx]))
            }
            Element::NewLine => TextElementReference::WhiteSpace(WhiteSpace::NewLine),
            Element::Space => TextElementReference::WhiteSpace(WhiteSpace::Space),
        })
    }
}

struct WorkingLetter {
    dynamic: DynamicSprite,
    // the x offset of the current letter with respect to the start of the current letter group
    x_position: i32,
    // where to render the letter from x_min to x_max
    x_offset: i32,
}

impl WorkingLetter {
    fn new(size: Size) -> Self {
        Self {
            dynamic: DynamicSprite::new(size),
            x_position: 0,
            x_offset: 0,
        }
    }

    fn reset(&mut self) {
        self.x_position = 0;
        self.x_offset = 0;
        self.dynamic.clear(0);
    }
}

pub struct Configuration {
    sprite_size: Size,
    palette: PaletteVram,
}

impl Configuration {
    #[must_use]
    pub fn new(sprite_size: Size, palette: PaletteVram) -> Self {
        Self {
            sprite_size,
            palette,
        }
    }
}

pub struct BufferedWordRender<'font> {
    word_render: WordRender<'font>,
    block: Words,
    current_word_length: usize,
    number_of_elements: usize,
    buffered_chars: VecDeque<char>,
    cache: WordRenderCache,
}

impl Write for BufferedWordRender<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for char in s.chars() {
            self.buffered_chars.push_back(char);
        }

        Ok(())
    }
}

impl<'font> BufferedWordRender<'font> {
    #[must_use]
    pub fn new(font: &'font Font, config: Configuration) -> Self {
        BufferedWordRender {
            word_render: WordRender::new(font, config),
            block: Words {
                letters: Letters(Vec::new()),
                word_lengths: Vec::new(),
            },
            current_word_length: 0,
            number_of_elements: 0,
            buffered_chars: VecDeque::new(),
            cache: WordRenderCache::new(),
        }
    }
}

impl BufferedWordRender<'_> {
    pub fn process(&mut self) {
        if let Some(char) = self.buffered_chars.pop_front() {
            if char == '\n' {
                if let Some(group) = self.word_render.finalise_letter() {
                    self.block.letters.0.push(group);
                    self.current_word_length += 1;
                }
                self.block
                    .word_lengths
                    .push(WordLength::from_element(Element::Word(
                        self.current_word_length as u8,
                    )));
                self.block
                    .word_lengths
                    .push(WordLength::from_element(Element::NewLine));
                self.number_of_elements += self.current_word_length + 1;
                self.current_word_length = 0;
            } else if char == ' ' {
                if let Some(group) = self.word_render.finalise_letter() {
                    self.block.letters.0.push(group);
                    self.current_word_length += 1;
                }
                self.block
                    .word_lengths
                    .push(WordLength::from_element(Element::Word(
                        self.current_word_length as u8,
                    )));
                self.block
                    .word_lengths
                    .push(WordLength::from_element(Element::Space));
                self.number_of_elements += self.current_word_length + 1;
                self.current_word_length = 0;
            } else if let Some(group) = self.word_render.render_char(char) {
                self.block.letters.0.push(group);
                self.current_word_length += 1;
            }
        }
    }

    pub fn update(&mut self, position: Rect<i32>, number_of_elements: usize) {
        while !self.buffered_chars.is_empty() && self.number_of_elements < number_of_elements {
            self.process();
        }

        self.cache.update(&self.block, number_of_elements, position);
    }

    pub fn commit(&self, oam: &mut OamIterator) {
        self.cache.commit(oam);
    }
}

struct WordRender<'font> {
    font: &'font Font,
    working: Working,
    config: Configuration,
}

struct Working {
    letter: WorkingLetter,
}

impl<'font> WordRender<'font> {
    #[must_use]
    fn new(font: &'font Font, config: Configuration) -> Self {
        WordRender {
            font,
            working: Working {
                letter: WorkingLetter::new(config.sprite_size),
            },
            config,
        }
    }
}

impl WordRender<'_> {
    #[must_use]
    fn finalise_letter(&mut self) -> Option<LetterGroup> {
        if self.working.letter.x_offset == 0 {
            return None;
        }

        let sprite = self
            .working
            .letter
            .dynamic
            .to_vram(self.config.palette.clone());
        let group = LetterGroup {
            sprite,
            width: self.working.letter.x_offset as u16,
            left: self.working.letter.x_position as i16,
        };
        self.working.letter.reset();

        Some(group)
    }

    #[must_use]
    fn render_char(&mut self, c: char) -> Option<LetterGroup> {
        let font_letter = self.font.letter(c);

        // uses more than the sprite can hold
        let group = if self.working.letter.x_offset + font_letter.width as i32
            > self.config.sprite_size.to_width_height().0 as i32
        {
            self.finalise_letter()
        } else {
            None
        };

        if self.working.letter.x_offset == 0 {
            self.working.letter.x_position = font_letter.xmin as i32;
        } else {
            self.working.letter.x_offset += font_letter.xmin as i32;
        }

        let y_position =
            self.font.ascent() - font_letter.height as i32 - font_letter.ymin as i32 + 4;

        for y in 0..font_letter.height as usize {
            for x in 0..font_letter.width as usize {
                let rendered = font_letter.bit_absolute(x, y);
                if rendered {
                    self.working.letter.dynamic.set_pixel(
                        x + self.working.letter.x_offset as usize,
                        (y_position + y as i32) as usize,
                        1,
                    );
                }
            }
        }

        self.working.letter.x_offset += font_letter.advance_width as i32;

        group
    }
}
