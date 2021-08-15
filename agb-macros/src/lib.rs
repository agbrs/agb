// This macro is based very heavily on the entry one in rust-embedded
use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::quote;
use rand::Rng;
use syn::{Ident, ItemFn, ReturnType, Type, Visibility};

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let f: ItemFn = syn::parse(input).expect("#[agb::entry] must be applied to a function");

    // Check that the function signature is correct
    assert!(
        f.sig.constness.is_none()
            && f.vis == Visibility::Inherited
            && f.sig.abi.is_none()
            && f.sig.inputs.is_empty()
            && f.sig.generics.params.is_empty()
            && f.sig.generics.where_clause.is_none()
            && match f.sig.output {
                ReturnType::Type(_, ref ty) => matches!(**ty, Type::Never(_)),
                _ => false,
            },
        "#[agb::entry] must have signature [unsafe] fn () -> !"
    );

    assert!(
        args.to_string() == "",
        "Must pass no args to #[agb::entry] macro"
    );

    let fn_name = random_ident();

    let attrs = f.attrs;
    let stmts = f.block.stmts;

    quote!(
        #[export_name = "main"]
        #(#attrs)*
        pub fn #fn_name() -> ! {
            #(#stmts)*
        }
    )
    .into()
}

fn random_ident() -> Ident {
    let mut rng = rand::thread_rng();
    Ident::new(
        &(0..16)
            .map(|i| {
                if i == 0 || rng.gen() {
                    (b'a' + rng.gen::<u8>() % 25) as char
                } else {
                    (b'0' + rng.gen::<u8>() % 10) as char
                }
            })
            .collect::<String>(),
        Span::call_site(),
    )
}
