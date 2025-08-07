# Basic Canister Example

A very basic canister backend for storing persons can look as follows:

```rust
use ic_rusqlite::with_connection;

#[ic_cdk::update]
fn add(name: String, data: String, age: u32) {
    with_connection(|conn| {
        // execute statement with parameters
        conn.execute(
            "INSERT INTO person (name, data, age) VALUES (?1, ?2, ?3)",
            (&name, &data, age),
        )
        .unwrap();
    })
}

#[ic_cdk::query]
fn list() -> Vec<(u64, String, String, u32)> {
    with_connection(|conn| {
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

    })
}

#[ic_cdk::init]
fn init() {
    with_connection(|conn| {
        // create the initial tables on the first deployment

        conn.execute(
            "CREATE TABLE IF NOT EXISTS person (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            data  TEXT,
            age   INTEGER
         )",
            (),
        )
        .unwrap();

    })

}
```

For a complete picture of what you can accomplish with a `Connection`, see the [**rusqlite** documentation](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html).

