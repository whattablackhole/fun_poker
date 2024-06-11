use std::io::Result;
use std::fs;
use prost_build::Config;

fn main() -> Result<()> {
    let mut config = Config::new();
    config.out_dir("src/protos_rs");

    let proto_files: Vec<_> = fs::read_dir("../protos")?
        .filter_map(Result::ok)
        .filter(|entry| {
            if let Some(extension) = entry.path().extension() {
                return extension == "proto";
            }
            false
        })
        .map(|entry| entry.path())
        .collect();

    let proto_files: Vec<_> = proto_files.iter()
        .map(|path| path.to_str().unwrap())
        .collect();

    config.compile_protos(&proto_files, &["../protos"])?;

    Ok(())
}