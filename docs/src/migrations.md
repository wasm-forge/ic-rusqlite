# Migrations

During canister development you might come across the situation when you need upgrade your canister's database structure. To do it, check out the [`ic-sql-migrate`](https://crates.io/crates/ic-sql-migrate) library. With it you can keep all the database changes within your source code and automatically upgrade your database on every deployment.

