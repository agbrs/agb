#![no_std]
#![no_main]

use agb::{
    display::{
        object::{Alignment, LetterGroup, ObjectUnmanaged, PaletteVram, Size, TextBlock},
        palette16::Palette16,
        Font, HEIGHT, WIDTH,
    },
    include_font,
    input::Button,
};
use alloc::collections::vec_deque::VecDeque;

extern crate alloc;

use core::iter::Peekable;

static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}


struct MultiLineTextDisplay {
    block: Peekable<TextBlock<&'static str>>,
    letters: VecDeque<LetterGroup>,
    max_number_of_lines: i32,
    current_line: i32,
}

impl MultiLineTextDisplay {
    fn new(text: TextBlock<&'static str>, max_number_of_lines: i32) -> Self {
        Self {
            block: text.peekable(),
            letters: VecDeque::new(),
            max_number_of_lines,
            current_line: 0,
        }
    }

    fn is_done(&mut self) -> bool {
        self.block.peek().is_none()
    }

    fn is_showing_all_available_lines(&mut self) -> bool {
        let Some(next_letter) = self.block.peek() else {
            return false;
        };
        
        self.current_line + self.max_number_of_lines <= next_letter.line
    }

    fn increase_letters(&mut self) {
        let Some(next_letter) = self.block.peek() else {
            
            return;
        };

        if self.current_line + self.max_number_of_lines > next_letter.line {
            self.letters.push_back(self.block.next().unwrap());
        }
    }

    fn iter(&self) -> impl Iterator<Item = LetterGroup> + use<'_> {
        self.letters.iter().map(|x| LetterGroup{ letter: x.letter.clone(), x: x.x, line: x.line - self.current_line})
    }

    fn pop_line(&mut self) {
        while let Some(letter) = self.letters.front() {
            if letter.line == self.current_line {
                self.letters.pop_front();
            } else {
                break;
            }
        }

        self.current_line += 1;
    }
}

fn main(mut gba: agb::Gba) -> ! {
    let (mut unmanaged, _sprites) = gba.display.object.get_unmanaged();

    let mut palette = [0x0; 16];
    palette[1] = 0xFF_FF;
    palette[2] = 0x00_FF;
    let palette = Palette16::new(palette);
    let palette = PaletteVram::new(&palette).unwrap();

    let timer = gba.timers.timers();
    let mut timer: agb::timer::Timer = timer.timer2;

    timer.set_enabled(true);
    timer.set_divider(agb::timer::Divider::Divider256);

    let start = timer.value();
    let wr = TextBlock::new(&FONT, 
        "Woah!{change2} {player_name}! {change1}こんにちは! I have a bunch of text I want to show you. However, you will find that the amount of text I can display is limited. Who'd have thought! Good thing that my text system supports scrolling! It only took around 20 jank versions to get here!",
        palette,
        Alignment::Left,
        (WIDTH - 8) as u32,
        Size::S16x16,
    );

    let end = timer.value();

    agb::println!(
        "Write took {} cycles",
        256 * (end.wrapping_sub(start) as u32)
    );

    let vblank = agb::interrupt::VBlank::get();
    let mut input = agb::input::ButtonController::new();

    let mut frame = 0;

    let mut multi_line = MultiLineTextDisplay::new(wr, 2);

    loop {
        input.update();

        let start = timer.value();

        if frame % 2 == 0 {
            multi_line.increase_letters();
        }

        if multi_line.is_showing_all_available_lines() && input.is_just_pressed(Button::A) {
            multi_line.pop_line();
        }

        let end = timer.value();
        agb::println!(
            "Update took {} cycles",
            256 * (end.wrapping_sub(start) as u32)
        );
        let start = timer.value();


        let mut frame_oam = unmanaged.iter();

        for letter in multi_line.iter() {
            let mut object = ObjectUnmanaged::new(letter.letter);
            object.set_position((4 + letter.x, HEIGHT - 32 + letter.line * 16).into()).show();
            frame_oam.set_next(&object);
        }

        let end = timer.value();
        agb::println!(
            "Draw took {} cycles",
            256 * (end.wrapping_sub(start) as u32)
        );

        vblank.wait_for_vblank();
        frame += 1;
    }
}
