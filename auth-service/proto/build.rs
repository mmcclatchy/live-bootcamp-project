use std::env;
use std::io::Write;
use std::panic;
use std::path::PathBuf;

use log::{error, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("Log: auth-service/proto/build.rs::main invoked");
    std::io::stderr().flush().unwrap();
    std::io::stdout().flush().unwrap();

    panic::set_hook(Box::new(|panic_info| {
        error!("Panic occurred: {:?}", panic_info);
    }));

    tonic_build::compile_protos("src/auth.proto")?;

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("auth_descriptor.bin"))
        .out_dir(&out_dir) // This ensures the file is generated in the OUT_DIR
        .compile(&["src/auth.proto"], &["src"])?;

    info!("cargo:rerun-if-changed=src/auth.proto");
    info!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
