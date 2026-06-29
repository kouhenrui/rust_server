# AGENTS.md

thumbor 是一个用 Rust + axum 实现的图片处理服务（动态剪切、缩放、加水印、滤镜）。
本文档只写「不读代码就猜不到」的事，每条都是 agent 容易踩坑的点。

---

## 1. 命令速查

```bash
cargo check --all-targets   # 快速类型检查
cargo test --lib            # 仅跑 lib 的单元测试（推荐，秒级）
cargo test --lib proc::filter  # 跑某个模块的测试
cargo run --release         # 性能才够用；debug 模式 JPEG 编码会慢一个数量级
cargo build --release       # 产物在 target/release/thumbor(.exe)
```

注意：所有 `cargo` 必须在仓库根目录运行（`Cargo.toml` 在那）。CI 跑 `cargo test --all-targets` 而不是 `--lib`，
可以加上 `--doc` 看文档测试。

---

## 2. 运行时配置（环境变量，全部 `THUMBOR_` 前缀）

| 变量 | 默认 | 说明 |
|---|---|---|
| `THUMBOR_BIND` | `0.0.0.0:8080` | 监听地址 |
| `THUMBOR_MAX_SOURCE_BYTES` | `26214400` (25 MiB) | 源图最大字节数，超过返回 413 |
| `THUMBOR_FETCH_TIMEOUT_MS` | `10000` | 远程源 HTTP 超时（毫秒） |
| `THUMBOR_WATERMARK_FONT` | _无_ | 文本水印必需的 `.ttf` 路径；不设则文本水印返回 502 `watermark_font_missing` |
| `THUMBOR_ALLOW_REMOTE` | `true` | 关闭后拒绝 `http(s)://` 源（返回 502 `remote_disabled`） |
| `THUMBOR_LOCAL_SOURCE_ROOT` | _无_ | 相对路径源的前缀目录 |
| `RUST_LOG` | `info,thumbor=info,tower_http=info` | tracing-subscriber 的 EnvFilter |

解析在 `src/config.rs::Config::from_env()`，无效值用 `tracing::warn!` 记一行后落回默认。

---

## 3. 唯一的 HTTP 接口

```
GET  /health
GET  /img?src=...&w=...&h=...&fit=...&crop=...&filters=...&watermark=...&format=...
POST /img   Content-Type: application/x-protobuf
```

- `src`：必填。`http(s)://`、`file://`、或相对路径（拼上 `THUMBOR_LOCAL_SOURCE_ROOT`）。
- `w`/`h`：像素整数，至少给一个；`0` 会被拒绝。
- `fit`：`cover`(默认) | `contain` | `stretch`。`contain` 走透明画布 letterbox。
- `crop`：`x,y,w,h`（源像素坐标），越界会返回 400。
- `filters`：冒号分隔，例 `grayscale:brightness(20):blur(2.0)`。详见 §5。
- `watermark`：`Hello@10,10`（文本）或 `image:logo.png@10,10`（图像叠加）。
- `format`：`png`（默认）| `jpeg` | `webp`。WebP 当前是无损。

**GET** 响应带 `Cache-Control: public, max-age=86400`，错误统一是 JSON：
```json
{"error":{"code":"bad_request","message":"..."}}
```

**POST** 走 `proto/api.proto`（`package thumbor.v1`），请求是 `ImageRequest`、响应是 `ImageResponse`。
HTTP 状态码跟 GET 路径完全一致（`AppError::status()` 决定），但**响应体里
同时填了 `ErrorInfo`**——便于经过反向代理改状态码时客户端仍能用 `code` 字段做程序化分派。
MIME type 始终是 `application/x-protobuf`，没有缓存头（proto 客户端是后端对后端，浏览器缓存语义不适用）。

错误码见 `src/error.rs::AppError::code()`。

---

## 4. 处理流水线（按这个顺序跑）

定义在 `src/handler.rs::img`：

```
load_source → decode → transform::apply(crop, resize+fit) → filters::apply → watermark::apply → encode
```

任何一步失败整条请求就 fail。`transform::apply` 的 `crop` 一定先于 `resize`，
因为裁切后再缩放比反着做少几倍像素计算。

---

## 5. 加新 filter / transform / watermark

**Filter**（`src/proc/filter.rs`）：
1. 给 `enum Filter` 加一个 variant。
2. `Filter::apply` 的 `match` 加一支（参考 `Grayscale` 的写法）。
3. `FilterChain::parse_one` 的 `match name` 加一支解析参数（用 `exactly_one(&parts, name)?`）。
4. 在 `proc::filter::tests` 加 parse 测试。

**FitMode**（`src/proc/transform.rs`）：
- 只改 `FitMode` enum + `resize_with_fit` 的 match；query 解析在 `params::ImgParams::parse`。

**Watermark 类型**（`src/proc/watermark.rs`）：
- 在 `enum WatermarkSpec` 加 variant，在 `parse_watermark`（`params.rs`）加解析分支。

---

## 6. 依赖里容易踩的几个坑

- `ab_glyph = "0.2"` **必须显式列在 `[dependencies]`**，即使 `imageproc` 内部已经在用它。
  Rust 不会通过传递依赖把符号暴露给你的 crate，直接 `use ab_glyph::...` 会编译失败。
- `image` 的 feature 默认开启会带 `tiff`/`hdr` 等不必要的格式。`Cargo.toml` 里已显式 `default-features = false` 并只开 `jpeg, png, webp, bmp, gif`。
- `reqwest` 也关掉了默认 feature，只留 `rustls-tls`，避免在无 OpenSSL 的容器里链接失败。
- `Box<dyn ImageEncoder>` **不能移动**（unsized），所以 `handler::encode` 是按格式 `match` 而不是装箱成 trait object。改它时记得保持三支分别落盘。
- `image::guess_format` 是新 API（image 0.25+），旧文档里的 `ImageFormat::from_bytes` 不存在。

