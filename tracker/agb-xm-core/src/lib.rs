use proc_macro2::TokenStream;
use proc_macro_error::abort;

use quote::quote;

pub fn agb_xm_core(args: TokenStream) -> TokenStream {
    if args.is_empty() {
        abort!(args, "must pass a filename");
    }

    quote! {
        fn hello_world() {}
    }
}
