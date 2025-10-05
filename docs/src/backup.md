# Backup and Recover

## Downloading and Uploading the Database File

To create a backup of your database, you can use the [`icml`](https://crates.io/crates/icml) tool. The `download` command lets you save the canister’s database to your local drive.

Later, you can restore it by uploading the database back with the `upload` command. 

With downloading and uploading the database it is also possible to move the database from one canister to another. 

```admonish warning title="Do not read or write database with connections open"
Before running `download` or `upload`, the canister's database connection must be closed. This ensures that the cached pages of the SQLite engine do not conflict with the database written on disk and your database does not get corrupted.
```

## Recovering from a Canister Snapshot

If a canister becomes unusable and its functions cannot be called, but you can still create and download a snapshot, `icml` can recover the SQLite database directly from the stable memory snapshot file.