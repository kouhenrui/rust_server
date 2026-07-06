# thumbor OpenSpec 工作计划

与当前代码库对齐的阶段性任务总览。详细任务见 `changes/<domain>/tasks.md`，
验收标准见 `specs/<domain>/spec.md`。OpenSpec CLI 与 Cursor 集成说明见 [README](./README.md)。

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
| 二进制生命周期 | `runtime-config` | ✅ | `main.rs`：dotenv → logger → connect → bootstrap → axum |
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
| 授权中间件 | `auth` | ✅ | `authorize_middleware`（JWT 解析 + Casbin RBAC） |
| 工具函数 | `shared-infra` | ✅ | `util/`（`parse_or_warn`、`redact_url`） |
| HTTP 客户端 | `shared-infra` | ✅ | `http_client::HttpClient`（超时、大小上限） |
| 响应宏 | `shared-infra` | ✅ | `span!`、`ok!`、`err!`；`AuthClaims` / `TraceId` 提取器 |
| 健康检查 | `http-api` + `shared-infra` | ✅ | `/health` + `check_health`（cache/db ping） |

## 阶段 2：可插拔后端 ✅

| 领域 | Change | 状态 | 说明 |
|------|--------|------|------|
| 缓存 | `cache-backend` | ✅ | memory/redis + `/img` 结果缓存（`ImgParams::cache_key`） |
| 数据库连接 | `database-backend` | ✅ | postgres/mysql/sqlite/mongodb 连接 + ping |
| 实体表 | `entity` | ✅ | `accounts`、`casbin_rule` DDL + migrate + repository 分层 |

## 阶段 3：图片处理流水线 ✅

| 领域 | Change | 状态 | 说明 |
|------|--------|------|------|
| 请求参数 | `params` | ✅ | `ImgParamsRaw` → `parse` → `build`；query + proto 共用校验 |
| 源加载 | `image-source` | ✅ | 远程/本地/`file://`；经 `HttpClient::fetch` |
| 处理编排 | `image-pipeline` | ✅ | `process_image`：cache → load → decode → transform → filter → watermark → encode |
| 几何变换 | `image-transform` | ✅ | crop、cover/contain/stretch（单元测试） |
| 滤镜 | `image-filters` | ✅ | grayscale、brightness、contrast、blur 链 |
| 水印 | `image-watermark` | ✅ | 文本/图像；`FontCache` 懒加载字体 |
| 输出编码 | `image-pipeline` | ✅ | PNG / JPEG(85) / WebP |

## 阶段 4：认证与授权 ✅

| 领域 | Change | 状态 | 说明 |
|------|--------|------|------|
| 密码与 JWT | `auth` | ✅ | `bcrypt` + `JwtAuth` + `bearer_token` |
| 配置与状态 | `auth` | ✅ | `Config.jwt_*`、`AppState.jwt` + `AppState.casbin` |
| 错误类型 | `auth` | ✅ | `Unauthorized` / `InvalidToken` / `Forbidden` |
| 登录 API | `auth` | ✅ | `POST /login`、`GET /me` |
| 账户持久化 | `entity` + `auth` | ✅ | SQL `accounts` 表；`AccountRepository` + `authenticate` |
| Casbin RBAC | `auth` + `entity` | ✅ | SQL `casbin_rule` 表；`CasbinRuleRepository`；非 CSV |
| Bootstrap 管理员 | `auth` | ✅ | `THUMBOR_BOOTSTRAP_*` 环境变量 |
| 用户注册 | `auth` | 📋 | `POST /register` 未实现 |

## 阶段 5：测试与 CI ✅

| 领域 | Change | 状态 | 说明 |
|------|--------|------|------|
| 单元测试 | 各模块 | ✅ | 45 项（error、params、transform、auth、entity、cache、db…） |
| 集成测试 | `http-api` | ✅ | `tests/integration.rs` + `tests/common/`（11 项） |
| 测试辅助 | `entity` | ✅ | `entity/test_util.rs`（lib）；`tests/common/mod.rs`（集成） |
| CI | — | ✅ | `fmt` + `clippy -D warnings` + `test`（`.github/workflows/ci.yml`） |

## 阶段 6：生产化 🚧

| 任务 | 优先级 | 状态 | 说明 |
|------|--------|------|------|
| `/img` 结果缓存 | 高 | ✅ | memory 后端 + 集成测试 |
| 生产 CORS | 中 | ✅ | `THUMBOR_CORS_ORIGINS` |
| 默认 JWT secret 告警 | 中 | ✅ | 启动 `warn!` |
| CI clippy/fmt | 中 | ✅ | workflow 已启用 |
| 文档同步 | 低 | ✅ | `AGENTS.md`、`README.md` 已对齐 |
| `POST /register` | 低 | 📋 | 用户注册 API |
| 水印 E2E 测试 | 低 | 📋 | 文本/图像水印端到端用例 |

---

## 推荐执行顺序（未完成项）

1. `POST /register`（可选）
2. 水印 E2E 测试
3. 单维度 `w`/`h` 缩放语义（spec 与实现需对齐，见 `image-transform`）

## 代码目录速查（当前）

```
src/
├── auth/              # JWT、密码、Casbin、login 业务（authenticate）
├── entity/            # models + repositories + schema + migrate
│   ├── models/        # Account, AccountAuth, CasbinRulePolicy
│   ├── repositories/  # AccountRepository, CasbinRuleRepository
│   └── test_util.rs   # 单元测试 SQLite 辅助
├── cache/ db/         # 可插拔后端（client.rs 为 trait 入口）
├── controller/        # health.rs, auth.rs, img.rs（process_image）
├── http_client.rs
├── logger/
├── middleware/        # logging.rs + auth.rs（Casbin + JWT）
├── params.rs          # ImgParams::parse / build / cache_key
├── response.rs
├── router.rs
├── state.rs
├── util/
└── proc/ source/ proto/ error.rs config.rs
tests/
├── common/mod.rs      # 集成测试 setup（唯一 SQLite 库名）
└── integration.rs
.github/workflows/ci.yml
```

## 已实现能力对照表

| 代码模块 | Spec | Tasks |
|----------|------|-------|
| `config.rs` | `runtime-config` | `changes/runtime-config` |
| `main.rs` | `runtime-config` | `changes/runtime-config` §3 |
| `error.rs` | `api-response`, `http-api` | `changes/http-api` §1 |
| `response.rs` | `api-response` | `changes/api-response` |
| `router.rs` + `controller/` | `http-api`, `auth` | `changes/http-api`, `changes/auth` |
| `params.rs` | `params` | `changes/params` |
| `controller/img.rs` | `image-pipeline`, `cache-backend` | 各 change tasks |
| `source.rs` | `image-source` | `changes/image-source` |
| `proc/*` | `image-transform/filters/watermark` | 各 change tasks |
| `logger/` + `middleware/` | `observability`, `auth` | 各 change tasks |
| `util/` + `http_client.rs` | `shared-infra` | `changes/shared-infra` |
| `cache/` | `cache-backend` | `changes/cache-backend` |
| `db/` | `database-backend` | `changes/database-backend` |
| `entity/` | `entity`, `database-backend` | `changes/database-backend`, `changes/auth` |
| `auth/` | `auth` | `changes/auth` |
| `proto/` + `build.rs` | `proto-api` | `changes/proto-api` |
