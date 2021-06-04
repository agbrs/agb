pub struct Bitarray<const N: usize> {
    a: [u32; N],
}

impl<const N: usize> Bitarray<N> {
    pub fn new() -> Self {
        Bitarray { a: [0; N] }
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        if index < N * 32 {
            Some((self.a[index / 32] >> (N % 32) & 1) != 0)
        } else {
            None
        }
    }

    pub fn set(&mut self, index: usize, value: bool) {
        let value = value as u32;
        let mask = 1 << (N % 32);
        let value_mask = value << (N % 32);
        self.a[index / 32] = self.a[index / 32] & !mask | value_mask
    }
}

#[test_case]
fn write_and_read(_gba: &mut crate::Gba) {
    let mut a: Bitarray<2> = Bitarray::new();
    assert_eq!(a.get(55).unwrap(), false, "expect unset values to be false");
    a.set(62, true);
    assert_eq!(a.get(62).unwrap(), true, "expect set value to be true");
    assert_eq!(a.get(120), None, "expect out of range to give None");
}

#[test_case]
fn test_everything(_gba: &mut crate::Gba) {
    for i in 0..64 {
        let mut a: Bitarray<2> = Bitarray::new();
        a.set(i, true);
        for j in 0..64 {
            let expected = if i == j { true } else { false };
            assert_eq!(
                a.get(j).unwrap(),
                expected,
                "set index {} and read {}, expected {} but got {}. u32 of this is {:#b}",
                i,
                j,
                expected,
                a.get(j).unwrap(),
                a.a[j / 32],
            );
        }
    }
}
