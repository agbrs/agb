use std::sync::Arc;

#[derive(Debug, Clone, Default)]
struct BufferData {
    buffer: Vec<f64>,
    frequency: f64,
}

#[derive(Debug, Default)]
pub struct Audio {
    buffer: arc_swap::ArcSwap<BufferData>,
}

impl Audio {
    pub fn set_buffer(&self, buffer: Vec<f64>, frequency: f64) {
        self.buffer
            .store(Arc::new(BufferData { buffer, frequency }));
    }

    pub fn play(
        &self,
        data: &mut [f32],
        channel_count: usize,
        frequency: f64,
        mut pos: f64,
    ) -> f64 {
        let buffer_data = self.buffer.load();

        if buffer_data.buffer.is_empty() {
            return pos;
        }

        if pos as usize >= buffer_data.buffer.len() {
            pos = 0.0;
        }

        for samples in data.chunks_exact_mut(channel_count) {
            let value = buffer_data.buffer[pos as usize];
            pos = (pos + buffer_data.frequency / frequency) % (buffer_data.buffer.len() as f64);

            for sample in samples {
                *sample = value as f32 * 0.25;
            }
        }

        pos
    }
}
