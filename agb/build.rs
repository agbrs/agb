fn main() {
    println!("cargo:rerun-if-changed=gba.ld");
    println!("cargo:rerun-if-changed=gba_mb.ld");
    println!("cargo:rerun-if-changed=build.rs");
}
