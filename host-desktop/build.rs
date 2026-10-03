use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = PathBuf::from("../proto/streamdeck.proto");
    println!("cargo:rerun-if-changed={}", proto_file.display());
    prost_build::compile_protos(&[proto_file], &[PathBuf::from("../proto")])?;
    Ok(())
}
