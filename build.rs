use glob::glob;
use std::env;
use std::path::PathBuf;

fn main() {
    let compile_sqlite = std::env::var("CARGO_FEATURE_COMPILE_SQLITE").is_ok();
    let precompiled = std::env::var("CARGO_FEATURE_PRECOMPILED").is_ok();

    // Figure out the target triple (host vs. wasm)
    let target = env::var("TARGET").unwrap();

    if !target.starts_with("wasm32") {
        panic!("Targets other than WASM32 are not supported!");
    }

    if compile_sqlite && precompiled {
        panic!(
            "Features `compile_sqlite` and `precompiled` cannot be enabled at the same time.\nEnable `compile_sqlite` if you want to compile sqlite3 from source, enable `precompiled` if you want to link the precompiled version."
        );
    }

    if !compile_sqlite && !precompiled {
        panic!(
            "Features either `compile_sqlite` or `precompiled` must be enabled.\nEither enable `compile_sqlite` if you want to compile sqlite3 from source, enable `precompiled` if you want to link the precompiled version."
        );
    }

    if precompiled {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let lib_dir = manifest_dir.join("lib");

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=sqlite3");
        println!("cargo:rerun-if-changed={}/libsqlite3.a", lib_dir.display());
    }

    if compile_sqlite {
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
