use std::{fmt, str::FromStr, sync::Arc};

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use tokio::sync::RwLock;
use uuid::Uuid;

use auth_service::{
    get_postgres_pool, get_redis_client,
    utils::constants::{test, DATABASE_URL, DEFAULT_REDIS_HOST_NAME},
};

#[derive(Clone, Debug)]
pub struct DbName(String);

impl DbName {
    fn new(name: String) -> Self {
        Self(name)
    }
}

impl AsRef<str> for DbName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DbName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub async fn configure_postgresql() -> (PgPool, DbName) {
    let postgresql_conn_url = test::DATABASE_URL.to_owned();

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    println!("postgresql_conn_url: {postgresql_conn_url}");
    println!("db_name:             {db_name}");

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    let pg_pool = get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!");

    (pg_pool, DbName::new(db_name))
}

pub async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

pub async fn delete_database(db_name: &str) -> Result<(), String> {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let connection_options =
        PgConnectOptions::from_str(&postgresql_conn_url).expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .map_err(|e| format!("Failed to drop the database: {:?}", e))?;

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .map_err(|e| format!("Failed to drop the database: {:?}", e))?;

    Ok(())
}

pub fn configure_redis() -> Arc<RwLock<redis::Connection>> {
    let conn = get_redis_client(DEFAULT_REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis Client")
        .get_connection()
        .expect("Failed to get Redis Connection");
    Arc::new(RwLock::new(conn))
}
