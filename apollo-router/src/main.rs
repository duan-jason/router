fn main() {
    // JASON customization: apollo_router --> uhg_custom_appollo_roouter
    match uhg_custom_appollo_roouter::main() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1)
        }
    }
}
