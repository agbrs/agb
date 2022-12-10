use crate::memory_mapped::MemoryMapped;
use crate::timer::Timer;

const fn dma_source_addr(dma: usize) -> usize {
    0x0400_00b0 + 0x0c * dma
}

const fn dma_dest_addr(dma: usize) -> usize {
    0x0400_00b4 + 0x0c * dma
}

const fn dma_control_addr(dma: usize) -> usize {
    0x0400_00ba + 0x0c * dma
}

// Once we have proper DMA support, we should use that rather than hard coding these here too
const DMA1_SOURCE_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(dma_source_addr(1)) };
const DMA1_DEST_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(dma_dest_addr(1)) };
const DMA1_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(dma_control_addr(1)) };

const DMA2_SOURCE_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(dma_source_addr(2)) };
const DMA2_DEST_ADDR: MemoryMapped<u32> = unsafe { MemoryMapped::new(dma_dest_addr(2)) };
const DMA2_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(dma_control_addr(2)) };

const FIFO_A_DEST_ADDR: u32 = 0x0400_00a0;
const FIFO_B_DEST_ADDR: u32 = 0x0400_00a4;

const SOUND_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0082) };
const SOUND_CONTROL_X: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0084) };

const SOUND_BIAS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0088) };

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

pub(super) fn enable_dma_for_sound(sound_memory: *const i8, lr: LeftOrRight) {
    match lr {
        LeftOrRight::Left => enable_dma1_for_sound(sound_memory),
        LeftOrRight::Right => enable_dma2_for_sound(sound_memory),
    }
}

fn enable_dma1_for_sound(sound_memory: *const i8) {
    DMA1_CONTROL.set(0);
    DMA1_SOURCE_ADDR.set(sound_memory as u32);
    DMA1_DEST_ADDR.set(FIFO_A_DEST_ADDR);
    DMA1_CONTROL.set(DMA_CONTROL_SETTING_FOR_SOUND);
}

fn enable_dma2_for_sound(sound_memory: *const i8) {
    DMA2_CONTROL.set(0);
    DMA2_SOURCE_ADDR.set(sound_memory as u32);
    DMA2_DEST_ADDR.set(FIFO_B_DEST_ADDR);
    DMA2_CONTROL.set(DMA_CONTROL_SETTING_FOR_SOUND);
}

pub(super) fn set_sound_control_register_for_mixer() {
    let sound_a_volume_100: u16 = 1 << 2;
    let sound_a_rout: u16 = 0 << 8; // sound A is for left channel only
    let sound_a_lout: u16 = 1 << 9;
    let sound_a_fifo_reset: u16 = 1 << 11;

    let sound_b_volume_100: u16 = 1 << 3;
    let sound_b_rout: u16 = 1 << 12;
    let sound_b_lout: u16 = 0 << 13;
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

    // Set the sound bias PWM resampling rate to 8bit at 65536Hz (default for most games)
    SOUND_BIAS.set(SOUND_BIAS.get() | 1 << 14);
}

pub(super) fn set_timer_counter_for_frequency_and_enable(timer: &mut Timer, frequency: i32) {
    timer.set_overflow_amount((16777216 / frequency) as u16);
    timer.set_enabled(true);
}
