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

    cc::Build::new()
        .flag("-w")
        .include("/usr/include/ibus-1.0")
        .include("/usr/include/glib-2.0")
        .include("/usr/lib64/glib-2.0/include")
        .out_dir(&out_dir)
        .file(out_dir.join("engine.c"))
        .compile("engine");

    println!("cargo:rerun-if-changed={}", vala_file);
    println!("cargo:rustc-link-lib=ibus-1.0");
    println!("cargo:rustc-link-lib=glib-2.0");
    println!("cargo:rustc-link-lib=gobject-2.0");
}
