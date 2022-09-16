use core::cmp;
use agb::save::{Error, MediaInfo};
use agb::sync::InitOnce;

fn init_sram(gba: &mut agb::Gba) -> &'static MediaInfo {
    static ONCE: InitOnce<MediaInfo> = InitOnce::new();
    ONCE.get(|| {
        crate::save_setup(gba);
        gba.save.access().unwrap().media_info().clone()
    })
}

#[derive(Clone)]
struct Rng(u32);
impl Rng {
    fn iter(&mut self) {
        self.0 = self.0.wrapping_mul(2891336453).wrapping_add(100001);
    }
    fn next_u8(&mut self) -> u8 {
        self.iter();
        (self.0 >> 22) as u8 ^ self.0 as u8
    }
    fn next_under(&mut self, under: u32) -> u32 {
        self.iter();
        let scale = 31 - under.leading_zeros();
        ((self.0 >> scale) ^ self.0) % under
    }
}

const MAX_BLOCK_SIZE: usize = 4 * 1024;

#[allow(clippy::needless_range_loop)]
fn do_test(
    gba: &mut agb::Gba, seed: Rng, offset: usize, len: usize, block_size: usize,
) -> Result<(), Error> {
    let mut buffer = [0; MAX_BLOCK_SIZE];

    let timers = gba.timers.timers();
    let mut access = gba.save.access_with_timer(timers.timer2)?;

    // writes data to the save media
    let mut prepared = access.prepare_write(offset..offset + len)?;
    let mut rng = seed.clone();
    let mut current = offset;
    let end = offset + len;
    while current != end {
        let cur_len = cmp::min(end - current, block_size);
        for i in 0..cur_len {
            buffer[i] = rng.next_u8();
        }
        prepared.write(current, &buffer[..cur_len])?;
        current += cur_len;
    }

    // validates the save media
    rng = seed;
    current = offset;
    while current != end {
        let cur_len = cmp::min(end - current, block_size);
        access.read(current, &mut buffer[..cur_len])?;
        for i in 0..cur_len {
            let cur_byte = rng.next_u8();
            assert_eq!(
                buffer[i], cur_byte,
                "Read does not match earlier write: {} != {} @ 0x{:05x}",
                buffer[i], cur_byte, current + i,
            );
        }
        current += cur_len;
    }

    Ok(())
}

#[test_case]
fn test_4k_blocks(gba: &mut agb::Gba) {
    let info = init_sram(gba);

    if info.len() >= (1 << 12) {
        do_test(gba, Rng(2000), 0, info.len(), 4 * 1024).expect("Test encountered error");
    }
}

#[test_case]
fn test_512b_blocks(gba: &mut agb::Gba) {
    let info = init_sram(gba);
    do_test(gba, Rng(1000), 0, info.len(), 512).expect("Test encountered error");
}

#[test_case]
fn test_partial_writes(gba: &mut agb::Gba) {
    let info = init_sram(gba);

    // test with random segments now.
    let mut rng = Rng(12345);
    for i in 0..8 {
        let rand_length = rng.next_under((info.len() >> 1) as u32) as usize + 50;
        let rand_offset = rng.next_under(info.len() as u32 - rand_length as u32) as usize;
        let block_size = cmp::min(rand_length >> 2, MAX_BLOCK_SIZE - 100);
        let block_size = rng.next_under(block_size as u32) as usize + 50;

        do_test(gba, Rng(i * 10000), rand_offset, rand_length, block_size)
            .expect("Test encountered error");
    }
}