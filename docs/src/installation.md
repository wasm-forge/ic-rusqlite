# Installation

## Required Tools
- [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)
- [DFX](https://internetcomputer.org/docs/current/developer-docs/setup/install/)

## Setting Up the Environment

You can setup your build environment via script:
```sh
curl -fsSL https://raw.githubusercontent.com/wasm-forge/ic-rusqlite/main/prepare.sh | sh
```

The script will:
- install `wasi2ic`: `cargo install wasi2ic`
- install WASI target: `rustup target add wasm32-wasip1`
- download `WASI-SDK` and WASI-oriented `clang`: [WASI-SDK](https://github.com/WebAssembly/wasi-sdk/releases/). 
- Finally, define variables in your `.bashrc`:
```sh
export WASI_SDK=<path to wasi-sdk>
export PATH=$WASI_SDK/bin:$PATH
```
