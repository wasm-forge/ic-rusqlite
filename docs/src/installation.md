# Installation

## Required Tools
- [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)
- [DFX](https://internetcomputer.org/docs/current/developer-docs/setup/install/)
- [WASI-SDK](https://github.com/WebAssembly/wasi-sdk/releases)
- `wasi2ic` tool (rewires WASI binaries for the Internet Computer).

## Setting Up the Environment

It is assumed that you have [rust](https://doc.rust-lang.org/book/ch01-01-installation.html), [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install/) installed.

To compile a project with `ic-rusqlite` dependency, you will need to:

- install wasi2ic: `cargo install wasi2ic`
- install WASI target: `rustup target add wasm32-wasip1`
- install WASI-SDK and WASI-oriented clang: [WASI-SDK](https://github.com/WebAssembly/wasi-sdk/releases/). 
- Finally, set the `WASI_SDK` and `PATH`:
```bash
export WASI_SDK=/opt/wasi-sdk
export PATH=$WASI_SDK/bin:$PATH
```

You can automate this by launching the preparation script:
```bash
curl -fsSL https://raw.githubusercontent.com/wasm-forge/ic-rusqlite/main/prepare.sh | sh
```


