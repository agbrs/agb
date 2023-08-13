use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

#[derive(Debug, Clone, Default)]
struct BufferData {
    buffer: Vec<f64>,
    frequency: f64,
}

#[derive(Debug)]
pub struct Audio {
    buffer: arc_swap::ArcSwap<BufferData>,

    pos: AtomicU64,
    should_loop: AtomicBool,
    should_play: AtomicBool,
}

impl Default for Audio {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            pos: Default::default(),
            should_loop: AtomicBool::new(true),
            should_play: Default::default(),
        }
    }
}

impl Audio {
    pub fn set_buffer(&self, buffer: Vec<f64>, frequency: f64) {
        self.buffer
            .store(Arc::new(BufferData { buffer, frequency }));
        self.pos.store(0.0f64.to_bits(), Ordering::SeqCst);
    }

    pub fn should_loop(&self) -> bool {
        self.should_loop.load(Ordering::SeqCst)
    }

    pub fn set_should_loop(&self, new_value: bool) {
        self.should_loop.store(new_value, Ordering::SeqCst);
    }

    pub fn start_playing(&self) {
        self.should_play.store(true, Ordering::SeqCst);
        self.pos.store(0.0f64.to_bits(), Ordering::SeqCst);
    }

    pub fn play(&self, data: &mut [f32], channel_count: usize, frequency: f64) {
        let original_pos = self.pos.load(Ordering::SeqCst);
        let mut pos = f64::from_bits(original_pos);

        let should_loop = self.should_loop.load(Ordering::SeqCst);

        let buffer_data = self.buffer.load();

        if buffer_data.buffer.is_empty() || !self.should_play.load(Ordering::SeqCst) {
            data.fill(0.0);
            return;
        }

        if pos as usize >= buffer_data.buffer.len() {
            if should_loop {
                pos = 0.0;
            } else {
                self.should_play.store(false, Ordering::SeqCst);
                data.fill(0.0);
            }
        }

        let buffer_len = buffer_data.buffer.len() as f64;

        for samples in data.chunks_exact_mut(channel_count) {
            let value = buffer_data.buffer[pos as usize];
            pos += buffer_data.frequency / frequency;

            if pos >= buffer_len {
                if should_loop {
                    pos -= buffer_len;
                } else {
                    self.should_play.store(false, Ordering::SeqCst);
                    break;
                }
            }

            for sample in samples {
                *sample = value as f32 * 0.25;
            }
        }

        let _ = self.pos.compare_exchange(
            original_pos,
            pos.to_bits(),
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
    }
}
