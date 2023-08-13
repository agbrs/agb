use std::{borrow::Cow, f64::consts::PI};

use super::{stretch_frequency_shift, BlockType, Input};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FundamentalShapeType {
    Sine,
    Square,
    Triangle,
    Saw,
}

impl FundamentalShapeType {
    pub fn all() -> impl Iterator<Item = FundamentalShapeType> + 'static {
        [Self::Sine, Self::Square, Self::Triangle, Self::Saw].into_iter()
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Sine => "Sine",
            Self::Square => "Square",
            Self::Triangle => "Triangle",
            Self::Saw => "Saw",
        }
    }

    fn value(self, index: f64) -> f64 {
        match self {
            Self::Sine => (index * PI * 2.0).sin(),
            Self::Square => {
                if index < 0.5 {
                    -1.0
                } else {
                    1.0
                }
            }
            Self::Triangle => {
                if index < 0.25 {
                    index * 4.0
                } else if index < 0.75 {
                    (index - 0.5) * -4.0
                } else {
                    (index - 0.75) * 4.0 - 1.0
                }
            }
            Self::Saw => {
                if index < 0.5 {
                    index * 2.0
                } else {
                    index * 2.0 - 2.0
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct FundamentalShapeBlock {
    fundamental_shape_type: FundamentalShapeType,
    periods: f64,
    base_frequency: f64,
    base_amplitude: f64,
    offset: f64,
}

impl FundamentalShapeBlock {
    pub fn new(fundamental_shape_type: FundamentalShapeType) -> Self {
        Self {
            fundamental_shape_type,
            periods: 1.0,
            base_frequency: 256.0,
            base_amplitude: 0.5,
            offset: 0.0,
        }
    }
}

impl BlockType for FundamentalShapeBlock {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.fundamental_shape_type.name())
    }

    fn inputs(&self) -> Vec<(Cow<'static, str>, Input)> {
        vec![
            ("Frequency".into(), Input::Frequency(self.base_frequency)),
            ("Amplitude".into(), Input::Amplitude(self.base_amplitude)),
            ("Periods".into(), Input::Periods(self.periods)),
            ("Offset".into(), Input::Periods(self.offset)),
        ]
    }

    fn set_input(&mut self, index: usize, value: &Input) {
        match (index, value) {
            (0, Input::Frequency(new_frequency)) => {
                if *new_frequency != 0.0 {
                    self.base_frequency = *new_frequency;
                }
            }
            (1, Input::Amplitude(new_amplitude)) => {
                self.base_amplitude = *new_amplitude;
            }
            (2, Input::Periods(new_periods)) => {
                self.periods = *new_periods;
            }
            (3, Input::Periods(new_offset)) => {
                self.offset = new_offset.clamp(0.0, 1.0);
            }
            (name, value) => panic!("Invalid input {name} with value {value:?}"),
        }
    }

    fn calculate(&self, global_frequency: f64, inputs: &[Option<&[f64]>]) -> Vec<f64> {
        let periods = if self.periods == 0.0 {
            1.0
        } else {
            self.periods
        };

        let period_length = (global_frequency / self.base_frequency).ceil();
        let length = (period_length * periods) as usize;

        let mut ret = Vec::with_capacity(length);
        for i in 0..length {
            let frequency_at_i = self.base_frequency
                * stretch_frequency_shift(
                    inputs[0]
                        .map(|frequency_input| frequency_input[i % frequency_input.len()])
                        .unwrap_or(0.0),
                )
                .clamp(0.1, 10_000.0);

            let amplitude_at_i = (self.base_amplitude
                * inputs[1]
                    .map(|amplitude_input| amplitude_input[i % amplitude_input.len()])
                    .unwrap_or(1.0))
            .clamp(-1.0, 1.0);

            let period_length_at_i = global_frequency / frequency_at_i;

            ret.push(
                self.fundamental_shape_type
                    .value((i as f64 / period_length_at_i + self.offset).fract())
                    * amplitude_at_i,
            );
        }

        ret
    }
}
