# thumbor

基于 Rust + [axum](https://github.com/tokio-rs/axum) 的图片处理 HTTP 服务，支持动态裁剪、缩放、水印与滤镜链（灰度 / 亮度 / 对比度 / 模糊）。

仓库地址：[https://github.com/kouhenrui/rust_server](https://github.com/kouhenrui/rust_server)

## 特性

- **图片处理**：裁剪、缩放（cover / contain / stretch）、滤镜链、文字/图片水印
- **双协议接口**：`GET /img`（Query 参数 + JSON）与 `POST /img`（Protobuf）
- **认证授权**：JWT 登录 + Casbin RBAC（策略存 SQL 表）
- **统一响应格式**：成功与失败均返回结构化信封，并附带 `trace_id`
- **可观测性**：封装 `tracing` 日志宏，HTTP 中间件记录请求参数与耗时
- **可插拔缓存**：`disabled` / `memory` / `redis`
- **可插拔数据库**：`postgres` / `mysql` / `sqlite` / `mongodb`

## 快速开始

```bash
# 复制环境变量模板（可选）
copy .env.example .env

# 启动服务（默认监听 0.0.0.0:8080）
cargo run --release
```

```bash
# 健康检查
curl http://localhost:8080/health

# 缩放远程图片并转灰度
curl "http://localhost:8080/img?src=https://example.com/cat.jpg&w=400&h=400&fit=cover&filters=grayscale" -o cat.png

# 文字水印（需设置 THUMBOR_WATERMARK_FONT）
set THUMBOR_WATERMARK_FONT=C:\Windows\Fonts\arial.ttf
cargo run --release
curl "http://localhost:8080/img?src=cat.jpg&w=600&watermark=Hello@20,20" -o cat.png

# 图片水印
curl "http://localhost:8080/img?src=cat.jpg&watermark=image:logo.png@10,10" -o cat.png
```

## API

### `GET /health`

返回服务健康状态（缓存、数据库 ping）。

### `POST /login`

JSON  body：`{ "username": "...", "password": "..." }`。成功返回 JWT（`data.token`）。

### `GET /me`

需 `Authorization: Bearer <token>`，返回当前用户信息。

### `GET /img`

通过 Query 参数处理图片：

| 参数 | 必填 | 示例 | 说明 |
|---|---|---|---|
| `src` | 是 | `cat.jpg` / `https://...` / `file:///abs/path` | 源图地址 |
| `w` | 否 | `400` | 目标宽度（像素） |
| `h` | 否 | `300` | 目标高度（像素） |
| `fit` | 否 | `cover` \| `contain` \| `stretch` | 缩放策略（默认 `cover`） |
| `crop` | 否 | `10,20,400,400` | 源图裁剪区域 `x,y,w,h` |
| `filters` | 否 | `grayscale:brightness(20):blur(2)` | 滤镜链，冒号分隔 |
| `watermark` | 否 | `Hello@10,10` 或 `image:logo.png@10,10` | 文字或图片水印 |
| `format` | 否 | `png` \| `jpeg` \| `webp` | 输出格式（默认 `png`） |

### `POST /img`

请求体为 Protobuf（`Content-Type: application/x-protobuf`），定义见 `proto/api.proto`。

## 响应格式

所有 JSON 接口使用统一信封：

**成功**（HTTP 200）：

```json
{
  "code": 0,
  "message": "success",
  "data": { },
  "trace_id": "..."
}
```

**失败**（HTTP 状态码与 `code` 一致）：

```json
{
  "code": 400,
  "message": "错误描述",
  "err": { "kind": "bad_request" },
  "trace_id": "..."
}
```

客户端可在请求头传入 `X-Trace-Id`；未传入时服务端自动生成（nanoid）。

## 配置

环境变量均以 `THUMBOR_` 为前缀，完整说明见 `AGENTS.md` 与 `.env.example`。

| 变量 | 默认值 | 说明 |
|---|---|---|
| `THUMBOR_BIND` | `0.0.0.0:8080` | 监听地址 |
| `THUMBOR_MAX_SOURCE_BYTES` | `26214400` | 源图最大字节数 |
| `THUMBOR_FETCH_TIMEOUT_MS` | `10000` | 远程源 HTTP 超时（毫秒） |
| `THUMBOR_WATERMARK_FONT` | _未设置_ | 文字水印字体路径 |
| `THUMBOR_ALLOW_REMOTE` | `true` | 是否允许 `http(s)://` 源 |
| `THUMBOR_LOCAL_SOURCE_ROOT` | _未设置_ | 相对路径源的前缀目录 |
| `THUMBOR_CACHE_BACKEND` | `disabled` | 缓存后端：`disabled` / `memory` / `redis` |
| `THUMBOR_DB_BACKEND` | `sqlite` | 数据库后端 |
| `THUMBOR_JWT_SECRET` | _启动时随机_ | JWT 签名密钥（生产必设） |
| `THUMBOR_CORS_ORIGINS` | _未设置_ | CORS 允许来源（逗号分隔） |
| `RUST_LOG` | `info,thumbor=info,tower_http=info` | 日志过滤级别 |

无效的环境变量值会记录警告日志并回退到默认值。

## 开发

```bash
cargo fmt --all -- --check  # 格式检查
cargo clippy --all-targets -- -D warnings  # lint
cargo test --lib              # 单元测试
cargo test                    # 单元测试 + 集成测试
cargo check --all-targets     # 类型检查
cargo run                     # 调试模式（编解码较慢）
```

## 项目结构

```
src/
├── main.rs              # 二进制入口：日志初始化、优雅停机
├── lib.rs               # 库对外导出
├── router.rs            # 路由注册
├── config.rs            # 环境变量配置
├── error.rs             # AppError
├── response.rs          # 统一 API 响应信封
├── state.rs             # AppState（缓存、数据库、HTTP 客户端）
├── source.rs            # 图片源加载
├── params.rs            # Query / proto 共用 ImgParams::build
├── controller/          # HTTP 处理器
│   ├── health.rs
│   ├── auth.rs          # /login, /me
│   └── img.rs
├── auth/                # JWT、密码、Casbin
├── entity/              # accounts / casbin_rule 模型与仓储
├── middleware/          # 日志 + JWT/Casbin 授权
├── logger/              # tracing 封装（config / init / macros）
├── cache/               # 缓存后端（memory / redis）
├── db/                  # 数据库后端（sql / mongo）
└── proc/                # 图片处理流水线
    ├── transform.rs     # 裁剪、缩放
    ├── filter.rs        # 滤镜链
    └── watermark.rs     # 水印
proto/
└── api.proto            # Protobuf 定义
tests/
├── common/mod.rs        # 集成测试共享 setup
└── integration.rs       # 端到端测试
```

## 许可证

MIT OR Apache-2.0
