//! Shared state handed to every handler. Cloned cheaply (HTTP client is
//! internally `Arc`-backed, font is loaded once into a `FontRef`).

use crate::auth::{CasbinAuth, JwtAuth};
use crate::cache::{Cache, CacheBackendConfig};
use crate::config::Config;
use crate::db::{Db, DbBackendConfig};
use crate::error::AppError;
use crate::http_client::HttpClient;
use crate::response::{ComponentHealth, HealthData};
use imageproc::image::ImageFormat;
use once_cell::sync::OnceCell;

/// Cached font loaded from `Config::watermark_font`. Loading happens lazily
/// the first time a watermark is requested, so deployments without a font
/// still start cleanly.
pub struct FontCache {
    inner: OnceCell<Option<Vec<u8>>>,
}

impl FontCache {
    /// 构造一个空缓存。`OnceCell::new` 零成本，第一次 [`FontCache::get`]
    /// 调用时才会去读文件系统 —— 服务启动时不需要字体存在。
    pub fn new() -> Self {
        Self { inner: OnceCell::new() }
    }

    /// 返回字体的原始字节；只在配置了字体路径且文件能读时才返回 `Some`。
    ///
    /// 故意不返回 `Result`：字体读取失败只是「暂不可用」，水印请求拿到
    /// `None` 后会再翻译成具体的 `AppError`。把 `io::Error` 在这里就转
    /// 成 `Option` 是因为水印路径里我们只关心「能不能用」，不关心
    /// 「为什么不能」，错误原因到 `watermark.rs` 里再补。
    /// `get_or_init` 闭包只跑一次，后续命中已缓存的字节。
    pub fn get(&self, path: &std::path::Path) -> Option<&[u8]> {
        self.inner
            .get_or_init(|| std::fs::read(path).ok())
            .as_deref()
    }
}

impl Default for FontCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-process shared state. Cloning is cheap.
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub http: HttpClient,
    pub fonts: std::sync::Arc<FontCache>,
    pub cache: Cache,
    pub db: Db,
    pub jwt: JwtAuth,
    pub casbin: CasbinAuth,
    pub img_cache_ttl_secs: Option<u64>,
}

impl AppState {
    /// 从环境变量加载并连接缓存、数据库，再构建运行时状态。
    pub async fn connect(config: Config) -> Result<Self, AppError> {
        let cache_cfg = CacheBackendConfig::from_env();
        let cache = Cache::connect(&cache_cfg).await?;
        crate::info!(backend = %cache.backend_name(), "cache ready");

        let db_cfg = DbBackendConfig::from_env()?;
        let db = Db::connect(&db_cfg).await?;
        crate::info!(backend = %db.backend_name(), "database ready");

        let http = HttpClient::build(config.fetch_timeout)?;
        let img_cache_ttl_secs = config.img_cache_ttl_secs;

        if let Some(pool) = db.sql_pool() {
            crate::auth::migrate(pool, db.backend_name()).await?;
        }

        let casbin = match db.sql_pool() {
            Some(pool) => CasbinAuth::new(&config, pool, db.backend_name()).await?,
            None => {
                return Err(AppError::Internal(
                    "casbin policy storage requires a SQL database backend".into(),
                ));
            }
        };

        let state = Self {
            config: config.clone(),
            http,
            fonts: std::sync::Arc::new(FontCache::new()),
            cache,
            db,
            jwt: JwtAuth::new(&config),
            casbin,
            img_cache_ttl_secs,
        };

        crate::controller::auth::bootstrap_admin(&state).await?;
        Ok(state)
    }

    /// Ping cache and database; used by `/health`.
    pub async fn check_health(&self) -> HealthData {
        let cache_ok = self.cache.ping().await.is_ok();
        let db_ok = self.db.ping().await.is_ok();
        HealthData {
            status: if cache_ok && db_ok { "ok" } else { "degraded" },
            cache: ComponentHealth {
                backend: self.cache.backend_name(),
                ok: cache_ok,
            },
            database: ComponentHealth {
                backend: self.db.backend_name(),
                ok: db_ok,
            },
        }
    }

    /// 通过 magic number 嗅探图像格式。
    ///
    /// 关键点：**不**用 URL 扩展名或 `Content-Type` 头来判。`src=cat.jpg`
    /// 实际是 PNG 的情况真实存在（被 CDN 转码过、被上游搞错），靠扩展名
    /// 会让解码炸成 422。用 `image::guess_format` 直接看字节头是唯一
    /// 可靠的方式。注意 `ImageFormat::from_bytes` 是不存在的（image 0.25
    /// 的 API 变更）—— 别凭印象去 `use` 它。
    pub fn sniff_format(bytes: &[u8]) -> Result<ImageFormat, AppError> {
        image::guess_format(bytes).map_err(|_| AppError::UnsupportedFormat)
    }
}

#[cfg(test)]
impl AppState {
    /// 测试用：内存 SQLite + 禁用缓存，避免依赖外部服务。
    pub async fn test(config: Config) -> Result<Self, AppError> {
        std::env::set_var("THUMBOR_DB_BACKEND", "sqlite");
        std::env::set_var("THUMBOR_DB_URL", "sqlite:file:memdb1?mode=memory&cache=shared");
        Self::connect(config).await
    }
}
