# Installation

## Prerequisites
- [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)
- [DFX](https://internetcomputer.org/docs/current/developer-docs/setup/install/)

- install `wasi2ic`: `cargo install wasi2ic`
- install WASI target: `rustup target add wasm32-wasip1`



## Setting Up the Environment for SQLite compilation from source

If you intend to compile the sqlite from source, you will need to install WASI-SDK. You can setup your build environment via script:
```sh
curl -fsSL https://raw.githubusercontent.com/wasm-forge/ic-rusqlite/main/prepare.sh | sh
```

The script will:
- download `WASI-SDK` and WASI-oriented `clang`: [WASI-SDK](https://github.com/WebAssembly/wasi-sdk/releases/). 
- Define variables in your `.bashrc`:
```sh
export WASI_SDK_PATH=<path to wasi-sdk>
export PATH=$WASI_SDK_PATH/bin:$PATH
```
