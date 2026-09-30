//! Teaches the build where the static library lands, so the C ABI tests can find
//! it.
//!
//! The C tests in `tests/c_abi.rs` compile and link real C programs against
//! `include/vime_engine.h`, which means they need a path to `libvime.a`. Cargo
//! does not hand a build script that path — only `OUT_DIR`, which is three
//! directories below the profile directory the artifact sits in. So the path is
//! reconstructed here and passed on as an environment variable.
//!
//! This script deliberately does *not* run `cargo build` to produce the library.
//! A build script that invokes cargo would re-enter itself, and cargo takes the
//! build directory lock that the outer invocation is holding. The C tests handle
//! freshness themselves, where they can do it once per run and report a clear
//! message if the rebuild is not possible.

use std::path::PathBuf;

fn main() {
    let out_dir =
        PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set for build scripts"));

    // `OUT_DIR` is `<target>/<profile>/build/<pkg>-<hash>/out`, so the profile
    // directory — the one holding `libvime.a` — is three levels up.
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("OUT_DIR is at least three levels below the profile directory")
        .to_path_buf();

    let staticlib = profile_dir.join("libvime.a");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=include/vime_engine.h");

    // Let the test binaries find the artifact without `-L`, so a C test that links
    // it cannot get the search path wrong.
    println!("cargo:rustc-link-search=native={}", profile_dir.display());
    println!("cargo:rustc-env=VIME_PROFILE_DIR={}", profile_dir.display());

    // Reported whether or not the file is there yet: freshness is the test's
    // problem, and it can answer that better than this script can.
    println!("cargo:rustc-env=VIME_STATICLIB={}", staticlib.display());
}
