mod gwilym_encoding;
mod load_dwarf;

use addr2line::gimli::{self, EndianReader};
pub use gwilym_encoding::{gwilym_decode, GwilymDecodeError};
pub use load_dwarf::{load_dwarf, GimliDwarf, LoadDwarfError};
use thiserror::Error;

pub use addr2line;

pub struct AddressInfo {
    pub location: Location,
    pub is_interesting: bool,
    pub is_inline: bool,
    pub function: String,
}

#[derive(Debug, Error)]
pub enum AddressInfoError {
    #[error(transparent)]
    Gimli(#[from] gimli::Error),
}

pub struct Location {
    pub filename: String,
    pub line: u32,
    pub col: u32,
}

pub type Addr2LineContext = addr2line::Context<gimli::EndianRcSlice<gimli::RunTimeEndian>>;

impl Default for Location {
    fn default() -> Self {
        Self {
            filename: "??".to_string(),
            line: 0,
            col: 0,
        }
    }
}

pub fn address_info(
    ctx: &Addr2LineContext,
    address: u64,
) -> Result<Vec<AddressInfo>, AddressInfoError> {
    let mut frames = ctx.find_frames(address).skip_all_loads()?;

    let mut is_first = true;

    let mut infos = Vec::new();

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

        infos.push(AddressInfo {
            location,
            is_interesting,
            is_inline: is_first,
            function: function_name,
        });
        is_first = false;
    }

    Ok(infos)
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
