use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType, Type};

/// Main entry point for embassy-agb async applications
///
/// This macro creates an async main function that runs on the embassy executor.
/// The function must take a single `Spawner` parameter and can be async.
///
/// # Example
///
/// ```rust,no_run
/// #![no_std]
/// #![no_main]
///
/// use embassy_agb::time::Timer;
/// use embassy_executor::Spawner;
///
/// #[embassy_agb::main]
/// async fn main(spawner: Spawner) {
///     let gba = embassy_agb::init(Default::default());
///     
///     // Your async game code here
///     loop {
///         Timer::after_millis(16).await; // 60 FPS
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn main(_args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    // Validate function signature
    if f.sig.inputs.len() != 1 {
        return quote! {
            compile_error!("embassy_agb::main function must take exactly one parameter: Spawner");
        }
        .into();
    }

    // Check return type
    let returns_never =
        matches!(f.sig.output, ReturnType::Type(_, ref ty) if matches!(**ty, Type::Never(_)));

    let fn_name = &f.sig.ident;
    let fn_args = &f.sig.inputs;
    let fn_body = &f.block;
    let fn_attrs = &f.attrs;

    // Extract the spawner parameter name
    let spawner_param = match f.sig.inputs.first() {
        Some(syn::FnArg::Typed(pat_type)) => match &*pat_type.pat {
            syn::Pat::Ident(ident) => &ident.ident,
            _ => panic!("Expected spawner parameter to be a simple identifier"),
        },
        _ => panic!("Expected spawner parameter"),
    };

    let result = if returns_never {
        // Function returns ! - run forever
        quote! {
            #[::embassy_agb::agb::entry]
            fn agb_main(gba: ::embassy_agb::agb::Gba) -> ! {
                // Store the gba instance globally so embassy-agb can access it
                unsafe {
                    ::embassy_agb::_internal::set_agb_instance(gba);
                }

                unsafe fn __make_static<T>(t: &mut T) -> &'static mut T {
                    ::core::mem::transmute(t)
                }

                let mut executor = ::embassy_agb::Executor::new();
                let executor = unsafe { __make_static(&mut executor) };
                executor.run(|spawner| {
                    spawner.spawn(main_task(spawner).unwrap());
                });
            }

            #[::embassy_executor::task]
            async fn main_task(#fn_args) -> ! {
                #(#fn_attrs)*
                async fn #fn_name(#fn_args) -> ! #fn_body
                #fn_name(#spawner_param).await
            }
        }
    } else {
        // Function returns () - run and then loop
        quote! {
            #[::embassy_agb::agb::entry]
            fn agb_main(gba: ::embassy_agb::agb::Gba) -> ! {
                // Store the gba instance globally so embassy-agb can access it
                unsafe {
                    ::embassy_agb::_internal::set_agb_instance(gba);
                }

                unsafe fn __make_static<T>(t: &mut T) -> &'static mut T {
                    ::core::mem::transmute(t)
                }

                let mut executor = ::embassy_agb::Executor::new();
                let executor = unsafe { __make_static(&mut executor) };
                executor.run(|spawner| {
                    spawner.spawn(main_task(spawner).unwrap());
                });
            }

            #[::embassy_executor::task]
            async fn main_task(#fn_args) {
                #(#fn_attrs)*
                async fn #fn_name(#fn_args) #fn_body
                #fn_name(#spawner_param).await;

                // If main returns, just halt forever
                loop {
                    ::embassy_agb::agb::halt();
                }
            }
        }
    };

    result.into()
}

/// Task macro for embassy-agb
///
/// This is a re-export of the embassy_executor::task macro for convenience.
#[proc_macro_attribute]
pub fn task(args: TokenStream, input: TokenStream) -> TokenStream {
    // Just pass through to embassy_executor::task with fully qualified path
    let args = proc_macro2::TokenStream::from(args);
    let input = proc_macro2::TokenStream::from(input);

    quote! {
        #[::embassy_executor::task(#args)]
        #input
    }
    .into()
}
