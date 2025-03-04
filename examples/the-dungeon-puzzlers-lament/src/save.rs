use agb::external::portable_atomic::{AtomicU32, Ordering};
use agb::{
    Gba,
    save::{Error, SaveManager},
};

static MAXIMUM_LEVEL: AtomicU32 = AtomicU32::new(0);
static SAVE_OFFSET: usize = 0xFF;

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
        access.read(SAVE_OFFSET, &mut buffer)?;
        let max_level = u32::from_le_bytes(buffer);

        if max_level > 100 {
            MAXIMUM_LEVEL.store(0, Ordering::SeqCst)
        } else {
            MAXIMUM_LEVEL.store(max_level, Ordering::SeqCst)
        }
    }

    Ok(())
}

pub fn load_max_level() -> u32 {
    MAXIMUM_LEVEL.load(Ordering::SeqCst)
}

pub fn save_max_level(save: &mut SaveManager, level: u32) -> Result<(), Error> {
    save.access()?
        .prepare_write(SAVE_OFFSET..SAVE_OFFSET + 4)?
        .write(SAVE_OFFSET, &level.to_le_bytes())?;
    MAXIMUM_LEVEL.store(level, Ordering::SeqCst);
    Ok(())
}
