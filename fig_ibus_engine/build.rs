use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let vala_file = manifest_dir.join("src/engine.vala");
    let vala_file = vala_file.to_str().unwrap();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let mut command = Command::new("valac");

    command.arg("--ccode").arg("--pkg").arg("ibus-1.0").arg(vala_file);

    #[cfg(debug_assertions)]
    {
        // add extra checks and such for debug builds
        command.arg("--enable-checking").arg("--enable-gobject-tracing");
    }

    let exit = command
        .current_dir(&out_dir)
        .spawn()
        .expect("Failed starting vala compiler")
        .wait()
        .expect("Failed running vala compiler");

    if !exit.success() {
        panic!("Vala compiler exited with code {}", exit.code().unwrap());
    }

    let libs = ["ibus-1.0", "glib-2.0", "gobject-2.0"];

    cc::Build::new()
        .flag("-w")
        .includes(
            libs.iter()
                .flat_map(|lib| pkg_config::probe_library(lib).unwrap().include_paths),
        )
        .out_dir(&out_dir)
        .file(out_dir.join("engine.c"))
        .compile("engine");

    println!("cargo:rerun-if-changed={}", vala_file);

    for lib in libs {
        println!("cargo:rustc-link-lib={lib}");
    }
}
