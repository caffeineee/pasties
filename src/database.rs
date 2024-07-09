//! `database` is responsible for handling a connection to an SQLite database that stores pastes

use sqlx::{Row, SqlitePool};

use crate::model::{Paste, PasteReturn};

#[derive(Debug)]
pub enum DatabaseError {
    Retrieval(sqlx::Error),
    Insert(sqlx::Error),
    Delete(sqlx::Error),
    BadRequest(sqlx::Error),
}

pub async fn init_database() -> SqlitePool {
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

/// Creates a new paste record in the database using the specified pool.
///
/// **Arguments**
/// * `pool`: an `&SqlitePool` reference towards the database
/// * `paste`: a `Paste` struct to create a record of
pub async fn insert_paste(pool: &SqlitePool, paste: Paste) -> Result<PasteReturn, DatabaseError> {
    let paste_return = paste.clone().into();
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
        Ok(_) => Ok(paste_return),
        Err(e) => Err(DatabaseError::Insert(e)),
    }
}

/// Deletes a paste from the database using the specified pool.
///
/// **Arguments**
/// * `pool`: an `&SqlitePool` reference towards the database
/// * `url`: a paste's custom URL that uniquely identifies it
pub async fn delete_paste(pool: &SqlitePool, url: &String) -> Result<(), DatabaseError> {
    let query = "delete from pastes where url=?";
    match sqlx::query(query).bind(url).execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(DatabaseError::Delete(e)),
    }
}

/// Fetches a paste from the database using the specified pool.
///
/// **Arguments**
/// * `pool`: an `&SqlitePool` reference towards the database
/// * `url`: a paste's custom URL
pub async fn retrieve_paste(pool: &SqlitePool, url: &String) -> Result<Paste, DatabaseError> {
    let query = "select * from pastes where url=?1";
    match sqlx::query(query).bind(url).fetch_one(pool).await {
        Ok(row) => Ok(Paste {
            id:             row.get("id"),
            url:            row.get("url"),
            password_hash:  row.get("password"),
            content:        row.get("content"),
            date_published: row.get("date_published"),
            date_edited:    row.get("date_edited"),
        }),
        Err(e) => Err(DatabaseError::Insert(e)),
    }
}
