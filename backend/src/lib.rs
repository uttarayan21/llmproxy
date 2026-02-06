pub mod api_key;
pub mod auth;
pub mod config;
pub mod db;
pub mod embedded;
pub mod error;
pub mod handlers;
pub mod models;
pub mod proxy;
pub mod repository;

use config::Config;
use repository::Repository;

#[derive(Clone)]
pub struct AppState {
    pub repository: Repository,
    pub config: Config,
}
