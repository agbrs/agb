use std::{
    collections::VecDeque,
    error::Error,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{anyhow, Context};
use clap::Parser;
use image_compare::compare_image;
use mgba::{LogLevel, Logger, MCore, MemoryBacked, VFile};

mod image_compare;

static LOGGER: Logger = Logger::new(my_logger);

static LOGGER_BUFFER: Mutex<VecDeque<(String, LogLevel, String)>> = Mutex::new(VecDeque::new());

fn my_logger(category: &str, level: LogLevel, s: String) {
    LOGGER_BUFFER
        .lock()
        .unwrap()
        .push_back((category.to_string(), level, s));
}

#[derive(Parser)]
struct CliArguments {
    rom: PathBuf,
}

struct TestRunner {
    mgba: MCore,
}

enum Timer {
    Start(u64),
    Total(u64),
}

impl TestRunner {
    fn new<V: VFile>(rom: V) -> Result<Self, Box<dyn Error>> {
        let mut mgba = MCore::new().ok_or(anyhow!("cannot create core"))?;

        mgba::set_global_default_logger(&LOGGER);

        mgba.load_rom(rom);

        Ok(Self { mgba })
    }

    fn run(mut self) -> Result<(), Box<dyn Error>> {
        let mut timer: Timer = Timer::Total(0);

        let mut mark_tests_as_soft_failed = false;
        let mut mark_this_test_as_soft_failed = false;
        loop {
            self.mgba.step();
            while let Some((category, level, message)) = LOGGER_BUFFER.lock().unwrap().pop_front() {
                match (category.as_ref(), level, message.as_ref()) {
                    (_, LogLevel::Fatal, fatal_message) => {
                        return Err(anyhow!("Failed with fatal message: {}", fatal_message).into());
                    }
                    ("GBA I/O", _, "Stub I/O register write: FFF800") => match timer {
                        Timer::Start(time) => {
                            let total_cycles = self.mgba.current_cycle() - time;
                            timer = Timer::Total(total_cycles);
                        }
                        Timer::Total(_) => {
                            timer = Timer::Start(self.mgba.current_cycle());
                        }
                    },
                    ("GBA Debug", _, debug_message) => {
                        if let Some(image_path) = debug_message.strip_prefix("image:") {
                            match compare_image(image_path, self.mgba.video_buffer()).with_context(
                                || anyhow!("Could not open image {} for comparison", image_path),
                            ) {
                                Ok(compare) => {
                                    if !compare.success() {
                                        eprintln!("Image and video buffer do not match");
                                        mark_tests_as_soft_failed = true;
                                        mark_this_test_as_soft_failed = true;
                                    }
                                }
                                Err(e) => eprintln!("{}", e),
                            }
                        } else if debug_message.ends_with("...") {
                            eprint!("{}", debug_message);
                        } else if debug_message == "[ok]" {
                            let cycles = match timer {
                                Timer::Start(_) => panic!("test completed with invalid timing"),
                                Timer::Total(c) => c,
                            };
                            if mark_this_test_as_soft_failed {
                                mark_this_test_as_soft_failed = false;
                                eprintln!(
                                    "[fail: {} c ≈ {} s]",
                                    cycles,
                                    ((cycles as f64 / (16.78 * 1_000_000.0)) * 100.0).round()
                                        / 100.0
                                );
                            } else {
                                eprintln!(
                                    "[ok: {} c ≈ {} s]",
                                    cycles,
                                    ((cycles as f64 / (16.78 * 1_000_000.0)) * 100.0).round()
                                        / 100.0
                                );
                            }
                        } else {
                            eprintln!("{}", debug_message);
                        }
                    }
                    _ => {}
                }

                if message == "Tests finished successfully" {
                    if mark_tests_as_soft_failed {
                        eprintln!("Tests failed");
                        return Err(anyhow!("Tests failed").into());
                    } else {
                        eprintln!("{}", message);
                        return Ok(());
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArguments::parse();

    let rom = load_rom(args.rom)?;
    let rom = MemoryBacked::new(rom);

    TestRunner::new(rom)?.run()?;

    Ok(())
}

fn load_rom<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<u8>> {
    let mut input_file = File::open(path)?;
    let mut input_file_buffer = Vec::new();

    input_file.read_to_end(&mut input_file_buffer)?;

    let mut elf_buffer = Vec::new();

    if agb_gbafix::write_gba_file(
        &input_file_buffer,
        Default::default(),
        agb_gbafix::PaddingBehaviour::DoNotPad,
        &mut elf_buffer,
    )
    .is_ok()
    {
        Ok(elf_buffer)
    } else {
        Ok(input_file_buffer)
    }
}
