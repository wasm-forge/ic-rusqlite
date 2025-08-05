# Installation

## Required Tools
- [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)
- [DFX](https://internetcomputer.org/docs/current/developer-docs/setup/install/)
- [WASI-SDK](https://github.com/WebAssembly/wasi-sdk/releases)
- `wasi2ic` tool (rewires WASI binaries for the Internet Computer).

## Setting Up the Environment

Install the WASI target:
```bash
rustup target add wasm32-wasip1
```

The SQLite written in C, to compile it you will need the [WASI-SDK](https://github.com/WebAssembly/wasi-sdk/releases).

The `*.deb` package will install into `/opt/wasi-sdk` by default. You can choose any other folder if you download a `*.tar.gz` file.

Add these commands to your canister project `build.sh` script or `.bashrc` to automate:
```bash
export WASI_SDK=/opt/wasi-sdk
export PATH=$WASI_SDK/bin:$PATH
```

Run `clang --version` to see that the correct compiler is available.

Finally, install the (`wasi2ic`)[https://github.com/wasm-forge/wasi2ic]:
```bash
cargo install wasi2ic
```

This tool (rewires a WASI binary)[https://www.youtube.com/watch?v=oQb5TUiby7Q] and replaces calls to WASI functions with the corresponding IC function implementations.

