# thumbor OpenSpec 工作计划

与当前代码库对齐的阶段性任务总览。详细任务见 `changes/<domain>/tasks.md`，
验收标准见 `specs/<domain>/spec.md`。

## 图例

| 标记 | 含义 |
|------|------|
| ✅ | 已实现并通过测试 |
| 🚧 | 部分实现，仍有 openspec 任务未完成 |
| 📋 | 已规划，未开始 |

---

## 阶段 0：基础运行时 ✅

| 领域 | Change | 状态 | 说明 |
|------|--------|------|------|
| 运行时配置 | `runtime-config` | ✅ | `Config`、`THUMBOR_*`、`.env` / `THUMBOR_DOTENV_PATH` |
| 二进制生命周期 | `runtime-config` | ✅ | `main.rs`：dotenv → logger → connect → axum；SIGINT/SIGTERM 优雅退出 |
| 代码生成 | `proto-api` | ✅ | `build.rs` + `protoc-bin-vendored` + `prost-build` |
| HTTP 路由 | `http-api` | ✅ | `router.rs` + `controller/`（`Arc<AppState>`） |
| 统一响应 | `api-response` | ✅ | JSON/Protobuf 信封、`trace_id`、`err.kind` 稳定码 |
| 统一错误 | `http-api` | ✅ | `AppError`、`AppResultExt`、`IntoResponse` → `api_error` |
| Protobuf API | `proto-api` | ✅ | `POST /img`、`proto/api.proto`、`src/proto.rs` |

## 阶段 1：可观测性与共享基础设施 ✅

| 领域 | Change | 状态 | 说明 |
|------|--------|------|------|
| 日志子系统 | `observability` | ✅ | `logger/`（`LoggerConfig`、`RUST_LOG` / `THUMBOR_LOG_LEVEL`） |
| 封装宏 | `observability` | ✅ | `trace!`/`debug!`/`info!`/`warn!`/`error!`（info+ 带 module/file/line） |
| HTTP 中间件 | `observability` | ✅ | `logging_middleware`、nanoid `trace_id`、请求摘要、响应注入 |
| 工具函数 | `shared-infra` | ✅ | `util/`（`parse_or_warn`、`redact_url`） |
| HTTP 客户端 | `shared-infra` | ✅ | `http_client::HttpClient`（超时、大小上限） |
| 响应宏 | `shared-infra` | ✅ | `span!`、`ok!`、`err!`；`TraceId` 提取器 |
| 健康检查 | `http-api` + `shared-infra` | ✅ | `/health` + `check_health`（cache/db ping） |

## 阶段 2：可插拔后端 ✅（连接层）

| 领域 | Change | 状态 | 说明 |
|------|--------|------|------|
| 缓存 | `cache-backend` | ✅ | memory/redis + `/img` 结果缓存 |
| 数据库 | `database-backend` | ✅ | postgres/mysql/sqlite/mongodb 连接 + ping；业务 ORM 未做 |

## 阶段 3：图片处理流水线 ✅

| 领域 | Change | 状态 | 说明 |
|------|--------|------|------|
| 请求参数 | `params` | ✅ | `ImgParamsRaw` → `ImgParams`；query + proto 双入口 |
| 源加载 | `image-source` | ✅ | 远程/本地/`file://`；经 `HttpClient::fetch` |
| 处理编排 | `image-pipeline` | ✅ | `process_image`：load → decode → transform → filter → watermark → encode |
| 几何变换 | `image-transform` | ✅ | crop、cover/contain/stretch |
| 滤镜 | `image-filters` | ✅ | grayscale、brightness、contrast、blur 链 |
| 水印 | `image-watermark` | ✅ | 文本/图像；`FontCache` 懒加载字体 |
| 输出编码 | `image-pipeline` | ✅ | PNG / JPEG(85) / WebP |

## 阶段 4：认证 ✅

