//#[macro_use]
//extern crate ic_cdk_macros;
#[macro_use]
extern crate serde;

use candid::CandidType;
use ic_cdk::call::RejectCode;
use ic_cdk::query;
use ic_cdk::update;
use ic_rusqlite::with_connection;

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

#[query]
fn query(params: QueryParams) -> Result {
    with_connection(|conn| {
        // prepare statement with parameters
        let mut stmt = match conn.prepare("select * from person limit ?1 offset ?2") {
            Ok(e) => e,
            Err(err) => {
                return Err(Error::CanisterError {
                    message: format!("{:?}", err),
                });
            }
        };

        // query with parameters and process it on a row-by-row basis
        let person_iter = match stmt.query_map((params.limit, params.offset), |row| {
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
