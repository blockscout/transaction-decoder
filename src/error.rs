use actix_web::{error, http::StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("not found")]
    NotFound,

    #[error("Error while getting tx info: {0}")]
    GetTxInfo(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl error::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Error::NotFound => StatusCode::BAD_REQUEST,
            Error::GetTxInfo(_) => StatusCode::BAD_REQUEST,
            Error::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
