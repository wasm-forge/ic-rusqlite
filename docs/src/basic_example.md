# Basic Example

A very basic canister backend of storing persons can look as follows:

```rust
#[ic_cdk::update]
fn add(name: String, data: String, age: u32) {
    // execute statement with parameters
    ic_rusqlite::get_connection()
        .execute(
            "INSERT INTO person (name, data, age) VALUES (?1, ?2, ?3)",
            (&name, &data, age),
        )
        .unwrap();
}

#[ic_cdk::query]
fn list() -> Vec<(u64, String, String, u32)> {
    // get DB connection
    let conn = ic_rusqlite::get_connection();

    // prepare SQL statement
    let mut stmt = conn
        .prepare("SELECT id, name, data, age FROM person")
        .unwrap();

    // execute statement and map results into an iterator
    let iter = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .unwrap();

    iter.map(|r| r.unwrap()).collect()
}

#[ic_cdk::init]
fn init() {
    // create the initial tables on the first deployment
    ic_rusqlite::get_connection()
        .execute(
            "CREATE TABLE IF NOT EXISTS person (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            data  TEXT,
            age   INTEGER
         )",
            (),
        )
        .unwrap();
}
```

To fully understand what you can do with a connection, refer to the [`rusqlite` documentation](https://docs.rs/rusqlite/latest/rusqlite/).
