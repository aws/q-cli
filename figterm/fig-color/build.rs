use std::io::Result;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/color.h");
    println!("cargo:rerun-if-changed=src/color.c");

    let binding = bindgen::Builder::default()
        .header("src/color.h")
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .unwrap();

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    binding.write_to_file(out_path.join("color.rs"))?;

    cc::Build::new().file("src/color.c").compile("color");

    Ok(())
}
