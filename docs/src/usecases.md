# When Is SQLite Appropriate?

SQLite is a fast, lightweight SQL database that stores all data in a single file. This works well with a canister's memory model, but there are pros and cons to consider before choosing it.

## When to Use SQLite

SQLite is ideal when you need structured, reliable, and transactional data storage but don’t want the complexity of running a database server.

- **You need complex queries.**
  SQLite supports standard SQL, allowing you to perform advanced queries using `SELECT`, `JOIN`, `GROUP BY`, and other operations.

- **You need transactions.**
  SQLite is ACID-compliant, so it guarantees data consistency, supports rollback, and handles multi-step operations safely.

- **Tabular data with multiple indexes.**
  SQLite is useful, if you have a large table and you want to find records by multiple indexed fields.

## When to Use Stable Structures

If your data needs are simple, stable structures may be faster and easier to use.

**Use a stable structure like `Vec`, `BTreeMap`, or `BTreeSet` when:**

- **You don’t need transactions.**
  If you don’t need rollback or atomic updates.

- **You don’t need SQL-style queries.**
  For simple `key -> value` dictionary search, stable data structures are faster and easier to work with.

- **You need speed over flexibility.**
  Structures like `Vec` provide O(1) index-based access, and `BTreeMap` gives sorted access with logarithmic complexity — both are faster than SQLite for many common tasks.

## When to Use the File System

For file-based hierarchical data, the file system may be the best choice.

**Use the file system storage when:**

- **You want to organize data into directories and subfolders.**
  Storing backups and documents.

- **You work with large data.**
  Files are larger than the canister's heap memory and/or contain large sections of zeroed memory ([sparse files](https://en.wikipedia.org/wiki/Sparse_file)).

- **You perform byte-level data manipulation.**
  Editing or seeking within large files is faster with standard file I/O than loading and modifying data through an SQL database.

