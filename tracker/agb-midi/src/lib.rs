use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro]
pub fn include_xm(args: TokenStream) -> TokenStream {
    agb_midi_core::agb_midi_core(args.into()).into()
}
