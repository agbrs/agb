use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro]
pub fn include_xm(args: TokenStream) -> TokenStream {
    agb_xm_core::agb_xm_core(args.into()).into()
}
