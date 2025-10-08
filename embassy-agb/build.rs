use std::env;
use std::fs;
use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::quote;

fn main() {
    // Set up basic configs for GBA target
    println!("cargo:rustc-check-cfg=cfg(time_driver_timer0)");
    println!("cargo:rustc-check-cfg=cfg(time_driver_timer1)");
    println!("cargo:rustc-check-cfg=cfg(time_driver_timer2)");
    println!("cargo:rustc-check-cfg=cfg(time_driver_timer3)");

    // Determine which timer to use for time driver
    let time_driver = if env::var("CARGO_FEATURE_TIME_DRIVER_TIMER0").is_ok() {
        "timer0"
    } else if env::var("CARGO_FEATURE_TIME_DRIVER_TIMER1").is_ok() {
        "timer1"
    } else if env::var("CARGO_FEATURE_TIME_DRIVER_TIMER2").is_ok() {
        "timer2"
    } else if env::var("CARGO_FEATURE_TIME_DRIVER_TIMER3").is_ok() {
        "timer3"
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
        #[allow(unused_imports)]
        pub mod peripherals {
            use super::*;

            /// Display peripheral singleton
            #[derive(Debug, Copy, Clone)]
            pub struct DISPLAY { _private: () }
            impl DISPLAY {
                /// # Safety
                /// This function should only be called once to obtain the peripheral singleton
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            /// Mixer peripheral singleton
            #[derive(Debug, Copy, Clone)]
            pub struct MIXER { _private: () }
            impl MIXER {
                /// # Safety
                /// This function should only be called once to obtain the peripheral singleton
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            /// Input peripheral singleton
            #[derive(Debug, Copy, Clone)]
            pub struct INPUT { _private: () }
            impl INPUT {
                /// # Safety
                /// This function should only be called once to obtain the peripheral singleton
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            /// Timer0 peripheral singleton
            #[derive(Debug, Copy, Clone)]
            pub struct TIMER0 { _private: () }
            impl TIMER0 {
                /// # Safety
                /// This function should only be called once to obtain the peripheral singleton
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            /// Timer1 peripheral singleton
            #[derive(Debug, Copy, Clone)]
            pub struct TIMER1 { _private: () }
            impl TIMER1 {
                /// # Safety
                /// This function should only be called once to obtain the peripheral singleton
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            /// Timer2 peripheral singleton
            #[derive(Debug, Copy, Clone)]
            pub struct TIMER2 { _private: () }
            impl TIMER2 {
                /// # Safety
                /// This function should only be called once to obtain the peripheral singleton
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }

            /// Timer3 peripheral singleton
            #[derive(Debug, Copy, Clone)]
            pub struct TIMER3 { _private: () }
            impl TIMER3 {
                /// # Safety
                /// This function should only be called once to obtain the peripheral singleton
                pub unsafe fn steal() -> Self { Self { _private: () } }
            }
        }

        /// GBA Peripherals struct
        #[allow(non_snake_case)]
        pub struct Peripherals {
            /// Display peripheral
            pub DISPLAY: peripherals::DISPLAY,
            /// Mixer peripheral
            pub MIXER: peripherals::MIXER,
            /// Input peripheral
            pub INPUT: peripherals::INPUT,
            /// Timer0 peripheral
            pub TIMER0: peripherals::TIMER0,
            /// Timer1 peripheral
            pub TIMER1: peripherals::TIMER1,
            /// Timer2 peripheral
            pub TIMER2: peripherals::TIMER2,
            /// Timer3 peripheral
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
