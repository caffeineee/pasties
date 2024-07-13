//! `model` manages the CRUD loop for pastes
//! It also handles the logic before database operations

use core::fmt;
use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::{
    database,
    utility::{self, hash_string, pseudoid, unix_timestamp},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The complete representation of a paste from the database. Used to interface with the `database` module, is never served to the end user
pub struct Paste {
    pub id:             i64,
    pub url:            String,
    pub password_hash:  String,
    pub content:        String,
    pub date_published: i64,
    pub date_edited:    i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Partial representation of a paste from the database. Used to handle interactions with existing pastes. Used to interface with the `database` module, is never served to the end user
pub struct ExistingPaste {
    pub url:           String,
    pub password_hash: String,
    pub content:       String,
    pub date_edited:   i64,
}

#[derive(Serialize, Deserialize, Debug)]
/// Paste data specified by the user to be used to create a new paste
pub struct PasteCreate {
    pub url:     String,
    pub content: String,
    password:    String,
}

#[derive(Serialize, Deserialize, Debug)]
/// Paste data specified by the user, used to update existing pastes
pub struct PasteUpdate {
    pub url:     String,
    pub content: String,
    password:    String,
}

#[derive(Serialize, Deserialize, Debug)]
/// Paste data specified by the user, used to delete a paste
pub struct PasteDelete {
    pub url:      String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Paste data to be served to the end user, containing information meant to be displayed by a client
pub struct PasteReturn {
    pub url:            String,
    pub content:        String,
    pub date_published: i64,
    pub date_edited:    i64,
}

impl From<Paste> for PasteReturn {
    fn from(paste: Paste) -> Self {
        Self {
            url:            paste.url,
            content:        paste.content,
            date_published: paste.date_published,
            date_edited:    paste.date_edited,
        }
    }
}

/// Various errors that may occur while processing requests made through `PasteManager`
pub enum PasteError {
    //GET
    NotFound,

    // POST
    AlreadyExists,
    InvalidContent,
    InvalidUrl,

    // PATCH, DELETE
    PasswordIncorrect,

    // other...
    Other(String),
}

impl IntoResponse for PasteError {
    fn into_response(self) -> Response {
        use crate::model::PasteError::*;
        match self {
            NotFound => (
                StatusCode::NOT_FOUND,
                "No paste with this URL has been found",
            )
                .into_response(),
            AlreadyExists => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "A paste with this URL already exists",
            )
                .into_response(),
            InvalidContent => (
                StatusCode::BAD_REQUEST,
                "The specified content is invalid, or is the wrong length",
            )
                .into_response(),
            InvalidUrl => (
                StatusCode::BAD_REQUEST,
                "The specified URL is invalid, or is the wrong length",
            )
                .into_response(),
            PasswordIncorrect => (
                StatusCode::UNAUTHORIZED,
                "The specified password is incorrect",
            )
                .into_response(),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unspecified error occured with the paste manager",
            )
                .into_response(),
        }
    }
}

impl Display for PasteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "No paste with this URL has been found"),
            Self::AlreadyExists => {
                write!(f, "A paste with this URL already exists")
            }
            Self::InvalidContent => write!(
                f,
                "The specified content is invalid, or is the wrong length"
            ),
            Self::InvalidUrl => write!(f, "The specified URL is invalid, or is the wrong length"),
            Self::PasswordIncorrect => write!(f, "The specified password is incorrect"),
            Self::Other(e) => write!(f, "An unspecified error occured with the paste manager.\nThe following error was passed: {:?}", e),
        }
    }
}

/// CRUD manager for pastes
#[derive(Clone)]
pub struct PasteManager {
    pool: SqlitePool, // SqlitePool <=> Pool<Sqlite>
}

impl PasteManager {
    /// **Returns:** a new instance of `PasteManager`
    pub async fn init() -> Self {
        Self {
            pool: database::init_database().await,
        }
    }

