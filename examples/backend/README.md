## Examples Container

Build a basic example of using SQLite in an IC Canister.


## Prerequisites

It is assumed that you have [rust](https://doc.rust-lang.org/book/ch01-01-installation.html), [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install/).


You also need the Wasm-oriented [clang](https://github.com/WebAssembly/wasi-sdk/releases/) installation. 
Once installed the `clang` compiler should be available from the path `/opt/wasi-sdk/bin/`. 

If your WASI installation is in a different folder, assign it to environment variable `WASI_SDK`:
```bash
export WASI_SDK=/opt/wasi-sdk
```

Finally, install the `wasi2ic` tool:
```bash
cargo install wasi2ic
```

## Building

To build and deploy the example canister, start the build script:

```bash
build.sh
```


## Testing

After deployment, you can open the backend in your browser or run the test script from a command line:

```bash
test.sh
```
