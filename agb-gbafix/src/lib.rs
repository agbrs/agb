use anyhow::{anyhow, bail, ensure, Result};
use std::io::Write;

pub fn write_gba_file<W: Write>(
    input: &[u8],
    mut header: gbafix::GBAHeader,
    output: &mut W,
) -> Result<()> {
    let elf_file = elf::ElfBytes::<elf::endian::AnyEndian>::minimal_parse(input)?;

    let section_headers = elf_file
        .section_headers()
        .ok_or_else(|| anyhow!("Failed to parse as elf file"))?;

    let mut bytes_written = 0;
    for section_header in section_headers.iter() {
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
            bytes_written += GBA_HEADER_SIZE as u64;
        }

        output.write_all(data)?;
        bytes_written += data.len() as u64;
    }

    if !bytes_written.is_power_of_two() {
        let required_padding = bytes_written.next_power_of_two() - bytes_written;

        for _ in 0..required_padding {
            output.write_all(&[0])?;
        }
    }

    Ok(())
}
