mod vfs;

use rusqlite::{Connection, OpenFlags};
use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::{DefaultMemoryImpl, memory_manager::MemoryManager};

const FS_MOUNT_RANGE: Range<u8> = 200..210;
const DEFAULT_MOUNTED_DB_ID: u8 = 20;

// re-export some of the core dependencies for others to use
pub use ic_wasi_polyfill;
pub use rusqlite;

thread_local! {
    pub static CONNECTION: RefCell<Option<Rc<Connection>>> = const { RefCell::new(None) };

    pub static DB_FILE_NAME: RefCell<Option<String>> = const { RefCell::new(None) };

    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = {
        let m = MemoryManager::init(DefaultMemoryImpl::default());

        // initialize ic-wasi-polyfill
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, FS_MOUNT_RANGE);

        RefCell::new(m)
    };
}

fn init_db() -> Rc<Connection> {
    let memory = MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(DEFAULT_MOUNTED_DB_ID)));

    sqlite_vfs::register("vfs", vfs::VfsPages::new(memory), true).unwrap();

    let conn = Connection::open_with_flags_and_vfs(
        vfs::DB_NAME,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        "vfs",
    )
    .unwrap();

    conn.execute_batch(
        r#"
            PRAGMA page_size=4096;
            PRAGMA journal_mode=MEMORY;
            PRAGMA locking_mode=EXCLUSIVE;
            PRAGMA synchronous=0;
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

/// Set random seed used by the database
pub fn init_random_seed(seed: &[u8]) {
    //
}

/// Limit the maximum database size allowed
pub fn set_max_size(size: u64) {
    //
}
