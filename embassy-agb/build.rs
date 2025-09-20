use std::env;
use std::fs;
use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::quote;

fn main() {
    // Set up basic configs for GBA target
    println!("cargo:rustc-check-cfg=cfg(time_driver_timer0)");
    println!("cargo:rustc-check-cfg=cfg(time_driver_timer1)");

    // Determine which timer to use for time driver
    let time_driver = if env::var("CARGO_FEATURE_TIME_DRIVER_TIMER0").is_ok() {
        "timer0"
    } else if env::var("CARGO_FEATURE_TIME_DRIVER_TIMER1").is_ok() {
        "timer1"
    } else {
        ""
    };

    if !time_driver.is_empty() {
        println!("cargo:rustc-cfg=time_driver_{}", time_driver);
    }

    // Generate the peripheral definitions
    let mut g = TokenStream::new();

    // Define GBA peripherals as singletons
    g.extend(quote! {
        /// GBA peripheral singletons
        pub mod peripherals {
            use super::*;

            #[derive(Debug, Copy, Clone)]
            pub struct DISPLAY { _private: () }
            impl DISPLAY {
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            #[derive(Debug, Copy, Clone)]
            pub struct MIXER { _private: () }
            impl MIXER {
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            #[derive(Debug, Copy, Clone)]
            pub struct INPUT { _private: () }
            impl INPUT {
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            #[derive(Debug, Copy, Clone)]
            pub struct TIMER0 { _private: () }
            impl TIMER0 {
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            #[derive(Debug, Copy, Clone)]
            pub struct TIMER1 { _private: () }
            impl TIMER1 {
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            #[derive(Debug, Copy, Clone)]
            pub struct TIMER2 { _private: () }
            impl TIMER2 {
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            #[derive(Debug, Copy, Clone)]
            pub struct TIMER3 { _private: () }
            impl TIMER3 {
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }
        }

        /// GBA Peripherals struct
        #[allow(non_snake_case)]
        pub struct Peripherals {
            pub DISPLAY: peripherals::DISPLAY,
            pub MIXER: peripherals::MIXER,
            pub INPUT: peripherals::INPUT,
            pub TIMER0: peripherals::TIMER0,
            pub TIMER1: peripherals::TIMER1,
            pub TIMER2: peripherals::TIMER2,
            pub TIMER3: peripherals::TIMER3,
        }

        impl Peripherals {
            /// Take the peripherals singleton
            pub fn take() -> Self {
                static mut TAKEN: bool = false;
                critical_section::with(|_| unsafe {
                    if TAKEN {
                        panic!("Peripherals already taken");
                    }
                    TAKEN = true;
                    Self {
                        DISPLAY: peripherals::DISPLAY::steal(),
                        MIXER: peripherals::MIXER::steal(),
                        INPUT: peripherals::INPUT::steal(),
                        TIMER0: peripherals::TIMER0::steal(),
                        TIMER1: peripherals::TIMER1::steal(),
                        TIMER2: peripherals::TIMER2::steal(),
                        TIMER3: peripherals::TIMER3::steal(),
                    }
                })
            }
        }
    });

    // Write the generated code
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_generated.rs");
    fs::write(&out_file, g.to_string()).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
