use agb_image_converter::{convert_image, Colour, ImageConverterConfig, TileSize};

fn main() {
    println!("cargo:rerun-if-changed=crt0.s");
    println!("cargo:rerun-if-changed=gba_mb.ld");
    println!("cargo:rerun-if-changed=interrupt_simple.s");
    println!("cargo:rerun-if-changed=src/sound/mixer/mixer.s");
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

    if !out.status.success() {
        panic!("{}", String::from_utf8_lossy(&out.stderr));
    }

    println!("cargo:rustc-link-search={}", out_dir);

    convert_image(
        ImageConverterConfig::builder()
            .transparent_colour(Colour::from_rgb(1, 1, 1))
            .tile_size(TileSize::Tile8)
            .input_image("gfx/test_logo.png".into())
            .output_file(format!("{}/test_logo.rs", out_dir).into())
            .crate_prefix("crate".to_owned())
            .build(),
    );
}
