pub fn blit_4(target: &mut [u32], src: &[u32]) {
    assert_eq!(target.len(), src.len());

    for (a, &b) in target.iter_mut().zip(src) {
        let hi = b & 0x8888_8888;
        let lo = b & 0x7777_7777;

        let set_nybbles = (hi | ((lo + 0x7777_7777) & 0x8888_8888)) >> 3;
        let mask = set_nybbles * 0xf;

        *a = (*a & !mask) | b;
    }
}

#[cfg(test)]
mod tests {
    use crate::Gba;

    use super::*;

    #[test_case]
    fn test_blit4_simple_case(_: &mut Gba) {
        let a = &mut [0x89ABCDEF];
        let b = &[0x01030507];

        blit_4(a, b);
        assert_eq!(a[0], 0x81a3c5e7);
    }
}
