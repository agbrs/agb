use anyhow::{anyhow, bail, ensure, Result};
use clap::{arg, value_parser};

use std::{
    fs,
    io::{BufWriter, Write},
    path::PathBuf,
};

fn main() -> Result<()> {
    let matches = clap::Command::new("agb-gbafix")
        .about("Convert elf files directly to a valid GBA ROM")
        .arg(arg!(<INPUT> "Input elf file").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-o --output <OUTPUT> "Set output file, defaults to replacing INPUT's extension to .gba").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-t --title <TITLE> "Set the title. At most 12 bytes. Defaults to truncating the input file name"))
        .arg(arg!(-c --gamecode <GAME_CODE> "Sets the game code, 4 bytes"))
        .arg(arg!(-m --makercode <MAKER_CODE> "Set the maker code, 0-65535").value_parser(value_parser!(u16)))
        .arg(arg!(-r --gameversion <VERSION> "Set the version of the game, 0-255").value_parser(value_parser!(u8)))
        .get_matches();

    let input = matches.get_one::<PathBuf>("INPUT").unwrap();
    let output = match matches.get_one::<PathBuf>("output") {
        Some(output) => output.clone(),
        None => input.with_extension("gba"),
    };

    let mut header = gbafix::GBAHeader::default();

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
            header.title[i] = c;
        }
    }

    if let Some(maker_code) = matches.get_one::<u16>("makercode") {
        header.maker_code = maker_code.to_le_bytes();
    }

    if let Some(game_version) = matches.get_one::<u8>("gameversion") {
        header.version = *game_version;
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

fn write_gba_file<W: Write>(
    input: &[u8],
    mut header: gbafix::GBAHeader,
    output: &mut W,
) -> Result<()> {
    let elf_file = elf::ElfBytes::<elf::endian::AnyEndian>::minimal_parse(input)?;

    let section_headers = elf_file
        .section_headers()
        .ok_or_else(|| anyhow!("Failed to parse as elf file"))?;

    const GBA_START_ADDRESS: u64 = 0x8000000;
    let mut address = GBA_START_ADDRESS;

    for section_header in section_headers.iter() {
        const SHT_NOBITS: u32 = 8;
        const SHT_NULL: u32 = 0;
        const SHF_ALLOC: u64 = 2;

        if (section_header.sh_type == SHT_NOBITS || section_header.sh_type == SHT_NULL)
            || section_header.sh_flags & SHF_ALLOC == 0
        {
            continue;
        }

        if address < section_header.sh_addr {
            for _ in address..section_header.sh_addr {
                output.write_all(&[0])?;
            }

            address = section_header.sh_addr;
        }

        let (mut data, compression) = elf_file.section_data(&section_header)?;
        if let Some(compression) = compression {
            bail!("Cannot decompress elf content, but got compression header {compression:?}");
        }

        if address == GBA_START_ADDRESS {
            const GBA_HEADER_SIZE: usize = 192;

            ensure!(
                data.len() > GBA_HEADER_SIZE,
                "first section must be at least as big as the gba header"
            );

            header.start_code = data[0..4].try_into().unwrap();
            header.update_checksum();

            let header_bytes = bytemuck::bytes_of(&header);
            output.write_all(header_bytes)?;

            data = &data[GBA_HEADER_SIZE..];
            address += GBA_HEADER_SIZE as u64;
        }

        output.write_all(data)?;
        address += data.len() as u64;
    }

    let length = address - GBA_START_ADDRESS;

    if !length.is_power_of_two() {
        let required_padding = length.next_power_of_two() - length;

        for _ in 0..required_padding {
            output.write_all(&[0])?;
        }
    }

    Ok(())
}
