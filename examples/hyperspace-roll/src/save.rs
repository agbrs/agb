use agb::save::{Error, SaveManager};
use agb::sync::Static;
use agb::Gba;

static HIGH_SCORE: Static<u32> = Static::new(0);

pub fn init_save(gba: &mut Gba) -> Result<(), Error> {
    gba.save.init_sram();

    let mut access = gba.save.access()?;

    let mut buffer = [0; 1];
    access.read(0, &mut buffer)?;

    if buffer[0] != 0 {
        access.prepare_write(0..1)?.write(0, &[0])?;
        core::mem::drop(access);
        save_high_score(&mut gba.save, 0)?;
    } else {
        let mut buffer = [0; 4];
        access.read(1, &mut buffer)?;
        let high_score = u32::from_le_bytes(buffer);

        if high_score > 100 {
            HIGH_SCORE.write(0)
        } else {
            HIGH_SCORE.write(high_score)
        }
    }

    Ok(())
}

pub fn load_high_score() -> u32 {
    HIGH_SCORE.read()
}

pub fn save_high_score(save: &mut SaveManager, score: u32) -> Result<(), Error> {
    save.access()?
        .prepare_write(1..5)?
        .write(1, &score.to_le_bytes())?;
    HIGH_SCORE.write(score);
    Ok(())
}
