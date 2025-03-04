use std::{
    borrow::Cow,
    error::Error,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    time::SystemTime,
};

use agb_debug::{AddressInfo, Location, address_info};
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

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();

    let modification_time = fs::metadata(&cli.elf_path)?
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let file = fs::read(&cli.elf_path)?;
    let dwarf = agb_debug::load_dwarf(&file)?;

    let ctx = addr2line::Context::from_dwarf(dwarf)?;

    for (i, address) in agb_debug::gwilym_decode(&cli.dump)?.enumerate() {
        let infos = address_info(&ctx, address.into())?;
        for info in infos {
            print_address_info(&info, i, modification_time)?;
        }
    }

    Ok(())
}

fn print_address_info(
    info: &AddressInfo,
    index: usize,
    elf_modification_time: SystemTime,
) -> Result<(), Box<dyn Error>> {
    let function_name_to_print = &info.function;

    if !info.is_inline {
        print!("{index}:\t{function_name_to_print}");
    } else {
        print!("\t(inlined into) {function_name_to_print}");
    }

    println!(
        " {}:{}",
        prettify_path(&info.location.filename).green(),
        info.location.line.to_string().green()
    );

    if info.location.line != 0 && info.is_interesting {
        print_line_of_code(&info.location, elf_modification_time)?;
    }

    Ok(())
}

fn print_line_of_code(
    location: &Location,
    elf_modification_time: SystemTime,
) -> Result<(), Box<dyn Error>> {
    let filename = &location.filename;
    let Ok(mut file) = File::open(filename) else {
        return Ok(());
    };

    let modification_time = fs::metadata(filename)?
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH);

    if modification_time > elf_modification_time {
        eprintln!(
            "Warning: File {filename} modified more recently than the binary, line info may be incorrect"
        );
    }

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
