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

    pub jwt_secret:String,
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
            match v.parse() {
                Ok(addr) => cfg.bind_addr = addr,
                Err(e) => crate::warn!(value = %v, error = %e, "invalid THUMBOR_BIND"),
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_MAX_SOURCE_BYTES") {
            match v.parse() {
                Ok(n) => cfg.max_source_bytes = n,
                Err(e) => crate::warn!(error = %e, "invalid THUMBOR_MAX_SOURCE_BYTES"),
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_FETCH_TIMEOUT_MS") {
            match v.parse::<u64>() {
                Ok(ms) => cfg.fetch_timeout = Duration::from_millis(ms),
                Err(e) => crate::warn!(error = %e, "invalid THUMBOR_FETCH_TIMEOUT_MS"),
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_WATERMARK_FONT") {
            cfg.watermark_font = Some(PathBuf::from(v));
        }
        if let Ok(v) = std::env::var("THUMBOR_ALLOW_REMOTE") {
            // 接受 1/true/yes 任意大小写 —— kubectl/Helm 的 value 文件
            // 有时给的是 "True" / "YES"，苛求 "true" 反而失之过严。
            cfg.allow_remote_sources = matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes");
        }
        if let Ok(v) = std::env::var("THUMBOR_LOCAL_SOURCE_ROOT") {
            cfg.local_source_root = Some(PathBuf::from(v));
        }
        if let Ok(v) = std::env::var("THUMBOR_LOG_LEVEL") {
            cfg.log_level = v.to_string();
        }
        cfg
    }
}
