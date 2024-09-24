use std::{error::Error, fs, path::Path};

use agb_xm_core::parse_module;
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::LitStr;
use xmrs::{
    amiga::amiga_module::AmigaModule, module::Module, s3m::s3m_module::S3mModule,
    xm::xmmodule::XmModule,
};

#[proc_macro_error]
#[proc_macro]
pub fn include_xm(args: TokenStream) -> TokenStream {
    agb_xm_core(args, |content| Ok(XmModule::load(content)?.to_module()))
}

#[proc_macro_error]
#[proc_macro]
pub fn include_s3m(args: TokenStream) -> TokenStream {
    agb_xm_core(args, |content| Ok(S3mModule::load(content)?.to_module()))
}

#[proc_macro_error]
#[proc_macro]
pub fn include_mod(args: TokenStream) -> TokenStream {
    agb_xm_core(args, |content| Ok(AmigaModule::load(content)?.to_module()))
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
