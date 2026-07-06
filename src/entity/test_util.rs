//! 单元测试用 SQLite 内存库辅助。

use super::{migrate, SqlBackend};
use sqlx::AnyPool;

pub async fn memory_sqlite_pool(db_name: &str) -> AnyPool {
    sqlx::any::install_default_drivers();
    let url = format!("sqlite:file:{db_name}?mode=memory&cache=shared");
    AnyPool::connect(&url).await.expect("sqlite memory pool")
}

pub async fn migrated_pool(db_name: &str) -> AnyPool {
    let pool = memory_sqlite_pool(db_name).await;
    migrate(&pool, SqlBackend::Sqlite).await.unwrap();
    pool
}
