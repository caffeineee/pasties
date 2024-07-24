//! `model` manages the CRUD loop for pastes
//! It also handles the logic before database operations

use core::fmt;
use std::fmt::Display;

use askama_axum::{IntoResponse, Response};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::{
    database::{self, DatabaseError},
    utility::{self, hash_string, is_url_safe},
};

pub enum PasteError {
    // Errors that may occur when creating a paste
    InvalidUrl,
    InvalidPassword,
    InvalidContent,
    AlreadyExists,
    Database(DatabaseError),
    // todo!()
    NotFound,
    IncorrectPassword,
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
            Self::InvalidPassword => write!(f, "The specified password is invalid, or is the wrong length"),
            Self::IncorrectPassword => write!(f, "The specified password is incorrect"),
            Self::Database(e) => write!(f, "An unspecified error occured with the database.\nThe following error was passed: {:?}", e),
        }
    }
}

impl IntoResponse for PasteError {
    fn into_response(self) -> Response {
        use crate::model::PasteError::*;
        match self {
            NotFound => (StatusCode::NOT_FOUND, format!("{}", NotFound)).into_response(),
            AlreadyExists => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}", AlreadyExists),
            )
                .into_response(),
            InvalidContent => {
                (StatusCode::BAD_REQUEST, format!("{}", InvalidContent)).into_response()
            }
            InvalidUrl => (StatusCode::BAD_REQUEST, format!("{}", InvalidUrl)).into_response(),
            InvalidPassword => {
                (StatusCode::BAD_REQUEST, format!("{}", InvalidPassword)).into_response()
            }
            IncorrectPassword => {
                (StatusCode::UNAUTHORIZED, format!("{}", IncorrectPassword)).into_response()
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unspecified error occured with the paste manager",
            )
                .into_response(),
        }
    }
}

/// Represents the database's paste schema as a struct, excluding the primary key, as a randomly generated i64 ID uniquely identifies any paste
pub struct DatabasePaste {
    pub id:             i64,
    pub url:            String,
    pub content:        String,
    pub password_hash:  String,
    pub date_published: i64,
    pub date_edited:    i64,
}

impl From<NewPasteData> for DatabasePaste {
    fn from(paste: NewPasteData) -> Self {
        DatabasePaste {
            id:             utility::pseudoid(),
            url:            paste.url,
            content:        paste.content,
            password_hash:  utility::hash_string(paste.password),
            date_published: utility::unix_timestamp(),
            date_edited:    utility::unix_timestamp(),
        }
    }
}

/// Represents the "mutable" fields on a paste within the database. Used for interacting with (and editing) existing paste records.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PartialDatabasePaste {
    pub url:           String,
    pub content:       String,
    pub password_hash: String,
    pub date_edited:   i64,
}

/// Data provided by the user to create a new paste from, or update an existing paste with
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewPasteData {
    pub url:      String,
    pub content:  String,
    pub password: String,
}

/// Struct to identify and authorize access to pastes
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PasteCredentials {
    pub url:      String,
    pub password: String,
}

/// Struct to be served to the end user, only contains data that is displayed on the front-end
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PasteReturn {
    pub url:            String,
    pub content:        String,
    pub date_published: i64,
    pub date_edited:    i64,
}

impl From<DatabasePaste> for PasteReturn {
    fn from(paste: DatabasePaste) -> Self {
        Self {
            url:            paste.url,
            content:        paste.content,
            date_published: paste.date_published,
            date_edited:    paste.date_edited,
        }
    }
}

#[derive(Clone)]
pub struct Manager {
    pool: SqlitePool,
}

impl Manager {
    pub async fn init() -> Self {
        Self {
            pool: database::init_database().await,
        }
    }
    pub async fn create_paste(&self, mut paste: NewPasteData) -> Result<(), PasteError> {
        // Check if the provided URL contains only accepted ASCII, and if it is short enough
        if !is_url_safe(&paste.url) || paste.url.len() > 250 {
            return Err(PasteError::InvalidUrl);
        }

        // Provide a default URL if it is empty, or throw an error if an already registered URL is given as input
        if paste.url.is_empty() {
            // Even though random collisions are unlikely, it is ensured here that random URLs will be unique
            let mut random_url = utility::random_string();
            while database::retrieve_paste(&self.pool, &random_url)
                .await
                .is_ok()
            {
                random_url = utility::random_string()
            }
            paste.url = random_url
        } else if database::retrieve_paste(&self.pool, &paste.url)
            .await
            .is_ok()
        {
            return Err(PasteError::AlreadyExists);
        }

        // Provide a default password, or throw an error if the one given as input is too long
        if paste.password.is_empty() {
            paste.password = utility::random_string();
        } else if paste.password.len() > 250 {
            return Err(PasteError::InvalidPassword);
        }

        // Check the content's length
        if paste.content.is_empty() || paste.content.len() > 200_000 {
            return Err(PasteError::InvalidContent);
        }

        let new_paste: DatabasePaste = paste.into();

        match database::insert_paste(&self.pool, new_paste).await {
            Ok(_) => Ok(()),
            Err(e) => Err(PasteError::Database(e)),
        }
    }
    pub async fn update_paste(
        &self,
        paste_credentials: PasteCredentials,
        mut paste: NewPasteData,
    ) -> Result<(), PasteError> {
        let existing_paste =
            match database::retrieve_paste(&self.pool, &paste_credentials.url).await {
                Ok(paste) => paste,
                Err(_) => return Err(PasteError::NotFound),
            };
        if existing_paste.password_hash
            != utility::hash_string(paste_credentials.password.to_owned())
        {
            return Err(PasteError::IncorrectPassword);
        }
        if paste.url.is_empty() {
            paste_credentials.url.clone_into(&mut paste.url)
        }
        if !paste.password.is_empty() && paste.password.len() > 250 {
            return Err(PasteError::InvalidPassword);
        }
        let password_hash = match paste.password.is_empty() {
            true => hash_string(paste_credentials.password),
            false => hash_string(paste.password),
        };
        // Check the content's length
        if paste.content.is_empty() || paste.content.len() > 200_000 {
            return Err(PasteError::InvalidContent);
        }

        let updated_paste = PartialDatabasePaste {
            url: paste.url,
            content: paste.content,
            password_hash,
            date_edited: utility::unix_timestamp(),
        };
        match database::update_paste(&self.pool, paste_credentials.url, updated_paste).await {
            Ok(_) => Ok(()),
            Err(e) => Err(PasteError::Database(e)),
        }
    }

    pub async fn delete_paste(&self, paste: PasteCredentials) -> Result<(), PasteError> {
        let existing_paste = match database::retrieve_paste(&self.pool, &paste.url).await {
            Ok(paste) => paste,
            Err(_) => return Err(PasteError::NotFound),
        };
        if existing_paste.password_hash != hash_string(paste.password) {
            return Err(PasteError::IncorrectPassword);
        }
        match database::delete_paste(&self.pool, &paste.url).await {
            Ok(_) => Ok(()),
            Err(e) => Err(PasteError::Database(e)),
        }
    }

    pub async fn retrieve_paste(&self, url: String) -> Result<PasteReturn, PasteError> {
        match database::retrieve_paste(&self.pool, &url).await {
            Ok(database_paste) => Ok(PasteReturn::from(database_paste)),
            Err(_) => Err(PasteError::NotFound),
        }
    }
}
