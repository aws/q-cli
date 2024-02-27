use std::io::Result;
use std::path::PathBuf;
use std::process::Command;

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

fn download_protoc() {
    let protoc_version = "25.3";
    let checksum = "f853e691868d0557425ea290bf7ba6384eef2fa9b04c323afab49a770ba9da80";

    let tmp_folder = tempfile::tempdir().unwrap();

    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "osx",
        os => panic!("Unsupported os: {os}"),
    };

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch_64",
        arch => panic!("Unsupported arch: {arch}"),
    };

    let mut download_command = Command::new("curl");
    download_command
        .arg("-Lf")
        .arg(format!(
            "https://github.com/protocolbuffers/protobuf/releases/download/v{protoc_version}/protoc-{protoc_version}-{os}-{arch}.zip"
        ))
        .arg("-o")
        .arg(tmp_folder.path().join("protoc.zip"));
    assert!(download_command.spawn().unwrap().wait().unwrap().success());

    let mut checksum_command = Command::new("sha256sum");
    checksum_command.arg(tmp_folder.path().join("protoc.zip"));
    let checksum_output = checksum_command.output().unwrap();
    let checksum_output = String::from_utf8(checksum_output.stdout).unwrap();

    eprintln!("checksum: {checksum_output:?}");
    assert!(checksum_output.starts_with(checksum));

    let mut unzip_comamnd = Command::new("unzip");
    unzip_comamnd
        .arg("-o")
        .arg(tmp_folder.path().join("protoc.zip"))
        .current_dir(tmp_folder.path());
    assert!(unzip_comamnd.spawn().unwrap().wait().unwrap().success());

    let out_bin = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("protoc");

    let mut mv = Command::new("mv");
    mv.arg(tmp_folder.path().join("bin/protoc")).arg(&out_bin);
    assert!(mv.spawn().unwrap().wait().unwrap().success());
    
    std::env::set_var("PROTOC", out_bin);
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let proto_files = std::fs::read_dir("../../proto")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|t| t.is_file()))
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "proto"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    for file in &proto_files {
        println!("cargo:rerun-if-changed={}", file.display());
    }

    // --experimental_allow_proto3_optional is supported only on version of protoc >= 3.12
    // if the version of the system protoc is too old, we must panic
    match protoc_version() {
        Some(Version::V1([0..=2, _, _] | [3, 0..=11, _])) => download_protoc(),
        Some(Version::V1(_) | Version::V2(_)) => {},
        None => download_protoc(),
    };

    let mut config = prost_build::Config::new();

    config.protoc_arg("--experimental_allow_proto3_optional");

    #[cfg(feature = "arbitrary")]
    config.type_attribute(
        ".",
        "#[cfg_attr(feature = \"arbitrary\", derive(arbitrary::Arbitrary))]",
    );

    config.extern_path(".fig_common.Empty", "()");

    prost_reflect_build::Builder::new()
        .file_descriptor_set_path(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("file_descriptor_set.bin"))
        .descriptor_pool("crate::DESCRIPTOR_POOL")
        .compile_protos_with_config(config, &proto_files, &["../../proto"])?;

    Ok(())
}
