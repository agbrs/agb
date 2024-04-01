use std::{
    borrow::Cow,
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

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

    /// Whether the actual code should be shown
    #[arg(short, long)]
    code: bool,
}

struct Location {
    filename: String,
    line: u32,
    col: u32,
}

impl Default for Location {
    fn default() -> Self {
        Self {
            filename: "??".to_string(),
            line: 0,
            col: 0,
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

        print_address(&ctx, i, address, cli.code)?;
    }

    Ok(())
}

fn print_address(
    ctx: &addr2line::Context<impl gimli::Reader>,
    index: usize,
    address: u64,
    include_code: bool,
) -> anyhow::Result<()> {
    let mut frames = ctx.find_frames(address).skip_all_loads()?;

    let mut is_first = true;

    while let Some(frame) = frames.next()? {
        let function_name = if let Some(ref func) = frame.function {
            func.demangle()?.into_owned()
        } else {
            "unknown function".to_string()
        };

        let location = frame
            .location
            .as_ref()
            .map(|location| Location {
                filename: location.file.unwrap_or("??").to_owned(),
                line: location.line.unwrap_or(0),
                col: location.column.unwrap_or(0),
            })
            .unwrap_or_default();

        let is_interesting = is_interesting_function(&function_name, &location.filename);
        let function_name_to_print = if is_interesting {
            function_name.bold()
        } else {
            function_name.normal()
        };

        if is_first {
            print!("{index}:\t{function_name_to_print}");
        } else {
            print!("\t(inlined by) {function_name_to_print}");
        }

        println!(
            " {}:{}",
            prettify_path(&location.filename).green(),
            location.line.to_string().green()
        );

        if include_code && location.line != 0 && is_interesting {
            print_line_of_code(&frame, location)?;
        }

        is_first = false;
    }

    Ok(())
}

fn print_line_of_code(
    frame: &addr2line::Frame<'_, impl gimli::Reader>,
    location: Location,
) -> anyhow::Result<()> {
    let Some(filename) = frame.location.as_ref().and_then(|location| location.file) else {
        return Ok(());
    };

    let Ok(mut file) = File::open(filename) else {
        return Ok(());
    };

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let Some(line_of_code) = content.split('\n').nth(location.line as usize - 1) else {
        eprintln!("File {filename} does not have line {}", location.line);
        return Ok(());
    };

    let trimmed = line_of_code.trim_start();
    let trimmed_len = line_of_code.len() - trimmed.len();
    println!("\t\t{}", trimmed);

    if location.col != 0 {
        println!(
            "\t\t{}{}",
            " ".repeat(location.col as usize - trimmed_len - 1),
            "^".bright_blue()
        );
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

fn is_interesting_function(function_name: &str, path: &str) -> bool {
    if function_name == "rust_begin_unwind" {
        return false; // this is the unwind exception call
    }

    if path.ends_with("panicking.rs") {
        return false; // probably part of rust's internal panic mechanisms
    }

    true
}
