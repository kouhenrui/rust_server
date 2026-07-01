//! Image processing controllers.
//!
//! `/img` accepts two request wire formats but always returns the unified
//! [`ApiResponse`](crate::response::ApiResponse) envelope:
//!
//! - `GET /img?src=...` → JSON (`data.image` is base64)
//! - `POST /img` + `Content-Type: application/x-protobuf` → protobuf

use crate::error::{AppError, AppResult};
use crate::params::{CropRect, ImgParams, ImgParamsRaw, OutputFormat, WatermarkSpec};
use crate::proc;
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
pub async fn img_post(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Response {
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
    // 一次性解构 `req` 而不是按字段零散借用：proto 字段里有几个
    // 拥有所有权的类型（String、Option<CropRect>、Option<Watermark>），
    // 零散取会让 borrow checker 在 `req.fit()` / `req.format()` 这种
    // 末尾调用上抱怨「partially moved」。一次解构干净。
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

    let src = if src.is_empty() {
        return Err(AppError::BadRequest("missing required 'src'".into()));
    } else {
        src
    };

    // w/h 必须 `> 0` 或者「都不给」(target=None)。proto 的 optional
    // 与 query 一样可以表达「都不给」。
    let target = match (w, h) {
        (Some(0), _) | (_, Some(0)) => {
            return Err(AppError::BadRequest("w and h must be > 0".into()));
        }
        (None, None) => None,
        (w, h) => Some((w.unwrap_or(0), h.unwrap_or(0))),
    };
    if let Some((0, 0)) = target {
        return Err(AppError::BadRequest("w and h must be > 0".into()));
    }

    let fit = match fit_enum {
        // `Unspecified` (0) 与未知值都回退到 Cover,跟 query 路径的
        // `None | Some("cover") | Some("") => FitMode::Cover` 同语义。
        2 => FitMode::Contain,
        3 => FitMode::Stretch,
        _ => FitMode::Cover,
    };

    // proto 路径上没在 query 那种字符串解析层把 `w=0` `h=0` 拦下，
    // 这里兜底 —— 0 维度的 crop 在后续 `transform::apply` 也会被
    // 当作「保持原尺寸」处理掉，所以这里直接过滤。
    let crop = crop
        .map(|c| CropRect { x: c.x, y: c.y, w: c.w, h: c.h })
        .filter(|c| c.w > 0 && c.h > 0);

    let filters = if filters.is_empty() {
        crate::proc::filter::FilterChain::default()
    } else {
        crate::proc::filter::FilterChain::parse(&filters)?
    };

    let watermark = watermark.and_then(|w| match w.kind? {
        api::image_request::watermark::Kind::Text(t) => Some(WatermarkSpec::Text {
            text: t.text,
            x: t.x,
            y: t.y,
        }),
        api::image_request::watermark::Kind::Image(i) => Some(WatermarkSpec::Image {
            path: i.path,
            x: i.x,
            y: i.y,
        }),
    });

    let format = match format_enum {
        // 与 fit 同样的「UNSPECIFIED / 未知值回退到默认」策略:
        // 0=Unspecified,1=Png,2=Jpeg,3=Webp。
        2 => OutputFormat::Jpeg,
        3 => OutputFormat::Webp,
        _ => OutputFormat::Png,
    };

    Ok(ImgParams {
        src,
        target,
        fit,
        crop,
        filters,
        watermark,
        format,
    })
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

    // 顺序与 `transform::apply` 文档保持一致：先裁后缩。
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

fn unpack_cached(cached: &[u8], format: OutputFormat) -> AppResult<(Bytes, OutputFormat, &'static str)> {
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
fn encode(
    img: &image::DynamicImage,
    format: OutputFormat,
) -> AppResult<(Bytes, &'static str)> {
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
