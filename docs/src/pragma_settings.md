# Pragma settings

Following [pragma](https://sqlite.org/pragma.html) settings are set in `ic-rusqlite` by default:

*[journal_mode](https://sqlite.org/pragma.html#pragma_journal_mode)*
    - `PERSIST`
    - persist journal file (is faster than deleting the file every time). Setting it to `OFF` works even faster, but [disallows atomic COMMIT/ROLLBACK](https://sqlite.org/pragma.html#pragma_journal_mode).

*[synchronous](https://sqlite.org/pragma.html#synchronous)*
    - `OFF`
    - because writes are always written and not cached, we do not need to explicity "flush" data changes to the disk.

*[page_size](https://sqlite.org/pragma.html#page_size)*
    - `4096`
    - bigger page size works a bit faster by reduring sequential writes. The performance degrades, however, if there are many small writes at random locations.

*[locking_mode](https://sqlite.org/pragma.html#locking_mode)*          
    - `EXCLUSIVE`    
    - exclusive mode is faster because we only lock the database once (the canister is the only user of the database).

*[temp_store](https://sqlite.org/pragma.html#temp_store)*
    - `MEMORY`       
    - avoids creating temporary file on disk, saves extra I/O operations.

*[cache_size](https://sqlite.org/pragma.html#cache_size)*
    - `1000000`
    - gives a significant performance boost at the expence of the canister's memory used.


The pragma settings can be adjusted with SQL queries, you might want to do that during the `init` or the `post_upgrade` events:

```sql
    PRAGMA cache_size=1000; -- Reduce the maximum DB cache size
```
