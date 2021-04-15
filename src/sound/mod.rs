use crate::memory_mapped::MemoryMapped;

const CHANNEL_1_SWEEP: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0060) };
const CHANNEL_1_LENGTH_DUTY_ENVELOPE: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0062) };
const CHANNEL_1_FREQUENCY_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0064) };

const MASTER_SOUND_VOLUME_ENABLE: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0080) };
const MASTER_SOUND_VOLUME_MIXING: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0082) };
const MASTER_SOUND_STATUS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0084) };

#[non_exhaustive]
pub struct Sound {}

impl Sound {
    pub(crate) const unsafe fn new() -> Self {
        Sound {}
    }

    pub fn channel1(&self) -> Channel1 {
        Channel1 {}
    }

    pub fn enable(&self) {
        MASTER_SOUND_STATUS.set_bits(1, 1, 7);

        MASTER_SOUND_VOLUME_ENABLE.set(0b1111_1111_0_111_0_111);
        MASTER_SOUND_VOLUME_MIXING.set(0b10);
    }
}

#[non_exhaustive]
pub struct Channel1 {}

impl Channel1 {
    pub fn play_sound(&self) {
        CHANNEL_1_SWEEP.set(0b00000000_111_0_010);
        CHANNEL_1_LENGTH_DUTY_ENVELOPE.set(0b111_1_001_01_111111);
        CHANNEL_1_FREQUENCY_CONTROL.set(0b1_0_000_01000000000);
    }
}
