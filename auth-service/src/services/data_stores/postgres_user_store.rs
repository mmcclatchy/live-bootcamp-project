use color_eyre::eyre;
use secrecy::ExposeSecret;
use sqlx::PgPool;

use crate::{
    domain::{
        data_stores::{UserStore, UserStoreError},
        email::Email,
        password::Password,
        user::{DbUser, NewUser, User},
    },
    utils::auth::async_compute_password_hash,
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
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: NewUser) -> Result<(), UserStoreError> {
        let password_hash = async_compute_password_hash(user.password.as_ref().clone())
            .await
            .map_err(UserStoreError::UnexpectedError)?;
        let result = sqlx::query_as!(
            DbUser,
            r#"
            INSERT INTO users (email, password_hash, requires_2fa)
            VALUES ($1, $2, $3)
            "#,
            user.email.as_ref().expose_secret(),
            password_hash.expose_secret(),
            user.requires_2fa
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(UserStoreError::UserAlreadyExists)
            }
            Err(e) => Err(UserStoreError::UnexpectedError(e.into())),
        }
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let user = sqlx::query_as!(
            DbUser,
            r#"
            SELECT *
            FROM users
            WHERE email = $1
            "#,
            email.as_ref().expose_secret(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| UserStoreError::UserNotFound)?;
        Ok(user.to_user())
    }

    #[tracing::instrument(name = "Updating user password in PostgreSQL", skip_all)]
    async fn update_password(&mut self, email: &Email, password: Password) -> Result<(), UserStoreError> {
        let password_hash = async_compute_password_hash(password.as_ref().clone())
            .await
            .map_err(UserStoreError::UnexpectedError)?;
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = $1
            WHERE email = $2
            "#,
            password_hash.expose_secret(),
            email.as_ref().expose_secret(),
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
            Err(e) => Err(UserStoreError::UnexpectedError(e.into())),
        }
    }

    #[tracing::instrument(name = "Verify password hash", skip_all)]
    async fn validate_user(&self, email: &Email, password: &Password) -> eyre::Result<User> {
        let user = sqlx::query_as!(
            DbUser,
            r#"
            SELECT *
            FROM users
            WHERE email = $1
            "#,
            email.as_ref().expose_secret(),
        )
        .fetch_one(&self.pool)
        .await?;
        user.verify_password(password)?;
        Ok(user.to_user())
    }
}
