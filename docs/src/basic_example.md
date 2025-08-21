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

## Creating a custom database connection

You can create a customized database connection, in case you want to store it in another file mount onto another virtual memory or customize its initial pragma settings:
```rust
#[init]
fn init() {
    // default configuration
    let mut config = ic_rusqlite::ConnectionConfig::new();

    // optinally, create a custom connection to a database different from the default one
    config.db_file_name = "/my_custom_path/my_base.db".to_string(); // some custom path to the database
    config.db_file_mount_id = Some(150); // store database in the virtual memory ID 150
    config
        .pragma_settings
        .insert("cache_size".to_string(), "10000".to_string()); // modify the default pragma settings

    ic_rusqlite::set_connection_config(config);
    //...
    // The actual connection is not needed here, it will be done automatically on the next "with_connection" statement.
}

#[pre_upgrade]
fn pre_upgrade() {
    // closing connection explicitly unlocks the database before canister upgrade
    ic_rusqlite::close_connection();
}

#[post_upgrade]
fn post_upgrade() {
    // same initialization
    init();
}
```

In this example, we explicitly close the database connection in the `pre_upgrade` hook. This ensures that the database lock is released, preventing any issues when creating a new connection after the canister upgrade.

```admonish note
**Note:** Since the canister operates in single-user mode, there is no risk of conflicts from concurrent connections. Therefore, `ic-rusqlite` will attempt to delete the lock file when establishing a database connection, if one is found. This means that even if you do not explicitly close the connection, you will not be locked out of the database after an upgrade. However, the situation is different if the database remains locked and, after a canister upgrade, you attempt to use it with an [`ATTACH DATABASE`](https://www.sqlite.org/lang_attach.html) query.
```