    /// Creates a new `Paste` from the input `PasteCreate`
    ///
    /// **Arguments**:
    /// * `paste`: a `PasteCreate` instance
    ///
    /// **Returns:** `Result<(), PasteError>`
    pub async fn create_paste(
        &self,
        mut paste: PasteCreate,
    ) -> Result<(String, String), PasteError> {
        // Provide default URL
        if paste.url.is_empty() {
            let mut random_id: String = utility::random_string().chars().take(10).collect();
            while database::retrieve_paste(&self.pool, &random_id)
                .await
                .is_ok()
            {
                random_id = utility::random_string().chars().take(10).collect();
            }
            paste.url = random_id;
        }
        // Throw an error if a paste with the specified URL already exists
        if database::retrieve_paste(&self.pool, &paste.url)
            .await
            .is_ok()
        {
            return Err(PasteError::AlreadyExists);
        }
        // Provide default password
        if paste.password.is_empty() {
            paste.password = utility::random_string().chars().take(10).collect();
        }
        // validate `paste.url`
        if paste.url.len() > 250 {
            return Err(PasteError::InvalidUrl);
        }
        if !paste
            .url
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(PasteError::InvalidUrl);
        }
        // validate `paste.content` by length
        if paste.content.len() > 200_000 || paste.content.is_empty() {
            return Err(PasteError::InvalidContent);
        }
        let paste_to_insert = Paste {
            id:             pseudoid(),
            url:            paste.url.to_owned(),
            password_hash:  hash_string(paste.password.to_owned()),
            content:        paste.content,
            date_published: unix_timestamp(),
            date_edited:    unix_timestamp(),
        };
        match database::insert_paste(&self.pool, paste_to_insert).await {
            Ok(_) => Ok((paste.url, paste.password)),
            Err(e) => Err(PasteError::Other(format!("{:?}", e))),
        }
    }

    /// Updates a `Paste` in `PasteManager` by its `url`
    ///
    /// **Arguments**:
    /// * `paste_url`: a paste's custom URL
    ///
    /// **Returns:** `Result<PasteReturn, PasteError>`
    pub async fn update_paste(&self, paste: PasteUpdate) -> Result<(), PasteError> {
        let original_paste = match database::retrieve_paste(&self.pool, &paste.url).await {
            Ok(paste) => paste,
            Err(_) => return Err(PasteError::NotFound),
        };
        if hash_string(paste.password.to_owned()) != original_paste.password_hash {
            return Err(PasteError::PasswordIncorrect);
        }
        if paste.content.len() > 200_000 || paste.content.is_empty() {
            return Err(PasteError::InvalidContent);
        }
        let updated_paste = ExistingPaste {
            url:           paste.url.to_owned(),
            password_hash: hash_string(paste.password),
            content:       paste.content,
            date_edited:   unix_timestamp(),
        };
        match database::update_paste(&self.pool, paste.url, updated_paste).await {
            Ok(_) => Ok(()),
            Err(e) => Err(PasteError::Other(format!("{:?}", e))),
        }
    }

    /// Retrieves a `Paste` from `PasteManager` by its `url`
    ///
    /// **Arguments**:
    /// * `paste_url`: a paste's custom URL
    ///
    /// **Returns:** `Result<PasteReturn, PasteError>`
    pub async fn get_paste_by_url(&self, paste_url: String) -> Result<PasteReturn, PasteError> {
        let searched_paste = database::retrieve_paste(&self.pool, &paste_url);
        match searched_paste.await {
            Ok(p) => Ok(PasteReturn {
                url:            p.url.to_owned(),
                content:        p.content.to_owned(),
                date_published: p.date_published,
                date_edited:    p.date_edited,
            }),
            Err(_) => Err(PasteError::NotFound),
        }
    }

    /// Removes a `Paste` from `PasteManager` by its `url`
    ///
    /// **Arguments**:
    /// * `paste_to_delete`: an instance of `PasteDelete`
    ///
    /// **Returns:** `Option<PasteReturn>`, where `None` signifies that the paste has not been found
    pub async fn delete_paste(&self, paste_to_delete: PasteDelete) -> Result<(), PasteError> {
        let existing_paste = match database::retrieve_paste(&self.pool, &paste_to_delete.url).await
        {
            Ok(p) => p,
            Err(_) => return Err(PasteError::NotFound),
        };
        if hash_string(paste_to_delete.password) != existing_paste.password_hash {
            return Err(PasteError::PasswordIncorrect);
        }
        match database::delete_paste(&self.pool, &paste_to_delete.url).await {
            Ok(_) => Ok(()),
            Err(e) => Err(PasteError::Other(format!("{:?}", e))),
        }
    }
}
