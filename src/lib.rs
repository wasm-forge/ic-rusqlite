use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;

use candid::CandidType;
use candid::Deserialize;
use rusqlite::Connection;

use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::{DefaultMemoryImpl, memory_manager::MemoryManager};

const FS_MOUNT_RANGE: Range<u8> = 200..210;
const DEFAULT_MOUNTED_DB_ID: u8 = 20;
const DB_DEFAULT_FILE_NAME: &str = "db.db3";

// re-export some of the core dependencies for others to use
pub use ic_wasi_polyfill;
pub use rusqlite;

thread_local! {
    pub static DB: RefCell<Option<Connection>> = const { RefCell::new(None) };

    pub static DB_FILE_NAME: RefCell<Option<String>> = const { RefCell::new(None) };

    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = {
        let m = MemoryManager::init(DefaultMemoryImpl::default());

        // initialize ic-wasi-polyfill
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, FS_MOUNT_RANGE);

        RefCell::new(m)
    };
}

#[derive(CandidType, Deserialize)]
enum Error {
    InvalidCanister,
    CanisterError { message: String },
}

pub struct DbConnectionBuilder {
    file_name: String,
    db_mount_id: Option<u8>,
    pragma_settings: HashMap<String, String>,
}

impl DbConnectionBuilder {
    pub fn new() -> Self {
        DbConnectionBuilder::default()
    }
}

impl Default for DbConnectionBuilder {
    fn default() -> Self {
        let mut deafult_pragmas = HashMap::new();

        // do not create and destroy the journal file every time, set its size to 0 instead
        deafult_pragmas.insert("journal_mode".to_string(), "TRUNCATE".to_string());

        // reduce synchronizations
        deafult_pragmas.insert("synchronous".to_string(), "0".to_string());

        // reduce locks and unlocks count
        deafult_pragmas.insert("locking_mode".to_string(), "EXCLUSIVE".to_string());

        // temp_store = MEMORY, disables creating temp files, improves performance,
        // this workaround also avoids sqlite error on creating a tmp file on complex queries
        deafult_pragmas.insert("temp_store".to_string(), "MEMORY".to_string());

        // minimize disk reads and mostly work in canister memory instead
        deafult_pragmas.insert("cache_size".to_string(), "1000000".to_string());

        Self {
            file_name: DB_DEFAULT_FILE_NAME.to_string(),
            db_mount_id: Some(DEFAULT_MOUNTED_DB_ID),
            pragma_settings: deafult_pragmas,
        }
    }
}

impl DbConnectionBuilder {
    pub fn with_file_name(mut self, name: impl Into<String>) -> Self {
        self.file_name = name.into();
        self
    }

    pub fn with_db_mount_id(mut self, id: Option<u8>) -> Self {
        self.db_mount_id = id;
        self
    }

    pub fn with_pragma(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.pragma_settings.insert(key.into(), value.into());
        self
    }

    pub fn connect(self) -> anyhow::Result<()> {
        connect(self)
    }
}

/// connect to a database using the connection parameters provided
pub fn connect(connection_parameters: DbConnectionBuilder) -> anyhow::Result<()> {
    DB.with(|db| {
        let mut db = db.borrow_mut();

        if db.is_some() {
            return Err(anyhow::anyhow!("Database is already connected!"));
        }

        // mount DB file
        MEMORY_MANAGER.with(|m| {
            let m = m.borrow();

            // if there is a mount id provided, create a mounted database file
            if let Some(id) = connection_parameters.db_mount_id {
                let memory = m.get(MemoryId::new(id));
                ic_wasi_polyfill::mount_memory_file(
                    &connection_parameters.file_name,
                    Box::new(memory),
                );
            }
        });

        // connect database
        *db = Some(Connection::open(connection_parameters.file_name)?);

        // set pragmas
        let conn = db.as_ref().expect("DB Connection failed!");

        for (k, v) in &connection_parameters.pragma_settings {
            if !v.is_empty() {
                conn.pragma_update(None, k, v)?;
            }
        }

        Ok(())
    })
}

/// close database connection
pub fn disconnect() -> anyhow::Result<()> {
    DB.with(|db| {
        let mut db = db.borrow_mut();

        if let Some(conn) = db.take() {
            // Try to close the connection if it is open
            conn.close().map_err(|(_conn, err)| anyhow::anyhow!(err))?;

            // close the connection and free the resources
            *db = None;

            // unmount the corresponding file
            DB_FILE_NAME.with(|name| {
                let name = name.borrow();

                if let Some(name) = name.as_ref() {
                    // also close the db mount
                    ic_wasi_polyfill::unmount_memory_file(name);
                }
            })
        }

        Ok(())
    })
}

pub fn execute<P: rusqlite::Params>(sql: &str, p: P) -> rusqlite::Result<usize> {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        db.execute(sql, p)
    })
}

pub fn query<P, F, T>(sql: String, params: P, mut f: F) -> rusqlite::Result<Vec<T>>
where
    P: rusqlite::Params,
    F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
{
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let mut stmt = db.prepare(&sql)?;
        let iter = stmt.query_map(params, |row| f(row))?;

        iter.collect()
    })
}
