# Working with the Large Datasets


- **Use indexed queries.**

Plan ahead, which fields will be critical to search the right elements and create indexes on those fields. If you need to quickly find a person by `name`, make sure this `name` field is indexed:

```sql
CREATE INDEX IF NOT EXISTS idx_persons_name ON customers(name);
```


Plan ahead, how you store your data. If you store the `first_name` and the `last_name` concatinated in the same table column, it won't be possible to search by the `last_name` without a full table scan, eg. `... WHERE name LIKE '%Johnson'`.

- **Check Instructions passed to see if you want to quit early and bulk insertions.**

1. You can process queries iteratively and check the timing constraints in the loop, if there is not enough time to finish the operation, exit eary with a limited amount of work done.
2. Every call to `execute` causes transactions to open and close on each statement. If you open a transaction before inserting the first, the database will not commit untill you explicitly commit the changes:

```rust
#[ic_cdk::update]
fn add_customers(id_offset: u64) {
    let start = ic_instruction_counter();

    with_connection(|mut conn| {
        let tx = conn.transaction().unwrap();

        let sql =
            String::from("insert into customers (firstname, lastname, email) values (?, ?, ?)");

        {
            let mut stmt = tx.prepare_cached(&sql).unwrap();

            let mut i = 0;

            // do as many rows as the timing allows
            while i < 100000000 {
                let id = id_offset + i + 1;
                let name = format!("{id}customer_name{id}");
                let last_name = format!("{id}customer_last_name{id}");
                let email = format!("{id}customer@example.com");

                stmt.execute(ic_rusqlite::params![name, last_name, email])
                    .expect("insert of a user failed!");

                i += 1;

                //
                let end = ic_instruction_counter();
                if end - start > 20000000000 {
                    break;
                }
            }
        }

        tx.commit().expect("COMMIT USER INSERTION FAILED!");
    });
}

```


- **Examine query plan.**

To identify problems with a complex or slow query, study its [query plan](https://www.sqlite.org/eqp.html).
See if there are any full scans on a table, you might want to change the query logic and/or introduce indexes.


- **Use pagination.**

To avoid running out of instructions and the returned data size on potentially large datasets, it is possible to limit the number of data rows returned by the SQL query. Use `LIMIT <N>` to limit the number of rows returned by a query, and `OFFSET <N>` to skip the first `N` rows of a response.

To return at most 5 persons and skip the first 3, write:
```sql
SELECT * FROM persons LIMIT 5 OFFSET 3
```

