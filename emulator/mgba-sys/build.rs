use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
    process::Command,
};

type MyError = Result<(), Box<dyn Error>>;

fn main() -> MyError {
    have_submodule()?;

    generate_bindings()?;

    compile()?;

    Ok(())
}

fn have_submodule() -> MyError {
    if !Path::new("mgba/src").exists() {
        let _ = Command::new("git")
            .args(["submodule", "update", "--init", "mgba"])
            .status()?;
    }

    Ok(())
}

fn generate_bindings() -> MyError {
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .opaque_type("mTiming")
        .allowlist_type("mCore")
        .allowlist_type("VFile")
        .allowlist_type("VDir")
        .allowlist_type("mLogger")
        .allowlist_type("mLogLevel")
        .allowlist_var("MAP_WRITE")
        .allowlist_var("BYTES_PER_PIXEL")
        .allowlist_function("GBACoreCreate")
        .allowlist_function("mCoreInitConfig")
        .allowlist_function("mLogSetDefaultLogger")
        .allowlist_function("blip_set_rates")
        .allowlist_function("blip_read_samples")
        .allowlist_function("blip_samples_avail")
        .allowlist_function("mCoreConfigLoadDefaults")
        .allowlist_function("mCoreLoadConfig")
        .allowlist_function("mTimingGlobalTime")
        .allowlist_function("mLogCategoryName")
        .generate_cstr(true)
        .derive_default(true)
        .clang_arg("-I./mgba/include")
        .generate()?;

    let out_path = PathBuf::from(env::var("OUT_DIR")?);
    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}

fn compile() -> MyError {
    let dst = cmake::Config::new("mgba")
        .define("LIBMGBA_ONLY", "1")
        .define("M_CORE_GBA", "1")
        .define("M_CORE_GB", "0")
        .define("USE_DEBUGGERS", "1")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=mgba");

    Ok(())
}
