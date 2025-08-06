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

Internally, the `ic_rusqlite` uses [stable structures](https://github.com/dfinity/stable-structures). The database connection is established with the first call to `ic_rusqlite::get_connection()`. You don't need to explicitly create the database or the memory manager, unless you also use other stable structures in your build. The virtual memories `200..210` are used for the canister's file system, and the virtual memory `120` is used as a storage for the database file.

```admonish note
Mounting a virtual memory onto a file is a special feature of [`stable-fs`](https://github.com/wasm-forge/stable-fs) that makes I/O operations with that file faster.
```

## Using File System

The `ic-rusqlite` uses `ic-wasi-polyfill`, this allows you to also use a file system. You can read or write files with the standard Rust functions. The database is stored in root `/main.db`. In the current implementation you only have a single database file, which you access by calling `ic_rusqlite::get_connection()`.

## Other Stable Structures

You can freely create other stable structures for your extra storage needs, just make sure to use a virtual memory ID that is not yet occupied.

```admonish warning title="Use Memory Manager"
Make sure [**you are using the memory manager**](https://docs.rs/ic-stable-structures/latest/ic_stable_structures/#example-canister) or you will destroy the database and the file system stored in the stable memory.
```

