# AGENTS.md

thumbor 是一个用 Rust + axum 实现的图片处理服务（动态剪切、缩放、加水印、滤镜）。
本文档只写「不读代码就猜不到」的事，每条都是 agent 容易踩坑的点。

---

## 1. 命令速查

```bash
cargo check --all-targets   # 快速类型检查
cargo fmt --all -- --check  # 格式检查（CI 同款）
cargo clippy --all-targets -- -D warnings  # lint（CI 同款）
cargo test --lib            # 仅跑 lib 的单元测试（推荐，秒级）
cargo test --lib proc::filter  # 跑某个模块的测试
cargo run --release         # 性能才够用；debug 模式 JPEG 编码会慢一个数量级
cargo build --release       # 产物在 target/release/thumbor(.exe)
```

注意：所有 `cargo` 必须在仓库根目录运行（`Cargo.toml` 在那）。CI 跑 `fmt` + `clippy -D warnings` + `cargo test --all-targets`。

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
| `THUMBOR_JWT_SECRET` | 随机生成（启动告警） | JWT 签名密钥；生产必须显式设置 |
| `THUMBOR_JWT_EXPIRE_SECS` | `86400` | JWT 有效期（秒） |
| `THUMBOR_BOOTSTRAP_USERNAME` / `PASSWORD` | _无_ | 可选：启动时 upsert 管理员并绑定 Casbin `admin` 角色 |
| `THUMBOR_CORS_ORIGINS` | _无_ | 逗号分隔允许来源；空 = 允许所有 |
| `THUMBOR_IMG_CACHE_TTL_SECS` | _无_ | `/img` 结果缓存 TTL（秒） |
| `THUMBOR_CASBIN_MODEL` | `config/casbin_model.conf` | Casbin RBAC model 文件；策略存 SQL 表 `casbin_rule` |
| `THUMBOR_DB_BACKEND` | `sqlite` | 数据库后端；Casbin/账户表需 postgres/mysql/sqlite |
| `RUST_LOG` | `info,thumbor=info,tower_http=info` | tracing-subscriber 的 EnvFilter |

解析在 `src/config.rs::Config::from_env()`，无效值用 `tracing::warn!` 记一行后落回默认。

---

## 3. HTTP 接口

```
GET  /health
POST /login          JSON { username, password } → JWT
GET  /me             需 Bearer JWT
GET  /img?src=...&w=...&h=...&fit=...&crop=...&filters=...&watermark=...&format=...
POST /img            Content-Type: application/x-protobuf
```

Casbin RBAC（`authorize_middleware`）：匿名可访问 `/health`、`/login`、`/img`；`user` 角色可 `/me`；策略持久化在 `casbin_rule` 表。

- `src`：必填。`http(s)://`、`file://`、或相对路径（拼上 `THUMBOR_LOCAL_SOURCE_ROOT`）。
- `w`/`h`：像素整数，至少给一个；`0` 会被拒绝。
- `fit`：`cover`(默认) | `contain` | `stretch`。`contain` 走透明画布 letterbox。
- `crop`：`x,y,w,h`（源像素坐标），越界会返回 400。
- `filters`：冒号分隔，例 `grayscale:brightness(20):blur(2.0)`。详见 §5。
- `watermark`：`Hello@10,10`（文本）或 `image:logo.png@10,10`（图像叠加）。
- `format`：`png`（默认）| `jpeg` | `webp`。WebP 当前是无损。

**JSON 接口**（`/health`、`/login`、`/me`、GET `/img`）统一信封，见 `src/response.rs`：

```json
{"code":0,"message":"success","data":{...},"trace_id":"..."}
{"code":401,"message":"...","err":{"kind":"unauthorized"},"trace_id":"..."}
```

**GET `/img`** 成功时 `data.image` 为 base64；带 `Cache-Control`（若启用结果缓存则按 TTL）。

**POST `/img`** 走 `proto/api.proto`（`package thumbor.v1`），请求 `ImageRequest`、响应 `ImageResponse`。
HTTP 状态码与 GET 一致；proto 体里同时填 `ErrorInfo`。MIME 为 `application/x-protobuf`。

GET query 与 POST proto 的参数校验共用 `params::ImgParams::build`（query 经 `parse`，proto 经 `controller/img.rs::img_request_to_params`）。

错误码见 `src/error.rs::AppError::code()`。

---

## 4. 处理流水线（按这个顺序跑）

定义在 `src/controller/img.rs::process_image`：

```
load_source → decode → transform::apply(crop, resize+fit) → filters::apply → watermark::apply → encode
```

启用缓存时：`cache.get` →  miss 则上述流水线 → `cache.set`（key 来自 `ImgParams::cache_key`）。

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
- 只改 `FitMode` enum + `resize_with_fit` 的 match；query 解析在 `params::ImgParams::parse`，proto 在 `img_request_to_params`，共用 `ImgParams::build`。

**Watermark 类型**（`src/proc/watermark.rs`）：
- 在 `enum WatermarkSpec` 加 variant，在 `parse_watermark`（`params.rs`）加解析分支。

---

## 6. 依赖里容易踩的几个坑

- `ab_glyph = "0.2"` **必须显式列在 `[dependencies]`**，即使 `imageproc` 内部已经在用它。
  Rust 不会通过传递依赖把符号暴露给你的 crate，直接 `use ab_glyph::...` 会编译失败。
