# OPS-CONSOLE-INDEPENDENT-RELEASE-001 — 运维控制台与原子化 App 独立发布

> 状态：🟡 Draft v0.1（设计基线，待实装）
> 日期：2026-11-14
> 设计原型：Claude Design 画布「7 · 运维控制台（仅管理员）」
> 关联：ADR-0054（Argo CD + Kargo + Argo Rollouts）、`spec/ops/01-ops-requirements-spec.md`、`OPS-BASIC-DESIGN-001.md`、`deploy/gitops/`、`.github/workflows/publish-star-canvas-services.yml`

## §0 目的

1. 定义一个**只有平台管理员可见**的运维控制台：运维数据、租户管理、用户与权限、原子化 App 集群的单独更新、审计。
2. 核对现有架构能否支持**按 App 单独升级**，列出差距，给出目标架构和重构步骤，作为后续重构的依据。

本文不改动已有 ADR 正文；与旧规格的冲突在 §1.3 显式列出。

## §1 角色与可见性

### 1.1 平台管理员

| 项 | 规则 |
|---|---|
| 角色名 | `platform_operator`（复用 `star-context::ActorContext::is_platform_operator()`） |
| 初始管理员 | 由部署配置 `STAR_PLATFORM_BOOTSTRAP_ADMIN` 指定的一个账号（邮箱），首次启动时写入；**真实邮箱不入库、不进镜像**，只存在于部署密钥中 |
| 数量 | 当前阶段唯一管理员；可扩展为多名，但系统必须至少保留 1 名 |
| 最后一名保护 | 最后一名管理员不能被降级、停用或删除（服务端校验，非前端隐藏） |
| 变更方式 | 增删管理员需二次确认 + 写审计；不允许管理员给自己以外的账号静默授权 |
| 可见性 | 运维入口只在用户资料菜单中对 `platform_operator` 渲染；路由与 API 均在服务端按角色拒绝（403），不能只靠隐藏 |

### 1.2 控制台页签

| 页签 | 内容 | 主要数据来源 |
|---|---|---|
| 运维数据 | 在线用户、运行中的 Run、API 错误率、事件积压、集群与节点、告警 | `star-telemetry`、`star-ops` F-01/F-03、Kargo/Argo 状态 |
| App 集群 | 每个原子化 App 的版本、可升级版本、契约兼容性、健康度、单独更新 / 回滚 | Kargo Freight/Stage、Argo CD Application、Argo Rollouts |
| 租户 | 租户列表、成员与 Project 数、配额（Run 并发 / 存储）、停用 | `domain-tenant`、`star-quota` |
| 用户与权限 | 平台角色、租户内角色、Agent 服务账号、吊销 | `domain-identity`、`domain-permission` |
| 审计 | 所有控制台操作与自动回滚记录 | `domain-audit`（WORM） |

未接通的数据源一律显示「未接入」，不得用示例数据填充（AGENTS.md §0）。

### 1.3 与旧规格的差异（需在实装前确认）

| 旧规格 | 本设计 | 处理 |
|---|---|---|
| `01-ops-requirements-spec.md` §1：SRE Ops 后台与 platform admin 后台分离，租户 CRUD / RBAC 不属于 Ops | 合并为一个运维控制台，同一 `platform_operator` 角色 | 以本设计为准，旧规格加 superseded 注记（另行提交） |
| 旧规格依赖「SRE Lead 真人」 | 唯一平台管理员 | 与 AGENTS.md v0.64「去掉等待真人流程」一致 |

## §2 现状核对：能否按 App 单独升级

| 层 | 现状（2026-11-14 实测） | 能否单独升级 |
|---|---|---|
| 构建 | `publish-star-canvas-services.yml` 按 App 矩阵分别构建 `canvas-engine` / `domain-canvas` / `canvas-realtime`，每个镜像独立 Cargo 上下文、独立 digest | ✅ |
| 晋级 | `deploy/gitops/kargo/kargo-dev-staging.yaml` 只有**一个** Warehouse `star-services`，同时订阅 3 个镜像 → 一个 Freight 打包三者一起晋级 | ❌ 一个 App 升级会把其他 App 一起带上 |
| 调谐 | `deploy/gitops/argocd/argocd-applications.yaml` 按环境建 Application（`star-dev` / `star-staging`），每个指向整个 `deploy/k3s-local` 目录 | ❌ 同步粒度是整个环境 |
| 渐进发布 | ADR-0054 选定 Argo Rollouts，但仓内尚无按 App 的 Rollout / AnalysisTemplate | ❌ 未落地 |
| 数据库 | `db/migrations/` 是单一共享目录（110 个文件），未标注所属 App，也没有扩展/收缩（expand/contract）约定 | ❌ 迁移与 App 版本耦合不清 |
| 前端 | Vercel 独立发布 | ✅ |
| 桌面 | Tauri 独立发布链（ADR-0054 §5.1） | ✅ |
| 其他服务 | `star-api-rest`、`star-bff` 有独立 Deployment，但不在 Kargo Warehouse 中 | ⚠️ 未纳入晋级链 |

结论：**构建层已原子化，晋级、调谐、数据库三层还是整体发布**，目前不支持真正的单独更新。

## §3 目标架构

### 3.1 App 清单（App Manifest）

每个原子化 App 在仓内有一份 `deploy/apps/<app>/app.yaml`（新增），是单独发布的唯一事实来源：

