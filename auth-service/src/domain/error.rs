use std::fmt;

use tonic;

#[derive(Debug)]
pub enum AuthAPIError {
    UserAlreadyExists,
    InvalidCredentials,
    InvalidEmail(String),
    InvalidLoginAttemptId,
    InvalidPassword(String),
    InvalidTwoFactorAuthCode,
    UserNotFound,
    UnexpectedError,
    MissingToken,
    InvalidToken,
}

impl fmt::Display for AuthAPIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthAPIError::UserAlreadyExists => write!(f, "User already exists"),
            AuthAPIError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthAPIError::InvalidEmail(msg) => write!(f, "Invalid email: {}", msg),
            AuthAPIError::InvalidPassword(msg) => write!(f, "Invalid password: {}", msg),
            AuthAPIError::UserNotFound => write!(f, "User not found"),
            AuthAPIError::UnexpectedError => write!(f, "Unexpected error occurred"),
            AuthAPIError::MissingToken => write!(f, "Missing auth token"),
            AuthAPIError::InvalidToken => write!(f, "Invalid auth token"),
            AuthAPIError::InvalidLoginAttemptId => write!(f, "Invalid login attempt id"),
            AuthAPIError::InvalidTwoFactorAuthCode => {
                write!(f, "Invalid two factor authentication code")
            }
        }
    }
}

impl From<AuthAPIError> for tonic::Status {
    fn from(error: AuthAPIError) -> Self {
        match error {
            AuthAPIError::UserAlreadyExists => tonic::Status::already_exists(error.to_string()),
            AuthAPIError::InvalidCredentials => tonic::Status::unauthenticated(error.to_string()),
            AuthAPIError::InvalidEmail(_) | AuthAPIError::InvalidPassword(_) => {
                tonic::Status::invalid_argument(error.to_string())
            }
            AuthAPIError::UserNotFound => tonic::Status::not_found(error.to_string()),
            AuthAPIError::UnexpectedError => tonic::Status::internal(error.to_string()),
            AuthAPIError::MissingToken => tonic::Status::unauthenticated(error.to_string()),
            AuthAPIError::InvalidToken => tonic::Status::unauthenticated(error.to_string()),
            AuthAPIError::InvalidLoginAttemptId => {
                tonic::Status::unauthenticated(error.to_string())
            }
            AuthAPIError::InvalidTwoFactorAuthCode => {
                tonic::Status::unauthenticated(error.to_string())
            }
        }
    }
}

impl std::error::Error for AuthAPIError {}
