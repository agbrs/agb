#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::env;

use app::TapirSoundApp;

mod app;
mod audio;
mod calculate;
mod save_load;
mod state;
mod widget;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        maximized: true,
        app_id: Some("tapir-sounds".to_string()),
        ..Default::default()
    };

    let args: Vec<_> = env::args().collect();

    eframe::run_native(
        "Tapir Sounds",
        options,
        Box::new(move |cc| Box::new(TapirSoundApp::new(cc, args.get(1).cloned()))),
    )
}
