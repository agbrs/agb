use agb::Gba;
use agb::save::Error;
use agb::sync::Static;

static HIGHSCORE: Static<u32> = Static::new(0);

pub fn init_save(gba: &mut Gba) -> Result<(), Error> {
    gba.save.init_sram();

    let mut access = gba.save.access()?;

    let mut buffer = [0; 1];
    access.read(0, &mut buffer)?;

    if buffer[0] != 0 {
        access.prepare_write(0..1)?.write(0, &[0])?;
        core::mem::drop(access);
        save_high_score(gba, 0)?;
    } else {
        let mut buffer = [0; 4];
        access.read(1, &mut buffer)?;
        let high_score = u32::from_le_bytes(buffer);

        if high_score > 100 {
            HIGHSCORE.write(0)
        } else {
            HIGHSCORE.write(high_score)
        }
    }

    Ok(())
}

pub fn load_high_score() -> u32 {
    HIGHSCORE.read()
}

pub fn save_high_score(gba: &mut Gba, score: u32) -> Result<(), Error> {
    gba.save.access()?.prepare_write(1..5)?.write(1, &score.to_le_bytes())?;
    HIGHSCORE.write(score);
    Ok(())
}
