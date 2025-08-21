# Creating Your SQLite Canister

## Configure your canister
To integrate SQLite into your backend Rust Canister project, you will need to add the `ic-rusqlite` dependency and adjust the build process.

The first part is easy:
```bash
cargo add ic-rusqlite
```

The second part requires you to update the `dfx.json` to specify path to `wasm` binary, set cansiter `type` to `custom`, and 
specify custom build steps to enforce compilation to the `wasm32-wasip1` target.
Finally, use `wasi2ic` to produce a Wasm executable on the Internet Computer.

Example `dfx.json`:
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

## Accessing the Database

The database connection is established with the first call to `ic_rusqlite::with_connection(...)`, so there is no need to explicitly initialize or create a database.

Internally, the `ic-rusqlite` uses [stable structures](https://dfinity.github.io/stable-structures/) with the [memory manager](https://dfinity.github.io/stable-structures/concepts/memory-manager.html). The virtual memories `101..119` are reserved for the file system.


The virtual memory with the ID `120` is the default storage memory for the database database file is `/DB/main.db`. These settings are default, but can be change via the `set_connection_config(...)` function.

```admonish note
The ability to associate a file with a virtual memory is a special feature of [`stable-fs`](https://github.com/wasm-forge/stable-fs). This allows to create dedicated files with fast I/O access.
```

## Using File System

`ic-rusqlite` is compiled to the WASI target and then processed by the [`wasi2ic` workflow](https://github.com/wasm-forge/wasi2ic), embedding `ic-wasi-polyfill` and `stable-fs` into the output binary. This enables the use of standard Rust I/O APIs for file operations.

```admonish note
By default the main database file is stored in the root folder: `/DB/main.db` and there are a few additional [helper files](https://www.sqlite.org/tempfiles.html) that can be created by the databsae engine.
```

## Using Stable Structures

You can freely create other stable structures for your extra storage needs, just make sure to use a virtual memory ID that is not yet occupied.

```admonish warning title="Use Memory Manager"
Make sure [**you are using the memory manager**](https://docs.rs/ic-stable-structures/latest/ic_stable_structures/#example-canister) or you will destroy the database and the file system stored in the stable memory.
```

