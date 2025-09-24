#[macro_use]
extern crate serde;

use candid::CandidType;
use ic_cdk::call::RejectCode;
use ic_cdk::init;
use ic_cdk::post_upgrade;
use ic_cdk::pre_upgrade;
use ic_cdk::query;
use ic_cdk::update;
use ic_rusqlite::with_connection;

use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

const DB_FILENAME: &str = "/my_custom_path/my_base.db";
const CHUNK_SIZE: usize = 2000000; // 2 MB for uploading and downloading the database

#[init]
fn init() {
    // default configuration
    let mut config = ic_rusqlite::ConnectionConfig::new();

    // optionally, create a custom connection to a database different from the default one
    config.db_file_name = DB_FILENAME.to_string(); // some custom path to the database
    config.db_file_mount_id = Some(150); // store database in the virtual memory ID 150
    config
        .pragma_settings
        .insert("cache_size".to_string(), "10000".to_string()); // modify the default pragma settings

    ic_rusqlite::set_connection_config(config);
}

#[pre_upgrade]
fn pre_upgrade() {
    // closing connection explicitly unlocks the database before canister upgrade
    ic_rusqlite::close_connection();
}

#[post_upgrade]
fn post_upgrade() {
    // same initialization
    init();
}

// Basic implementation for downloading the database using the icml tool
// The real implementation should keep canister in "service" mode to prevent database updates during download,
// also make sure only the owner of the canister can call this method
#[query]
fn db_download(offset: u64) -> Vec<u8> {
    ic_rusqlite::close_connection();

    let mut file = match File::open(DB_FILENAME) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    // Get file length
    let file_len = match file.metadata() {
        Ok(meta) => meta.len(),
        Err(_) => return Vec::new(),
    };

    if offset >= file_len {
        return Vec::new();
    }

    // Seek to the requested offset
    if file.seek(SeekFrom::Start(offset)).is_err() {
        return Vec::new();
    }

    let mut buffer = Vec::with_capacity(CHUNK_SIZE);
    let mut handle = file.take(CHUNK_SIZE as u64);

    if handle.read_to_end(&mut buffer).is_err() {
        return Vec::new();
    }

    buffer
}

// Basic implementation to upload the database using the icml tool
// The real implementation should keep canister in "service" mode to prevent database updates during upload
// also make sure only the owner of the canister can call this method
#[update]
fn db_upload(offset: u64, content: Vec<u8>) {
    ic_rusqlite::close_connection();

    // open file for writing
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true) // create file if it doesn't exist
        .truncate(true)
        .open(DB_FILENAME)
    {
        if file.seek(SeekFrom::Start(offset)).is_ok() {
            // write bytes at given offset
            let _ = file.write_all(&content);
        }
    }
}

#[update]
fn create() -> Result {
    with_connection(|conn| {
        // create database table
        match conn.execute(
            "CREATE TABLE IF NOT EXISTS person (
            id   INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER
        )",
            [],
        ) {
            Ok(e) => Ok(format!("{:?}", e)),
            Err(err) => Err(Error::CanisterError {
                message: format!("{:?}", err),
            }),
        }
    })
}

#[update]
fn ls_path(path: String) -> Vec<String> {
    let mut entries = Vec::new();

    if let Ok(read_dir) = std::fs::read_dir(std::path::Path::new(&path)) {
        for entry in read_dir {
            if let Ok(entry) = entry {
                if let Some(name) = entry.path().to_str() {
                    entries.push(name.to_string());
                }
            }
        }
    }

    entries
}

type QueryResult<T = Vec<Vec<String>>, E = Error> = std::result::Result<T, E>;

#[ic_cdk::update]
fn execute(sql: String) -> Result {
    with_connection(|conn| match conn.execute(&sql, []) {
        Ok(_) => Ok(format!(
            "execute performance_counter: {:?}",
            ic_cdk::api::performance_counter(0)
        )),
        Err(err) => Err(Error::CanisterError {
            message: format!("execute: {err:?}"),
        }),
    })
}

