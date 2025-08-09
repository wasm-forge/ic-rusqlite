# Pagination

To avoid running out of instructions and the returned data size on potentially large datasets, it is possible to limit the number of data rows returned by the SQL query. Use `LIMIT <N>` to limit the number of rows returned by a query, and `OFFSET <N>` to skip the first `N` rows of a responce.

To return at most 5 persons and skip the first 3:
```sql
SELECT * FROM person LIMIT 5 OFFSET 3
```

