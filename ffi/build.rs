//! Locates the static library for C ABI tests.
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    // OUT_DIR is `<target>/<profile>/build/<pkg>-<hash>/out`, so profile dir is 3 levels up.
    let profile_dir = out_dir.ancestors().nth(3).unwrap().to_path_buf();
    let staticlib = profile_dir.join("libvime.a");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=include/vime_engine.h");

    // Let test binaries find the artifact without `-L`.
    println!("cargo:rustc-link-search=native={}", profile_dir.display());
    println!("cargo:rustc-env=VIME_PROFILE_DIR={}", profile_dir.display());
    println!("cargo:rustc-env=VIME_STATICLIB={}", staticlib.display());
}
