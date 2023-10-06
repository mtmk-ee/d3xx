use std::env;
use std::path::PathBuf;

const BINDINGS_OUTPUT_NAME: &str = "bindings.rs";

#[cfg(windows)]
const LINK_DIR: &str = "lib/windows";
#[cfg(not(windows))]
const LINK_DIR: &str = "lib/linux";

#[cfg(windows)]
const ALLOWED_HEADERS: [&str; 1] = ["include/windows/ftd3xx.h"];
#[cfg(not(windows))]
const ALLOWED_HEADERS: [&str; 1] = ["include/linux/ftd3xx.h"];

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    println!("cargo:rustc-link-search={}", LINK_DIR);
    println!("cargo:rustc-link-lib=ftd3xx");
    println!("cargo:rerun-if-changed=include/wrapper.h");

    let mut bindings = bindgen::Builder::default()
        .header("include/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));
    for file in ALLOWED_HEADERS {
        bindings = bindings.allowlist_file(file);
    }
    bindings
        .generate()
        .expect("unable to generate bindings")
        .write_to_file(out_path.join(BINDINGS_OUTPUT_NAME))
        .expect("could not write bindings");
}
