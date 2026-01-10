//! Utilities for graphics.

/// Copies the content of `src` into `target` but skipping transparent pixels.
///
/// Assumes 1 pixel is 4 bits, so useful for copying into 4-bit dynamic tiles
/// like in [`DynamicTile16`](crate::display::tiled::DynamicTile16) or
/// [`DynamicSprite16`](crate::display::object::DynamicSprite16).
///
/// Normally you shouldn't need to use this method, as you should be using
/// [`Object`s](crate::display::object::Object) or backgrounds. Only use this
/// if you specifically need dynamic sprites or tiles.
///
/// # Examples
///
/// ```
/// # #![no_main]
/// # #![no_std]
/// # #[agb::doctest]
/// # fn test(_: agb::Gba) {
/// use agb::display::utils::blit_16_colour;
///  
/// let a = &mut [0x89abcdef];
/// let b = &[0x01030507];
///
/// blit_16_colour(a, b);
/// assert_eq!(a[0], 0x81a3c5e7);
/// # }
/// ```
pub fn blit_16_colour(target: &mut [u32], src: &[u32]) {
    assert_eq!(target.len(), src.len());

    for (a, &b) in target.iter_mut().zip(src) {
        let hi = b & 0x8888_8888;
        let lo = b & 0x7777_7777;

        let set_nybbles = (hi | ((lo + 0x7777_7777) & 0x8888_8888)) >> 3;
        let mask = set_nybbles * 0xf;

        *a = (*a & !mask) | b;
    }
}
