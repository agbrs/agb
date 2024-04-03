use std::{borrow::Cow, collections::HashMap, io::Cursor, rc::Rc};

use addr2line::{
    gimli,
    object::{self, Object},
};
use anyhow::bail;

pub fn load_dwarf(
    file_content: &[u8],
) -> anyhow::Result<gimli::Dwarf<gimli::EndianRcSlice<gimli::RunTimeEndian>>> {
    if let Ok(object) = object::File::parse(file_content) {
        return load_from_object(&object);
    }

    let last_8_bytes = &file_content[file_content.len() - 8..];
    let len = u32::from_le_bytes(last_8_bytes[0..4].try_into()?) as usize;
    let version = u32::from_le_bytes(last_8_bytes[4..].try_into()?) as usize;

    if version != 1 {
        bail!("Only version 1 of the debug info is supported");
    }

    let compressed_debug_data = &file_content[file_content.len() - len - 8..file_content.len() - 8];

    let decompressing_reader =
        lz4_flex::frame::FrameDecoder::new(Cursor::new(compressed_debug_data));
    let debug_info: HashMap<String, Vec<u8>> = rmp_serde::decode::from_read(decompressing_reader)?;

    let dwarf = gimli::Dwarf::load(|id| {
        let data = debug_info
            .get(id.name())
            .map(|data| Cow::Borrowed(data.as_slice()))
            .unwrap_or(Cow::Borrowed(&[]));

        Result::<_, gimli::Error>::Ok(gimli::EndianRcSlice::new(
            Rc::from(&*data),
            gimli::RunTimeEndian::Little,
        ))
    })?;

    Ok(dwarf)
}

fn load_from_object<'file>(
    object: &object::File<'file, &'file [u8]>,
) -> anyhow::Result<gimli::Dwarf<gimli::EndianRcSlice<gimli::RunTimeEndian>>> {
    let endian = if object.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    fn load_section<'data: 'file, 'file, O, Endian>(
        id: gimli::SectionId,
        file: &'file O,
        endian: Endian,
    ) -> Result<gimli::EndianRcSlice<Endian>, gimli::Error>
    where
        O: object::Object<'data, 'file>,
        Endian: gimli::Endianity,
    {
        use object::ObjectSection;

        let data = file
            .section_by_name(id.name())
            .and_then(|section| section.uncompressed_data().ok())
            .unwrap_or(Cow::Borrowed(&[]));
        Ok(gimli::EndianRcSlice::new(Rc::from(&*data), endian))
    }

    let dwarf = gimli::Dwarf::load(|id| load_section(id, object, endian))?;
    Ok(dwarf)
}
