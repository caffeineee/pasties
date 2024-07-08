//! `model` manages the CRUD loop for pastes
//! It also handles the logic before database insertions

use core::fmt;
use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{
    database::Database,
    utility::{self, hash_string, pseudoid, unix_timestamp},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The complete representation of a paste from the database
pub struct Paste {
    pub id:             i64,
    pub url:            String,
    pub password_hash:  String,
    pub content:        String,
    pub date_published: i64,
    pub date_edited:    i64,
}

impl Paste {
    pub fn to_paste_return(&self) -> PasteReturn {
        PasteReturn {
            url:            self.url.clone(),
            content:        self.content.clone(),
            date_published: self.date_published,
            date_edited:    self.date_edited,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
/// Paste data specified by the user, to be handled by `PasteManager`
pub struct PasteCreate {
    pub url:     String,
    pub content: String,
    password:    String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Paste data to be served to the end user
pub struct PasteReturn {
    pub url:            String,
    pub content:        String,
    pub date_published: i64,
    pub date_edited:    i64,
}

#[derive(Serialize, Deserialize, Debug)]
/// Data to delete a paste specified by the user, to be handled by `PasteManager`
pub struct PasteDelete {
    pub url:      String,
    pub password: String,
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
            Self::Other => write!(f, "An unspecified error occured with the paste manager"),
        }
    }
}

/// CRUD manager for pastes
#[derive(Clone)]
pub struct PasteManager {
    database: Database,
}

impl PasteManager {
    /// **Returns:** a new instance of `PasteManager`
    pub async fn init() -> Self {
        Self {
            database: Database::init().await,
        }
    }

    /// Creates a new `Paste` from the input `PasteCreate`
    ///
    /// **Arguments**:
    /// * `paste`: a `PasteCreate` instance
    ///
    /// **Returns:** `Result<(), PasteError>`
    pub async fn create_paste(&self, mut paste: PasteCreate) -> Result<(), PasteError> {
        // check lengths
        if paste.url.len() > 250 {
            return Err(PasteError::InvalidUrl);
        }
        if paste.content.len() > 200_000 || paste.content.len() == 0 {
            return Err(PasteError::InvalidContent);
        }
        if !paste
            .url
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(PasteError::InvalidUrl);
        }

        // Provide defaults
        if paste.url.is_empty() {
            paste.url = utility::random_string().chars().take(10).collect();
        }
        if paste.password.is_empty() {
            paste.password = utility::random_string().chars().take(10).collect();
        }

        if let Ok(_) = self.database.retrieve_paste(&paste.url).await {
            return Err(PasteError::AlreadyExists);
        }

        let paste_to_insert = Paste {
            id:             pseudoid().abs(),
            url:            paste.url,
            content:        paste.content,
            password_hash:  hash_string(paste.password),
            date_published: unix_timestamp(),
            date_edited:    unix_timestamp(),
        };

        // TODO: Handle errors out here
        match self.database.insert_paste(paste_to_insert).await {
            Ok(_) => Ok(()),
            Err(e) => Err(PasteError::Other),
        }
    }

    /// Retrieves a `Paste` from `PasteManager` by its `url`
    ///
    /// **Arguments**:
    /// * `paste_url`: a paste's custom URL
    ///
    /// **Returns:** `Result<PasteReturn, PasteError>`
    pub async fn get_paste_by_url(&self, paste_url: String) -> Result<PasteReturn, PasteError> {
        let searched_paste = self.database.retrieve_paste(&paste_url);
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
        let existing_paste = match self.database.retrieve_paste(&paste_to_delete.url).await {
            Ok(p) => p,
            Err(_) => return Err(PasteError::NotFound),
        };
        if hash_string(paste_to_delete.password) != existing_paste.password_hash {
            return Err(PasteError::PasswordIncorrect);
        }
        match self.database.delete_paste(&paste_to_delete.url).await {
            Ok(_) => Ok(()),
            Err(_) => Err(PasteError::Other),
        }
    }
}
