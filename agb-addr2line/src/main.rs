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

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();

    let file = fs::read(cli.elf_path)?;
    let object = object::File::parse(file.as_slice())?;

    let ctx = addr2line::Context::new(&object)?;

    if let Some(location) = ctx.find_location(parse_address(&cli.dump)?)? {
        let file = location.file.unwrap_or("unknown file");
        let line = location
            .line
            .map(|line| line.to_string())
            .unwrap_or_else(|| "??".to_owned());

        println!("{file}:{line}");
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
