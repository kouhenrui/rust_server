//! Unified error type. Implements `IntoResponse` so handlers can `?` freely.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Crate-wide result alias.
pub type AppResult<T> = std::result::Result<T, AppError>;

/// All errors a handler can produce. Use `?` to propagate; the axum
/// `IntoResponse` impl turns these into structured JSON responses.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("source image not found: {0}")]
    SourceNotFound(String),

    #[error("source image too large (max {max} bytes)")]
    SourceTooLarge { max: usize },

    #[error("unsupported image format")]
    UnsupportedFormat,

    #[error("invalid image data: {0}")]
    Decode(String),

    #[error("remote sources are disabled")]
    RemoteDisabled,

    #[error("watermark requires a TTF font; set THUMBOR_WATERMARK_FONT")]
    WatermarkFontMissing,

    #[error("filter parse error: {0}")]
    Filter(String),

    #[error("upstream fetch failed: {0}")]
    Upstream(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// 把错误变体映射到稳定的 HTTP 状态码。
    ///
    /// 这是 API 契约的一部分：客户端依赖具体状态码做重试 / 缓存决策，
    /// 所以新增变体时**必须**同时决定它的状态码 —— 不能复用
    /// `Internal(500)` 来「暂时」兜底。映射理由：
    /// 4xx 表示「调用方的问题」（参数错、源没了、超大），
    /// 502 表示「上游 / 依赖的问题」（远程拒、字体缺、上游 fetch 失败），
    /// 500 是「我自己炸了」（编程错误）。
    /// 把错误变体映射到稳定的 HTTP 状态码。
    ///
    /// 这是 API 契约的一部分：客户端依赖具体状态码做重试 / 缓存决策，
    /// 所以新增变体时**必须**同时决定它的状态码 —— 不能复用
    /// `Internal(500)` 来「暂时」兜底。映射理由：
    /// 4xx 表示「调用方的问题」（参数错、源没了、超大），
    /// 502 表示「上游 / 依赖的问题」（远程拒、字体缺、上游 fetch 失败），
    /// 500 是「我自己炸了」（编程错误）。
    pub(crate) fn status(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) | AppError::Filter(_) => StatusCode::BAD_REQUEST,
            AppError::SourceNotFound(_) => StatusCode::NOT_FOUND,
            AppError::SourceTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::UnsupportedFormat | AppError::Decode(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::RemoteDisabled
            | AppError::WatermarkFontMissing
            | AppError::Upstream(_) => StatusCode::BAD_GATEWAY,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 把错误变体映射到稳定的**字符串**错误码。
    ///
    /// 这些字符串也是公开 API 的一部分（出现在 JSON 响应体里）——
    /// 客户端写 `switch (err.code)` 的时候要用。**绝对不要**把
    /// `e.to_string()` 直接当 code；to_string 文本会随 thiserror 模板
    /// 微调，是「给人看的」而不是「给程序读的」。
    /// 把错误变体映射到稳定的**字符串**错误码。
    ///
    /// 这些字符串也是公开 API 的一部分（出现在 JSON 响应体里）——
    /// 客户端写 `switch (err.code)` 的时候要用。**绝对不要**把
    /// `e.to_string()` 直接当 code；to_string 文本会随 thiserror 模板
    /// 微调，是「给人看的」而不是「给程序读的」。
    pub(crate) fn code(&self) -> &'static str {
        match self {
            AppError::BadRequest(_) => "bad_request",
            AppError::SourceNotFound(_) => "source_not_found",
            AppError::SourceTooLarge { .. } => "source_too_large",
            AppError::UnsupportedFormat => "unsupported_format",
            AppError::Decode(_) => "decode_failed",
            AppError::RemoteDisabled => "remote_disabled",
            AppError::WatermarkFontMissing => "watermark_font_missing",
            AppError::Filter(_) => "invalid_filter",
            AppError::Upstream(_) => "upstream_failed",
            AppError::Internal(_) => "internal",
        }
    }
}

/// reqwest 错误一律归到「上游问题」：不是我们能修的，也不算 4xx。
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Upstream(e.to_string())
    }
}

/// `image` crate 的解码失败归 422：因为字节已经在手、格式也对，
/// 但内容无效 —— 调用方应该改源而不是重试。
impl From<image::ImageError> for AppError {
    fn from(e: image::ImageError) -> Self {
        AppError::Decode(e.to_string())
    }
}

/// 走 `Internal` 而不是单独搞个 `Io` 变体：handler 层 IO 失败基本
/// 都是「磁盘炸了 / 路径被误改」，归内部问题更准，也省一个变体。
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(format!("io: {e}"))
    }
}

/// axum 把 `Result<_, AppError>` 转成 HTTP 响应的入口。
///
/// 故意把所有分支走同一个错误信封 —— `{ code, message, err }`，HTTP
/// 状态码由 `AppError::status()` 决定。
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        crate::response::api_error(&self)
    }
}
