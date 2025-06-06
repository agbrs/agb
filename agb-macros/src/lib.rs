// This macro is based very heavily on the entry one in rust-embedded
use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{FnArg, Ident, ItemFn, Pat, ReturnType, Token, Type, Visibility};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn token_stream_with_string_error(mut tokens: TokenStream, error_str: &str) -> TokenStream {
    tokens.extend(TokenStream::from(quote! { compile_error!(#error_str); }));
    tokens
}

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let f: ItemFn = match syn::parse(input.clone()) {
        Ok(it) => it,
        Err(_) => return input,
    };

    // Check that the function signature is correct
    if !(f.sig.constness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && match f.sig.output {
            ReturnType::Type(_, ref ty) => matches!(**ty, Type::Never(_)),
            _ => false,
        })
    {
        return token_stream_with_string_error(
            input,
            "#[agb::entry] must have signature [unsafe] fn (mut agb::Gba) -> !",
        );
    }

    // Check that the function signature takes 1 argument, agb::Gba
    let arguments: Vec<_> = f.sig.inputs.iter().collect();

    if arguments.len() != 1 {
        return token_stream_with_string_error(
            input,
            &format!(
                "#[agb::entry] must have signature [unsafe] fn (mut agb::Gba) -> !, but got {} arguments",
                arguments.len()
            ),
        );
    }

    let (argument_type, (argument_name, is_mutable)) = match arguments[0] {
        FnArg::Typed(pat_type) => (
            pat_type.ty.to_token_stream(),
            match &*pat_type.pat {
                Pat::Ident(ident) => {
                    if !(ident.attrs.is_empty() && ident.by_ref.is_none() && ident.subpat.is_none())
                    {
                        return token_stream_with_string_error(
                            input,
                            "#[agb::entry] must have signature [unsafe] fn (mut agb::Gba) -> !",
                        );
                    }

                    (ident.ident.clone(), ident.mutability.is_some())
                }
                _ => {
                    return token_stream_with_string_error(
                        input,
                        "Expected first argument to #[agb::entry] to be a basic identifier",
                    );
                }
            },
        ),
        _ => {
            return token_stream_with_string_error(
                input,
                "Expected first argument to #[agb::entry] to not be self",
            );
        }
    };

    if args.to_string() != "" {
        return token_stream_with_string_error(input, "Must pass no args to #[agb::entry] macro");
    }

    let fn_name = hashed_ident(&f);

    let attrs = f.attrs;
    let stmts = f.block.stmts;

    let mutable = if is_mutable {
        Some(Token![mut](Span::call_site()))
    } else {
        None
    };

    if !argument_type.to_string().ends_with("Gba") {
        return token_stream_with_string_error(input, "Expected first argument to have type 'Gba'");
    }

    quote!(
        #[unsafe(export_name = "main")]
        #[doc(hidden)]
        #(#attrs)*
        pub extern "C" fn #fn_name() -> ! {
            let gba = unsafe { #argument_type ::new_in_entry() };

            fn inner(#mutable #argument_name: #argument_type) -> ! {
                #(#stmts)*
            }

            #[cfg(test)]
            if cfg!(test) {
                agb::test_runner::agb_start_tests(gba, test_main);
            } else {
                inner(gba);
            }

            #[cfg(not(test))]
            inner(gba);
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn doctest(args: TokenStream, input: TokenStream) -> TokenStream {
    let f: ItemFn = match syn::parse(input.clone()) {
        Ok(it) => it,
        Err(_) => return input,
    };

    let fn_name = f.sig.ident.clone();

    entry(
        args,
        quote! {
            fn doctest_main(gba: agb::Gba) -> ! {
                #f

                #fn_name(gba);
                agb::println!("Tests finished successfully");

                loop {
                    agb::halt();
                }
            }
        }
        .into(),
    )
}

fn hashed_ident<T: Hash>(f: &T) -> Ident {
    let hash = calculate_hash(f);
    Ident::new(&format!("_agb_main_func_{hash}"), Span::call_site())
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
