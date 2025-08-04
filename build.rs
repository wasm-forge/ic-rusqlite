use glob::glob;
use std::env;
use std::path::PathBuf;

fn main() {
    // locate wasi builtins
    let sdk_path = if let Ok(sdk_path) = env::var("WASI_SDK") {
        sdk_path
    } else {
        match std::env::consts::OS {
            "linux" => "/opt/wasi-sdk".to_string(),
            "macos" => "/opt/wasi-sdk".to_string(),
            other => panic!("Unsupported OS: {other}"),
        }
    };

    let pattern = format!("{sdk_path}/lib/clang/*/lib/*wasip1");

    let paths: Vec<PathBuf> = glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect();

    if let Some(path) = paths.last() {
        // use the latest version we have found

        println!("cargo:rustc-link-search={}", path.display());

        match std::env::consts::OS {
            "linux" => println!("cargo:rustc-link-arg=-lclang_rt.builtins-wasm32"),
            "macos" => println!("cargo:rustc-link-arg=-lclang_rt.builtins.a"),
            other => panic!("Unsupported OS: {other}"),
        }
    } else {
        panic!("Could not find clang wasm32 builtins under '{pattern}'");
    }
}
