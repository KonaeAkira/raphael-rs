#[path = "src/util.rs"]
mod util;

fn main() {
    let toml_path = util::translation_toml_path();
    println!(
        "cargo:rerun-if-env-changed={}",
        util::RAPHAEL_TRANSLATIONS_TOML_ENV
    );
    println!(
        "cargo::rerun-if-changed={}",
        toml_path
            .to_str()
            .expect("Translation TOML file path is not valid unicode!")
    );
}
