use crate::memory_mapped::MemoryMapped;

const CHANNEL_1_SWEEP: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0060) };
const CHANNEL_1_LENGTH_DUTY_ENVELOPE: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0062) };
const CHANNEL_1_FREQUENCY_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0064) };

const CHANNEL_2_LENGTH_DUTY_ENVELOPE: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0068) };
const CHANNEL_2_FREQUENCY_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_006c) };

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

    pub fn channel2(&self) -> Channel2 {
        Channel2 {}
    }

    pub fn enable(&self) {
        MASTER_SOUND_STATUS.set_bits(1, 1, 7);

        #[allow(clippy::unusual_byte_groupings)] // I've split these like this for a reason
        MASTER_SOUND_VOLUME_ENABLE.set(0b1111_1111_0_111_0_111);
        MASTER_SOUND_VOLUME_MIXING.set(0b10);
    }
}

#[non_exhaustive]
pub struct Channel1 {}

impl Channel1 {
    pub fn play_sound(
        &self,
        frequency: u16,
        length: Option<u8>,
        sweep_settings: &SweepSettings,
        envelope_settings: &EnvelopeSettings,
        duty_cycle: DutyCycle,
    ) {
        CHANNEL_1_SWEEP.set(sweep_settings.as_bits());
        let length_bits = length.unwrap_or(0) as u16;
        assert!(length_bits < 64, "Length must be less than 64");

        let length_flag: u16 = length.map(|_| 1 << 14).unwrap_or(0);
        let initial: u16 = 1 << 15;

        assert!(frequency < 2048, "Frequency must be less than 2048");

        CHANNEL_1_LENGTH_DUTY_ENVELOPE
            .set(envelope_settings.as_bits() | duty_cycle.as_bits() | length_bits);
        CHANNEL_1_FREQUENCY_CONTROL.set(frequency | length_flag | initial);
    }
}

#[non_exhaustive]
pub struct Channel2 {}

impl Channel2 {
    pub fn play_sound(
        &self,
        frequency: u16,
        length: Option<u8>,
        envelope_settings: &EnvelopeSettings,
        duty_cycle: DutyCycle,
    ) {
        let length_bits = length.unwrap_or(0) as u16;
        assert!(length_bits < 64, "Length must be less than 64");

        let length_flag: u16 = length.map(|_| 1 << 14).unwrap_or(0);
        let initial: u16 = 1 << 15;

        assert!(frequency < 2048, "Frequency must be less than 2048");

        CHANNEL_2_LENGTH_DUTY_ENVELOPE
            .set(envelope_settings.as_bits() | duty_cycle.as_bits() | length_bits);
        CHANNEL_2_FREQUENCY_CONTROL.set(frequency | length_flag | initial);
    }
}

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

pub struct EnvelopeSettings {
    step_time: u8,
    direction: SoundDirection,
    initial_volume: u8,
}

impl EnvelopeSettings {
    pub fn new(step_time: u8, direction: SoundDirection, initial_volume: u8) -> Self {
        assert!(step_time < 8, "Step time must be less than 8");
        assert!(initial_volume < 16, "Initial volume must be less that 16");
        EnvelopeSettings {
            step_time,
            direction,
            initial_volume,
        }
    }

    fn as_bits(&self) -> u16 {
        (self.step_time as u16) << 8
            | (self.direction.as_bits() << 11)
            | ((self.initial_volume as u16) << 12)
    }
}

impl Default for EnvelopeSettings {
    fn default() -> Self {
        EnvelopeSettings::new(0, SoundDirection::Increase, 15)
    }
}

pub enum DutyCycle {
    OneEighth,
    OneQuarter,
    Half,
    ThreeQuarters,
}

impl DutyCycle {
    fn as_bits(&self) -> u16 {
        use DutyCycle::*;

        match &self {
            OneEighth => 0,
            OneQuarter => 1,
            Half => 2,
            ThreeQuarters => 3,
        }
    }
}
