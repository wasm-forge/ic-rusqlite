# ic-rusqlite
This is a convenience package to create a canister with the Sqlite support. 


## Prerequisites

It is assumed that you have [rust](https://doc.rust-lang.org/book/ch01-01-installation.html), [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install/).

To compile a project with this dependency, you will need to:
- install wasi2ic: `cargo install wasi2ic`
- install WASI target: `rustup target add wasm32-wasip1`
- install WASI-SDK and WASI-oriented clang: [WASI-SDK](https://github.com/WebAssembly/wasi-sdk/releases/). 
- Finally, set the `WASI_SDK` and `PATH`:
```bash
  export WASI_SDK=/opt/wasi-sdk
  export PATH=$WASI_SDK/bin:$PATH
```

You can automate this by simply launching the preparation script:
```bash
  curl -fsSL https://raw.githubusercontent.com/wasm-forge/ic-rusqlite/main/prepare.sh -o prepare.sh | source prepare.sh
```


## Developing casniter

To enable `rusqlite` in your canister, add the helper dependency into your backend canister:
```bash
cargo add ic-rusqlite
```

You will also need to update the `dfx.json` to specify path to `wasm`, set `type` to `custom`, and 
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

## Example use in your Rust code

Finally, use the `get_connection()` function to access your database:

```rust
    //...
    
    // get connection to the database
    let conn = ic_rusqlite::get_connection();

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

    //...
```


You can find a small example in the `"examples/backend"` folder.

