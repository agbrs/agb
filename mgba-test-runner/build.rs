use std::{env, path::PathBuf};

const MGBA_VERSION: &str = "0.9.1";

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mgba_directory = out_path.join(format!("mgba-{}", MGBA_VERSION));
    std::process::Command::new("bash")
        .arg("build-mgba.sh")
        .arg(MGBA_VERSION)
        .arg(&out_path)
        .output()
        .expect("should be able to build mgba");
    println!("cargo:rustc-link-search={}", out_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=static={}", "mgba-cycle");
    println!("cargo:rustc-link-lib=elf");

    cc::Build::new()
        .file("c/test-runner.c")
        .include(&mgba_directory.join("include"))
        .static_flag(true)
        .compile("test-runner");

    let bindings = bindgen::Builder::default()
        .header("c/test-runner.h")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(&out_path.join("runner-bindings.rs"))
        .expect("Couldn't write bindings!");
}
