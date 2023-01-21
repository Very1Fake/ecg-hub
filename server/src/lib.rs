pub mod app;
pub mod config;
pub mod error;
pub mod handlers;
pub mod keys;
pub mod models;
pub mod types;
pub mod utils;

pub type DB = sqlx::PgPool;
