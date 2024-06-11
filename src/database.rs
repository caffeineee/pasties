//! `database` is responsible for handling a connection to an SQLite database that stores pastes

use rusqlite::{params, Connection};

use crate::model::Paste;

pub struct Database {
    connection: Connection,
}

pub enum DatabaseError {
    Retrieval(rusqlite::Error),
    Insert(rusqlite::Error),
    Delete(rusqlite::Error),
    BadRequest,
}

impl Database {
    /// Initializes a `Database` struct, creates a `pastes` table if there isn't one already, and returns `Self` with the connection field filled
    pub fn init() -> Self {
        let c = Connection::open("main.db").unwrap();
        match c.execute(
            "create table if not exists pastes (
                 id             integer primary key,
                 url            text,
                 password       text,
                 content        text,
                 date_published integer,
                 date_edited    integer
             )",
            (),
        ) {
            Ok(_) => println!("Successfully connected to table."),
            Err(e) => panic!("Database creation failed with error message: {e}"),
        }
        Self { connection: c }
    }
    /// Creates a new paste record in the database.
    ///
    /// **Arguments**
    /// * `paste`: a `Paste` struct to create a record of
    ///
    /// **Returns:** `Result<i64, DatabaseError>` where the i64 is the paste's unique ID returned by the database
    pub fn insert_paste(&self, paste: Paste) -> Result<i64, DatabaseError> {
        match self.connection.execute(
            "insert into pastes(url,  password,  content,  date_published,  date_edited)
                         values (?1, ?2, ?3, ?4, ?5)",
            params![
                paste.url,
                paste.password_hash,
                paste.content,
                paste.date_published,
                paste.date_edited
            ],
        ) {
            Ok(_) => Ok(self.connection.last_insert_rowid()),
            Err(e) => Err(DatabaseError::Insert(e)),
        }
    }
    /// Deletes a paste from the database
    ///
    /// **Arguments**
    /// * `url`: a paste's custom URL
    ///
    /// **Returns:** `Result<(), DatabaseError>`
    pub fn delete_paste(&self, url: &String) -> Result<(), DatabaseError> {
        match self
            .connection
            .execute("delete from pastes where url=?1", [url])
        {
            Ok(_) => Ok(()),
            Err(e) => Err(DatabaseError::Delete(e)),
        }
    }
    /// Creates a new paste record in the database.
    ///
    /// **Arguments**
    /// * `url`: a paste's custom URL
    ///
    /// **Returns:** `Result<Paste, DatabaseError>`
    pub fn retrieve_paste(&self, url: &String) -> Result<Paste, DatabaseError> {
        let mut query = self
            .connection
            .prepare("select * from pastes where url=?1")
            .unwrap();
        let paste = query.query_row([url], |p| {
            Ok(Paste {
                id:             p.get(0)?,
                url:            p.get(1)?,
                password_hash:  p.get(2)?,
                content:        p.get(3)?,
                date_published: p.get(4)?,
                date_edited:    p.get(5)?,
            })
        });
        match paste {
            Ok(p) => Ok(p),
            Err(e) => Err(DatabaseError::Delete(e)),
        }
    }
}
