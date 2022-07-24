use agb::interrupt::free;
use bare_metal::Mutex;
use core::cell::RefCell;

const RAM_ADDRESS: *mut u8 = 0x0E00_0000 as *mut u8;
const HIGH_SCORE_ADDRESS_START: *mut u8 = RAM_ADDRESS.wrapping_offset(1);

static HIGHSCORE: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

pub fn init_save() {
    if (unsafe { RAM_ADDRESS.read_volatile() } == !0) {
        save_high_score(0);
        unsafe { RAM_ADDRESS.write_volatile(0) };
    }

    let mut a = [0; 4];
    for (idx, a) in a.iter_mut().enumerate() {
        *a = unsafe { HIGH_SCORE_ADDRESS_START.add(idx).read_volatile() };
    }

    let high_score = u32::from_le_bytes(a);

    free(|cs| {
        if high_score > 100 {
            HIGHSCORE.borrow(cs).replace(0);
        } else {
            HIGHSCORE.borrow(cs).replace(high_score);
        }
    });
}

pub fn load_high_score() -> u32 {
    free(|cs| *HIGHSCORE.borrow(cs).borrow())
}

pub fn save_high_score(score: u32) {
    let a = score.to_le_bytes();

    for (idx, &a) in a.iter().enumerate() {
        unsafe { HIGH_SCORE_ADDRESS_START.add(idx).write_volatile(a) };
    }

    free(|cs| HIGHSCORE.borrow(cs).replace(score));
}
