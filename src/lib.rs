use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::ops::Range;
use std::path::Path;

use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::{DefaultMemoryImpl, memory_manager::MemoryManager};

/// Virtual memory range used by the file system
const FS_MEMORY_RANGE: Range<u8> = 101..119;

/// Dedicated virtual memory used to store the database
const DEFAULT_MOUNTED_DB_ID: u8 = 120;

/// Database file name
const DEFAULT_DB_FILE_NAME: &str = "./DB/main.db";

// Re-export some of the core dependencies for others to use
pub use ic_wasi_polyfill;
pub use rusqlite;
pub use rusqlite::*;

thread_local! {

    /// Initialize memory manager
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = {

        let m = MemoryManager::init(DefaultMemoryImpl::default());

        // initialize ic-wasi-polyfill
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[("SQLITE_TMPDIR", "tmp")], &m, FS_MEMORY_RANGE);

        // add tmp folder
        let _ = std::fs::create_dir_all("tmp");

        RefCell::new(m)
    };

    /// Active connection
    pub static CONNECTION: RefCell<Option<Connection>> = RefCell::new(Some(create_connection()));

    /// Preconfigured connection that will be used on the next database Open call
    pub static CONNECTION_SETUP: RefCell<ConnectionConfig> = RefCell::new(ConnectionConfig::default());
}

/// Connection configuration for a database connection
#[derive(Clone, Debug)]
pub struct ConnectionConfig {
    /// File name of the database
    pub db_file_name: String,
    /// The ID of the mounted virtual memory for the database storage
    pub db_file_mount_id: Option<u8>,
    /// Default pragma settings to activate right after connection
    pub pragma_settings: HashMap<String, String>,
}

impl ConnectionConfig {
    pub fn new() -> Self {
        ConnectionConfig::default()
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        let mut default_pragmas = HashMap::new();

        // do not create and destroy the journal file every time, set its size to 0 instead
        default_pragmas.insert("journal_mode".to_string(), "PERSIST".to_string());

        // reduce synchronizations
        default_pragmas.insert("synchronous".to_string(), "OFF".to_string());

        // reduce locks and unlocks count
        default_pragmas.insert("locking_mode".to_string(), "EXCLUSIVE".to_string());

        // temp_store = MEMORY, disables creating temp files, improves performance,
        // this workaround also avoids sqlite error on creating a tmp file on complex queries
        default_pragmas.insert("temp_store".to_string(), "MEMORY".to_string());

        // default page size
        default_pragmas.insert("page_size".to_string(), "4096".to_string());

        // minimize disk reads and mostly work in canister memory instead
        default_pragmas.insert("cache_size".to_string(), "500000".to_string());

        Self {
            db_file_name: DEFAULT_DB_FILE_NAME.to_string(),
            db_file_mount_id: Some(DEFAULT_MOUNTED_DB_ID),
            pragma_settings: default_pragmas,
        }
    }
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

/// Get a clone of the current connection configuration
pub fn get_connection_config() -> ConnectionConfig {
    CONNECTION_SETUP.with(|setup| setup.borrow().clone())
}

/// Replace the current connection setup with a new one
pub fn set_connection_config(new_setup: ConnectionConfig) {
    CONNECTION_SETUP.with(|setup| {
        let mut s = setup.borrow_mut();
        *s = new_setup
    });
}

fn prepare_db_folder(db_file_name: &str) {
    // create DB folder before mounting and connecting to the database
    let path = Path::new(db_file_name);
    let parent = path.parent();

    if let Some(parent) = parent {
        let res = create_dir_all(parent);
        if let Err(er) = res {
            println!("Error creating parent folder {er:?}");
        }
    }

    // remove lock if it exists
    let _ = std::fs::remove_dir_all(format!("{}.lock", db_file_name));
}

fn remount_db_file(db_file_name: &str, mount_id: Option<u8>) {
    // unmount old mount, in case it was created before
    ic_wasi_polyfill::unmount_memory_file(db_file_name);

    if let Some(mount_id) = mount_id {
        let memory = MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(mount_id)));

        // dedicate a virtual memory to the database file
        let r = ic_wasi_polyfill::mount_memory_file(
            db_file_name,
            Box::new(memory),
            ic_wasi_polyfill::MountedFileSizePolicy::MemoryPages,
        );

        if r > 0 {
            println!("Error {r}: failed mounting database file {db_file_name}!");
        }
    }
}

fn create_connection() -> Connection {
    // init file system
    MEMORY_MANAGER.with_borrow(|_m| {
        // init file system
    });

    let setup = get_connection_config();

    // prepare database folder
    prepare_db_folder(&setup.db_file_name);

    // try mounting database
    remount_db_file(&setup.db_file_name, setup.db_file_mount_id);

    // Create a new connection to the DB file
    let conn =
        rusqlite::Connection::open(setup.db_file_name).expect("Failed opening the database!");

    // set pragmas
    for (k, v) in &setup.pragma_settings {
        conn.pragma_update(None, k, v as &dyn ToSql).unwrap();
    }

    conn
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
