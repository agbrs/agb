// This macro is based very heavily on the entry one in rust-embedded
use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, ItemFn, Pat, ReturnType, Token, Type, Visibility};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let f: ItemFn = syn::parse(input).expect("#[agb::entry] must be applied to a function");

    // Check that the function signature is correct
    assert!(
        f.sig.constness.is_none()
            && f.vis == Visibility::Inherited
            && f.sig.abi.is_none()
            && f.sig.generics.params.is_empty()
            && f.sig.generics.where_clause.is_none()
            && match f.sig.output {
                ReturnType::Type(_, ref ty) => matches!(**ty, Type::Never(_)),
                _ => false,
            },
        "#[agb::entry] must have signature [unsafe] fn (mut agb::Gba) -> !"
    );

    // Check that the function signature takes 1 argument, agb::Gba
    let arguments: Vec<_> = f.sig.inputs.iter().collect();

    assert_eq!(
        arguments.len(),
        1,
        "#[agb::entry] must have signature [unsafe] fn (mut agb::Gba) -> !, but got {} arguments",
        arguments.len(),
    );

    let (argument_type, (argument_name, is_mutable)) = match arguments[0] {
        FnArg::Typed(pat_type) => (
            pat_type.ty.to_token_stream(),
            match &*pat_type.pat {
                Pat::Ident(ident) => {
                    assert!(
                        ident.attrs.is_empty() && ident.by_ref.is_none() && ident.subpat.is_none(),
                        "#[agb::entry] must have signature [unsafe] fn (mut agb::Gba) -> !"
                    );

                    (ident.ident.clone(), ident.mutability.is_some())
                }
                _ => panic!("Expected first argument to #[agb::entry] to be a basic identifier"),
            },
        ),
        _ => panic!("Expected first argument to #[agb::entry] to not be self"),
    };

    assert!(
        args.to_string() == "",
        "Must pass no args to #[agb::entry] macro"
    );

    let fn_name = hashed_ident(&f);

    let attrs = f.attrs;
    let stmts = f.block.stmts;

    let mutable = if is_mutable {
        Some(Token![mut](Span::call_site()))
    } else {
        None
    };

    assert!(
        argument_type.to_string().ends_with("Gba"),
        "Expected first argument to have type 'Gba'"
    );

    quote!(
        #[cfg(not(test))]
        #[export_name = "main"]
        #(#attrs)*
        pub extern "C" fn #fn_name() -> ! {
            let #mutable #argument_name = unsafe { #argument_type ::new_in_entry() };

            #(#stmts)*
        }

        #[cfg(test)]
        #[export_name = "main"]
        #(#attrs)*
        pub extern "C" fn #fn_name() -> ! {
            let mut #argument_name = unsafe { #argument_type ::new_in_entry() };

            if cfg!(test) {
                agb::test_runner::agb_start_tests(#argument_name, test_main);
            } else {
                #(#stmts)*
            }
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

fn hashed_ident<T: Hash>(f: &T) -> Ident {
    let hash = calculate_hash(f);
    Ident::new(&format!("_agb_main_func_{}", hash), Span::call_site())
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
