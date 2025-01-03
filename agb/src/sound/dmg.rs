use crate::memory_mapped::MemoryMapped;

const CHANNEL_1_SWEEP: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0060) };
const CHANNEL_1_LENGTH_DUTY_ENVELOPE: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0062) };
const CHANNEL_1_FREQUENCY_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0064) };

const CHANNEL_2_LENGTH_DUTY_ENVELOPE: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0068) };
const CHANNEL_2_FREQUENCY_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_006c) };

const CHANNEL_4_LENGTH_ENVELOPE: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0078) };
const CHANNEL_4_FREQUENCY_CONTROL: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_007c) };

const MASTER_SOUND_VOLUME_ENABLE: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0080) };
const MASTER_SOUND_VOLUME_MIXING: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0082) };
const MASTER_SOUND_STATUS: MemoryMapped<u16> = unsafe { MemoryMapped::new(0x0400_0084) };

#[non_exhaustive]
pub struct Sound {}

impl Sound {
    pub(crate) const unsafe fn new() -> Self {
        Sound {}
    }

    #[must_use]
    pub fn channel1(&self) -> Channel1 {
        Channel1 {}
    }

    #[must_use]
    pub fn channel2(&self) -> Channel2 {
        Channel2 {}
    }

    #[must_use]
    pub fn noise(&self) -> Noise {
        Noise {}
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
        let length_bits = u16::from(length.unwrap_or(0));
        assert!(length_bits < 64, "Length must be less than 64");

        let length_flag: u16 = length.map_or(0, |_| 1 << 14);
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
        let length_bits = u16::from(length.unwrap_or(0));
        assert!(length_bits < 64, "Length must be less than 64");

        let length_flag: u16 = length.map_or(0, |_| 1 << 14);
        let initial: u16 = 1 << 15;

        assert!(frequency < 2048, "Frequency must be less than 2048");

        CHANNEL_2_LENGTH_DUTY_ENVELOPE
            .set(envelope_settings.as_bits() | duty_cycle.as_bits() | length_bits);
        CHANNEL_2_FREQUENCY_CONTROL.set(frequency | length_flag | initial);
    }
}

#[non_exhaustive]
pub struct Noise {}

impl Noise {
    pub fn play_sound(
        &self,
        length: Option<u8>,
        envelope_setting: &EnvelopeSettings,
        frequency_divider: u8,
        counter_step_width_15: bool,
        shift_clock_frequency: u8,
    ) {
        let length_bits = u16::from(length.unwrap_or(0));
        assert!(length_bits < 64, "length must be less than 16");

        assert!(
            frequency_divider < 8,
            "frequency divider must be less than 8"
        );
        assert!(
            shift_clock_frequency < 16,
            "frequency clock divider must be less than 16"
        );

        let length_flag: u16 = length.map_or(0, |_| 1 << 14);
        let initial: u16 = 1 << 15;

        let counter_step_bit = if counter_step_width_15 { 0 } else { 1 << 3 };

        CHANNEL_4_LENGTH_ENVELOPE.set(length_bits | envelope_setting.as_bits());
        CHANNEL_4_FREQUENCY_CONTROL.set(
            u16::from(frequency_divider)
                | counter_step_bit
                | (u16::from(shift_clock_frequency) << 4)
                | length_flag
                | initial,
        );
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
    #[must_use]
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
        (u16::from(self.number_of_sweep_shifts) & 0b111)
            | ((1 - self.sound_direction.as_bits()) << 3) // sweep works backwards 
            | ((u16::from(self.sweep_time) & 0b111) << 4)
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
    #[must_use]
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
        (u16::from(self.step_time) << 8)
            | (self.direction.as_bits() << 11)
            | (u16::from(self.initial_volume) << 12)
    }
}

impl Default for EnvelopeSettings {
    fn default() -> Self {
        EnvelopeSettings::new(0, SoundDirection::Increase, 15)
    }
}

#[derive(Copy, Clone)]
pub enum DutyCycle {
    OneEighth,
    OneQuarter,
    Half,
    ThreeQuarters,
}

impl DutyCycle {
    fn as_bits(self) -> u16 {
        use DutyCycle::*;

        match self {
            OneEighth => 0,
            OneQuarter => 1,
            Half => 2,
            ThreeQuarters => 3,
        }
    }
}
