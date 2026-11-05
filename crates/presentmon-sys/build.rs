use std::path::PathBuf;
use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=vendor/PresentMonAPI.h");

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let header = manifest.join("vendor/PresentMonAPI.h");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // The header refers to enum/struct types without their tags, so it only parses
    // as C++ even though bindgen treats a .h input as C.
    // Its <cstdint> pulls in the MSVC C++ STL, which rejects the bundled clang; the
    // shim shadows that include with the C typedefs the header actually needs.
    let shim = out_dir.join("include");
    fs::create_dir_all(&shim).unwrap();
    fs::write(
        shim.join("cstdint"),
        "#pragma once\n#include <stdint.h>\n#include <stddef.h>\n",
    )
    .unwrap();

    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .clang_arg("-xc++")
        .clang_arg("-isystem")
        .clang_arg(shim.display().to_string())
        .clang_arg("-fdeclspec")
        .clang_arg("-Wno-ignored-attributes")
        // Keep the CRT's own defines and typedefs (SAL version macros, NULL, ...)
        // out of the generated API surface.
        .allowlist_file(".*PresentMonAPI\\.h")
        .prepend_enum_name(false)
        .default_enum_style(bindgen::EnumVariation::Consts)
        .derive_default(true)
        .derive_debug(true)
        .generate()
        .expect("bindgen failed for PresentMonAPI.h");

    let out = out_dir.join("bindings.rs");
    bindings.write_to_file(out).unwrap();
}
