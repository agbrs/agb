use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("tests/text_render");
    let file_data = fs::read(path)?;
    let file_data = file_data.as_slice();

    let elf_file = elf::ElfBytes::<elf::endian::AnyEndian>::minimal_parse(file_data)?;

    let (section_headers, strtab) = elf_file.section_headers_with_strtab()?;
    let section_headers = section_headers.expect("Expected section headers");
    let strtab = strtab.expect("Expected string table");

    let output = fs::File::create("out.gba")?;
    let mut buf_writer = io::BufWriter::new(output);

    for section_header in section_headers.iter() {
        let section_name = strtab.get(section_header.sh_name as usize)?;

        const SHT_NOBITS: u32 = 8;
        const SHT_NULL: u32 = 0;
        const SHF_ALLOC: u64 = 2;

        if (section_header.sh_type == SHT_NOBITS || section_header.sh_type == SHT_NULL)
            || section_header.sh_flags & SHF_ALLOC == 0
        {
            continue;
        }

        println!("{section_name}");

        let (data, compression) = elf_file.section_data(&section_header)?;
        if let Some(compression) = compression {
            panic!("Cannot decompress elf content, but got compression header {compression:?}");
        }

        buf_writer.write_all(data)?;
    }

    buf_writer.flush()?;

    Ok(())
}
