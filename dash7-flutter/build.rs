use lib_flutter_rust_bridge_codegen::codegen::{self, Config};

fn main() -> anyhow::Result<()> {
    // Uncomment the line below, if you only want to generate bindings on api directory change.
    //
    // NOTE: This accelerates the build process, but you will need to manually trigger binding
    // generation whenever there are changes to definitions outside of the api directory that it
    // depends on.
    //
    println!("cargo:rerun-if-changed=src/api");
    println!("cargo:rerun-if-changed=flutter_rust_bridge.yaml");
    println!("cargo:rerun-if-changed=pubspec.yaml");

    // If you want to see logs
    // Alternatively, use `cargo build -vvv` (instead of `cargo build`) to see logs on screen
    // configure_opinionated_logging("./logs/", true)?;

    // Execute code generator with auto-detected config
    codegen::generate(
        Config::from_config_file("flutter_rust_bridge.yaml")?.unwrap(),
        Default::default(),
    )
}
