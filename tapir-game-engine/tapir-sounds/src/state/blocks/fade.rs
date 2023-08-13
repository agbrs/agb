use std::borrow::Cow;

use super::BlockType;

#[derive(Clone)]
pub struct Fade {
    amplitude: f64,
    offset: f64,
}

impl Default for Fade {
    fn default() -> Self {
        Self {
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl Fade {
    pub fn name() -> super::BlockName {
        super::BlockName {
            category: super::BlockCategory::Combine,
            name: "Fade".to_owned(),
        }
    }
}

impl BlockType for Fade {
    fn name(&self) -> super::BlockName {
        Self::name()
    }

    fn inputs(&self) -> Vec<(Cow<'static, str>, super::Input)> {
        vec![
            ("Amplitude".into(), super::Input::Amplitude(self.amplitude)),
            ("Offset".into(), super::Input::Periods(self.offset)),
        ]
    }

    fn set_input(&mut self, index: usize, value: &super::Input) {
        match (index, value) {
            (0, super::Input::Amplitude(new_amplitude)) => {
                self.amplitude = *new_amplitude;
            }
            (1, super::Input::Periods(new_offset)) => {
                self.offset = new_offset.clamp(0.0, 1.0);
            }
            _ => panic!("Invalid input {index} {value:?}"),
        }
    }

    fn calculate(&self, _global_frequency: f64, inputs: &[Option<&[f64]>]) -> Vec<f64> {
        let input = inputs[0].unwrap_or_default();

        let length = input.len() as f64 * (1.0 - self.offset);
        let offset_start = (input.len() as f64 * self.offset) as usize;

        if self.amplitude > 0.0 {
            // start at amplitude and end at 0. But only start doing something once we get to offset
            input
                .iter()
                .enumerate()
                .map(|(i, value)| {
                    if i < offset_start {
                        *value * self.amplitude
                    } else {
                        let amount = self.amplitude * (1.0 - (i - offset_start) as f64 / length);
                        *value * amount
                    }
                })
                .collect()
        } else {
            let offset_start = input.len() - offset_start;

            // start at 0 and end at -amplitude. But only start doing something once we get to offset
            input
                .iter()
                .enumerate()
                .map(|(i, value)| {
                    if i > offset_start {
                        *value * -self.amplitude
                    } else {
                        let amount = -self.amplitude * (i as f64 / length);
                        *value * amount
                    }
                })
                .collect()
        }
    }
}
