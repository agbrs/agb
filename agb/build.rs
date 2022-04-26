fn main() {
    println!("cargo:rerun-if-changed=crt0.s");
    println!("cargo:rerun-if-changed=gba_mb.ld");
    println!("cargo:rerun-if-changed=src/sound/mixer/mixer.s");
    println!("cargo:rerun-if-changed=src/asm_include.s");
    println!("cargo:rerun-if-changed=interrupt_handler.s");
    println!("cargo:rerun-if-changed=gfx/test_logo.png");

    let out_file_name = "crt0.o";
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");
    let out_file_path = format!("{}/{}", out_dir, &out_file_name);

    let out = std::process::Command::new("arm-none-eabi-as")
        .arg("-mthumb-interwork")
        .arg("-mcpu=arm7tdmi")
        .arg("-g")
        .args(&["-o", out_file_path.as_str()])
        .arg("crt0.s")
        .output()
        .expect("failed to compile crt0.s");

    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    for warning_line in String::from_utf8_lossy(&out.stderr).split('\n') {
        if !warning_line.is_empty() {
            println!("cargo:warning={}", warning_line);
        }
    }

    println!("cargo:rustc-link-search={}", out_dir);
}
