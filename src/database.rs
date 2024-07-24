//! `database` a helper module for handling SQL queries via a connection pool to an SQLite database

use sqlx::{Row, SqlitePool};

use crate::model::{DatabasePaste, PartialDatabasePaste};

#[derive(Debug)]
pub enum DatabaseError {
    Retrieval(sqlx::Error),
    Insert(sqlx::Error),
    Update(sqlx::Error),
    Delete(sqlx::Error),
    BadRequest(sqlx::Error),
}

/// Connects to the database at `<project root>/main.db` and returns an `SqlitePool` for other database helper functions to use
/// Also handles creating the schema for paste storage, if the table does not already exist
/// **Panics** if anything goes wrong, as the lack of an `SqlitePool` is a non-recoverable error for pasties
pub async fn init_database() -> SqlitePool {
    // Connect to the SQLite
    let pool = match SqlitePool::connect("sqlite://main.db").await {
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

/// Creates a new paste record in a database using the specified pool.
///
/// **Arguments**
/// * `pool`: an `&SqlitePool` reference
/// * `paste`: a `DatabasePaste` struct to create a record of
pub async fn insert_paste(pool: &SqlitePool, paste: DatabasePaste) -> Result<(), DatabaseError> {
    let query = "insert into pastes(
        id,
        url,  
        password,  
        content,  
        date_published,  
        date_edited
    ) values (?, ?, ?, ?, ?, ?)";
    match sqlx::query(query)
        .bind(paste.id)
        .bind(paste.url)
        .bind(paste.password_hash)
        .bind(paste.content)
        .bind(paste.date_published)
        .bind(paste.date_edited)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(DatabaseError::Insert(e)),
    }
}

/// Updates a paste in a database using the specified pool.
///
/// **Arguments**
/// * `pool`: an `&SqlitePool` reference
/// * `paste`: a `PartialDatabasePaste` struct
pub async fn update_paste(
    pool: &SqlitePool,
    url: String,
    paste: PartialDatabasePaste,
) -> Result<(), DatabaseError> {
    let query =
        "update pastes set url = ?, password = ?, content = ?, date_edited = ? where url = ?";
    match sqlx::query(query)
        .bind(paste.url)
        .bind(paste.password_hash)
        .bind(paste.content)
        .bind(paste.date_edited)
        .bind(url)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(DatabaseError::Update(e)),
    }
}

/// Deletes a paste from a database using the specified pool. The identification of the paste happens through its URL, which is guaranteed to be unique by the `model` module
///
/// **Arguments**
/// * `pool`: an `&SqlitePool` reference
/// * `url`: a paste's custom URL that uniquely identifies it
pub async fn delete_paste(pool: &SqlitePool, url: &String) -> Result<(), DatabaseError> {
    let query = "delete from pastes where url=?";
    match sqlx::query(query).bind(url).execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(DatabaseError::Delete(e)),
    }
}

/// Fetches a paste from a database using the specified pool. The identification of the paste happens through its URL, which is guaranteed to be unique by the `model` module
///
/// **Arguments**
/// * `pool`: an `&SqlitePool` reference
/// * `url`: a paste's custom URL
pub async fn retrieve_paste(
    pool: &SqlitePool,
    url: &String,
) -> Result<DatabasePaste, DatabaseError> {
    let query = "select * from pastes where url=?1";
    match sqlx::query(query).bind(url).fetch_one(pool).await {
        Ok(row) => Ok(DatabasePaste {
            id:             row.get("id"),
            url:            row.get("url"),
            password_hash:  row.get("password"),
            content:        row.get("content"),
            date_published: row.get("date_published"),
            date_edited:    row.get("date_edited"),
        }),
        Err(e) => Err(DatabaseError::Retrieval(e)),
    }
}
