use std::io::Result;
use std::path::PathBuf;

enum Version {
    V1([u32; 3]),
    V2([u32; 2]),
}

/// Try to find the version of protoc installed on the system.
fn protoc_version() -> Option<Version> {
    let output = std::process::Command::new("protoc").arg("--version").output().ok()?;
    let version = String::from_utf8(output.stdout).ok()?;
    eprintln!("protoc version: {version:?}");

    let version = version.trim();
    eprintln!("version: {version:?}");
    let version = version.split(' ').last().expect("No version");
    let version = version.split('.').collect::<Vec<_>>();
    let version = version
        .iter()
        .map(|s| s.parse::<u32>().ok())
        .collect::<Option<Vec<_>>>()?;
    match version.len() {
        3 => Some(Version::V1([version[0], version[1], version[2]])),
        2 => Some(Version::V2([version[0], version[1]])),
        _ => None,
    }
}

fn main() -> Result<()> {
    let proto_files = std::fs::read_dir("../../proto")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter(|entry| entry.path().extension().map(|ext| ext == "proto").unwrap_or(false))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    for file in &proto_files {
        println!("cargo:rerun-if-changed={}", file.display());
    }

    // --experimental_allow_proto3_optional is supported only on version of protoc >= 3.12
    // if the version of the system protoc is too old, we must panic
    match protoc_version() {
        Some(Version::V1(version @ ([0..=2, _, _] | [3, 0..=11, _]))) => {
            let version_str = version.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(".");
            panic!("protoc version {version_str} is too old, please install version 3.12 or newer",)
        },

        Some(Version::V2(_)) => {},
        None => panic!("protoc not found"),
        _ => (),
    }

    let mut config = prost_build::Config::new();

    config.bytes(["."]);

    #[cfg(feature = "arbitrary")]
    config.type_attribute(
        ".",
        "#[cfg_attr(feature = \"arbitrary\", derive(arbitrary::Arbitrary))]",
    );

    prost_reflect_build::Builder::new()
        .file_descriptor_set_path(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("file_descriptor_set.bin"))
        .descriptor_pool("crate::DESCRIPTOR_POOL")
        .compile_protos_with_config(config, &proto_files, &["../../proto"])?;

    Ok(())
}
