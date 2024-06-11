//! `database` is responsible for handling a connection to an SQLite database that stores pastes

use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};

use crate::model::{Paste, PasteError};

pub type Result<T> = std::result::Result<T, PasteError>;

pub struct Database {
    connection: Connection,
}

pub enum DatabaseError {
    Retrieval,
    Insertion,
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
    pub fn insert_paste(&self, paste: Paste) -> i64 {
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
            Ok(_) => (),
            Err(e) => panic!("Database insert failed: {e}"),
        };
        self.connection.last_insert_rowid()
    }
    pub fn get_paste_by_url(&self, url: &String) -> Option<Paste> {
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
        paste.ok()
    }
}

/// Client manager for the database
#[derive(Clone)]
pub struct ClientManager<T> {
    /// Temporary store, it's labeled "pool" but is not currently a pool
    pool: ConnectionPool<T>,
}

type ConnectionPool<T> = Arc<Mutex<Vec<T>>>;

impl<T: std::ops::Index<String, Output = String> + Clone> ClientManager<T> {
    /// Create new [`Client`]
    pub fn new(pool: ConnectionPool<T>) -> Self {
        Self { pool }
    }

    // This is not needed when replaced with SQL methods
    pub fn len(&self) -> usize {
        // obtain client from pool
        let client = self.pool.lock().unwrap();

        // return
        client.len()
    }

    // functions below should be altered when proper database support is added
    // we only really need to select one thing at a time since the api is pretty basic,
    // so these functions only do what is needed ... optionally this could all be replaced
    // with a `run_query` function (or something)

    /// Select by a given `field`
    ///
    /// ## Arguments:
    /// * `field` - the field we are selecting by
    /// * `equals` - what the field value needs to equal
    pub fn select_single(&self, field: String, equals: &str) -> Result<T> {
        // obtain client from pool
        let client = self.pool.lock().unwrap();

        // select
        // (replace with sql "SELECT FROM ... WHERE ... LIMIT 1", this just implements a basic version)
        let entry = client.iter().clone().find(|r| r[field.clone()] == equals);

        match entry {
            // we need T to impl Clone so we can do this
            Some(r) => Ok((*r).to_owned()),
            None => Err(PasteError::NotFound),
        }
    }

    /// Insert `T`
    ///
    /// ## Arguments:
    /// * `value`: `T`
    pub fn insert_row(&self, value: T) -> Result<()> {
        // obtain client from pool
        let mut client = self.pool.lock().unwrap();

        // push and return
        client.push(value);
        Ok(())
    }

    /// Remove row by `field`
    ///
    /// ## Arguments:
    /// * `field` - the field we are selecting by
    /// * `equals` - what the field value needs to equal
    pub fn remove_single(&self, field: String, equals: &str) -> Result<()> {
        // obtain client from pool
        let mut client = self.pool.lock().unwrap();

        // remove
        // (replace with sql "REMOVE FROM ... WHERE ... LIMIT 1", this just implements a basic version)

        // this is very bad and only for testing, it'll go through everything to find what we want
        for (i, row) in client.clone().iter().enumerate() {
            if row[field.clone()] != equals {
                continue;
            }

            client.remove(i);
            break;
        }

        // return
        Ok(())
    }
}
