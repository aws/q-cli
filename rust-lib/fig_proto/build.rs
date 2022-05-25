use std::io::Result;

const PROTO_FILES: &[&str] = &[
    "../../proto/local.proto",
    "../../proto/figterm.proto",
    "../../proto/daemon.proto",
    "../../proto/fig_common.proto",
    "../../proto/fig.proto",
];

fn main() -> Result<()> {
    for file in PROTO_FILES {
        println!("cargo:rerun-if-changed={}", file);
    }

    prost_reflect_build::Builder::new().compile_protos(PROTO_FILES, &["../../proto"])?;

    Ok(())
}
