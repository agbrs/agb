use std::{fs::File, io::BufWriter};

use quote::quote;

fn generate_lut_table<F>(f: F) -> [i16; 256]
where
    F: Fn(f64) -> f64,
{
    let mut table = [0; 256];

    for (i, v) in table.iter_mut().enumerate() {
        let conversion = (1 << 8) as f64;

        let x = (i as f64) / conversion;
        let c = f(x);
        let p = c * f64::from(1 << 11);

        *v = p as i16;
    }

    table
}

fn output_lut_table(file: &mut dyn std::io::Write, name: &str, values: &[i16]) {
    let ident = quote::format_ident!("{}", name);

    let s = quote! {
        pub static #ident: &[i16] = &[
            #(#values),*
        ];
    };

    writeln!(file, "{s}").expect("Should be able to write to file");
}

fn main() {
    let build_dir = std::env::var("OUT_DIR").expect("OUT_DIR must be specified");

    let mut file = BufWriter::new(
        File::create(format!("{build_dir}/lut.rs")).expect("Should be able to open file"),
    );

    output_lut_table(
        &mut file,
        "COS",
        &generate_lut_table(|x| (x * std::f64::consts::TAU).cos()),
    );
}
