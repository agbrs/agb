use agb_debug::{Addr2LineContext, addr2line::Context, load_dwarf};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn decode_backtrace(backtrace: &str) -> Result<Vec<u32>, JsError> {
    Ok(agb_debug::gwilym_decode(backtrace)?.collect())
}

#[wasm_bindgen]
pub struct DebugFile {
    dwarf: Addr2LineContext,
}

#[wasm_bindgen(getter_with_clone)]
pub struct AddressInfo {
    pub filename: String,
    pub function_name: String,
    pub line_number: u32,
    pub column: u32,
    pub is_interesting: bool,
    pub is_inline: bool,
}

#[wasm_bindgen]
impl DebugFile {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<DebugFile, JsError> {
        Ok(DebugFile {
            dwarf: Context::from_dwarf(load_dwarf(data)?)?,
        })
    }

    pub fn address_info(&self, address: u32) -> Result<Vec<AddressInfo>, JsError> {
        let info = agb_debug::address_info(&self.dwarf, address.into())?;
        let address_infos = info
            .into_iter()
            .map(|x| AddressInfo {
                filename: x.location.filename,
                line_number: x.location.line,
                column: x.location.col,
                is_interesting: x.is_interesting,
                is_inline: x.is_inline,
                function_name: x.function,
            })
            .collect();

        Ok(address_infos)
    }
}
