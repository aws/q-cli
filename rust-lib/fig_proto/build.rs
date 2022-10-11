use std::io::Result;

/// Try to find the version of protoc installed on the system.
fn protoc_version() -> Option<[u32; 3]> {
    let output = std::process::Command::new("protoc").arg("--version").output().ok()?;
    let version = String::from_utf8(output.stdout).ok()?;
    let version = version.trim();
    let version = version.split(' ').last().expect("No version");
    let version = version.split('.').collect::<Vec<_>>();
    if version.len() != 3 {
        return None;
    }
    let version = version
        .iter()
        .map(|s| s.parse::<u32>().ok())
        .collect::<Option<Vec<_>>>()?;
    Some([version[0], version[1], version[2]])
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
        Some(version @ ([0..=2, _, _] | [3, 0..=11, _])) => {
            let version_str = version.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(".");
            panic!("protoc version {version_str} is too old, please install version 3.12 or newer",)
        },
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

    prost_reflect_build::Builder::new().compile_protos_with_config(config, &proto_files, &["../../proto"])?;

    Ok(())
}
