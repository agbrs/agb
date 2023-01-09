use std::path;

fn main() {
    let asm = &[
        "src/crt0.s",
        "src/interrupt_handler.s",
        "src/sound/mixer/mixer.s",
        "src/agbabi/memset.s",
        "src/agbabi/memcpy.s",
        "src/save/asm_routines.s",
        "src/sound/tracker/mm_effect.s",
        "src/sound/tracker/mm_main_gba.s",
        "src/sound/tracker/mm_main.s",
        "src/sound/tracker/mm_mas_arm.s",
        "src/sound/tracker/mm_mas.s",
        "src/sound/tracker/mm_mixer_gba.s",
    ];

    println!("cargo:rerun-if-changed=gba.ld");
    println!("cargo:rerun-if-changed=gba_mb.ld");
    println!("cargo:rerun-if-changed=src/asm_include.s");
    println!("cargo:rerun-if-changed=src/agbabi/macros.inc");
    println!("cargo:rerun-if-changed=gfx/test_logo.png");

    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");
    let mut o_files = vec![];

    for &a in asm.iter() {
        println!("cargo:rerun-if-changed={a}");
        let filename = path::Path::new(a);
        let filename = filename.with_extension("o");
        let filename = filename
            .file_name()
            .expect("should have filename")
            .to_str()
            .expect("Please make it valid utf-8");

        let out_file_path = format!("{out_dir}/{filename}");

        let out = std::process::Command::new("arm-none-eabi-as")
            .arg("-mthumb-interwork")
            .arg("-mcpu=arm7tdmi")
            .arg("-g")
            .args(["-o", out_file_path.as_str()])
            .arg(a)
            .output()
            .unwrap_or_else(|_| panic!("failed to compile {a}"));

        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );

        for warning_line in String::from_utf8_lossy(&out.stderr).split('\n') {
            if !warning_line.is_empty() {
                println!("cargo:warning={warning_line}");
            }
        }

        o_files.push(out_file_path);
    }

    let archive = format!("{out_dir}/agb.a");
    let _ = std::fs::remove_file(&archive);
    let ar_out = std::process::Command::new("arm-none-eabi-ar")
        .arg("-crs")
        .arg(&archive)
        .args(&o_files)
        .output()
        .expect("Failed to create static library");

    assert!(
        ar_out.status.success(),
        "{}",
        String::from_utf8_lossy(&ar_out.stderr)
    );

    println!("cargo:rustc-link-search={out_dir}");
}
