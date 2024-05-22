use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Read},
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use clap::Parser;
use image::DynamicImage;
use image_generate::generate_image;
use mgba::{LogLevel, Logger, MCore, MemoryBacked, VFile};

mod image_generate;

static LOGGER: Logger = Logger::new(my_logger);

fn my_logger(_category: &str, _level: LogLevel, _s: String) {}

#[derive(Parser)]
struct CliArguments {
    #[arg(long)]
    rom: PathBuf,
    #[arg(long)]
    frames: usize,
    #[arg(long)]
    output: PathBuf,
}

struct ScreenshotGenerator {
    mgba: MCore,
}

impl ScreenshotGenerator {
    fn new<V: VFile>(rom: V) -> Result<Self, Box<dyn Error>> {
        let mut mgba = MCore::new().ok_or(anyhow!("cannot create core"))?;

        mgba::set_global_default_logger(&LOGGER);

        mgba.load_rom(rom);

        Ok(Self { mgba })
    }

    fn run(mut self, frames: usize) -> DynamicImage {
        for _ in 0..frames {
            self.mgba.frame();
        }

        generate_image(self.mgba.video_buffer())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArguments::parse();

    let rom = load_rom(args.rom)?;
    let rom = MemoryBacked::new(rom);

    let image = ScreenshotGenerator::new(rom)?.run(args.frames);

    let mut output = BufWriter::new(
        File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(args.output)?,
    );
    image.write_to(&mut output, image::ImageOutputFormat::Png)?;

    Ok(())
}

fn load_rom<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<u8>> {
    let mut input_file = File::open(path)?;
    let mut input_file_buffer = Vec::new();

    input_file.read_to_end(&mut input_file_buffer)?;

    let mut elf_buffer = Vec::new();

    let inculde_debug_info = false;
    if agb_gbafix::write_gba_file(
        &input_file_buffer,
        Default::default(),
        agb_gbafix::PaddingBehaviour::DoNotPad,
        inculde_debug_info,
        &mut elf_buffer,
    )
    .is_ok()
    {
        Ok(elf_buffer)
    } else {
        Ok(input_file_buffer)
    }
}
