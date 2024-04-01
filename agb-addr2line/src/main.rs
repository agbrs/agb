use std::{borrow::Cow, fs, path::PathBuf};

use addr2line::{gimli, object};
use clap::Parser;
use colored::Colorize;

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

    for (i, address) in cli.dump.split('-').enumerate() {
        let mut address = u64::from_str_radix(address, 16)?;
        if address <= 0xFFFF {
            address += 0x0800_0000;
        }

        print_address(&ctx, i, address)?;
    }

    Ok(())
}

fn print_address(
    ctx: &addr2line::Context<impl gimli::Reader>,
    index: usize,
    address: u64,
) -> anyhow::Result<()> {
    let mut frames = ctx.find_frames(address).skip_all_loads()?;

    let mut is_first = true;

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

        if is_first {
            print!("{index}:\t{}", function_name.bold());
        } else {
            print!("\t(inlined by) {function_name}");
        }

        println!(
            " {}:{}",
            prettify_path(&location.filename).green(),
            location.line.to_string().green()
        );

        is_first = false;
    }

    Ok(())
}

fn prettify_path(path: &str) -> Cow<'_, str> {
    if let Some(src_index) = path.rfind("/src/") {
        let crate_name_start = path[0..src_index].rfind('/');
        let crate_name = crate_name_start
            .map(|crate_name_start| &path[crate_name_start + 1..src_index])
            .unwrap_or("<crate>");

        Cow::Owned(format!("<{crate_name}>/{}", &path[src_index + 5..]))
    } else {
        Cow::Borrowed(path)
    }
}
