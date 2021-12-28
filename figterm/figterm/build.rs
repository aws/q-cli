use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["../../proto/local.proto"], &["../../proto"])?;
    prost_build::compile_protos(&["figterm.proto"], &["."])?;
    Ok(())
}
