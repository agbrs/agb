use std::path;

fn find_mgba_library() -> Option<&'static str> {
    const POTENTIAL_LIBRARY_LOCATIONS: &[&str] = &[
        "/usr/lib/libmgba.so.0.9.0",
        "/usr/local/lib/libmgba.so.0.9.0",
    ];

    POTENTIAL_LIBRARY_LOCATIONS
        .iter()
        .find(|file_path| path::Path::new(file_path).exists())
        .copied()
}

fn main() {
    let mgba_library = find_mgba_library().expect("Need mgba 0.9.0 installed");

    cc::Build::new()
        .file("c/test-runner.c")
        .object(mgba_library)
        .include("c/include")
        .compile("test-runner");
}
