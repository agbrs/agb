use anyhow::{anyhow, bail, ensure, Result};
use std::{collections::HashMap, io::Write};

const GBA_HEADER_SIZE: usize = 192;

const NINTENDO_LOGO: &[u8] = &[
    0x24, 0xFF, 0xAE, 0x51, 0x69, 0x9A, 0xA2, 0x21, 0x3D, 0x84, 0x82, 0x0A, 0x84, 0xE4, 0x09, 0xAD,
    0x11, 0x24, 0x8B, 0x98, 0xC0, 0x81, 0x7F, 0x21, 0xA3, 0x52, 0xBE, 0x19, 0x93, 0x09, 0xCE, 0x20,
    0x10, 0x46, 0x4A, 0x4A, 0xF8, 0x27, 0x31, 0xEC, 0x58, 0xC7, 0xE8, 0x33, 0x82, 0xE3, 0xCE, 0xBF,
    0x85, 0xF4, 0xDF, 0x94, 0xCE, 0x4B, 0x09, 0xC1, 0x94, 0x56, 0x8A, 0xC0, 0x13, 0x72, 0xA7, 0xFC,
    0x9F, 0x84, 0x4D, 0x73, 0xA3, 0xCA, 0x9A, 0x61, 0x58, 0x97, 0xA3, 0x27, 0xFC, 0x03, 0x98, 0x76,
    0x23, 0x1D, 0xC7, 0x61, 0x03, 0x04, 0xAE, 0x56, 0xBF, 0x38, 0x84, 0x00, 0x40, 0xA7, 0x0E, 0xFD,
    0xFF, 0x52, 0xFE, 0x03, 0x6F, 0x95, 0x30, 0xF1, 0x97, 0xFB, 0xC0, 0x85, 0x60, 0xD6, 0x80, 0x25,
    0xA9, 0x63, 0xBE, 0x03, 0x01, 0x4E, 0x38, 0xE2, 0xF9, 0xA2, 0x34, 0xFF, 0xBB, 0x3E, 0x03, 0x44,
    0x78, 0x00, 0x90, 0xCB, 0x88, 0x11, 0x3A, 0x94, 0x65, 0xC0, 0x7C, 0x63, 0x87, 0xF0, 0x3C, 0xAF,
    0xD6, 0x25, 0xE4, 0x8B, 0x38, 0x0A, 0xAC, 0x72, 0x21, 0xD4, 0xF8, 0x07,
];

#[derive(Debug, Default)]
pub struct GbaHeader {
    pub start_code: [u8; 4],
    pub game_title: [u8; 12],
    pub game_code: [u8; 4],
    pub maker_code: [u8; 2],
    pub software_version: u8,
}

impl GbaHeader {
    fn produce_header(&self) -> Vec<u8> {
        let mut header_result = vec![];

        header_result.extend_from_slice(&self.start_code);
        header_result.extend_from_slice(NINTENDO_LOGO);
        header_result.extend_from_slice(&self.game_title);
        header_result.extend_from_slice(&self.game_code);
        header_result.extend_from_slice(&self.maker_code);
        header_result.push(0x96); // must be 96
        header_result.push(0); // main unit code (0 for current GBA models)
        header_result.push(0); // device type, usually 0
        header_result.extend_from_slice(&[0; 7]); // reserved area, should be zero filled
        header_result.push(self.software_version);

        let checksum = Self::calculate_checksum(&header_result);
        header_result.push(checksum); // checksum
        header_result.extend_from_slice(&[0; 2]); // reserved area, should be zero filled

        assert_eq!(header_result.len(), GBA_HEADER_SIZE);

        header_result
    }

    fn calculate_checksum(header: &[u8]) -> u8 {
        let mut chk = 0u8;
        for value in header.iter().take(0xBC).skip(0xA0) {
            chk = chk.wrapping_sub(*value);
        }

        chk = chk.wrapping_sub(0x19);
        chk
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PaddingBehaviour {
    Pad,
    #[default]
    DoNotPad,
}

pub fn write_gba_file<W: Write>(
    input: &[u8],
    mut header: GbaHeader,
    padding_behaviour: PaddingBehaviour,
    include_debug: bool,
    output: &mut W,
) -> Result<()> {
    let elf_file = elf::ElfBytes::<elf::endian::AnyEndian>::minimal_parse(input)?;

    let section_headers = elf_file
        .section_headers()
        .ok_or_else(|| anyhow!("Failed to parse as elf file"))?;

    let mut bytes_written = 0;
    for section_header in section_headers {
        const SHT_NOBITS: u32 = 8;
        const SHT_NULL: u32 = 0;
        const SHF_ALLOC: u64 = 2;

        if (section_header.sh_type == SHT_NOBITS || section_header.sh_type == SHT_NULL)
            || section_header.sh_flags & SHF_ALLOC == 0
        {
            continue;
        }

        let align = bytes_written % section_header.sh_addralign;
        if align != 0 {
            for _ in 0..(section_header.sh_addralign - align) {
                output.write_all(&[0])?;
                bytes_written += 1;
            }
        }

        let (mut data, compression) = elf_file.section_data(&section_header)?;
        if let Some(compression) = compression {
            bail!("Cannot decompress elf content, but got compression header {compression:?}");
        }

        if bytes_written == 0 {
            ensure!(
                data.len() > GBA_HEADER_SIZE,
                "first section must be at least as big as the gba header"
            );

            header.start_code = data[0..4].try_into().unwrap();

            let header_bytes = header.produce_header();
            output.write_all(&header_bytes)?;

            data = &data[GBA_HEADER_SIZE..];
            bytes_written += GBA_HEADER_SIZE as u64;
        }

        output.write_all(data)?;
        bytes_written += data.len() as u64;
    }

    if include_debug {
        bytes_written += write_debug(&elf_file, output)?;
    }

    if !bytes_written.is_power_of_two() && padding_behaviour == PaddingBehaviour::Pad {
        let required_padding = bytes_written.next_power_of_two() - bytes_written;

        for _ in 0..required_padding {
            output.write_all(&[0])?;
        }
    }

    Ok(())
}

fn write_debug<W: Write>(
    elf_file: &elf::ElfBytes<'_, elf::endian::AnyEndian>,
    output: &mut W,
) -> Result<u64> {
    let (Some(section_headers), Some(string_table)) = elf_file.section_headers_with_strtab()?
    else {
        bail!("Could not find either the section headers or the string table");
    };

    let mut debug_sections = HashMap::new();

    for section_header in section_headers {
        let section_name = string_table.get(section_header.sh_name as usize)?;
        if !section_name.starts_with(".debug") {
            continue;
        }

        let (data, compression) = elf_file.section_data(&section_header)?;
        if compression.is_some() {
            bail!("Do not support compression in elf files, section {section_name} was compressed");
        }

        debug_sections.insert(section_name.to_owned(), data);
    }

    let mut debug_data = vec![];
    {
        let mut compressed_writer = lz4_flex::frame::FrameEncoder::new(&mut debug_data);
        rmp_serde::encode::write(&mut compressed_writer, &debug_sections)?;
        compressed_writer.flush()?;
    }

    output.write_all(&debug_data)?;
    output.write_all(&(debug_data.len() as u32).to_le_bytes())?;
    output.write_all(&(1u32).to_le_bytes())?;

    Ok(debug_data.len() as u64 + 4)
}
