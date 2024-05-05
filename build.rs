use std::io::Result;

fn main() -> Result<()> {
    let mut config = prost_build::Config::new();
    config.out_dir("src/protos_rs");
    config.compile_protos(&["protos/*.proto"], &[""])?;
    Ok(())
}