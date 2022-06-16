use std::path;

fn main() {
    let asm = &["crt0.s", "interrupt_handler.s", "src/sound/mixer/mixer.s"];

    println!("cargo:rerun-if-changed=gba.ld");
    println!("cargo:rerun-if-changed=gba_mb.ld");
    println!("cargo:rerun-if-changed=src/asm_include.s");
    println!("cargo:rerun-if-changed=gfx/test_logo.png");

    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");

    for &a in asm.iter() {
        println!("cargo:rerun-if-changed={}", a);
        let filename = path::Path::new(a);
        let filename = filename.with_extension("o");
        let filename = filename
            .file_name()
            .expect("should have filename")
            .to_str()
            .expect("Please make it valid utf-8");

        let out_file_path = format!("{}/{}", out_dir, filename);

        let out = std::process::Command::new("arm-none-eabi-as")
            .arg("-mthumb-interwork")
            .arg("-mcpu=arm7tdmi")
            .arg("-g")
            .args(&["-o", out_file_path.as_str()])
            .arg(a)
            .output()
            .unwrap_or_else(|_| panic!("failed to compile {}", a));

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
        println!("cargo:rustc-link-arg={}", out_file_path);
    }

    println!("cargo:rustc-link-search={}", out_dir);
    // println!("cargo:rustc-link-arg={}/crt0.o", out_dir);
}
