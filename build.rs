use glob::glob;
use std::env;
use std::path::PathBuf;

fn main() {
    let bundled = std::env::var("CARGO_FEATURE_BUNDLED").is_ok();
    let precompiled = std::env::var("CARGO_FEATURE_PRECOMPILED").is_ok();

    if bundled && precompiled {
        panic!(
            "Features `bundled` and `precompiled` cannot be enabled at the same time.\nEnable `bundled` if you want to compile sqlite3 from source, enable `precompiled` if you want to link the precompiled version."
        );
    }

    if !bundled && !precompiled {
        panic!(
            "Features either `bundled` or `precompiled` must be enabled.\nEnable `bundled` if you want to compile sqlite3 from source, enable `precompiled` if you want to link the precompiled version."
        );
    }

    if precompiled {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let lib_dir = manifest_dir.join("lib");

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=sqlite3");
        println!("cargo:rerun-if-changed=lib/libsqlite3.a");
    }

    if bundled {
        // locate WASI builtins
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
}
