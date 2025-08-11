# Tips on Working with Large Datasets


- **Use indexed queries.**

Plan ahead, which fields will be critical to search the right elements and create indexes on those fields. For example, if you need to quickly find a person by `name`, make sure this `name` field is indexed:

```sql
CREATE INDEX IF NOT EXISTS idx_persons_name ON customers(name);
```

Also plan how you store your data. If you store the first_name and last_name concatenated in the same column, it won’t be possible to efficiently search by last_name without performing a full table scan, e.g.:
```sql
... WHERE name LIKE '%Johnson'
```

Plan ahead, how you store your data. If you store the `first_name` and the `last_name` concatinated in the same table column, it won't be possible to search by the `last_name` without a full table scan, eg. .

- **Check Instructions passed to see if you want to quit early and bulk insertions.**

1. When processing queries iteratively, check timing constraints inside the loop. If there isn’t enough time to complete the operation, exit early with a partial result rather than letting the process overrun.

2. Every call to `execute` opens and closes a transaction. To improve performance when inserting many records, open a transaction before the first insert and commit changes only once after all inserts are complete. This avoids committing after each row:


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

