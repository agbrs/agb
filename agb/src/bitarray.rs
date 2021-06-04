pub struct Bitarray<const N: usize> {
    a: [u32; N],
}

impl<const N: usize> Bitarray<N> {
    pub fn new() -> Self {
        Bitarray { a: [0; N] }
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        if index < N * 32 {
            Some((self.a[index / N] >> (N % 32) & 1) != 0)
        } else {
            None
        }
    }

    pub fn set(&mut self, index: usize, value: bool) {
        let value = value as u32;
        let mask = 1 << (N % 32);
        let value_mask = value << (N % 32);
        self.a[index / N] = self.a[index / N] & !mask | value_mask
    }
}
