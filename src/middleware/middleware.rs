//! HTTP access logging middleware: trace id + request parameter logs.

use axum::body::Body;
use axum::extract::{FromRequestParts, Request};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use bytes::Bytes;
use http_body_util::BodyExt;
use prost::Message;
use serde_json::{json, Value};
use std::ops::Deref;
use std::time::Instant;
use tracing::Instrument;

use crate::error::AppError;
use crate::proto::api::{self, ImageRequest};
use crate::response::{JSON_CONTENT_TYPE, PROTOBUF_CONTENT_TYPE};
use async_trait::async_trait;

/// Request / response header carrying the trace identifier.
pub const TRACE_ID_HEADER: &str = "x-trace-id";

/// Per-request trace identifier stored in request extensions.
#[derive(Clone, Debug)]
pub struct TraceId(pub String);

impl Deref for TraceId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for TraceId
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<TraceId>()
            .cloned()
            .ok_or_else(|| AppError::Internal("trace id missing from request extensions".into()))
    }
}

/// Assign `trace_id`, log request params, inject id into the response.
pub async fn logging_middleware(mut request: Request, next: Next) -> Response {
    let trace_id = resolve_trace_id(
        request
            .headers()
            .get(TRACE_ID_HEADER)
            .and_then(|v| v.to_str().ok()),
    );

    request.extensions_mut().insert(TraceId(trace_id.clone()));

    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let query = request.uri().query().unwrap_or("").to_string();
    let req_content_type = request
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    let (parts, body) = request.into_parts();
    let req_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };

    let req_summary = summarize_request(
        &method,
        &path,
        &query,
        req_content_type.as_deref(),
        &req_bytes,
    );
    crate::info!(
        trace_id = %trace_id,
        method = %method,
        path = %path,
        query = %query,
        request = %req_summary,
        "http request received"
    );

    let request = Request::from_parts(parts, Body::from(req_bytes));
    let started = Instant::now();

    let span = crate::span!(
        "http",
        trace_id = %trace_id,
        method = %method,
        path = %path,
    );

    async move {
        let response = next.run(request).await;
        let latency_ms = started.elapsed().as_millis() as u64;
        let status = response.status().as_u16();

        let (mut parts, body) = response.into_parts();
        let resp_bytes = match body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => Bytes::new(),
        };
        let resp_content_type = parts
            .headers
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok());
        let resp_bytes = inject_trace_id(resp_content_type, &resp_bytes, &trace_id);

        set_trace_response_header(&mut parts.headers, &trace_id);
        crate::info!(
            trace_id = %trace_id,
            method = %method,
            path = %path,
            status = status,
            latency_ms = latency_ms,
            "http request completed"
        );

        Response::from_parts(parts, Body::from(resp_bytes))
    }
    .instrument(span)
    .await
}

fn new_trace_id() -> String {
    nanoid::nanoid!()
}

fn resolve_trace_id(incoming: Option<&str>) -> String {
    incoming
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(new_trace_id)
}

fn set_trace_response_header(headers: &mut axum::http::HeaderMap, trace_id: &str) {
    headers.insert(
        axum::http::HeaderName::from_static(TRACE_ID_HEADER),
        axum::http::HeaderValue::from_str(trace_id)
            .expect("trace_id must be a valid header value"),
    );
}

fn inject_trace_id(content_type: Option<&str>, body: &Bytes, trace_id: &str) -> Bytes {
    if is_json(content_type) {
        inject_trace_id_json(body, trace_id)
    } else if is_protobuf(content_type) {
        inject_trace_id_proto(body, trace_id)
    } else {
        body.clone()
    }
}

fn is_protobuf(content_type: Option<&str>) -> bool {
    content_type.is_some_and(|ct| ct.starts_with(PROTOBUF_CONTENT_TYPE))
}

fn is_json(content_type: Option<&str>) -> bool {
    content_type.is_some_and(|ct| ct.starts_with(JSON_CONTENT_TYPE))
}

fn inject_trace_id_json(body: &Bytes, trace_id: &str) -> Bytes {
    let Ok(mut value) = serde_json::from_slice::<Value>(body) else {
        return body.clone();
    };
    if let Some(obj) = value.as_object_mut() {
        obj.insert("trace_id".into(), json!(trace_id));
    }
    Bytes::from(serde_json::to_vec(&value).unwrap_or_else(|_| body.to_vec()))
}

fn inject_trace_id_proto(body: &Bytes, trace_id: &str) -> Bytes {
    let Ok(mut resp) = api::ApiResponse::decode(body.as_ref()) else {
        return body.clone();
    };
    resp.trace_id = trace_id.to_string();
    Bytes::from(resp.encode_to_vec())
}

fn summarize_request(
    method: &str,
    path: &str,
    query: &str,
    content_type: Option<&str>,
    body: &Bytes,
) -> String {
    if method == "GET" {
        if query.is_empty() {
            return format!("GET {path}");
        }
        return format!("GET {path}?{query}");
    }

    if is_protobuf(content_type) {
        return summarize_proto_request(body)
            .unwrap_or_else(|| format!("{method} {path} protobuf_body_len={}", body.len()));
    }

    format!(
        "{method} {path} content_type={} body_len={}",
        content_type.unwrap_or("-"),
        body.len()
    )
}

fn summarize_proto_request(body: &Bytes) -> Option<String> {
    let req = ImageRequest::decode(body.as_ref()).ok()?;
    Some(format!(
        "POST /img src={} w={:?} h={:?} fit={} filters={} format={}",
        req.src, req.w, req.h, req.fit, req.filters, req.format,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injects_trace_id_into_json() {
        let body = Bytes::from(
            serde_json::to_vec(&json!({
                "code": 0,
                "message": "success",
                "data": { "status": "ok" }
            }))
            .unwrap(),
        );
        let out = inject_trace_id_json(&body, "abc-123");
        let v: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["trace_id"], "abc-123");
    }

    #[test]
    fn summarizes_proto_request_fields() {
        let req = ImageRequest {
            src: "a.png".into(),
            w: Some(10),
            h: Some(20),
            fit: 1,
            crop: None,
            filters: "grayscale".into(),
            watermark: None,
            format: 2,
        };
        let body = Bytes::from(req.encode_to_vec());
        let summary = summarize_proto_request(&body).unwrap();
        assert!(summary.contains("src=a.png"));
        assert!(summary.contains("grayscale"));
    }
}
