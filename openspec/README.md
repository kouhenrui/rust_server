# thumbor OpenSpec

OpenSpec CLI：**v1.5.0**（`@fission-ai/openspec` 官方最新） · 配置见 [config.yaml](./config.yaml)

与实现同步的 living specs 与变更包。工作计划见 [ROADMAP.md](./ROADMAP.md)；规划上下文见 [config.yaml](./config.yaml) 的 `context` 段。

## Cursor 集成（v1.5.0 OPSX）

Slash 命令（`.cursor/commands/`）：

| 命令 | 用途 |
|------|------|
| `/opsx:propose` | 创建 change 并生成 proposal / design / specs / tasks |
| `/opsx:apply` | 按 tasks 实现 |
| `/opsx:sync` | 将 delta specs 同步到主 specs |
| `/opsx:archive` | 归档已完成 change |
| `/opsx:explore` | 探索性分析（不写代码） |

维护命令：

```bash
npm install -g @fission-ai/openspec@latest   # 升级 CLI
openspec update --force                       # 刷新 Cursor 指令文件
openspec validate --specs --strict            # 校验 living specs
```

## 主规格（canonical specs）

| 域 | 路径 |
|----|------|
| api-response | [specs/api-response/spec.md](./specs/api-response/spec.md) |
| auth | [specs/auth/spec.md](./specs/auth/spec.md) |
| cache-backend | [specs/cache-backend/spec.md](./specs/cache-backend/spec.md) |
| database-backend | [specs/database-backend/spec.md](./specs/database-backend/spec.md) |
| entity | [specs/entity/spec.md](./specs/entity/spec.md) |
| http-api | [specs/http-api/spec.md](./specs/http-api/spec.md) |
| image-filters | [specs/image-filters/spec.md](./specs/image-filters/spec.md) |
| image-pipeline | [specs/image-pipeline/spec.md](./specs/image-pipeline/spec.md) |
| image-source | [specs/image-source/spec.md](./specs/image-source/spec.md) |
| image-transform | [specs/image-transform/spec.md](./specs/image-transform/spec.md) |
| image-watermark | [specs/image-watermark/spec.md](./specs/image-watermark/spec.md) |
| observability | [specs/observability/spec.md](./specs/observability/spec.md) |
| params | [specs/params/spec.md](./specs/params/spec.md) |
| proto-api | [specs/proto-api/spec.md](./specs/proto-api/spec.md) |
| runtime-config | [specs/runtime-config/spec.md](./specs/runtime-config/spec.md) |
| shared-infra | [specs/shared-infra/spec.md](./specs/shared-infra/spec.md) |

实现速查：仓库根 [README.md](../README.md)、[AGENTS.md](../AGENTS.md)。
