use agb::{
    save::{Error, SaveManager},
    sync::Static,
    Gba,
};

static MAXIMUM_LEVEL: Static<u32> = Static::new(0);

pub fn init_save(gba: &mut Gba) -> Result<(), Error> {
    gba.save.init_sram();

    let mut access = gba.save.access()?;

    let mut buffer = [0; 1];
    access.read(0, &mut buffer)?;

    if buffer[0] != 0 {
        access.prepare_write(0..1)?.write(0, &[0])?;
        core::mem::drop(access);
        save_max_level(&mut gba.save, 0)?;
    } else {
        let mut buffer = [0; 4];
        access.read(1, &mut buffer)?;
        let max_level = u32::from_le_bytes(buffer);

        if max_level > 100 {
            MAXIMUM_LEVEL.write(0)
        } else {
            MAXIMUM_LEVEL.write(max_level)
        }
    }

    Ok(())
}

pub fn load_max_level() -> u32 {
    MAXIMUM_LEVEL.read()
}

pub fn save_max_level(save: &mut SaveManager, level: u32) -> Result<(), Error> {
    save.access()?
        .prepare_write(1..5)?
        .write(1, &level.to_le_bytes())?;
    MAXIMUM_LEVEL.write(level);
    Ok(())
}