#[ic_cdk::update]
fn query(sql: String) -> QueryResult {
    // get connection
    ic_rusqlite::with_connection(|conn| {
        let mut stmt = conn.prepare(&sql).unwrap();
        let cnt = stmt.column_count();
        let mut rows = stmt.query([]).unwrap();
        let mut res: Vec<Vec<String>> = Vec::new();

        loop {
            match rows.next() {
                Ok(row) => match row {
                    Some(row) => {
                        let mut vec: Vec<String> = Vec::new();
                        for idx in 0..cnt {
                            let v = row.get_ref_unwrap(idx);
                            match v.data_type() {
                                ic_rusqlite::rusqlite::types::Type::Null => {
                                    vec.push(String::from(""))
                                }
                                ic_rusqlite::rusqlite::types::Type::Integer => {
                                    vec.push(v.as_i64().unwrap().to_string())
                                }
                                ic_rusqlite::rusqlite::types::Type::Real => {
                                    vec.push(v.as_f64().unwrap().to_string())
                                }
                                ic_rusqlite::rusqlite::types::Type::Text => {
                                    vec.push(v.as_str().unwrap().parse().unwrap())
                                }
                                ic_rusqlite::rusqlite::types::Type::Blob => {
                                    vec.push(hex::encode(v.as_blob().unwrap()))
                                }
                            }
                        }
                        res.push(vec)
                    }
                    None => break,
                },
                Err(err) => {
                    return Err(Error::CanisterError {
                        message: format!("{err:?}"),
                    });
                }
            }
        }
        Ok(res)
    })
}

#[query]
fn query_filter(params: FilterParams) -> Result {
    with_connection(|conn| {
        let mut stmt = match conn.prepare("select * from person where name=?1") {
            Ok(e) => e,
            Err(err) => {
                return Err(Error::CanisterError {
                    message: format!("{:?}", err),
                });
            }
        };

        let person_iter = match stmt.query_map((params.name,), |row| {
            Ok(PersonQuery {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                age: row.get(2).unwrap(),
            })
        }) {
            Ok(e) => e,
            Err(err) => {
                return Err(Error::CanisterError {
                    message: format!("{:?}", err),
                });
            }
        };
        let mut persons = Vec::new();
        for person in person_iter {
            persons.push(person.unwrap());
        }
        let res = serde_json::to_string(&persons).unwrap();
        Ok(res)
    })
}

#[update]
fn insert(person: Person) -> Result {
    with_connection(|conn| {
        // execute insertion query
        match conn.execute(
            "INSERT INTO person (name, age) values (?1, ?2);",
            (person.name, person.age),
        ) {
            Ok(e) => Ok(format!("{:?}", e)),
            Err(err) => Err(Error::CanisterError {
                message: format!("{:?}", err),
            }),
        }
    })
}

#[update]
fn delete(id: usize) -> Result {
    with_connection(
        |conn| match conn.execute("delete from person where id=?1", (id,)) {
            Ok(e) => Ok(format!("{:?}", e)),

            Err(err) => Err(Error::CanisterError {
                message: format!("{:?}", err),
            }),
        },
    )
}

#[update]
fn update(params: UpdateParams) -> Result {
    with_connection(|conn| {
        match conn.execute(
            "update person set name=?1 where id=?2",
            (params.name, params.id),
        ) {
            Ok(e) => Ok(format!("{:?}", e)),
            Err(err) => Err(Error::CanisterError {
                message: format!("{:?}", err),
            }),
        }
    })
}

#[derive(CandidType, Debug, Serialize, Deserialize, Default)]
struct Person {
    name: String,
    age: usize,
}

#[derive(CandidType, Debug, Serialize, Deserialize, Default)]
struct PersonQuery {
    id: usize,
    name: String,
    age: usize,
}

#[derive(CandidType, Debug, Serialize, Deserialize, Default)]
struct QueryParams {
    limit: usize,
    offset: usize,
}

#[derive(CandidType, Debug, Serialize, Deserialize, Default)]
struct FilterParams {
    name: String,
}

#[derive(CandidType, Debug, Serialize, Deserialize, Default)]
struct UpdateParams {
    id: usize,
    name: String,
}

#[derive(CandidType, Deserialize)]
enum Error {
    InvalidCanister,
    CanisterError { message: String },
}

type Result<T = String, E = Error> = std::result::Result<T, E>;

impl From<(RejectCode, String)> for Error {
    fn from((code, message): (RejectCode, String)) -> Self {
        match code {
            RejectCode::CanisterError => Self::CanisterError { message },
            _ => Self::InvalidCanister,
        }
    }
}
