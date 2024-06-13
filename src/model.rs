//! `model` manages the CRUD loop for pastes
//! It also handles the logic before database insertions

use core::fmt;
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{
    database::Database,
    utility::{hash_string, unix_timestamp},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The complete representation of a paste from the database
pub struct Paste {
    pub id:             Option<i64>,
    pub url:            String,
    pub content:        String,
    pub password_hash:  String,
    pub date_published: u64,
    pub date_edited:    u64,
}

#[derive(Serialize, Deserialize, Debug)]
/// Paste data specified by the user, to be handled by `PasteManager`
pub struct PasteCreate {
    url:      String,
    content:  String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Paste data to be served to the end user
pub struct PasteReturn {
    pub url:            String,
    pub content:        String,
    pub date_published: u64,
    pub date_edited:    u64,
}

#[derive(Serialize, Deserialize, Debug)]
/// Data to delete a paste specified by the user, to be handled by `PasteManager`
pub struct PasteDelete {
    pub url:      String,
    pub password: String,
}

/// Various errors that may occur while processing requests made through `PasteManager`
pub enum PasteError {
    PasswordIncorrect,
    AlreadyExists,
    NotFound,
    Other,
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
            Self::PasswordIncorrect => write!(f, "The specified password is incorrect"),
            Self::Other => write!(f, "An unspecified error occured with the paste manager"),
        }
    }
}

#[derive(Clone)]
pub struct PasteManager {
    database: Arc<Mutex<Database>>,
}

/// CRUD manager for pastes
impl PasteManager {
    /// **Returns:** a new instance of `PasteManager`
    pub async fn init() -> Self {
        Self {
            database: Arc::new(Mutex::new(Database::init())),
        }
    }

    /// Creates a new `Paste` from the input `PasteCreate`
    ///
    /// **Arguments**:
    /// * `paste`: a `PasteCreate` instance
    ///
    /// **Returns:** `Result<(), PasteError>`
    pub async fn create_paste(&self, paste: PasteCreate) -> Result<(), PasteError> {
        if let Ok(_) = self.database.lock().unwrap().retrieve_paste(&paste.url) {
            return Err(PasteError::AlreadyExists);
        }

        let paste_to_insert = Paste {
            id:             None,
            url:            paste.url,
            content:        paste.content,
            password_hash:  hash_string(paste.password),
            date_published: unix_timestamp(),
            date_edited:    unix_timestamp(),
        };

        // TODO: Handle errors out here
        match self.database.lock().unwrap().insert_paste(paste_to_insert) {
            Ok(_) => Ok(()),
            Err(_) => Err(PasteError::Other),
        }
    }

    /// Retrieves a `Paste` from `PasteManager` by its `url`
    ///
    /// **Arguments**:
    /// * `paste_url`: a paste's custom URL
    ///
    /// **Returns:** `Result<PasteReturn, PasteError>`
    pub async fn get_paste_by_url(&self, paste_url: String) -> Result<PasteReturn, PasteError> {
        let searched_paste = self.database.lock().unwrap().retrieve_paste(&paste_url);
        match searched_paste {
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
    pub async fn delete_paste_by_url(
        &self,
        paste_to_delete: PasteDelete,
    ) -> Result<(), PasteError> {
        let existing_paste = match self
            .database
            .lock()
            .unwrap()
            .retrieve_paste(&paste_to_delete.url)
        {
            Ok(p) => p,
            Err(_) => return Err(PasteError::NotFound),
        };
        if hash_string(paste_to_delete.password) != existing_paste.password_hash {
            return Err(PasteError::PasswordIncorrect);
        }
        match self
            .database
            .lock()
            .unwrap()
            .delete_paste(&paste_to_delete.url)
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PasteError::Other),
        }
    }
}
