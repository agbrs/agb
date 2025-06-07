use crate::{
    Gba,
    display::{
        Priority,
        tiled::{AffineBackground, AffineBackgroundSize, AffineBackgroundWrapBehaviour},
    },
};

#[test_case]
fn can_create_100_affine_backgrounds_one_at_a_time(gba: &mut Gba) {
    let mut gfx = gba.graphics.get();

    for _ in 0..100 {
        let bg = AffineBackground::new(
            Priority::P0,
            AffineBackgroundSize::Background64x64,
            AffineBackgroundWrapBehaviour::NoWrap,
        );
        let mut frame = gfx.frame();
        bg.show(&mut frame);
        frame.commit();
    }
}