```yaml
app: domain-canvas
image: ghcr.io/<owner>/star-domain-canvas
provides:
  api: { canvas-domain: v3 }
  events: [canvas.element.v2]
requires:
  apps: { canvas-engine: ">=0.3.0 <0.4.0" }
  events: [canvas.realtime.v1]
db:
  owner_schema: canvas_domain        # 只能迁移自己拥有的 schema
  policy: expand-contract
rollout:
  strategy: canary
  steps: [10, 30, 60, 100]
  analysis: [error-rate, p95-latency, event-backlog]
```

### 3.2 晋级与调谐按 App 拆分

| 层 | 目标 |
|---|---|
| Kargo | 每个 App 一个 Warehouse（只订阅自己的镜像）和自己的 Stage 链 dev → staging → prod；prod 需要平台管理员审批 |
| Argo CD | 用 ApplicationSet 按「App × 环境」生成 Application，每个指向 `deploy/apps/<app>/overlays/<env>` |
| Argo Rollouts | 每个 App 一个 Rollout + AnalysisTemplate；分析失败只回滚该 App |
| 控制台「单独更新」 | 调用 Kargo Promotion API 晋级该 App 的 Freight；「回滚」= 晋级上一个已验证的 Freight |

### 3.3 兼容性门禁（晋级前必过）

由 Kargo Stage 的 verification 步骤执行，结果回显到控制台：

1. 镜像签名与 SBOM 许可证检查（沿用现有 cosign 与许可闭包检查）。
2. API 契约：新版本 `provides` 不得删除或改变已被其他 App `requires` 的版本；只允许新增。
3. 依赖满足：新版本 `requires` 在目标环境当前部署的版本中可满足。
4. 事件 schema：新版本发布的事件对旧消费者向后兼容（字段只增不删）。
5. 数据库：只允许迁移自己 `owner_schema` 的表；破坏性变更必须拆成「扩展」→ 全量切换 → 「收缩」三次独立发布，且收缩只能在所有依赖方升级后进行。

任一项失败，控制台禁止该 App 晋级并显示原因。

### 3.4 运行期约束

- **N-1 兼容**：任一 App 的新版本必须能与其他 App 的当前版本和上一版本共存。
- **数据所有权**：App 之间不跨 schema 直接写表，只通过版本化 API 或事件交互（与 AGENTS.md「跨 owner 不直接写表」一致）。
- **消息传输**：事件 profile 与版本固定在消息上，升级不复用旧 dedupe 窗口（与 AGENTS.md 消息传输约束一致）。
- **功能开关**：新行为默认关闭，由 Feature Flags 在单独更新完成后逐步开启。

## §4 数据模型（W/T/M）

| 表 | 类 | 用途 |
|---|---|---|
| `ops.platform_admin` | M（SCD2） | 平台管理员名单，含最后一名保护约束 |
| `ops.app_release` | T（append-only） | 每次单独更新 / 回滚的记录：app、from、to、环境、操作者、结果 |
| `ops.app_compat_check` | T（append-only） | 每次兼容性门禁的检查项与结果 |
| `ops.console_session` | W（TTL 12h） | 控制台会话与二次确认令牌 |

所有 T/M 表带 RLS；控制台操作同时写 `audit_audit_event`。

## §5 API（`star-ops` 扩展）

| 方法 | 路径 | 权限 |
|---|---|---|
| GET | `/ops/v1/apps` | platform_operator |
| GET | `/ops/v1/apps/{app}/candidates` | platform_operator |
| POST | `/ops/v1/apps/{app}/promote` | platform_operator + 二次确认 |
| POST | `/ops/v1/apps/{app}/rollback` | platform_operator + 二次确认 |
| GET/POST/PATCH | `/ops/v1/tenants[/{id}]` | platform_operator |
| GET/PATCH | `/ops/v1/users/{id}/roles` | platform_operator（最后一名管理员保护） |
| GET | `/ops/v1/audit` | platform_operator |

## §6 重构步骤（建议顺序）

1. 新增 `deploy/apps/<app>/app.yaml`，先覆盖 3 个画布服务 + `star-api-rest` + `star-bff`。
2. 拆分 Kargo：每个 App 一个 Warehouse 与 Stage 链；保留现有合并 Warehouse 直到新链路验证通过。
3. Argo CD 改为 ApplicationSet（App × 环境），每个 App 有自己的 overlay。
4. 为每个 App 落地 Rollout + AnalysisTemplate。
5. 给 `db/migrations/` 标注所属 App 与 schema，并引入 expand/contract 检查脚本，接进兼容性门禁。
6. `star-ops` 增加 §5 API 与平台管理员引导；前端实现运维控制台。
7. 旧规格 `01-ops-requirements-spec.md` 加 superseded 注记。

## §7 已知缺口

| # | 缺口 | 影响 |
|---|---|---|
| G-1 | 现有 110 个迁移文件未标注所属 App，需要逐个归属 | 步骤 5 工作量未估 |
| G-2 | prod 环境与 prod 集群尚未接入（ADR-0054 第一阶段只有 dev/staging） | 控制台 prod 相关数据显示「未接入」 |
| G-3 | `star-api-rest` / `star-bff` 尚未进入镜像发布矩阵 | 需先纳入 CI 构建 |
| G-4 | 跨 App 的契约测试套件尚不存在 | 门禁第 2–4 项需新建 |
| G-5 | 旧 Ops 规格与本设计的合并需要确认 | 见 §1.3 |

## §8 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-11-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 初版：运维控制台角色与页签、单独发布现状核对、目标架构、兼容性门禁、重构步骤 |
