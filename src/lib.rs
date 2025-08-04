use rusqlite::Connection;
use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::{DefaultMemoryImpl, memory_manager::MemoryManager};

/// Virtual memory range used by the file system
const FS_MEMORY_RANGE: Range<u8> = 200..210;

/// Dedicated virtual memory used to store the database
const DEFAULT_MOUNTED_DB_ID: u8 = 120;

/// Database file name
const DB_FILE_NAME: &str = "main.db";

// re-export some of the core dependencies for others to use
pub use ic_wasi_polyfill;
pub use rusqlite;

thread_local! {
    pub static CONNECTION: RefCell<Option<Rc<Connection>>> = const { RefCell::new(None) };

    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = {

        let m = MemoryManager::init(DefaultMemoryImpl::default());
        // initialize ic-wasi-polyfill
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, FS_MEMORY_RANGE);
        RefCell::new(m)
    };
}

fn init_db() -> Rc<Connection> {
    let memory = MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(DEFAULT_MOUNTED_DB_ID)));

    // dedicate a virtual memory to the database file
    ic_wasi_polyfill::mount_memory_file(DB_FILE_NAME, Box::new(memory));

    // remove lock if it exists
    let _ = std::fs::remove_dir_all(format!("{DB_FILE_NAME}.lock"));

    // Create a new connection to the file
    let conn = rusqlite::Connection::open(DB_FILE_NAME).expect("Failed opening the database!");

    // set pragmas
    conn.execute_batch(
        r#"
            PRAGMA page_size=4096;
            PRAGMA journal_mode=MEMORY;
            PRAGMA synchronous=0;
            PRAGMA locking_mode=EXCLUSIVE;
            PRAGMA temp_store=MEMORY;
            PRAGMA cache_size=100000;
            "#,
    )
    .unwrap();

    CONNECTION.with_borrow_mut(|c| {
        *c = Some(Rc::new(conn));
    });

    CONNECTION.with_borrow(|c| c.clone().unwrap())
}

/// Use this function to get a new active connection to the database
pub fn get_connection() -> Rc<Connection> {
    if let Some(conn) = CONNECTION.with_borrow(|c| c.clone()) {
        Rc::clone(&conn)
    } else {
        init_db()
    }
}
