#![no_std]
#![no_main]

use agb::{
    display::{object::Object, palette16::Palette16, tiled::VRAM_MANAGER, HEIGHT, WIDTH},
    include_aseprite,
    input::ButtonController,
    save::{Error, SaveManager},
};

extern crate alloc;
use agb_fixnum::{vec2, Num, Vector2D};

include_aseprite!(
    mod sprites,
    "examples/gfx/chicken.aseprite"
);

struct Save {
    position: Vector2D<Num<i32, 8>>,
}

impl Save {
    fn write(&self, save: &mut SaveManager) -> Result<(), Error> {
        let mut access = save.access()?;

        let x = self.position.x.to_raw();
        let y = self.position.y.to_raw();

        let [a, b, c, d] = i32::to_ne_bytes(x);
        let [e, f, g, h] = i32::to_ne_bytes(y);

        access
            .prepare_write(0..9)?
            .write(0, &[0, a, b, c, d, e, f, g, h])
    }

    fn new(save: &mut SaveManager) -> Result<Self, Error> {
        save.init_sram();

        let mut access = save.access()?;

        let mut has_existing_save_buf = 0;

        access.read(0, core::slice::from_mut(&mut has_existing_save_buf))?;

        if has_existing_save_buf != 0 {
            Ok(Save {
                position: vec2(WIDTH / 2, HEIGHT / 2).change_base(),
            })
        } else {
            let mut p = [0; 8];
            access.read(1, &mut p)?;

            let x = i32::from_ne_bytes([p[0], p[1], p[2], p[3]]);
            let y = i32::from_ne_bytes([p[4], p[5], p[6], p[7]]);

            Ok(Save {
                position: vec2(Num::from_raw(x), Num::from_raw(y)),
            })
        }
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let vblank = agb::interrupt::VBlank::get();

    let mut gfx = gba.display.graphics.get();
    let mut save = Save::new(&mut gba.save).expect("able to read save data");
    let mut button = ButtonController::new();

    VRAM_MANAGER.set_background_palette(
        0,
        &Palette16::new([0xffff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    );

    loop {
        let mut frame = gfx.frame();
        button.update();

        save.position.x += button.x_tri() as i32;
        save.position.y += button.y_tri() as i32;

        save.position.x = save.position.x.clamp(0.into(), (WIDTH - 8).into());
        save.position.y = save.position.y.clamp(0.into(), (HEIGHT - 8).into());

        save.write(&mut gba.save).expect("able to write save data");

        Object::new(sprites::IDLE.sprite(0))
            .set_position(save.position.floor())
            .show(&mut frame);

        vblank.wait_for_vblank();
        frame.commit();
    }
}
