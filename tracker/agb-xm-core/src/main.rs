fn main() -> Result<(), Box<dyn std::error::Error>> {
    let module = agb_xm_core::load_module_from_file(&std::path::Path::new(
        "../agb-tracker/examples/ajoj.xm",
    ))?;
    let output = agb_xm_core::parse_module(&module);

    Ok(())
}
