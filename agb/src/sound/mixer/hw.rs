use crate::memory_mapped::MemoryMapped;

// Once we have proper DMA support, we should use that rather than hard coding these here too
const DMA1_SOURCE_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(0x0400_00bc) };
const DMA1_DEST_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(0x0400_00c0) };
const _DMA1_WORD_COUNT: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_00c4) }; // sound ignores this for some reason
const DMA1_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_00c6) };

const DMA2_SOURCE_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(0x0400_00ca) };
const DMA2_DEST_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(0x0400_00cc) };
const _DMA2_WORD_COUNT: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_00d0) };
const DMA2_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_00d2) };

const FIFOA_DEST_ADDR: u32 = 0x0400_00a0;
const FIFOB_DEST_ADDR: u32 = 0x0400_00a4;

// Similarly for proper timer support
const TIMER0_COUNTER: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0100) };
const TIMER0_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0102) };

const SOUND_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0082) };
const SOUND_CONTROL_X: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0084) };

const DMA_CONTROL_SETTING_FOR_SOUND: u16 = {
    let dest_fixed: u16 = 2 << 5; // dest addr control = fixed
    let repeat: u16 = 1 << 9;
    let transfer_type: u16 = 1 << 10; // transfer in words
    let dma_start_timing: u16 = 3 << 12; // sound fifo timing
    let enable: u16 = 1 << 15; // enable

    dest_fixed | repeat | transfer_type | dma_start_timing | enable
};

#[derive(Copy, Clone)]
pub(super) enum LeftOrRight {
    Left,
    Right,
}

pub(super) fn enable_dma_for_sound(sound_memory: &[i8], lr: LeftOrRight) {
    match lr {
        LeftOrRight::Left => enable_dma1_for_sound(sound_memory),
        LeftOrRight::Right => enable_dma2_for_sound(sound_memory),
    }
}

fn enable_dma1_for_sound(sound_memory: &[i8]) {
    DMA1_CONTROL.set(0);
    DMA1_SOURCE_ADDR.set(sound_memory.as_ptr() as u32);
    DMA1_DEST_ADDR.set(FIFOA_DEST_ADDR);
    DMA1_CONTROL.set(DMA_CONTROL_SETTING_FOR_SOUND);
}

fn enable_dma2_for_sound(sound_memory: &[i8]) {
    DMA2_CONTROL.set(0);
    DMA2_SOURCE_ADDR.set(sound_memory.as_ptr() as u32);
    DMA2_DEST_ADDR.set(FIFOB_DEST_ADDR);
    DMA2_CONTROL.set(DMA_CONTROL_SETTING_FOR_SOUND);
}

pub(super) fn set_sound_control_register_for_mixer() {
    let sound_a_volume_100: u16 = 1 << 2;
    let sound_a_rout: u16 = 0 << 8; // sound A is for left channel only
    let sound_a_lout: u16 = 1 << 9;
    let sound_a_fifo_reset: u16 = 1 << 11;

    let sound_b_volume_100: u16 = 1 << 3;
    let sound_b_rout: u16 = 1 << 12;
    let sound_b_lout: u16 = 1 << 13;
    let sound_b_fifo_reset: u16 = 1 << 15;

    SOUND_CONTROL.set(
        sound_a_volume_100
            | sound_a_rout
            | sound_a_lout
            | sound_a_fifo_reset
            | sound_b_volume_100
            | sound_b_rout
            | sound_b_lout
            | sound_b_fifo_reset,
    );

    // master sound enable
    SOUND_CONTROL_X.set(1 << 7);
}

pub(super) fn set_timer_counter_for_frequency_and_enable(frequency: i32) {
    let counter = 65536 - (16777216 / frequency);
    TIMER0_COUNTER.set(counter as u16);

    TIMER0_CONTROL.set(1 << 7); // enable the timer
}
