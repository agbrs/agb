use agb_fixnum::Num;

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let sine = (0..64).map(|i| (Num::<i32, 8>::new(i) / 64).sin());

    let square = (0..64).map(|i| {
        if i < 32 {
            Num::<i32, 8>::new(-1)
        } else {
            Num::<i32, 8>::new(1)
        }
    });

    let saw = (0..64).map(|i| (Num::<i32, 8>::new(i) - 32) / 32);

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("lookups.rs");

    fs::write(
        &dest_path,
        format!(
            "
            pub(crate) static SINE_LOOKUP: [agb_fixnum::Num<i32, 8>; 64] = [{sine_lookup}];
            pub(crate) static SQUARE_LOOKUP: [agb_fixnum::Num<i32, 8>; 64] = [{square_lookup}];
            pub(crate) static SAW_LOOKUP: [agb_fixnum::Num<i32, 8>; 64] = [{saw_lookup}];
            ",
            sine_lookup = gen_lookup(sine),
            square_lookup = gen_lookup(square),
            saw_lookup = gen_lookup(saw),
        ),
    )
    .unwrap();

    println!("cargo::rerun-if-changed=build.rs");
}

fn gen_lookup(input: impl IntoIterator<Item = Num<i32, 8>>) -> String {
    let output: Vec<_> = input
        .into_iter()
        .map(|v| format!("agb_fixnum::Num::from_raw({})", v.to_raw()))
        .collect();

    output.join(", ")
}
