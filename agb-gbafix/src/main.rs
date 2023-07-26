use anyhow::{anyhow, bail, Result};
use clap::{arg, value_parser};

use std::{
    fs,
    io::{BufWriter, Write},
    path::PathBuf,
};

use agb_gbafix::{write_gba_file, GbaHeader};

fn main() -> Result<()> {
    let matches = clap::Command::new("agb-gbafix")
        .about("Convert elf files directly to a valid GBA ROM")
        .arg(arg!(<INPUT> "Input elf file").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-o --output <OUTPUT> "Set output file, defaults to replacing INPUT's extension to .gba").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-t --title <TITLE> "Set the title. At most 12 bytes. Defaults to truncating the input file name"))
        .arg(arg!(-c --gamecode <GAME_CODE> "Sets the game code, 4 bytes"))
        .arg(arg!(-m --makercode <MAKER_CODE> "Set the maker code, 2 bytes"))
        .arg(arg!(-r --gameversion <VERSION> "Set the version of the game, 0-255").value_parser(value_parser!(u8)))
        .arg(arg!(-p --padding "Ignored for compatibility with gbafix"))
        .get_matches();

    let input = matches.get_one::<PathBuf>("INPUT").unwrap();
    let output = match matches.get_one::<PathBuf>("output") {
        Some(output) => output.clone(),
        None => input.with_extension("gba"),
    };

    let mut header = GbaHeader::default();

    {
        let title = if let Some(title) = matches.get_one::<String>("title") {
            title.clone()
        } else {
            let title = input
                .file_stem()
                .ok_or_else(|| anyhow!("Invalid filename {}", input.to_string_lossy()))?
                .to_string_lossy();
            title.into_owned()
        };

        for (i, &c) in title.as_bytes().iter().enumerate().take(12) {
            header.game_title[i] = c;
        }
    }

    if let Some(maker_code) = matches.get_one::<String>("makercode") {
        let maker_code = maker_code.as_bytes();
        if maker_code.len() > 2 {
            bail!(
                "Maker code must be at most 2 bytes, got {}",
                maker_code.len()
            );
        }

        header.maker_code = [
            *maker_code.first().unwrap_or(&0),
            *maker_code.get(1).unwrap_or(&0),
        ];
    }

    if let Some(game_version) = matches.get_one::<u8>("gameversion") {
        header.software_version = *game_version;
    }

    if let Some(game_code) = matches.get_one::<String>("gamecode") {
        for (i, &c) in game_code.as_bytes().iter().enumerate().take(4) {
            header.game_code[i] = c;
        }
    }

    let mut output = BufWriter::new(fs::File::create(output)?);
    let file_data = fs::read(input)?;

    write_gba_file(file_data.as_slice(), header, &mut output)?;

    output.flush()?;

    Ok(())
}
