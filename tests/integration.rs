//! Integration tests for the unified `ApiResponse` envelope.

use std::path::PathBuf;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use base64::Engine;
use image::{ImageBuffer, Rgba};
use prost::Message;
use serde_json::Value;
use tower::ServiceExt;

use thumbor::auth::{upsert_account_for_backend, SqlBackend};
use thumbor::config::Config;
use thumbor::proto::api::{ApiResponse, ImageRequest};
use thumbor::response::{SUCCESS_CODE, SUCCESS_MESSAGE};
use thumbor::state::AppState;

fn make_test_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "thumbor-it-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(2, 2, |x, y| {
        Rgba([x as u8 * 100, y as u8 * 100, 200, 255])
    });
    img.save(root.join("tiny.png")).unwrap();
    root
}

async fn app_with_root(root: PathBuf) -> axum::Router {
    std::env::set_var("THUMBOR_DB_BACKEND", "sqlite");
    std::env::set_var("THUMBOR_DB_URL", "sqlite:file:memdb1?mode=memory&cache=shared");
    let cfg = Config {
        local_source_root: Some(root),
        ..Config::default()
    };
    let state = AppState::connect(cfg).await.unwrap();
    upsert_account_for_backend(
        state.db.sql_pool().unwrap(),
        SqlBackend::Sqlite,
        "testuser",
        "testpass",
    )
    .await
    .unwrap();
    thumbor::router::router(state)
}

#[tokio::test]
async fn health_returns_unified_envelope() {
    let root = make_test_root();
    let app = app_with_root(root.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get("x-trace-id").is_some());
    let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["code"], SUCCESS_CODE);
    assert_eq!(json["message"], SUCCESS_MESSAGE);
    assert_eq!(json["data"]["status"], "ok");
    assert_eq!(json["data"]["cache"]["ok"], true);
    assert_eq!(json["data"]["database"]["ok"], true);
    assert!(json.get("err").is_none());
    assert!(json["trace_id"].as_str().is_some_and(|s| !s.is_empty()));

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn post_img_protobuf_success() {
    let root = make_test_root();
    let app = app_with_root(root.clone()).await;

    let req = ImageRequest {
        src: "tiny.png".into(),
        w: Some(4),
        h: Some(4),
        fit: 0,
        crop: None,
        filters: "grayscale".into(),
        watermark: None,
        format: 0,
    };
    let body = req.encode_to_vec();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/img")
                .header("content-type", "application/x-protobuf")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    let resp = ApiResponse::decode(bytes.as_ref()).unwrap();

    assert_eq!(resp.code, SUCCESS_CODE);
    assert_eq!(resp.message, SUCCESS_MESSAGE);
    assert!(resp.err.is_none());
    let data = resp.data.expect("data should be populated");
    assert_eq!(data.content_type, "image/png");
    assert!(!data.image.is_empty());

    let decoded = image::load_from_memory(&data.image).unwrap();
    assert_eq!(decoded.width(), 4);
    assert_eq!(decoded.height(), 4);

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn post_img_protobuf_error_propagates_in_body() {
    let root = make_test_root();
    let app = app_with_root(root.clone()).await;

    let req = ImageRequest {
        src: String::new(),
        w: Some(10),
        h: Some(10),
        fit: 0,
        crop: None,
        filters: String::new(),
        watermark: None,
        format: 0,
    };
    let body = req.encode_to_vec();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/img")
                .header("content-type", "application/x-protobuf")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let resp = ApiResponse::decode(bytes.as_ref()).unwrap();

    assert_eq!(resp.code, 400);
    assert!(resp.message.contains("src"));
    assert!(resp.data.is_none());
    assert_eq!(resp.err.expect("err").kind, "bad_request");

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn post_img_protobuf_invalid_body_returns_bad_request() {
    let root = make_test_root();
    let app = app_with_root(root.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/img")
                .header("content-type", "application/x-protobuf")
                .body(Body::from("not a protobuf".as_bytes().to_vec()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let resp = ApiResponse::decode(bytes.as_ref()).unwrap();
    assert_eq!(resp.code, 400);
    assert_eq!(resp.err.expect("err").kind, "bad_request");
    assert!(resp.data.is_none());

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn get_img_returns_unified_json_envelope() {
    let root = make_test_root();
    let app = app_with_root(root.clone()).await;

    let qs = "src=tiny.png&w=4&h=4&fit=cover&filters=grayscale&format=png";
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/img?{qs}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap()),
        Some("application/json")
    );

    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["code"], SUCCESS_CODE);
    assert_eq!(json["message"], SUCCESS_MESSAGE);
    assert_eq!(json["data"]["content_type"], "image/png");
    assert!(json.get("err").is_none());
    assert!(json["trace_id"].as_str().is_some_and(|s| !s.is_empty()));

    let image_b64 = json["data"]["image"].as_str().unwrap();
    let raw = base64::engine::general_purpose::STANDARD
        .decode(image_b64)
        .unwrap();
    let decoded = image::load_from_memory(&raw).unwrap();
    assert_eq!(decoded.width(), 4);
    assert_eq!(decoded.height(), 4);

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn get_img_error_returns_unified_json_envelope() {
    let root = make_test_root();
    let app = app_with_root(root.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/img?w=4&h=4")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["code"], 400);
    assert_eq!(json["err"]["kind"], "bad_request");
    assert!(json.get("data").is_none());
    assert!(json["trace_id"].as_str().is_some_and(|s| !s.is_empty()));

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn login_returns_token() {
    let root = make_test_root();
    let app = app_with_root(root.clone()).await;

    let body = serde_json::json!({"username": "testuser", "password": "testpass"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["code"], SUCCESS_CODE);
    assert!(json["data"]["token"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(json["data"]["expires_at"].as_u64().is_some());

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn login_wrong_password_returns_unauthorized() {
    let root = make_test_root();
    let app = app_with_root(root.clone()).await;

    let body = serde_json::json!({"username": "testuser", "password": "wrong"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["err"]["kind"], "unauthorized");

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn me_requires_bearer_token() {
    let root = make_test_root();
    let app = app_with_root(root.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn me_returns_profile_with_valid_token() {
    let root = make_test_root();
    let app = app_with_root(root.clone()).await;

    let login_body = serde_json::json!({"username": "testuser", "password": "testpass"});
    let login_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(login_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let login_bytes = to_bytes(login_resp.into_body(), 4096).await.unwrap();
    let login_json: Value = serde_json::from_slice(&login_bytes).unwrap();
    let token = login_json["data"]["token"].as_str().unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/me")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 4096).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["data"]["username"], "testuser");

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn img_result_is_cached_with_memory_backend() {
    let root = make_test_root();
    std::env::set_var("THUMBOR_CACHE_BACKEND", "memory");
    std::env::set_var("THUMBOR_DB_BACKEND", "sqlite");
    std::env::set_var("THUMBOR_DB_URL", "sqlite:file:memdb1?mode=memory&cache=shared");

    let cfg = Config {
        local_source_root: Some(root.clone()),
        ..Config::default()
    };
    let state = AppState::connect(cfg).await.unwrap();
    let app = thumbor::router::router(state);

    let qs = "src=tiny.png&w=4&h=4";
    let uri = format!("/img?{qs}");

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    std::fs::remove_file(root.join("tiny.png")).unwrap();

    let second = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);

    let bytes = to_bytes(second.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["code"], SUCCESS_CODE);

    let _ = std::fs::remove_dir_all(&root);
}
