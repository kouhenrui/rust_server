//! Image source loading. Supports:
//! - `http://` / `https://` URLs (when `Config::allow_remote_sources`)
//! - `file://` URIs
//! - Plain relative paths joined onto `Config::local_source_root`
//!
//! Decoding uses `image::ImageReader` so format detection is automatic.

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use image::ImageReader;
use std::io::Cursor;

/// 把 `src` 字符串解析为**字节**。
///
/// 关键设计：**先分支、再去网络/磁盘**。`http://` / `file://` 前缀
/// 是「不变量」，提前识别能避免「相对路径恰好长得像 URL」的歧义；
/// 一旦分到本地路径分支，就不会再去发 HTTP 请求。`strip_prefix` 后
/// 还得手动把 scheme 拼回去（`reqwest` 不接受裸主机名），
/// 这是 `url` crate 没引进来换来的依赖体积。
pub async fn load_source(state: &AppState, src: &str) -> AppResult<Vec<u8>> {
    if let Some(rest) = src
        .strip_prefix("http://")
        .or_else(|| src.strip_prefix("https://"))
    {
        if !state.config.allow_remote_sources {
            return Err(AppError::RemoteDisabled);
        }
        // Re-prepend scheme for reqwest.
        let url = if src.starts_with("http://") {
            format!("http://{rest}")
        } else {
            format!("https://{rest}")
        };
        return state.http.fetch(&url, state.config.max_source_bytes).await;
    }

    if let Some(path) = src.strip_prefix("file://") {
        return read_local(path, state.config.max_source_bytes);
    }

    let resolved = match &state.config.local_source_root {
        Some(root) => root.join(src),
        None => std::path::PathBuf::from(src),
    };
    read_local(
        resolved.to_str().unwrap_or(src),
        state.config.max_source_bytes,
    )
}

/// 读本地文件，同样做大小检查。
///
/// 用 `metadata().len()` 提前 stat 一次是因为 `read_to_end` 会**真的读完
/// 才报错** —— 一旦超限就白花了 25 MiB 内存才拒。提前 stat
/// 拿 stat size 就走 `SourceTooLarge`，快得多。
fn read_local(path: &str, max_bytes: usize) -> AppResult<Vec<u8>> {
    let meta = std::fs::metadata(path).map_err(|_| AppError::SourceNotFound(path.into()))?;
    if meta.len() as usize > max_bytes {
        return Err(AppError::SourceTooLarge { max: max_bytes });
    }
    std::fs::read(path).map_err(|_| AppError::SourceNotFound(path.into()))
}

/// Decode bytes into a `DynamicImage` and validate the detected format.
pub fn decode(bytes: &[u8]) -> AppResult<image::DynamicImage> {
    let format = AppState::sniff_format(bytes)?;
    if matches!(
        format,
        image::ImageFormat::Png
            | image::ImageFormat::Jpeg
            | image::ImageFormat::WebP
            | image::ImageFormat::Bmp
            | image::ImageFormat::Gif
    ) {
        return ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(AppError::from)?
            .decode()
            .map_err(AppError::from);
    }
    Err(AppError::UnsupportedFormat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn cfg() -> Config {
        Config {
            allow_remote_sources: false,
            ..Config::default()
        }
    }

    #[tokio::test]
    async fn rejects_remote_when_disabled() {
        let st = AppState::test(cfg()).await.unwrap();
        let err = load_source(&st, "https://example.com/cat.png")
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::RemoteDisabled));
    }

    #[tokio::test]
    async fn missing_local_source_404s() {
        let st = AppState::test(cfg()).await.unwrap();
        let res = load_source(&st, "definitely-not-here.png").await;
        assert!(matches!(res, Err(AppError::SourceNotFound(_))));
    }
}
