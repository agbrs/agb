use core::cell::RefCell;

use bare_metal::Mutex;

use crate::interrupt::free;

pub struct RandomNumberGenerator {
    state: [u32; 4],
}

impl RandomNumberGenerator {
    pub const fn new() -> Self {
        Self {
            state: [1014776995, 476057059, 3301633994, 706340607],
        }
    }

    pub fn next(&mut self) -> i32 {
        let result = (self.state[0].wrapping_add(self.state[3]))
            .rotate_left(7)
            .wrapping_mul(9);
        let t = self.state[1].wrapping_shr(9);

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(11);

        result as i32
    }
}

static GLOBAL_RNG: Mutex<RefCell<RandomNumberGenerator>> =
    Mutex::new(RefCell::new(RandomNumberGenerator::new()));

pub fn next() -> i32 {
    free(|cs| GLOBAL_RNG.borrow(*cs).borrow_mut().next())
}
