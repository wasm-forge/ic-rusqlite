## Examples Container

Build a basic example of using SQLite in an IC Canister.


## Prerequisites

It is assumed that you have [rust](https://doc.rust-lang.org/book/ch01-01-installation.html), [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install/).

Install the `wasi2ic` tool:
```bash
cargo install wasi2ic
```

## Build and Deploy

Run dfx:
```bash
dfx start --clean --background
```

Deploy the example canister:
```bash
dfx deploy
```

## Test

After deployment, you can open the backend in your browser or run the test script from a command line:

```bash
test.sh
```