| 领域 | Change | 状态 | 说明 |
|------|--------|------|------|
| 密码与 JWT 库 | `auth` | ✅ | `bcrypt` + `JwtAuth` + `bearer_token` |
| 配置与状态 | `auth` | ✅ | `Config.jwt_*`、`AppState.jwt` |
| 错误类型 | `auth` | ✅ | `Unauthorized` / `InvalidToken` → 401 |
| 登录 API | `auth` | ✅ | `POST /login` |
| JWT 中间件 | `auth` | ✅ | 保护 `GET /me` |
| 用户持久化 | `auth` | ✅ | SQL `users` 表 + migrate；`POST /register` 待做 |

## 阶段 5：测试与验证 ✅

| 领域 | Change | 状态 | 说明 |
|------|--------|------|------|
| 单元测试 | 各模块 | ✅ | error、response、filter、source、auth、cache、db、util、middleware |
| 集成测试 | `http-api` | ✅ | `tests/integration.rs`（health、GET/POST /img、错误路径） |
| 测试辅助 | `database-backend` | ✅ | `AppState::connect` + 内存 SQLite；`AppState::test`（lib 测试） |

## 阶段 6：生产化 🚧

| 任务 | 优先级 | 状态 | 说明 |
|------|--------|------|------|
| `/img` 结果缓存 | 高 | ✅ | `ImgParams::cache_key` + memory 后端 |
| 生产 CORS | 中 | ✅ | `THUMBOR_CORS_ORIGINS` |
| 默认 JWT secret 告警 | 中 | ✅ | 启动 `warn!` |
| CI workflow | 中 | ✅ | `.github/workflows/ci.yml` |
| 更新 `AGENTS.md` | 低 | 📋 | 与当前目录结构对齐 |
| `POST /register` | 低 | 📋 | 用户注册 API |
| 水印 E2E 测试 | 低 | 📋 | 文本/图像水印端到端用例 |

---

## 推荐执行顺序（未完成项）

1. 文档同步 → `AGENTS.md`
2. `auth` → `POST /register`（可选）
3. 水印 E2E 测试

## 代码目录速查（当前）

```
src/
├── auth/           # bcrypt + JWT（库层，无 HTTP）
├── cache/ db/      # 可插拔后端
├── controller/     # health.rs, img.rs（process_image 编排）
├── http_client.rs  # 远程源 HTTP 封装
├── logger/         # init, config, formatter, macros
├── middleware/     # logging_middleware, TraceId
├── params.rs       # ImgParams 解析（query + proto 桥接）
├── response.rs     # 统一信封
├── router.rs
├── state.rs        # AppState, FontCache, check_health, sniff_format
├── util/
└── proc/ source/ proto/ error.rs config.rs
build.rs            # prost 代码生成
tests/integration.rs
```

## 已实现能力对照表

| 代码模块 | Spec | Tasks |
|----------|------|-------|
| `config.rs` | `runtime-config` | `changes/runtime-config` |
| `main.rs` | `runtime-config` | `changes/runtime-config` §3 |
| `error.rs` | `api-response`, `http-api` | `changes/http-api` §1 |
| `response.rs` | `api-response` | `changes/api-response` |
| `router.rs` + `controller/` | `http-api` | `changes/http-api` |
| `params.rs` | `params` | `changes/params` |
| `controller/img.rs` pipeline | `image-pipeline` | `changes/image-pipeline` |
| `source.rs` | `image-source` | `changes/image-source` |
| `proc/*` | `image-transform/filters/watermark` | 各 change tasks |
| `logger/` + `middleware/` | `observability` | `changes/observability` |
| `util/` + `http_client.rs` | `shared-infra` | `changes/shared-infra` |
| `cache/` | `cache-backend` | `changes/cache-backend` |
| `db/` | `database-backend` | `changes/database-backend` |
| `auth/` | `auth` | `changes/auth` |
| `proto/` + `build.rs` | `proto-api` | `changes/proto-api` |
