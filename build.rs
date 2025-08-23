use glob::glob;
use std::env;
use std::path::PathBuf;

fn main() {
    // locate wasi builtins
    let sdk_path = if let Ok(sdk_path) = env::var("WASI_SDK_PATH") {
        sdk_path
    } else {
        "/opt/wasi-sdk".to_string()
    };

    let sysroot = format!("{sdk_path}/share/wasi-sysroot");
    println!("cargo:rustc-env=WASI_SYSROOT={sysroot}",);

    let pattern = format!("{sdk_path}/lib/clang/*/lib/*wasip1");

    let paths: Vec<PathBuf> = glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect();

    if let Some(path) = paths.last() {
        // use the latest version that we have found
        println!("cargo:rustc-link-search={}", path.display());

        let builtins: Vec<PathBuf> = glob(&format!("{}/*", path.display()))
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .collect();

        if let Some(b) = builtins.first() {
            let name = b
                .file_stem()
                .expect("builtins not found")
                .to_string_lossy()
                .to_string();

            println!("cargo:rustc-link-arg=-l{}", &name[3..]);
        } else {
            panic!("Could not find clang wasm32 builtins under '{pattern}'");
        }
    } else {
        panic!("Could not find clang wasm32 builtins under '{pattern}'");
    }
}
