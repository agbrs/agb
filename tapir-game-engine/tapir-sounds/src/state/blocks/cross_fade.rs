#[derive(Clone)]
pub struct CrossFade {
    left_amount: f64,
    split: f64,
    right_amount: f64,
}

impl Default for CrossFade {
    fn default() -> Self {
        Self {
            left_amount: 1.0,
            split: 0.0,
            right_amount: 1.0,
        }
    }
}

impl super::BlockType for CrossFade {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "Cross fade".into()
    }

    fn inputs(&self) -> Vec<(std::borrow::Cow<'static, str>, super::Input)> {
        vec![
            ("Left".into(), super::Input::Amplitude(self.left_amount)),
            ("Cross".into(), super::Input::Amplitude(self.split)),
            ("Right".into(), super::Input::Amplitude(self.right_amount)),
        ]
    }

    fn set_input(&mut self, index: usize, value: &super::Input) {
        match (index, value) {
            (0, super::Input::Amplitude(new_left)) => {
                self.left_amount = *new_left;
            }
            (1, super::Input::Amplitude(new_cross)) => {
                self.split = *new_cross;
            }
            (2, super::Input::Amplitude(new_right)) => {
                self.right_amount = *new_right;
            }
            _ => panic!("Invalid input {index} {value:?}"),
        }
    }

    fn calculate(&self, _global_frequency: f64, inputs: &[Option<&[f64]>]) -> Vec<f64> {
        let left_input = inputs[0].unwrap_or(&[0.0]);
        let cross_input = inputs[1].unwrap_or(&[0.0]);
        let right_input = inputs[2].unwrap_or(&[0.0]);

        // we take the maximum length of all the inputs
        let output_length = left_input
            .len()
            .max(cross_input.len())
            .max(right_input.len());

        (0..output_length)
            .map(|i| {
                let cross = cross_input[i % cross_input.len()];

                // cross is between 0 and 1 for how much of left vs. right to take
                let cross = (cross + self.split).clamp(-1., 1.) / 2.0 + 0.5;

                let left = left_input[i % left_input.len()] * self.left_amount;
                let right = right_input[i % right_input.len()] * self.right_amount;

                (left * cross) + (right * (1.0 - cross))
            })
            .collect()
    }
}
