use std::env;
use std::path::PathBuf;
use pkg_config;

fn main() {
    let conf = pkg_config::probe_library("guile-3.0").unwrap();
    let guile_dir = env::var("GUILE_DIR").unwrap_or("/usr/local/lib/".to_string());
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={guile_dir}");

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=guile-3.0");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut builder = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));
    for p in conf.include_paths {
        builder = builder.clang_arg(format!("-I{}", p.to_str().unwrap()));
    }

        // Finish the builder and generate the bindings.
    let bindings = builder.generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");
     
    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
