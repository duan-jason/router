//! Main entry point for CLI command to start server.

fn main() {
    match uhg_custom_appollo_roouter::main() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1)
        }
    }
}
