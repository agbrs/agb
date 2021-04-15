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

pub enum SoundDirection {
    Increase,
    Decrease,
}

impl SoundDirection {
    fn as_bits(&self) -> u16 {
        match &self {
            SoundDirection::Increase => 1,
            SoundDirection::Decrease => 0,
        }
    }
}

impl Channel1 {
    pub fn play_sound(&self, sweep_settings: &SweepSettings) {
        CHANNEL_1_SWEEP.set(sweep_settings.as_bits());
        CHANNEL_1_LENGTH_DUTY_ENVELOPE.set(0b111_1_001_01_111111);
        CHANNEL_1_FREQUENCY_CONTROL.set(0b1_0_000_01000000000);
    }
}

pub struct SweepSettings {
    number_of_sweep_shifts: u8,
    sound_direction: SoundDirection,
    sweep_time: u8,
}

impl SweepSettings {
    pub fn new(
        number_of_sweep_shifts: u8,
        sound_direction: SoundDirection,
        sweep_time: u8,
    ) -> Self {
        assert!(
            number_of_sweep_shifts < 8,
            "Number of sweep shifts must be less than 8"
        );
        assert!(sweep_time < 8, "Sweep time must be less than 8");

        SweepSettings {
            number_of_sweep_shifts,
            sound_direction,
            sweep_time,
        }
    }

    fn as_bits(&self) -> u16 {
        ((self.number_of_sweep_shifts as u16) & 0b111)
            | (self.sound_direction.as_bits() << 3)
            | ((self.sweep_time as u16) & 0b111) << 4
    }
}

impl Default for SweepSettings {
    fn default() -> Self {
        SweepSettings::new(0, SoundDirection::Increase, 0)
    }
}
