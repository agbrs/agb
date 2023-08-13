use std::borrow::Cow;

use super::BlockType;

#[derive(Clone)]
pub struct Noise {
    base_frequency: f64,
    base_amplitude: f64,
    periods: f64,
    seed: f64,
}

impl Default for Noise {
    fn default() -> Self {
        Self {
            base_frequency: 128.0,
            base_amplitude: 0.5,
            periods: 1.0,
            seed: Default::default(),
        }
    }
}

impl BlockType for Noise {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Noise")
    }

    fn inputs(&self) -> Vec<(Cow<'static, str>, super::Input)> {
        vec![
            (
                "Frequency".into(),
                super::Input::Frequency(self.base_frequency),
            ),
            (
                "Amplitude".into(),
                super::Input::Amplitude(self.base_amplitude),
            ),
            ("Periods".into(), super::Input::Periods(self.periods)),
            ("Seed".into(), super::Input::Periods(self.seed)),
        ]
    }

    fn set_input(&mut self, index: usize, value: &super::Input) {
        match (index, value) {
            (0, super::Input::Frequency(new_frequency)) => {
                if *new_frequency != 0.0 {
                    self.base_frequency = *new_frequency;
                }
            }
            (1, super::Input::Amplitude(new_amplitude)) => {
                self.base_amplitude = *new_amplitude;
            }
            (2, super::Input::Periods(new_periods)) => {
                self.periods = *new_periods;
            }
            (3, super::Input::Periods(new_seed)) => {
                self.seed = *new_seed;
            }
            _ => panic!("Invalid input {index} {value:?}"),
        }
    }

    fn calculate(&self, global_frequency: f64, _inputs: &[Option<&[f64]>]) -> Vec<f64> {
        let mut rng = fastrand::Rng::with_seed(self.seed.to_bits());

        let periods = if self.periods == 0.0 {
            1.0
        } else {
            self.periods
        };

        let period_length = (global_frequency / self.base_frequency).ceil();
        let length = (period_length * periods) as usize;

        let mut ret = vec![0.0; length];

        ret.fill_with(|| (rng.f64() * 2.0 - 1.0) * self.base_amplitude);

        ret
    }
}
