use std::{borrow::Cow, collections::HashMap, io::Cursor, rc::Rc};

use addr2line::{
    gimli,
    object::{self, Object},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadDwarfError {
    #[error("Gba file is empty")]
    GbaFileEmpty,
    #[error("Failed to load debug information from ROM file, it might not have been included?")]
    NoDebugInformation,
    #[error("Failed to load debug information: {0}")]
    DeserializationError(#[from] rmp_serde::decode::Error),
    #[error(transparent)]
    GimliError(#[from] gimli::Error),
}

pub type GimliDwarf = gimli::Dwarf<gimli::EndianRcSlice<gimli::RunTimeEndian>>;

pub fn load_dwarf(file_content: &[u8]) -> Result<GimliDwarf, LoadDwarfError> {
    if let Ok(object) = object::File::parse(file_content) {
        return Ok(load_from_object(&object)?);
    }

    // the file might have been padded, so ensure we skip any padding before continuing
    let last_non_zero_byte = file_content
        .iter()
        .rposition(|&b| b != 0)
        .ok_or_else(|| LoadDwarfError::GbaFileEmpty)?;

    let file_content = &file_content[..last_non_zero_byte + 1];

    let last_8_bytes = &file_content[file_content.len() - 8..];
    let len = u32::from_le_bytes(
        last_8_bytes[0..4]
            .try_into()
            .or(Err(LoadDwarfError::NoDebugInformation))?,
    ) as usize;
    let version = &last_8_bytes[4..];

    if version != b"agb1" {
        return Err(LoadDwarfError::NoDebugInformation);
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
) -> Result<GimliDwarf, gimli::Error> {
    let endian = if object.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    fn load_section<'data, Endian>(
        id: gimli::SectionId,
        file: &impl object::Object<'data>,
        endian: Endian,
    ) -> Result<gimli::EndianRcSlice<Endian>, gimli::Error>
    where
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
