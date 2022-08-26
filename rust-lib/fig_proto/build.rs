use std::io::Result;

const PROTO_FILES: &[&str] = &[
    "../../proto/fig_common.proto",
    "../../proto/local.proto",
    "../../proto/figterm.proto",
    "../../proto/daemon.proto",
    "../../proto/fig.proto",
    "../../proto/secure.proto",
];

fn main() -> Result<()> {
    for file in PROTO_FILES {
        println!("cargo:rerun-if-changed={file}");
    }

    // TODO: remove this when protoc is newer in all repos
    #[cfg(target_os = "linux")]
    if std::env::var_os("LOCAL_PROTOC").is_none() {
        std::env::set_var("PROTOC", protobuf_src::protoc());
    }

    prost_reflect_build::Builder::new().compile_protos(PROTO_FILES, &["../../proto"])?;

    Ok(())
}
