use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out = &PathBuf::from(env::var("OUT_DIR").unwrap());
    let linker_script = if env::var("CARGO_FEATURE_MULTIBOOT").is_ok() {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/gba_mb.ld")).as_slice()
    } else {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/gba.ld")).as_slice()
    };

    fs::write(out.join("gba.ld"), linker_script)?;
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=src/gba.ld");
    println!("cargo:rerun-if-changed=src/gba_mb.ld");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_MULTIBOOT");

    Ok(())
}
