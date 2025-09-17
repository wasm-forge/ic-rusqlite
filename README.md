# ic-rusqlite
This is a convenience package to create a canister with the Sqlite support. 


## Prerequisites

It is assumed that you have [rust](https://doc.rust-lang.org/book/ch01-01-installation.html), [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install/).

To compile a project with this dependency, you will need to:
- install wasi2ic: `cargo install wasi2ic`
- install WASI target: `rustup target add wasm32-wasip1`


## Compiling SQLite

If you intend to compile the SQLite from the source, you will need to install WASI-SDK:

- install WASI-SDK and WASI-oriented clang: [WASI-SDK](https://github.com/WebAssembly/wasi-sdk/releases/). 
- set the `WASI_SDK_PATH` and `PATH` variables:
```bash
export WASI_SDK_PATH=/opt/wasi-sdk
export PATH=$WASI_SDK_PATH/bin:$PATH
```

You can automate this by launching the preparation script:
```sh
curl -fsSL https://raw.githubusercontent.com/wasm-forge/ic-rusqlite/main/prepare.sh | sh
```

Finally, to enable `rusqlite` in your canister, add the helper dependency into your backend canister:
```bash
cargo add ic-rusqlite
```

## Using Precompiled SQLite

If you don't want to install `WASI-SDK`, you can use the precompiled SQLite version for WASI, just activate the `precompiled` feature and disable the default features:
```sh
cargo add ic-rusqlite --no-default-features --features precompiled
```

## Developing Canister

You will need to update the `dfx.json` to specify path to `wasm`, set `type` to `custom`, and 
specify custom build steps to enforce compilation to the `wasm32-wasip1` target. 
Finally, use `wasi2ic` to produce wasm executable on the Internet Computer.

Example:
```json
{
  "canisters": {
    "backend": {
      "candid": "can.did",
      "package": "backend",
      "build": [
        "cargo build --release --target wasm32-wasip1",
        "wasi2ic target/wasm32-wasip1/release/backend.wasm target/wasm32-wasip1/release/backend_nowasi.wasm"
      ],
      "wasm": "target/wasm32-wasip1/release/backend_nowasi.wasm",
      "type": "custom",
      "metadata": [
        {
          "name": "candid:service"
        }
      ]
    }
  },
  "dfx": "0.28.0",
  "version": 1
}
```

## Example

Finally, use the `with_connection()` function to access your database:

```rust
    //...
    
    with_connection(|conn| {
      conn.execute(
          "CREATE TABLE person (
              id    INTEGER PRIMARY KEY,
              name  TEXT NOT NULL,
              data  BLOB
          )",
          (),
      )?;

      let data: Option<Vec<u8>> = None;

      conn.execute(
          "INSERT INTO person (name, data) VALUES (?1, ?2)",
          ("Steven", &data),
      )?;

    })


    //...
```

## Further Examples

You can find a small example in the `"examples/backend"` folder.

For more detailed explanations, see the [`ic-rusqlite` book](https://wasm-forge.github.io/ic-rusqlite/).

