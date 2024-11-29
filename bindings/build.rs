use cbindgen::Config;
use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_config(Config {
            usize_is_size_t: true,
            ..Default::default()
        })
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate C++ bindings")
        .write_to_file("bindings.h");

    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("raphael_bindings")
        .csharp_namespace("Raphael")
        .csharp_class_accessibility("public")
        .generate_csharp_file("NativeMethods.g.cs")
        .expect("Unable to generate C# bindings");
}
