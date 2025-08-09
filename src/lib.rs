use std::cell::RefCell;
use std::cell::RefMut;
use std::ops::Range;

use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::{DefaultMemoryImpl, memory_manager::MemoryManager};

/// Virtual memory range used by the file system
const FS_MEMORY_RANGE: Range<u8> = 101..119;

/// Dedicated virtual memory used to store the database
const DEFAULT_MOUNTED_DB_ID: u8 = 120;

/// Database file name
const DB_FILE_NAME: &str = "/main.db";

// re-export some of the core dependencies for others to use
pub use ic_wasi_polyfill;
pub use rusqlite;
pub use rusqlite::*;

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = {

        let m = MemoryManager::init(DefaultMemoryImpl::default());

        // initialize ic-wasi-polyfill
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, FS_MEMORY_RANGE);
        RefCell::new(m)
    };

    pub static CONNECTION: RefCell<Option<Connection>> = RefCell::new(Some(create_connection()))
}

/// Return the name of the database file
pub fn get_db_path() -> &'static str {
    DB_FILE_NAME
}

/// Close connection to safely unlock the database file and process it directly
pub fn close_connection() {
    CONNECTION.with(|conn| {
        let mut conn_mut = conn.borrow_mut();

        if let Some(c) = conn_mut.take() {
            drop(c);
        }
    })
}

fn create_connection() -> Connection {
    let memory = MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(DEFAULT_MOUNTED_DB_ID)));

    // dedicate a virtual memory to the database file
    ic_wasi_polyfill::mount_memory_file(DB_FILE_NAME, Box::new(memory));

    // remove lock if it exists
    let _ = std::fs::remove_dir_all(format!("{DB_FILE_NAME}.lock"));

    // Create a new connection to the file
    let conn = rusqlite::Connection::open(DB_FILE_NAME).expect("Failed opening the database!");

    // set pragmas
    set_pragmas(&conn);

    conn
}

fn set_pragmas(conn: &Connection) {
    // do not create and destroy the journal file every time
    conn.pragma_update(None, "journal_mode", &"PERSIST" as &dyn ToSql)
        .unwrap();

    // writes are not cached on mounted memory, no need to call sync
    conn.pragma_update(None, "synchronous", &"OFF" as &dyn ToSql)
        .unwrap();

    // reduce locks and unlocks
    conn.pragma_update(None, "locking_mode", &"EXCLUSIVE" as &dyn ToSql)
        .unwrap();

    // temp_store = MEMORY, disables creating temp files, improves performance,
    // (currently this workaround also avoids error on creating a tmp file on complex queries)
    conn.pragma_update(None, "temp_store", &"MEMORY" as &dyn ToSql)
        .unwrap();

    // default page size
    conn.pragma_update(None, "page_size", &4096 as &dyn ToSql)
        .unwrap();

    // reduce read operations by caching database pages (set the cache limit to 4096 * 500000 bytes)
    conn.pragma_update(None, "cache_size", &500000 as &dyn ToSql)
        .unwrap();
}

/// Open connection if it is closed and execute a function provided
pub fn with_connection<F, R>(f: F) -> R
where
    F: FnOnce(RefMut<'_, Connection>) -> R,
{
    CONNECTION.with(|conn| {
        let mut conn_mut: RefMut<'_, Option<_>> = conn.borrow_mut();

        if conn_mut.is_none() {
            *conn_mut = Some(create_connection());
        }

        let conn_ref = RefMut::map(conn_mut, |opt| opt.as_mut().unwrap());

        f(conn_ref)
    })
}
