use std::{error::Error, fs, path::Path};

use agb_xm_core::parse_module;
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::LitStr;
use xmrs::{module::Module, xm::xmmodule::XmModule};

#[proc_macro_error]
#[proc_macro]
pub fn include_xm(args: TokenStream) -> TokenStream {
    agb_xm_core(args, parse_xm)
}

fn agb_xm_core(
    args: TokenStream,
    load_module: impl Fn(&[u8]) -> Result<Module, Box<dyn Error>>,
) -> TokenStream {
    let input = match syn::parse::<LitStr>(args) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error().into(),
    };

    let filename = input.value();

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let path = Path::new(&root).join(&*filename);

    let include_path = path.to_string_lossy();

    let file_content = match fs::read(&path) {
        Ok(content) => content,
        Err(e) => abort!(input, e),
    };

    let module = match load_module(&file_content) {
        Ok(track) => track,
        Err(e) => abort!(input, e),
    };

    let parsed = parse_module(&module);

    quote! {
        {
            const _: &[u8] = include_bytes!(#include_path);

            #parsed
        }
    }
    .into()
}

fn parse_xm(file_content: &[u8]) -> Result<Module, Box<dyn Error>> {
    Ok(XmModule::load(file_content)?.to_module())
}
