use serde::Serialize;
use snx::{
    response::{IntoResponse, Response},
    Json, StatusCode,
};

pub mod posts;

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

impl ErrorResponse {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

/// Represents errors that occur in handlers.
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    InvalidJsonBody(#[from] snx::InvalidJsonBodyError),
    #[error("an unknown database error occurred")]
    UnknownDatabaseError(#[from] diesel::result::Error),
    #[error("the requested resource could not be found")]
    ResourceNotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::InvalidJsonBody(e) => (
                StatusCode::BadRequest,
                Json(ErrorResponse::new(e.to_string())),
            ),
            AppError::ResourceNotFound => (
                StatusCode::NotFound,
                Json(ErrorResponse::new(self.to_string())),
            ),
            _ => (
                StatusCode::InternalServerError,
                Json(ErrorResponse::new("something went wrong".to_string())),
            ),
        }
        .into_response()
    }
}

/// Represents a result which can be returned from handlers.
pub type Result<T> = std::result::Result<T, AppError>;
