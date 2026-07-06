//! Universal API response envelopes:
//!
//! - Success: `{ code: 0, message, data }` with HTTP 200
//! - Error:   `{ code: <status>, message, err }` with the matching HTTP status

use crate::error::AppError;
use crate::proto::api;
use axum::body::Bytes;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use base64::Engine;
use prost::Message;
use serde::Serialize;

pub const JSON_CONTENT_TYPE: &str = "application/json";
pub const PROTOBUF_CONTENT_TYPE: &str = "application/x-protobuf";
pub const SUCCESS_CODE: i32 = 0;
pub const SUCCESS_MESSAGE: &str = "success";

/// Success body: `{ code, message, data, trace_id }`.
#[derive(Debug, Clone, Serialize)]
pub struct ApiSuccess<T> {
    pub code: i32,
    pub message: &'static str,
    pub data: T,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub trace_id: String,
}

/// Error body: `{ code, message, err, trace_id }`.
#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorBody {
    pub code: i32,
    pub message: String,
    pub err: ErrBody,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub trace_id: String,
}

/// Structured error detail inside `err`.
#[derive(Debug, Clone, Serialize)]
pub struct ErrBody {
    pub kind: &'static str,
}

impl From<&AppError> for ErrBody {
    fn from(err: &AppError) -> Self {
        Self { kind: err.code() }
    }
}

/// `/img` success payload (JSON `data.image` is base64-encoded).
#[derive(Debug, Clone, Serialize)]
pub struct ImageData {
    #[serde(serialize_with = "serialize_image_base64")]
    pub image: Bytes,
    pub content_type: String,
}

/// `/health` success payload.
#[derive(Debug, Clone, Serialize)]
pub struct HealthData {
    pub status: &'static str,
    pub cache: ComponentHealth,
    pub database: ComponentHealth,
}

/// Per-dependency health in `/health` responses.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    pub backend: &'static str,
    pub ok: bool,
}

fn serialize_image_base64<S>(image: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = base64::engine::general_purpose::STANDARD.encode(image);
    serializer.serialize_str(&encoded)
}

fn status_code(err: &AppError) -> i32 {
    err.status().as_u16() as i32
}

/// Wire-agnostic result of the image pipeline.
#[derive(Debug)]
pub enum ImageOutcome {
    Ok {
        image: Bytes,
        content_type: &'static str,
    },
    Err(AppError),
}

impl ImageOutcome {
    pub fn from_result(
        result: Result<(Bytes, crate::params::OutputFormat, &'static str), AppError>,
    ) -> Self {
        match result {
            Ok((image, _, content_type)) => Self::Ok {
                image,
                content_type,
            },
            Err(err) => Self::Err(err),
        }
    }

    pub fn into_json_response(self) -> Response {
        match self {
            Self::Ok {
                image,
                content_type,
            } => api_success(ImageData {
                image,
                content_type: content_type.to_string(),
            }),
            Self::Err(err) => api_error(&err),
        }
    }

    pub fn into_proto_response(self) -> Response {
        match self {
            Self::Ok {
                image,
                content_type,
            } => proto_success(ImageData {
                image,
                content_type: content_type.to_string(),
            }),
            Self::Err(err) => proto_error(&err),
        }
    }
}

/// Successful response with typed `data`.
pub fn api_success<T: Serialize>(data: T) -> Response {
    let body = ApiSuccess {
        code: SUCCESS_CODE,
        message: SUCCESS_MESSAGE,
        data,
        trace_id: String::new(),
    };
    json_response(StatusCode::OK, body)
}

/// Error response: `{ code: <http status>, message, err }`.
pub fn api_error(err: &AppError) -> Response {
    let body = ApiErrorBody {
        code: status_code(err),
        message: err.to_string(),
        err: ErrBody::from(err),
        trace_id: String::new(),
    };
    json_response(err.status(), body)
}

fn json_response<T: Serialize>(status: StatusCode, body: T) -> Response {
    let mut resp = (status, Json(body)).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(JSON_CONTENT_TYPE),
    );
    resp
}

fn proto_success(data: ImageData) -> Response {
    let body = api::ApiResponse {
        code: SUCCESS_CODE,
        message: SUCCESS_MESSAGE.to_string(),
        data: Some(api::ImageData {
            image: data.image,
            content_type: data.content_type,
        }),
        err: None,
        trace_id: String::new(),
    };
    proto_bytes(StatusCode::OK, body)
}

fn proto_error(err: &AppError) -> Response {
    let body = api::ApiResponse {
        code: status_code(err),
        message: err.to_string(),
        data: None,
        err: Some(api::ErrInfo {
            kind: err.code().to_string(),
        }),
        trace_id: String::new(),
    };
    proto_bytes(err.status(), body)
}

fn proto_bytes(status: StatusCode, body: api::ApiResponse) -> Response {
    let bytes = body.encode_to_vec();
    let mut resp = (status, Bytes::from(bytes)).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(PROTOBUF_CONTENT_TYPE),
    );
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_envelope_shape() {
        let body = ApiSuccess {
            code: SUCCESS_CODE,
            message: SUCCESS_MESSAGE,
            data: ImageData {
                image: Bytes::from_static(b"png"),
                content_type: "image/png".into(),
            },
            trace_id: String::new(),
        };
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["code"], 0);
        assert_eq!(json["message"], "success");
        assert_eq!(json["data"]["content_type"], "image/png");
        assert_eq!(json["data"]["image"], "cG5n");
        assert!(json.get("err").is_none());
    }

    #[test]
    fn error_envelope_shape() {
        let err = AppError::BadRequest("missing src".into());
        let body = ApiErrorBody {
            code: status_code(&err),
            message: err.to_string(),
            err: ErrBody::from(&err),
            trace_id: String::new(),
        };
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["code"], 400);
        assert!(json["message"].as_str().unwrap().contains("src"));
        assert_eq!(json["err"]["kind"], "bad_request");
        assert!(json.get("data").is_none());
    }
}
