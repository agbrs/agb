use crate::memory_mapped::MemoryMapped;

#[non_exhaustive]
pub struct Mixer {}

impl Mixer {
    pub(crate) const unsafe fn new() -> Self {
        Mixer {}
    }

    pub fn enable(&self) {
        set_sound_control_register_for_mixer();
    }

    pub fn play_sound_starting_at(&self, sound_memory: &[u8]) {
        set_timer_counter_for_frequency_and_enable(SOUND_FREQUENCY);
        enable_dma1_for_sound(sound_memory);
    }
}

// I've picked one frequency that works nicely. But there are others that work nicely
// which we may want to consider in the future: https://web.archive.org/web/20070608011909/http://deku.gbadev.org/program/sound1.html
const SOUND_FREQUENCY: i32 = 10512;
const SOUND_BUFFER_SIZE: usize = 176;

// Once we have proper DMA support, we should use that rather than hard coding these here too
const DMA1_SOURCE_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(0x0400_00bc) };
const DMA1_DEST_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(0x0400_00c0) };
const _DMA1_WORD_COUNT: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_00c4) }; // sound ignores this for some reason
const DMA1_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_00c6) };

const FIFOA_DEST_ADDR: u32 = 0x0400_00a0;

// Similarly for proper timer support
const TIMER0_COUNTER: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0100) };
const TIMER0_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0102) };

const SOUND_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0082) };
const SOUND_CONTROL_X: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0084) };

fn enable_dma1_for_sound(sound_memory: &[u8]) {
    let dest_fixed: u16 = 2 << 5; // dest addr control = fixed
    let repeat: u16 = 1 << 9;
    let transfer_type: u16 = 1 << 10; // transfer in words
    let dma_start_timing: u16 = 3 << 12; // sound fifo timing
    let enable: u16 = 1 << 15; // enable

    let address: *const u8 = &sound_memory[0];
    DMA1_SOURCE_ADDR.set(address as u32);
    DMA1_DEST_ADDR.set(FIFOA_DEST_ADDR);
    DMA1_CONTROL.set(dest_fixed | repeat | transfer_type | dma_start_timing | enable);
}

fn set_sound_control_register_for_mixer() {
    let sound_a_volume_100: u16 = 1 << 2;
    let sound_a_rout: u16 = 1 << 8;
    let sound_a_lout: u16 = 1 << 9;
    let sound_a_fifo_reset: u16 = 1 << 11;

    SOUND_CONTROL.set(sound_a_volume_100 | sound_a_rout | sound_a_lout | sound_a_fifo_reset);

    // master sound enable
    SOUND_CONTROL_X.set(1 << 7);
}

fn set_timer_counter_for_frequency_and_enable(frequency: i32) {
    let counter = 65536 - (16777216 / frequency);
    TIMER0_COUNTER.set(counter as u16);

    TIMER0_CONTROL.set(1 << 7); // enable the timer
}
