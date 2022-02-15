use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &["../../proto/local.proto", "../../proto/figterm.proto"],
        &["../../proto"],
    )?;

    Ok(())
}
