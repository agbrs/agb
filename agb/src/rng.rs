use portable_atomic::{AtomicU128, Ordering};

/// A fast pseudo-random number generator. Note that the output of the
/// random number generator for a given seed is guaranteed stable
/// between minor releases, however could change in a major release.
pub struct RandomNumberGenerator {
    state: [u32; 4],
}

impl RandomNumberGenerator {
    /// Create a new random number generator with a fixed seed
    ///
    /// Note that this seed is guaranteed to be the same between minor releases.
    #[must_use]
    pub const fn new() -> Self {
        Self::new_with_seed([1014776995, 476057059, 3301633994, 706340607])
    }

    /// Produces a random number generator with the given initial state / seed.
    /// None of the values can be 0.
    #[must_use]
    pub const fn new_with_seed(seed: [u32; 4]) -> Self {
        // this can't be in a loop because const
        assert!(seed[0] != 0, "seed must not be 0");
        assert!(seed[1] != 0, "seed must not be 0");
        assert!(seed[2] != 0, "seed must not be 0");
        assert!(seed[3] != 0, "seed must not be 0");

        Self { state: seed }
    }

    /// Returns the next value for the random number generator
    pub fn gen(&mut self) -> i32 {
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

impl Default for RandomNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL_RNG: AtomicU128 = AtomicU128::new(unsafe {
    core::mem::transmute::<[u32; 4], u128>(RandomNumberGenerator::new().state)
});

/// Using a global random number generator, provides the next random number
#[must_use]
pub fn gen() -> i32 {
    let data: u128 = GLOBAL_RNG.load(Ordering::SeqCst);
    let data_u32: [u32; 4] = unsafe { core::mem::transmute(data) };
    let mut rng = RandomNumberGenerator { state: data_u32 };
    let value = rng.gen();
    GLOBAL_RNG.store(
        unsafe { core::mem::transmute::<[u32; 4], u128>(rng.state) },
        Ordering::SeqCst,
    );
    value
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Gba;

    #[test_case]
    fn should_be_reasonably_distributed(_gba: &mut Gba) {
        let mut values: [u32; 16] = Default::default();

        let mut rng = RandomNumberGenerator::new();
        for _ in 0..500 {
            values[(rng.gen().rem_euclid(16)) as usize] += 1;
        }

        for (i, &value) in values.iter().enumerate() {
            assert!(
                value >= 500 / 10 / 3,
                "{i} came up less than expected {value}"
            );
        }
    }

    #[test_case]
    fn global_rng_should_be_reasonably_distributed(_gba: &mut Gba) {
        let mut values: [u32; 16] = Default::default();

        for _ in 0..500 {
            values[super::gen().rem_euclid(16) as usize] += 1;
        }

        for (i, &value) in values.iter().enumerate() {
            assert!(
                value >= 500 / 10 / 3,
                "{i} came up less than expected {value}"
            );
        }
    }
}
