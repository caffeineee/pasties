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
    database::{ClientManager, Database},
    utility::{hash_string, unix_timestamp},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Paste {
    pub id:             Option<i64>,
    pub url:            String,
    pub content:        String,
    pub password_hash:  String,
    pub date_published: u64,
    pub date_edited:    u64,
}

// This is only needed when using Arc<Mutex<Vec<Paste>>>
// It only exists so we can do `paste[field]``
impl std::ops::Index<String> for Paste {
    type Output = String;
    fn index(&self, index: String) -> &Self::Output {
        match index.as_ref() {
            "url" => &self.url,
            _ => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PasteCreate {
    url:      String,
    content:  String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PasteReturn {
    pub url:            String,
    pub content:        String,
    pub date_published: u64,
    pub date_edited:    u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PasteDelete {
    pub url:      String,
    pub password: String,
}

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
            PasswordIncorrect => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "The given password is invalid.",
            )
                .into_response(),
            AlreadyExists => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "A paste with this URL already exists.",
            )
                .into_response(),
            NotFound => (
                StatusCode::NOT_FOUND,
                "No paste with this URL has been found.",
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
            Self::NotFound => write!(
                f,
                "A paste with the specified identifier could not be found"
            ),
            Self::AlreadyExists => {
                write!(f, "A paste with the specified identifier already exists")
            }
            Self::PasswordIncorrect => write!(f, "The specified password is incorrect"),
            Self::Other => write!(f, "An unspecified error occured with the paste manager"),
        }
    }
}

#[derive(Clone)]
pub struct PasteManager {
    // This will eventually be a lot more elaborate as a database is implemented, currently this mock storage is here so I can test the API
    manager:  ClientManager<Paste>,
    database: Arc<Mutex<Database>>,
}

/// CRUD manager for pastes
///
/// TODO: use an actual database instead of in-memory `Arc<Mutex<Vec<Paste>>>`
impl PasteManager {
    /// Returns a new instance of `PasteManager`
    pub async fn init() -> Self {
        Self {
            manager:  ClientManager::new(Arc::default()),
            database: Arc::new(Mutex::new(Database::init())),
        }
    }

    /// Creates a new `Paste` from the input `PasteCreate`
    ///
    /// **Arguments**:
    /// * `paste`: a `PasteCreate` instance
    /// **Returns:** `Result<(), PasteError>`
    pub async fn create_paste(&self, paste: PasteCreate) -> Result<(), PasteError> {
        // make sure paste doesn't already exist
        if let Some(_) = self.database.lock().unwrap().get_paste_by_url(&paste.url) {
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
        self.database.lock().unwrap().insert_paste(paste_to_insert);

        Ok(())
    }

    /// Retrieves a `Paste` from `PasteManager` by its `url`
    ///
    /// **Arguments**:
    /// * `paste_url`: a paste's custom URL
    /// **Returns:** `Result<PasteReturn, PasteError>`
    pub async fn get_paste_by_url(&self, paste_url: String) -> Result<PasteReturn, PasteError> {
        let searched_paste = self.database.lock().unwrap().get_paste_by_url(&paste_url);
        match searched_paste {
            Some(p) => Ok(PasteReturn {
                url:            p.url.to_owned(),
                content:        p.content.to_owned(),
                date_published: p.date_published,
                date_edited:    p.date_edited,
            }),
            None => Err(PasteError::NotFound),
        }
    }

    /// Removes a `Paste` from `PasteManager` by its `url`
    ///
    /// **Arguments**:
    /// * `paste_url`: a paste's custom URL
    /// **Returns:** `Option<PasteReturn>`, where `None` signifies that the paste has not been found
    pub async fn delete_paste_by_url(&self, paste: PasteDelete) -> Result<(), PasteError> {
        // make sure paste exists
        let existing = match self.manager.select_single(String::from("url"), &paste.url) {
            Ok(p) => p,
            Err(_) => return Err(PasteError::NotFound),
        };

        // check password
        // in the future, hashes should be compared here
        if paste.password != existing.password_hash {
            return Err(PasteError::PasswordIncorrect);
        }

        // return
        self.manager.remove_single(String::from("url"), &paste.url)
    }
}
