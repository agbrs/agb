use std::borrow::Cow;

use super::{BlockName, BlockType, Input};

#[derive(Clone)]
pub struct BandPassFilter {
    base_amplitude: f64,
    lower_bound: f64,
    upper_bound: f64,
}

impl Default for BandPassFilter {
    fn default() -> Self {
        Self {
            base_amplitude: 1.0,
            lower_bound: 0.0,
            upper_bound: 1.0,
        }
    }
}

impl BandPassFilter {
    pub fn name() -> BlockName {
        BlockName {
            category: super::BlockCategory::Alter,
            name: "Band pass filter".to_owned(),
        }
    }
}

impl BlockType for BandPassFilter {
    fn name(&self) -> BlockName {
        Self::name()
    }

    fn inputs(&self) -> Vec<(Cow<'static, str>, Input)> {
        vec![
            ("Amplitude".into(), Input::Amplitude(self.base_amplitude)),
            ("Lower bound".into(), Input::Periods(self.lower_bound)),
            ("Upper bound".into(), Input::Periods(self.upper_bound)),
        ]
    }

    fn set_input(&mut self, index: usize, value: &Input) {
        match (index, value) {
            (0, Input::Amplitude(amplitude)) => {
                self.base_amplitude = amplitude.clamp(0.0, 1.0);
            }
            (1, Input::Periods(new_lower_bound)) => {
                self.lower_bound = new_lower_bound.clamp(0.0, self.upper_bound);
            }
            (2, Input::Periods(new_upper_bound)) => {
                self.upper_bound = new_upper_bound.clamp(self.lower_bound, 1.0);
            }
            _ => panic!("Invalid input {index} {value:?}"),
        }
    }

    fn calculate(&self, _global_frequency: f64, inputs: &[Option<&[f64]>]) -> Vec<f64> {
        let input = inputs[0].unwrap_or_default();

        if input.is_empty() {
            return vec![];
        }

        let mut buffer: Vec<_> = input
            .iter()
            .map(rustfft::num_complex::Complex::from)
            .collect();

        let mut planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_forward(input.len());
        fft.process(&mut buffer);

        let lower_frequencies = (buffer.len() as f64 * self.lower_bound) as usize;
        let upper_frequencies = (buffer.len() as f64 * self.upper_bound) as usize;

        for (i, value) in buffer.iter_mut().enumerate() {
            if i < lower_frequencies || i > upper_frequencies {
                *value = 0.0.into();
            }
        }

        let inv_fft = planner.plan_fft_inverse(input.len());
        inv_fft.process(&mut buffer);

        let normalization_amount = 1.0 / (buffer.len() as f64) * self.base_amplitude;

        buffer
            .iter()
            .map(|complex| complex.re * normalization_amount)
            .collect()
    }
}
