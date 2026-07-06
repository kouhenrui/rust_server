//! Server configuration.
//!
//! Load order: [`Config::default`] → `.env` file (via [`Config::load_dotenv`]) →
//! `THUMBOR_*` environment variables (via [`Config::from_env`]).
//!
//! Call [`Config::load`] at startup, or `load_dotenv()` then `from_env()` manually.
//! Variables already set in the process environment are **not** overwritten by `.env`.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use crate::util::parse_or_warn;

/// Runtime configuration. Construct via [`Config::from_env`] or [`Config::default`].
#[derive(Debug, Clone)]
pub struct Config {
    /// Address the axum listener binds to. Default: `0.0.0.0:8080`.
    pub bind_addr: SocketAddr,

    /// Maximum size of the source image, in bytes. Default: 25 MiB.
    pub max_source_bytes: usize,

    /// HTTP fetch timeout for remote sources. Default: 10s.
    pub fetch_timeout: Duration,

    /// Optional path to a `.ttf` font used for text watermarks. If unset, text
    /// watermarks are rejected with [`AppError::WatermarkFontMissing`].
    pub watermark_font: Option<PathBuf>,

    /// Allow fetching arbitrary `http://` / `https://` URLs as the source image.
    /// Disable for fully-closed deployments that only serve local files.
    pub allow_remote_sources: bool,

    /// Base path prepended to relative source paths when the `src` is not a URL.
    pub local_source_root: Option<PathBuf>,

    pub log_level: String,

    /// HMAC secret for JWT signing (`THUMBOR_JWT_SECRET`).
    pub jwt_secret: String,

    /// JWT lifetime in seconds (`THUMBOR_JWT_EXPIRE_SECS`, default 86400).
    pub jwt_expire_secs: u64,

    /// Comma-separated allowed CORS origins (`THUMBOR_CORS_ORIGINS`); empty = permissive.
    pub cors_origins: Vec<String>,

    /// TTL for cached `/img` results in seconds; `None` = no expiry. (`THUMBOR_IMG_CACHE_TTL_SECS`)
    pub img_cache_ttl_secs: Option<u64>,

    /// Casbin model file (`THUMBOR_CASBIN_MODEL`, default `config/casbin_model.conf`).
    pub casbin_model: PathBuf,
}

/// 构造一组**可直接部署**的默认值。
///
/// `expect("valid default addr")` 这种写法是有意为之：默认值是字面量、
/// 编译期已知，parse 永远不该失败；如果失败了说明默认值被改坏，
/// 进程该 panic 而不是带病启动。
impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".parse().expect("valid default addr"),
            max_source_bytes: 25 * 1024 * 1024,
            fetch_timeout: Duration::from_secs(10),
            watermark_font: None,
            allow_remote_sources: true,
            local_source_root: None,
            log_level: "info".to_string(),
            jwt_secret: "secret".to_string(),
            jwt_expire_secs: 86400,
            cors_origins: Vec::new(),
            img_cache_ttl_secs: Some(3600),
            casbin_model: PathBuf::from("config/casbin_model.conf"),
        }
    }
}

impl Config {
    /// 加载 `.env` 后读取 `THUMBOR_*` 环境变量。进程入口推荐用这个。
    pub fn load() -> Self {
        Self::load_dotenv();
        Self::from_env()
    }

    /// 从 `.env` 注入环境变量（不覆盖已有变量）。
    ///
    /// 默认读项目根目录的 `.env`；可通过 `THUMBOR_DOTENV_PATH` 指定其他路径。
    /// 文件不存在时静默跳过。
    pub fn load_dotenv() {
        let path = std::env::var("THUMBOR_DOTENV_PATH").unwrap_or_else(|_| ".env".into());
        match dotenvy::from_filename(&path) {
            Ok(_) => {}
            Err(dotenvy::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => eprintln!("warning: failed to load {path}: {e}"),
        }
    }

    /// 从 `THUMBOR_*` 环境变量覆盖默认值。
    ///
    /// 关键设计：解析失败**回退到默认 + warn log**，而不是 panic 或返回
    /// `Result`。配置错误不该让服务起不来 —— 部署时人手少、CI 与
    /// 生产环境变量偶有漂移，宁可行为稍微变（用默认）也不能让服务消失。
    /// 副作用是用户可能拿到一个跟预期略不同的服务；但日志里有线索。
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("THUMBOR_BIND") {
            if let Some(addr) = parse_or_warn(&v, "invalid THUMBOR_BIND") {
                cfg.bind_addr = addr;
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_MAX_SOURCE_BYTES") {
            if let Some(n) = parse_or_warn(&v, "invalid THUMBOR_MAX_SOURCE_BYTES") {
                cfg.max_source_bytes = n;
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_FETCH_TIMEOUT_MS") {
            if let Some(ms) = parse_or_warn::<u64>(&v, "invalid THUMBOR_FETCH_TIMEOUT_MS") {
                cfg.fetch_timeout = Duration::from_millis(ms);
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_WATERMARK_FONT") {
            cfg.watermark_font = Some(PathBuf::from(v));
        }
        if let Ok(v) = std::env::var("THUMBOR_ALLOW_REMOTE") {
            // 接受 1/true/yes 任意大小写 —— kubectl/Helm 的 value 文件
            // 有时给的是 "True" / "YES"，苛求 "true" 反而失之过严。
            cfg.allow_remote_sources =
                matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes");
        }
        if let Ok(v) = std::env::var("THUMBOR_LOCAL_SOURCE_ROOT") {
            cfg.local_source_root = Some(PathBuf::from(v));
        }
        if let Ok(v) = std::env::var("THUMBOR_LOG_LEVEL") {
            cfg.log_level = v.to_string();
        }
        if let Ok(v) = std::env::var("THUMBOR_JWT_SECRET") {
            if !v.is_empty() {
                cfg.jwt_secret = v;
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_JWT_EXPIRE_SECS") {
            if let Some(secs) = parse_or_warn::<u64>(&v, "invalid THUMBOR_JWT_EXPIRE_SECS") {
                cfg.jwt_expire_secs = secs;
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_CORS_ORIGINS") {
            cfg.cors_origins = v
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
        }
        if let Ok(v) = std::env::var("THUMBOR_IMG_CACHE_TTL_SECS") {
            match v.parse::<u64>() {
                Ok(0) => cfg.img_cache_ttl_secs = None,
                Ok(secs) => cfg.img_cache_ttl_secs = Some(secs),
                Err(_) => {
                    let _ = parse_or_warn::<u64>(&v, "invalid THUMBOR_IMG_CACHE_TTL_SECS");
                }
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_CASBIN_MODEL") {
            if !v.is_empty() {
                cfg.casbin_model = PathBuf::from(v);
            }
        }
        cfg
    }
}