- `image` 的 feature 默认开启会带 `tiff`/`hdr` 等不必要的格式。`Cargo.toml` 里已显式 `default-features = false` 并只开 `jpeg, png, webp, bmp, gif`。
- `reqwest` 也关掉了默认 feature，只留 `rustls-tls`，避免在无 OpenSSL 的容器里链接失败。
- `Box<dyn ImageEncoder>` **不能移动**（unsized），所以 `controller/img.rs::encode` 是按格式 `match` 而不是装箱成 trait object。改它时记得保持三支分别落盘。
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
- **proto 路径上的 owning 字段**在 `img_request_to_params` 顶一次性解构，校验交给 `ImgParams::build`。

---

## 7. 状态、路由、错误、认证

- 路由在 `src/router.rs`；handler 在 `src/controller/`；中间件在 `src/middleware/`（`logging_middleware` + `authorize_middleware`）。
- `AppState` 被 `Arc::new` 包一层再 `.with_state(...)`，因为 `tower` 期望 state 本身是 `Clone`。
- `AppError` 同时实现 `From<reqwest::Error>`、`From<image::ImageError>`、`From<std::io::Error>`，处理器里随便 `?`。`IntoResponse` 映射成统一 JSON 信封（`response::api_error`）。
- JWT：`auth/jwt.rs`；密码 bcrypt：`auth/password.rs`；登录业务：`auth/account.rs` → `entity/repositories/account.rs`。
- Casbin：`auth/casbin.rs` + `auth/casbin_adapter.rs`（SQL 适配器）+ `entity/repositories/casbin_rule.rs`。
- 实体分层：`entity/models/`（struct）→ `entity/repositories/`（SQL）→ `entity/schema.rs`（DDL）。
- CORS：`THUMBOR_CORS_ORIGINS`；空则 permissive（开发友好，上线应显式配置）。

---

## 8. 字体与水印

文本水印要 `THUMBOR_WATERMARK_FONT` 指向一个真实可读的 `.ttf`。字体是**懒加载**的（`state::FontCache`），
所以服务启动时不强制存在 —— 只有第一次请求水印时报错。如果临时不想支持文本水印，可以直接传 `format=` query 不带 `watermark=`，或设 `THUMBOR_ALLOW_REMOTE=false` 之外的策略都行，但更干净的做法是后端拒绝（当前行为）。

---

## 9. 测试现状与扩展

**单元测试**（`src/**` 内 `#[cfg(test)] mod tests`）：
- `proc/filter.rs`：滤镜链解析
- `params.rs`：`ImgParams::parse` / `build` 校验一致性
- `source.rs`：远程源禁用、文件缺失
- `auth/`、`entity/`：登录、Casbin、迁移、仓储
- `entity/test_util.rs`：`migrated_pool()` 供单元测试复用

**集成测试**（`tests/integration.rs` + `tests/common/mod.rs`）：
端到端（`tower::ServiceExt::oneshot`）：
- `health_returns_unified_envelope`
- `get_img_returns_unified_json_envelope` / `get_img_error_returns_unified_json_envelope`
- `post_img_protobuf_success` / `post_img_protobuf_error_propagates_in_body` / `post_img_protobuf_invalid_body_returns_bad_request`
- `login_returns_token` / `login_wrong_password_returns_unauthorized`
- `me_requires_bearer_token` / `me_returns_profile_with_valid_token`
- `img_result_is_cached_with_memory_backend`

**建议补的测试**：
- `transform::apply` 三种 fit 模式尺寸断言
- 每个 `AppError` 变体对应 HTTP 状态码

跑单测时 `-- --nocapture` 可看 tracing；默认单元测试不初始化 subscriber。

---

## 10. 不要做的事

- 不要把 `AppState` clone 进 handler 又传 `Arc`：`with_state` 期望的状态本身就要 `Clone`，重复包 Arc 是没意义的。
- 不要在 `proc/filter.rs` 里用 `image::imageops::FilterType` —— 那是个缩放滤波器枚举，不是图像滤镜。
- 不要在 `ImageEncoder` 上加 `Send`/`Sync` 推导：编码器内部借 `&mut Vec<u8>`，根本不该跨线程。
- 不要把 `Cargo.lock` 加进 `.gitignore`：这是二进制的发布工程，不是库。

---

## 11. 常用阅读顺序（agent 进入仓库第一遍）

1. `src/main.rs` — 启动 + tracing + bootstrap_admin + graceful shutdown。
2. `src/router.rs` + `src/controller/` — 路由与 HTTP handler。
3. `src/middleware/auth.rs` — JWT + Casbin 授权。
4. `src/params.rs` — GET/POST 共用 `ImgParams::build`。
5. `src/controller/img.rs::process_image` — 图片流水线 + 缓存。
6. `src/proc/{transform,filter,watermark}.rs` — 处理逻辑。
7. `src/entity/` — 账户表、Casbin 策略表、repository 分层。
8. `src/auth/` — JWT、密码、Casbin 封装。
9. `src/source.rs` — 源加载。
10. `src/error.rs` + `src/response.rs` — 错误码与统一信封。
11. `proto/api.proto` + `build.rs` + `src/proto.rs` — POST wire 格式。
