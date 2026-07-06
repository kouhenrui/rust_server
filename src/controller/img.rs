//! Image processing controllers.
//!
//! `/img` accepts two request wire formats but always returns the unified
//! [`ApiResponse`](crate::response::ApiResponse) envelope:
//!
//! - `GET /img?src=...` → JSON (`data.image` is base64)
//! - `POST /img` + `Content-Type: application/x-protobuf` → protobuf

use crate::error::{AppError, AppResult};
use crate::params::{ImgParams, ImgParamsRaw, OutputFormat};
use crate::proc;
use crate::proc::filter::FilterChain;
use crate::proc::transform::FitMode;
use crate::proto::api;
use crate::response::ImageOutcome;
use crate::source;
use crate::state::AppState;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::response::Response;
use image::ImageEncoder;
use prost::Message;
use std::sync::Arc;

/// `GET /img` handler: query-string params → [`ImageOutcome`] → response.
pub async fn img_get(
    State(state): State<Arc<AppState>>,
    Query(raw): Query<ImgParamsRaw>,
) -> Response {
    let outcome = match ImgParams::parse(raw) {
        Ok(params) => ImageOutcome::from_result(process_image(&state, params).await),
        Err(err) => ImageOutcome::Err(err),
    };
    outcome.into_json_response()
}

/// `POST /img` handler: protobuf request → [`ImageOutcome`] → response.
pub async fn img_post(State(state): State<Arc<AppState>>, body: Bytes) -> Response {
    let req = match api::ImageRequest::decode(body.as_ref()) {
        Ok(r) => r,
        Err(e) => {
            let err = AppError::BadRequest(format!("invalid protobuf: {e}"));
            return ImageOutcome::Err(err).into_proto_response();
        }
    };

    let params = match img_request_to_params(req) {
        Ok(p) => p,
        Err(e) => return ImageOutcome::Err(e).into_proto_response(),
    };

    ImageOutcome::from_result(process_image(&state, params).await).into_proto_response()
}

/// 把 [`api::ImageRequest`] 转成 [`ImgParams`]，把枚举 / oneof 翻译成
/// 我们内部用的强类型。
///
/// **为什么在这里做转换而不是给 `ImgParams::parse` 加一个 proto 重载：**
/// proto 字段是 `i32`（枚举），query 是字符串，差异在「输入类型」
/// 就在「输入层」解决 —— 内部 `ImgParams` 是同一份，这样处理流水线
/// （crop → resize → filter → watermark）就一份代码。转换失败仍
/// 走 `AppError::BadRequest`，跟 query 路径完全一致。
fn img_request_to_params(req: api::ImageRequest) -> AppResult<ImgParams> {
    let api::ImageRequest {
        src,
        w,
        h,
        fit: fit_enum,
        crop,
        filters,
        watermark,
        format: format_enum,
    } = req;

    let fit = match fit_enum {
        2 => FitMode::Contain,
        3 => FitMode::Stretch,
        _ => FitMode::Cover,
    };

    let crop = crop.map(|c| crate::params::CropRect {
        x: c.x,
        y: c.y,
        w: c.w,
        h: c.h,
    });

    let filters = if filters.is_empty() {
        FilterChain::default()
    } else {
        FilterChain::parse(&filters)?
    };

    let watermark = watermark.and_then(|w| match w.kind? {
        api::image_request::watermark::Kind::Text(t) => Some(crate::params::WatermarkSpec::Text {
            text: t.text,
            x: t.x,
            y: t.y,
        }),
        api::image_request::watermark::Kind::Image(i) => {
            Some(crate::params::WatermarkSpec::Image {
                path: i.path,
                x: i.x,
                y: i.y,
            })
        }
    });

    let format = match format_enum {
        2 => OutputFormat::Jpeg,
        3 => OutputFormat::Webp,
        _ => OutputFormat::Png,
    };

    ImgParams::build(src, w, h, fit, crop, filters, watermark, format)
}

/// 业务核心：拿到校验过的 `ImgParams` 后跑完整流水线，输出
/// (编码字节, content_type)。
pub async fn process_image(
    state: &AppState,
    params: ImgParams,
) -> AppResult<(Bytes, OutputFormat, &'static str)> {
    let cache_key = params.cache_key();

    if state.cache.is_enabled() {
        if let Some(cached) = state.cache.get(&cache_key).await? {
            if let Ok(hit) = unpack_cached(&cached, params.format) {
                return Ok(hit);
            }
        }
    }

    let bytes = source::load_source(state, &params.src).await?;
    let mut img = source::decode(&bytes)?;

    proc::transform::apply(&mut img, params.crop, params.target, params.fit)?;
    params.filters.clone().apply(&mut img);
    if let Some(wm) = &params.watermark {
        proc::watermark::apply(&mut img, wm, state)?;
    }

    let (body, content_type) = encode(&img, params.format)?;

    if state.cache.is_enabled() {
        let packed = pack_cached(&body, content_type);
        state
            .cache
            .set(&cache_key, &packed, state.img_cache_ttl_secs)
            .await?;
    }

    Ok((body, params.format, content_type))
}

fn pack_cached(body: &[u8], content_type: &str) -> Vec<u8> {
    let mut v = Vec::with_capacity(content_type.len() + 1 + body.len());
    v.extend_from_slice(content_type.as_bytes());
    v.push(0);
    v.extend_from_slice(body);
    v
}

fn unpack_cached(
    cached: &[u8],
    format: OutputFormat,
) -> AppResult<(Bytes, OutputFormat, &'static str)> {
    let sep = cached
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| AppError::Internal("corrupt image cache entry".into()))?;
    let _content_type = std::str::from_utf8(&cached[..sep])
        .map_err(|e| AppError::Internal(format!("corrupt cache content-type: {e}")))?;
    let body = Bytes::copy_from_slice(&cached[sep + 1..]);
    Ok((body, format, format.content_type()))
}

/// 把 `DynamicImage` 编码成目标格式的字节。
fn encode(img: &image::DynamicImage, format: OutputFormat) -> AppResult<(Bytes, &'static str)> {
    let rgba8 = img.to_rgba8();
    let (w, h) = (rgba8.width(), rgba8.height());
    let raw = rgba8.into_raw();
    let mut buf: Vec<u8> = Vec::with_capacity(64 * 1024);
    let content_type = format.content_type();
    match format {
        OutputFormat::Png => {
            image::codecs::png::PngEncoder::new(&mut buf)
                .write_image(&raw, w, h, image::ExtendedColorType::Rgba8)
                .map_err(|e| AppError::Internal(format!("encode png: {e}")))?;
        }
        OutputFormat::Jpeg => {
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 85)
                .write_image(&raw, w, h, image::ExtendedColorType::Rgba8)
                .map_err(|e| AppError::Internal(format!("encode jpeg: {e}")))?;
        }
        OutputFormat::Webp => {
            image::codecs::webp::WebPEncoder::new_lossless(&mut buf)
                .write_image(&raw, w, h, image::ExtendedColorType::Rgba8)
                .map_err(|e| AppError::Internal(format!("encode webp: {e}")))?;
        }
    }
    Ok((Bytes::from(buf), content_type))
}
