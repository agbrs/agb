use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Sparkline},
};
use sysinfo::System;

const SAMPLE_INTERVAL_MS: u64 = 100;
const SAMPLE_BUFFER_SIZE: usize = 60;

pub struct CpuMonitor {
    sys: System,
    samples: VecDeque<u64>,
    last_sample: Instant,
}

impl CpuMonitor {
    pub fn new() -> Self {
        Self {
            sys: System::new(),
            samples: VecDeque::with_capacity(SAMPLE_BUFFER_SIZE),
            last_sample: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        if self.last_sample.elapsed() >= Duration::from_millis(SAMPLE_INTERVAL_MS) {
            self.sys.refresh_cpu_usage();
            let usage = self.sys.global_cpu_usage() as u64;

            if self.samples.len() >= SAMPLE_BUFFER_SIZE {
                self.samples.pop_front();
            }
            self.samples.push_back(usage);
            self.last_sample = Instant::now();
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let graph_width = area.width.saturating_sub(2) as usize;
        let cpu_data = self.samples_for_width(graph_width);

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("CPU {}%", self.current())),
            )
            .data(&cpu_data)
            .max(100)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(sparkline, area);
    }

    fn current(&self) -> u64 {
        self.samples.back().copied().unwrap_or(0)
    }

    fn samples_for_width(&self, width: usize) -> Vec<u64> {
        let recent: Vec<u64> = self.samples.iter().rev().take(width).copied().collect();
        let mut data = vec![0; width.saturating_sub(recent.len())];
        data.extend(recent.into_iter().rev());
        data
    }
}
