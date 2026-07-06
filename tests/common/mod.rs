//! Shared helpers for integration tests.

use std::path::PathBuf;

use thumbor::auth::hash_password;
use thumbor::config::Config;
use thumbor::entity::{AccountRepository, SqlBackend};
use thumbor::state::AppState;

fn unique_sqlite_name(prefix: &str) -> String {
    format!(
        "{prefix}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

pub fn set_sqlite_env(db_name: &str) {
    std::env::set_var("THUMBOR_DB_BACKEND", "sqlite");
    std::env::set_var(
        "THUMBOR_DB_URL",
        format!("sqlite:file:{db_name}?mode=memory&cache=shared"),
    );
}

pub async fn connect_state(cfg: Config) -> AppState {
    set_sqlite_env(&unique_sqlite_name("memdb"));
    AppState::connect(cfg).await.unwrap()
}

pub async fn seed_test_user(state: &AppState) {
    let password_hash = hash_password("testpass").unwrap();
    AccountRepository::upsert(
        state.db.sql_pool().unwrap(),
        SqlBackend::Sqlite,
        "testuser",
        &password_hash,
    )
    .await
    .unwrap();
}

pub async fn app_with_root(root: PathBuf) -> axum::Router {
    let cfg = Config {
        local_source_root: Some(root),
        ..Config::default()
    };
    let state = connect_state(cfg).await;
    seed_test_user(&state).await;
    thumbor::router::router(state)
}
