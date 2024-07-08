//! `database` is responsible for handling a connection to an SQLite database that stores pastes

use sqlx::{Row, SqlitePool};

use crate::model::{Paste, PasteReturn};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

#[derive(Debug)]
pub enum DatabaseError {
    Retrieval(sqlx::Error),
    Insert(sqlx::Error),
    Delete(sqlx::Error),
    BadRequest(sqlx::Error),
}

async fn init_database() -> SqlitePool {
    // Connect to the SQLite
    let pool = match SqlitePool::connect(&format!("sqlite://main.db")).await {
        Err(e) => panic!("Failed to connect to the database with the following error:\n    {e}"),
        Ok(pool) => pool,
    };
    // Create schema
    let res = sqlx::query(
        "create table if not exists pastes (
            primary_key    integer primary key,
            id             integer,
            url            text,
            password       text,
            content        text,
            date_published integer,
            date_edited    integer
         )",
    )
    .execute(&pool)
    .await;
    match res {
        Err(e) => panic!(
            "Failed to connect to the pastes table in the database with the following error:\n    {e}"
        ),
        Ok(_) => pool,
    }
}

impl Database {
    /// Initializes a `Database` struct, creates a `pastes` table if there isn't one already, and returns `Self` with the connection pool field (`self.pool`) filled
    pub async fn init() -> Self {
        Self {
            pool: init_database().await,
        }
    }
    /// Creates a new paste record in the database.
    ///
    /// **Arguments**
    /// * `paste`: a `Paste` struct to create a record of
    pub async fn insert_paste(&self, paste: Paste) -> Result<PasteReturn, DatabaseError> {
        let query = "insert into pastes(
                id,
                url,  
                password,  
                content,  
                date_published,  
                date_edited
            ) values (?, ?, ?, ?, ?, ?)";
        let paste_return = paste.to_paste_return();
        match sqlx::query(query)
            .bind(paste.id)
            .bind(paste.url)
            .bind(paste.password_hash)
            .bind(paste.content)
            .bind(paste.date_published)
            .bind(paste.date_edited)
            .execute(&self.pool)
            .await
        {
            Ok(_) => Ok(paste_return),
            Err(e) => Err(DatabaseError::Insert(e)),
        }
    }
    /// Deletes a paste from the database
    ///
    /// **Arguments**
    /// * `url`: a paste's custom URL
    ///
    /// **Returns:** `Result<(), DatabaseError>`
    pub async fn delete_paste(&self, url: &String) -> Result<(), DatabaseError> {
        let query = "delete from pastes where url=?";
        match sqlx::query(query).bind(url).execute(&self.pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(DatabaseError::Delete(e)),
        }
    }
    /// Fetches a paste from the database.
    ///
    /// **Arguments**
    /// * `url`: a paste's custom URL
    ///
    /// **Returns:** `Result<Paste, DatabaseError>`
    pub async fn retrieve_paste(&self, url: &String) -> Result<Paste, DatabaseError> {
        match sqlx::query("select * from pastes where url=?1")
            .bind(url)
            .fetch_one(&self.pool)
            .await
        {
            Err(e) => Err(DatabaseError::Insert(e)),
            Ok(row) => Ok(Paste {
                id:             row.get("id"),
                url:            row.get("url"),
                password_hash:  row.get("password"),
                content:        row.get("content"),
                date_published: row.get("date_published"),
                date_edited:    row.get("date_edited"),
            }),
        }
    }
}
