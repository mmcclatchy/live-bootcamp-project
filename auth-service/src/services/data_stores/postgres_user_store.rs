use sqlx::PgPool;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    password::Password,
    user::{DbUser, NewUser, User},
};

#[derive(Clone, Debug)]
pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: NewUser) -> Result<(), UserStoreError> {
        let result = sqlx::query_as!(
            DbUser,
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)",
            user.email.as_ref(),
            user.password.as_ref(),
            user.requires_2fa
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(UserStoreError::UserAlreadyExists)
            }
            Err(_) => Err(UserStoreError::UnexpectedError),
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let user = sqlx::query_as!(DbUser, "SELECT * FROM users WHERE email = $1", email.as_ref())
            .fetch_one(&self.pool)
            .await
            .map_err(|_| UserStoreError::UserNotFound)?;
        Ok(user.to_user())
    }

    async fn update_password(&mut self, email: &Email, password: Password) -> Result<(), UserStoreError> {
        let result = sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE email = $2",
            password.as_ref(),
            email.as_ref(),
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(pg_result) => {
                if pg_result.rows_affected() == 0 {
                    Err(UserStoreError::UserNotFound)
                } else {
                    Ok(())
                }
            }
            Err(_) => Err(UserStoreError::UnexpectedError),
        }
    }

    async fn validate_user(&self, email: &Email, password: &Password) -> Result<User, UserStoreError> {
        let user = sqlx::query_as!(DbUser, "SELECT * FROM users WHERE email = $1", email.as_ref())
            .fetch_one(&self.pool)
            .await
            .map_err(|_| UserStoreError::UserNotFound)?;
        user.verify_password(password)?;
        Ok(user.to_user())
    }
}
