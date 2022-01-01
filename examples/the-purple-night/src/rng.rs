struct RandomNumberGenerator {
    state: [u32; 4],
}

impl RandomNumberGenerator {
    const fn new() -> Self {
        Self {
            state: [1014776995, 476057059, 3301633994, 706340607],
        }
    }

    fn next(&mut self) -> i32 {
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

static mut RANDOM_GENERATOR: RandomNumberGenerator = RandomNumberGenerator::new();

pub fn get_random() -> i32 {
    unsafe { &mut RANDOM_GENERATOR }.next()
}
