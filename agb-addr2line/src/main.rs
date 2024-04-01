use std::{fs, path::PathBuf, str::FromStr};

use addr2line::object;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The filename of the elf file
    elf_path: PathBuf,

    /// The output of agb's dump
    dump: String,
}

struct Location {
    filename: String,
    line: u32,
}

impl Default for Location {
    fn default() -> Self {
        Self {
            filename: "??".to_string(),
            line: 0,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();

    let file = fs::read(cli.elf_path)?;
    let object = object::File::parse(file.as_slice())?;

    let ctx = addr2line::Context::new(&object)?;

    let mut frames = ctx
        .find_frames(parse_address(&cli.dump)?)
        .skip_all_loads()?;

    while let Some(frame) = frames.next()? {
        let function_name = if let Some(func) = frame.function {
            func.demangle()?.into_owned()
        } else {
            "unknown function".to_string()
        };

        let location = frame
            .location
            .map(|location| Location {
                filename: location.file.unwrap_or("??").to_owned(),
                line: location.line.unwrap_or(0),
            })
            .unwrap_or_default();

        println!(
            "{}:{} ({})",
            location.filename, location.line, function_name
        );
    }

    Ok(())
}

fn parse_address(input: &str) -> Result<u64, <u64 as FromStr>::Err> {
    if let Some(input) = input.strip_prefix("0x") {
        u64::from_str_radix(input, 16)
    } else {
        input.parse()
    }
}