## 6.1. Protobuf 路径的几个坑

- **`build.rs` 用 `protoc-bin-vendored`，**不是 `protobuf-src` / 系统 `protoc`。
  `protobuf-src` 在第一次 build 时会走 CMake 从源码编译 protoc —— Windows
  上没有现成 C 工具链的机器直接炸。`protoc-bin-vendored` 直接下预编译
  二进制，第一次 build 多等几十秒，之后无感。
- **`prost-build` 生成的 Rust 文件名由 `package` 决定，不是 proto 文件名**。
  `package thumbor.v1;` + 文件 `proto/api.proto` → 输出 `<OUT_DIR>/thumbor.v1.rs`。
  `src/proto.rs` 里 `include!` 路径要按这个规则来。
- **`config.bytes(["."])` 让所有 `bytes` 字段生成 `bytes::Bytes`，不是 `Vec<u8>`**。
  `ImageResponse::image` 字段因此是 `Bytes`，序列化时不要再 `.to_vec()`。
- **proto 枚举字段在 struct 里是 `i32`，不是 typed enum**。
  想拿 typed enum 调 `req.fit()` / `req.format()` 这种方法时 borrow checker
  会抱怨（前面要先 destructuring 拿字段）。简单粗暴：直接 `match fit_enum { 0|1 => ..., 2 => ... }`。
- **`AppError::status` / `AppError::code` 必须是 `pub(crate)`，不是 `fn`**。
  proto 错误响应（`proto_error_response`）要直接读它们来填
  `ErrorInfo`，不能用 `IntoResponse` 的 JSON 形状泄露给 proto 客户端。
- **proto 路径上的 `req.crop` / `req.watermark` / `req.src` 都是 owning**，
  多次用容易触发「partially moved」错误。在 `img_request_to_params` 函数顶
  一次性 `let api::ImageRequest { ... } = req;` 解构最干净。

---

## 7. 状态、路由、错误

- `AppState` 被 `Arc::new` 包一层再 `.with_state(...)`，因为 `tower` 期望 state 本身是 `Clone`，而我们想让 axum 内部零拷贝地共享它。
- `AppError` 同时实现 `From<reqwest::Error>`、`From<image::ImageError>`、`From<std::io::Error>`，
  处理器里随便 `?`。`IntoResponse` 把错误映射成 `StatusCode + JSON`。
- `CorsLayer::permissive()` 在 `main.rs` 是为了开发期方便；上线前要换成显式 origin 列表。

---

## 8. 字体与水印

文本水印要 `THUMBOR_WATERMARK_FONT` 指向一个真实可读的 `.ttf`。字体是**懒加载**的（`state::FontCache`），
所以服务启动时不强制存在 —— 只有第一次请求水印时报错。如果临时不想支持文本水印，可以直接传 `format=` query 不带 `watermark=`，或设 `THUMBOR_ALLOW_REMOTE=false` 之外的策略都行，但更干净的做法是后端拒绝（当前行为）。

---

## 9. 测试现状与扩展

- `src/proc/filter.rs::tests`：filter 链解析。
- `src/source.rs::tests`：远程源禁用、文件缺失。
- `tests/integration.rs`：端到端（`tower::ServiceExt::oneshot` 跑完整 axum 栈）
  - `post_img_protobuf_success` — POST 合法 `ImageRequest`，解码 `ImageResponse`，验证 image 真能 load
  - `post_img_protobuf_error_propagates_in_body` — POST 空 `src`，验证 400 + `ErrorInfo.code == "bad_request"`
  - `post_img_protobuf_invalid_body_returns_bad_request` — POST 非合法 protobuf 字节
  - `get_img_query_still_works` — GET 老 query 路径冒烟

**没有的测试**，agent 改动时建议补：
- 处理器级：`transform::apply` 的 cover/contain/stretch 三种 fit 模式各拍一张断言尺寸。
- 错误路径：每个 `AppError` 变体在 `IntoResponse` 后给到的状态码。

跑单测时加上 `-- --nocapture` 可以看 `tracing` 输出，但默认单元测试不会初始化 subscriber，
所以加 `#[test]` 时别直接调 `tracing::info!` 期望看到。

---

## 10. 不要做的事

- 不要把 `AppState` clone 进 handler 又传 `Arc`：`with_state` 期望的状态本身就要 `Clone`，重复包 Arc 是没意义的。
- 不要在 `proc/filter.rs` 里用 `image::imageops::FilterType` —— 那是个缩放滤波器枚举，不是图像滤镜。
- 不要在 `ImageEncoder` 上加 `Send`/`Sync` 推导：编码器内部借 `&mut Vec<u8>`，根本不该跨线程。
- 不要把 `Cargo.lock` 加进 `.gitignore`：这是二进制的发布工程，不是库。

---

## 11. 常用阅读顺序（agent 进入仓库第一遍）

1. `src/main.rs` — 启动 + tracing 初始化 + graceful shutdown。
2. `src/handler.rs::router` / `img_get` / `img_post` / `process_image` — 业务入口（双协议路由 + 共享核心）。
3. `src/params.rs` — 知道所有 query 字段（GET 路径走这里）。
4. `src/proc/{transform,filter,watermark}.rs` — 处理逻辑。
5. `src/source.rs` — 源加载策略。
6. `src/error.rs` — 错误码 → HTTP 状态码 映射。
7. `proto/api.proto` + `build.rs` + `src/proto.rs` — POST 路径的 wire 格式与代码生成。
