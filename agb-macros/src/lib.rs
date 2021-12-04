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

#[proc_macro]
pub fn num(input: TokenStream) -> TokenStream {
    let f = syn::parse_macro_input!(input as syn::LitFloat);
    let v: f64 = f.base10_parse().expect("The number should be parsable");

    let integer = v.trunc();
    let fractional = v.fract() * (1_u64 << 30) as f64;

    let integer = integer as i32;
    let fractional = fractional as i32;
    quote!((#integer, #fractional)).into()
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
