use color_eyre::eyre::Report;
use thiserror::Error;
use tonic;

#[derive(Debug, Error)]
pub enum AuthAPIError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid email")]
    InvalidEmail(String),
    #[error("Invalid login attempt id")]
    InvalidLoginAttemptId,
    #[error("Invalid password")]
    InvalidPassword(#[source] Report),
    #[error("Invalid auth token")]
    InvalidToken,
    #[error("Invalid two factor authentication code")]
    InvalidTwoFactorAuthCode,
    #[error("Missing auth token")]
    MissingToken,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
    #[error("User not found")]
    UserNotFound,
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
            AuthAPIError::UnexpectedError(report) => tonic::Status::internal(report.to_string()),
            AuthAPIError::MissingToken => tonic::Status::unauthenticated(error.to_string()),
            AuthAPIError::InvalidToken => tonic::Status::unauthenticated(error.to_string()),
            AuthAPIError::InvalidLoginAttemptId => tonic::Status::unauthenticated(error.to_string()),
            AuthAPIError::InvalidTwoFactorAuthCode => tonic::Status::unauthenticated(error.to_string()),
        }
    }
}

// impl std::error::Error for AuthAPIError {}
