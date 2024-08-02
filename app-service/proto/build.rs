use log::{error, info};
use std::{env, fs, panic, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    panic::set_hook(Box::new(|panic_info| {
        error!("Panic occurred: {:?}", panic_info);
    }));

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = PathBuf::from("src").join("generated");

    // Create the generated directory if it doesn't exist
    fs::create_dir_all(&dest_path)?;

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("auth_descriptor.bin"))
        .out_dir(&dest_path)
        .compile(&["src/auth.proto"], &["src"])?;

    // Create a mod.rs file in the generated directory
    fs::write(dest_path.join("mod.rs"), "pub mod auth;\n")?;

    info!("cargo:rerun-if-changed=src/auth.proto");
    info!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
