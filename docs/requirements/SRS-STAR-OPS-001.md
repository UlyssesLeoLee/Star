# SRS-STAR-OPS-001

> **STAR Ops Console 软件需求规格说明书 v0.1**
>
> - 状态: Requirements Baseline (Draft)
> - 目标阶段: 需求定义 → 基本设计 (MVP 入口 + 4 tab 占位)
> - 核心语言: Rust (后端) + TypeScript (前端)
> - 核心运行时: Tokio (后端) + Next.js (前端)
> - 核心架构: MVP-骨架 (4 tab 路由 + 8 REST stub + Hybrid AI mock + 真实通道 stub)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-08 JST

---

## §0 文档目的

本文档定义 STAR（`D:/Star`）项目"运维界面 (Ops Console)" 软件需求规格。

**触发**（per 2026-09-08 07:53 JST 用户发令）：在 TopBar 右上角 UserMenu 增加「运维」入口，承载运维应有的功能，包括不限于 (a) app 集群独立更新 (b) log AI 分析 (c) 运维数据 (d) 后续运维相关功能。

**MVP 范围**（per 2026-09-08 07:58 JST ask_user `ask_e76f2e614519fbc9eda16b53` 拍板）：
- 范围: **仅入口 + 4 个空壳页面 (4 tab 骨架 + 8 REST stub 端点)**
- AI 通道: **Hybrid: mock MVP + 真实通道 stub** (本地 mock 走 `scripts/automation/ai_log_mock.py` per 守门 #23; 同时留 OpenAI/Anthropic adapter 接口 + Fallback Ladder 4 级 per ADR-0026)
- 后端落位: **新建 crates/star-ops/** (跟 `star-context`/`star-mcp`/`star-api-rest` 平级, 47 → 48 package)
- 文档层: **仅需求 + 基本设计 2 段** (跳详设, 占位 §0 注明 "随实装迭代")

**不**做：
- 4 类功能端到端实装 (集群更新 / log AI / 运维数据 / 文档) — 仅 stub
- 跨域编排 / 任务卡 1:1 集成 (per AGENTS.md §4 #3 5 域独立 Lead 硬约束, 运维子域 Lead 真人到位前不实装)
- 真实 LLM 通道 (OpenAI/Anthropic) — 仅 stub
- PostgreSQL / Redis 持久化 — MVP 内存 + JSON stub

---

## §1 章节映射

需求章节用 IPA 模板，状态列：`✅ MVP 落档` / `🟡 MVP 部分` / `⏳ 待 [M]/[L] 子项` / `❌ 范围外`。

| # | 章节 | 状态 | 实证 / 待 |
|---|---|---|---|
| §2 | 业务目标 (Ops Console 入口) | ✅ | ask_user 拍板 Q1 opt1 |
| §3 | 范围与边界 (MVP-骨架 + 4 tab + 8 stub) | ✅ | ask_user 拍板 Q1/Q3/Q4 |
| §4 | 4 类功能定义 (F-01..F-04) | ✅ MVP stub / ⏳ 实装待子项 | 4 tab 占位 |
| §5 | AI 通道 (Hybrid: mock + stub) | ✅ | ask_user 拍板 Q2 + ADR-0026 Fallback Ladder 4 级 |
| §6 | 角色与责任 (守门 #3 + 守门 #14) | ✅ | Mavis 临时代签, 5 域 Lead 真人到位后追溯 |
| §7 | NFR (性能/可用/安全/可观测) | ✅ MVP / ⏳ 实装 | 守门 #1+#7+#13 派生 |
| §8 | 数据分类 (W/T/M 守门 #13) | ✅ | T/M/W 三类 100% 覆盖, 表 6 张 |
| §9 | 验收 (AC-001..AC-008 MVP 8 项) | ✅ | per 测试阶段落地 |
| §10 | 风险与依赖 | ✅ | 跨 6 子项 (P3-A 实装 2-3M token 估) |

---

## §2 业务目标

### 2.1 入口与定位

| 维度 | 描述 |
|---|---|
| 入口位置 | TopBar 右上角 UserMenu (per `frontend/src/components/UserMenu.tsx` line 131-353 现有菜单结构) |
| 入口文案 | `运维` (zh-CN) / `Ops` (en) / `運用` (ja) — i18n 走 `lib/i18n.ts` 现有机制 |
| 入口图标 | `lucide-react` `Wrench` (运维语义, 跟现有 `User/Terminal/Key/Cpu/Globe` 风格一致) |
| 路由 | `/ops` (Next.js App Router, 跟 `/automation-debug` 同级) |
| 权限 | 跟现有 "tenant_admin" 角色绑定 (per `frontend/src/components/UserMenu.tsx` line 113-115), 后续可拆 `ops_admin` 子角色 |

### 2.2 4 类功能总览 (per 用户发令 "运维应有的功能")

| ID | 功能 | MVP 范围 | 实装阶段 |
|---|---|---|---|
| F-01 | **app 集群独立更新** (K8s/Helm 灰度/回滚) | tab 占位 + 4 REST stub (列出 release / 触发灰度 / 回滚 / 状态) | [M] 子项 (估 ~600K token) |
| F-02 | **log AI 分析** (log 采集 + LLM 摘要 + 异常检测) | tab 占位 + 2 REST stub (上传 log / 查询分析结果), mock AI 走 `ai_log_mock.py` | [M] 子项 (估 ~800K token) |
| F-03 | **运维数据** (KPI / 仪表盘 / 趋势) | tab 占位 + 1 REST stub (metrics 列表), 内存 + JSON | [M] 子项 (估 ~400K token) |
| F-04 | **运维文档** (需求 / 设计 / 报告入口) | tab 占位 + 1 REST stub (列出 ops 相关 docs), 链接到 `docs/reports/` + `docs/requirements/` | [S] 子项 (估 ~200K token) |

**4 类功能 MVP 累计估 = 2.0-2.4M token** (后续 4 个 [M] 子项拍板后逐个推进)。

---

## §3 范围与边界

### 3.1 MVP-骨架 落档清单 (本次 commit)

| # | 文件 | 类型 | 说明 |
|---|---|---|---|
| 1 | `docs/requirements/SRS-STAR-OPS-001.md` | 需求 | 本文件 |
| 2 | `docs/basic-design/OPS-BASIC-DESIGN-001.md` | 基本设计 | 4 tab + 8 stub + W/T/M 表 + 组件/接口 |
| 3 | `docs/reports/PHASE-OPS-INTRY-REPORT.md` | 报告 | 7 段 (per AGENTS.md §3 模板) |
| 4 | `crates/star-ops/Cargo.toml` | crate 元数据 | name=star-ops, version=workspace, 47 → 48 package |
| 5 | `crates/star-ops/src/lib.rs` | lib 入口 | 暴露 `ops_api::routes` + `ops_domain::*` + `ops_ai::*` |
| 6 | `crates/star-ops/src/main.rs` | binary 入口 | `star-ops` 启动 axum server, 端口 8090 (per star-mcp 8080/8081 顺延) |
| 7 | `crates/star-ops/src/error.rs` | 错误模型 | 复用 star-mcp 6-field 错误模型 (code/message/source_module/source_kind/retriable/hint) |
| 8 | `crates/star-ops/src/ops_api.rs` | REST 路由 | 8 stub 端点 (4 tab × 2), 全部 `501 NOT_IMPLEMENTED` + mock data |
| 9 | `crates/star-ops/src/ops_domain/mod.rs` | 领域模块 | 3 子域: cluster / log / metrics, 各 1 数据结构 stub |
| 10 | `crates/star-ops/src/ops_domain/cluster.rs` | 集群子域 | `HelmRelease` struct, R-05 守门 (守门 #1) 暂不实装 helm client |
| 11 | `crates/star-ops/src/ops_domain/log.rs` | log 子域 | `LogEntry` + `LogAnalysis` struct, mock AI 接入点 |
| 12 | `crates/star-ops/src/ops_domain/metrics.rs` | 运维数据子域 | `OpsMetric` struct, 内存 + stub JSON |
| 13 | `crates/star-ops/src/ops_ai/mod.rs` | AI 通道模块 | Hybrid: mock + OpenAI stub + Anthropic stub + Fallback Ladder 4 级 (per ADR-0026 §2.2) |
| 14 | `crates/star-ops/src/ops_ai/mock.rs` | mock 通道 | 调用 `scripts/automation/ai_log_mock.py` subprocess, 跟 `console_server.py` 同模式 (per 守门 #24 v2) |
| 15 | `crates/star-ops/src/ops_ai/openai_stub.rs` | OpenAI stub | trait `AiChannel` + struct, 真实调用占位返回 mock, 标 `// TODO: 真实 API` |
| 16 | `crates/star-ops/src/ops_ai/anthropic_stub.rs` | Anthropic stub | 同上 |
| 17 | `crates/star-ops/src/ops_ai/ladder.rs` | Fallback Ladder | 4 级回退逻辑, mock 优先, 真实通道未配置自动回退 |
| 18 | `frontend/src/components/UserMenu.tsx` | 入口 (修改) | line ~192 工具区加 "运维" Link + `Wrench` icon, 1 commit 改动 |
| 19 | `frontend/src/app/ops/page.tsx` | 路由 (新建) | 4 tab skeleton (集群更新 / log AI / 运维数据 / 文档), 跟 `/automation-debug` 同 3D-style 视觉 |
| 20 | `frontend/src/app/ops/components/OpsHeroHeader.tsx` | 头部 | 跟 `automation-debug/components/HeroHeader.tsx` 同模式 |
| 21 | `frontend/src/app/ops/components/ClusterTab.tsx` | F-01 tab | 占位 + 4 卡片 (列出 release / 灰度 / 回滚 / 状态) |
| 22 | `frontend/src/app/ops/components/LogAITab.tsx` | F-02 tab | 占位 + 上传区 + AI 分析结果区 |
| 23 | `frontend/src/app/ops/components/MetricsTab.tsx` | F-03 tab | 占位 + 5 KPI 卡片 + 趋势占位 |
| 24 | `frontend/src/app/ops/components/DocsTab.tsx` | F-04 tab | 占位 + 文档链接列表 |
| 25 | `frontend/src/app/ops/lib/ops-api.ts` | 前端 API client | 8 个 fetch wrapper, 全部走 `process.env.NEXT_PUBLIC_OPS_URL || http://localhost:8090` |
| 26 | `frontend/src/lib/i18n.ts` | i18n (修改) | 加 `userMenu.ops` + `ops.*` 4 tab 翻译 (zh-CN/en/ja) |
| 27 | `scripts/automation/ai_log_mock.py` | AI mock 脚本 | 模板生成 (守门 #23), 输入 log 文本 → 输出 3 条建议 + features_context 联动 |
| 28 | `scripts/automation/registry.md` | 脚本索引 (修改) | 加 `ai_log_mock.py` 一行 (per 守门 #21 v21) |
| 29 | `docs/automation-design.md` | 自动化档 (修改) | §4 任务卡表加 OPS-INTRY 一行 (per 守门 #21 v21) |
| 30 | `Cargo.toml` | workspace (修改) | members 加 `crates/star-ops` 一行 |

### 3.2 不做什么 (守门 #11 缺标比错标安全)

- **不**实装 4 类功能端到端 (F-01..F-04) — 等 [M] 子项拍板
- **不**实装真实 K8s/Helm client (R-05 不 push 反转已落地, 仍守 R-05 工具/数据接口不接生产)
- **不**实装真实 OpenAI/Anthropic 调用 — 仅 stub (守门 #23 AI mock 不开外部 API)
- **不**实装 PostgreSQL/Redis 持久化 — MVP 内存 + JSON stub, 守门 #13 W/T/M 表结构先定
- **不**实装跨域编排 / 任务卡集成 — 守门 #3 5 域独立 Lead 硬约束, 运维子域真人到位前不实装
- **不**实装详设文档 — per 拍板 Q4 跳详设, 详设随实装迭代
- **不**实装 OpenAPI 3.1 spec 自动生成 — per `star-api-rest` v0.1 同模式
- **不**实装 OAuth 2.0 / mTLS — MVP 阶段 auth stub, 复用 `star-context::ActorContext` (per 守门 #4.2 v16)

---

## §4 4 类功能定义 (F-01..F-04)

### F-01 app 集群独立更新 (Cluster)

| 维度 | MVP stub | 实装阶段 |
|---|---|---|
| API | `GET /api/ops/cluster/releases` · `POST /api/ops/cluster/canary` · `POST /api/ops/cluster/rollback` · `GET /api/ops/cluster/status` | [M] 子项 (估 ~600K token) |
| 数据结构 | `HelmRelease { name, namespace, chart, revision, status: ReleaseStatus, last_deployed_at, canary_weight }` | stub 4 条 hardcoded |
| 业务规则 | 灰度权重 0-100, 回滚保留前 N 个 revision, 状态机: Pending → Deploying → Healthy/Degraded/Failed | MVP 仅 enum + 注释 |
| UI 卡片 | 4 张: 列出 release / 触发灰度 (滑块) / 回滚 (选择 revision) / 状态指示 | 占位 + 假数据 |
| 权限 | `ops_admin` (MVP 跟 tenant_admin 合并) | 后续 DDD Review 拆 |
| 审计 | T 类 `ops_cluster_action_log` (WORM) | 表结构见 §8 |
| 依赖 | K8s client (kube-rs) / Helm SDK — 实装时引入 | MVP 不引入 |
| **不**做 | 实装 helm exec / kubectl apply / 实数据源 | — |

### F-02 log AI 分析 (LogAI)

| 维度 | MVP stub | 实装阶段 |
|---|---|---|
| API | `POST /api/ops/log/upload` · `GET /api/ops/log/analysis/{id}` | [M] 子项 (估 ~800K token) |
| 数据结构 | `LogEntry { id, source, level: LogLevel, message, timestamp, trace_id }` + `LogAnalysis { log_id, summary, anomalies: Vec<Anomaly>, suggestions: Vec<Suggestion>, confidence: f32, generated_by: AiChannel }` | stub 1 条 + AI mock 输出 |
| AI 通道 | Hybrid per ADR-0026: mock (MVP) → OpenAI stub → Anthropic stub → Fallback Ladder 4 级 | mock 优先, 真实通道未配置自动回退 |
| 业务规则 | confidence < 0.5 必标 "需人工 review" (per 守门 #23); log 上传大小限制 1MB; retention 7 天 (W 类) | MVP 内存 + TTL 1 小时 |
| UI 卡片 | 2 张: 上传区 (拖拽 / 粘贴) + 分析结果 (摘要 + 异常 + 建议) | 占位 + 假数据 |
| 权限 | `ops_admin` | — |
| 审计 | T 类 `ops_log_query_log` | 表结构见 §8 |
| 依赖 | AI mock: `scripts/automation/ai_log_mock.py`; 真实通道实装时引入 reqwest + serde_json | MVP 调 subprocess (守门 #24) |
| **不**做 | 实装 log 采集 agent (Fluent Bit/Loki) / 真实 LLM 调用 | — |

### F-03 运维数据 (Metrics)

| 维度 | MVP stub | 实装阶段 |
|---|---|---|
| API | `GET /api/ops/metrics/summary` | [M] 子项 (估 ~400K token) |
| 数据结构 | `OpsMetric { name, value, unit, trend: TrendDirection, last_updated }` 5 条 stub | stub 5 KPI hardcoded |
| 业务规则 | 5 KPI: CPU 平均 / 内存平均 / 活跃任务卡数 / MCP 调用 QPS / LLM token 消耗 | MVP 假数据 |
| UI 卡片 | 5 张 KPI 胶囊 + 趋势占位 (折线/柱状) | 占位 |
| 权限 | `ops_admin` | — |
| 审计 | M 类 `ops_metrics_config` (slowly changing) | 表结构见 §8 |
| 依赖 | Prometheus client (实装时引入) / 内部 metrics (`star-telemetry` 已有, 复用) | MVP 复用 telemetry stub |
| **不**做 | 实装 Grafana 集成 / 时序数据库 / 告警 | — |

### F-04 运维文档 (Docs)

| 维度 | MVP stub | 实装阶段 |
|---|---|---|
| API | `GET /api/ops/docs` | [S] 子项 (估 ~200K token) |
| 数据结构 | `OpsDocRef { path, title, category, updated_at }` 列 `docs/reports/` + `docs/requirements/` 跟 ops 相关的 md | 扫描结果内存缓存 |
| 业务规则 | 类别: SRS / BAS / 报告; 按 updated_at 倒序; 支持 1 层跳转 | MVP 仅 GET, 不支持编辑 |
| UI 卡片 | 1 张: 文档链接列表 (10 条上限) | 占位 |
| 权限 | `ops_admin` | — |
| 审计 | T 类 `ops_doc_read_log` | 表结构见 §8 |
| 依赖 | 内部 doc 扫描 (walkdir) | MVP 不引入 walkdir, hardcoded 列表 |
| **不**做 | 实装 Markdown 渲染 / 全文搜索 / 编辑 | — |

---

## §5 AI 通道 (Hybrid per 守门 #23 + ADR-0026)

### 5.1 通道矩阵

| 通道 | MVP 状态 | 实现 | Fallback Level |
|---|---|---|---|
| **Local mock** (scripts/automation/ai_log_mock.py) | ✅ 启用 | subprocess.run, 模板生成 | L1 首选 |
| **OpenAI stub** | 🟡 接口就绪, 真实调用 `// TODO` | trait `AiChannel::complete()`, 占位返回 mock | L2 |
| **Anthropic stub** | 🟡 同上 | 同上 | L3 |
| **Git Only** (per ADR-0026 L4) | ❌ MVP 不涉及, ops 域不需要 git | — | L4 兜底 |

### 5.2 Fallback Ladder 4 级 (per ADR-0026 §2.2)

```
L1: mock (默认) ── 配置真实通道时升级 L2
   ↓ 配置未生效 / 真实通道失败
L2: OpenAI ──── 401/429/5xx 自动降级
   ↓
L3: Anthropic ── 同上
   ↓
L4: mock (兜底, 永远可用)
```

### 5.3 mock 实现 (守门 #23)

```python
# scripts/automation/ai_log_mock.py 摘要
# 输入: log 文本 (stdin)
# 输出: 3 条建议 (add_field / remove_method / rename_class) + features_context 联动
# confidence 永远 < 0.5, 提示用户手动 review
# 不开 OpenAI/Anthropic 第三方 API (守门 #23)
# API key 不走 UI 输入
# 可重放: subprocess.run 模式, 跟守门 #9 v3 实证一致
```

---

## §6 角色与责任 (守门 #3 + 守门 #14)

### 6.1 角色矩阵 (RACI per 守门 #14 v2)

| 角色 | R | A | C | I | 备注 |
|---|---|---|---|---|---|
| **架构师 (Mavis 接手)** | ✅ | ✅ | — | — | MVP-骨架 临时代签, 拍板 + 落地 |
| **运维子域 Lead** | ⏳ | ⏳ | — | — | 真人到位前 Mavis 临时代签, 到位后追溯签字 |
| **SRE Lead** | ⏳ | ⏳ | — | — | 集群/性能相关, 真人到位前 Mavis 代签 |
| **平台 Lead** | ⏳ | ⏳ | — | — | 部署/可观测, 真人到位前 Mavis 代签 |
| **评审主持** | ⏳ | ⏳ | — | — | DDD Review 阶段触发 |
| **PM** | ⏳ | ⏳ | — | — | 子项排期, 真人到位前 Mavis 代签 |

### 6.2 决策 scope (per 守门 #14 v2 拍板 9/3 19:43 JST)

- 跨域: ✅ (运维跨越 cluster/log/metrics 3 子域)
- 域内: ✅ (单一 ops 域内决策)

### 6.3 到位 timeline

- 5 域 Lead 真人到位: 待定 (per 守门 #14 + `docs/recruitment/5-business-domain-lead-referral.md` v0.1)
- 运维子域 Lead: 跟随 5 域 Lead, 暂未独立招聘
- Mavis 临时代签维持期: 真人到位前 (per 9/3 19:35 JST 拍板 D)

### 6.4 Mavis 代签边界 (per 守门 #14 v2 拍板 + 守门 #10)

- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10)
- 修订人 = `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per §2.3)
- 审批 = `架构师 (Mavis 接手 agent per DEC-008)` (per §2.2)
- 派生约束保留: 禁回溯叙事 / BAS git log --follow 实证 / 缺标比错标 / 子代理授权"无证据叙事=禁止"

---

## §7 NFR (非功能需求)

### 7.1 性能 (per 守门 #1 + #4 token-OLU)

| 指标 | MVP 目标 | 实装目标 |
|---|---|---|
| 首屏 LCP | < 1.5s (本地) | < 1.0s (生产) |
| API P95 | < 200ms (stub) | < 100ms (实装) |
| tab 切换 | < 100ms (本地) | < 50ms (生产) |
| AI mock 调用 | < 500ms (subprocess) | < 200ms (实 LLM 通道) |
| Cargo check | < 35s (per 守门 #1 v19 -j 4 派生) | 0 err |

### 7.2 可用性 (Availability)

- MVP: 单实例 axum 0.8, 端口 8090, 无 HA
- 实装: 跟 star-mcp 0.8 同模式, K8s deployment + health check `/healthz`

### 7.3 安全 (守门 #5 v2 派生 + 守门 #7 0 unsafe)

- API key 经 UI 配置, 不进环境变量 (守门 #5)
- 真实 LLM 通道 API key 走 star-credential crate 加密存储 (per `crates/star-credential`, 已有)
- 0 unsafe 代码 (守门 #7)
- AI mock 不开外部 API (守门 #23)
- 子代理 dispatch 必先 brief 落地 (守门 #20 v9)

### 7.4 可观测 (Observability)

- MVP: tracing crate 日志 (复用 workspace tracing 依赖)
- 实装: star-telemetry 集成 (已存在, 复用)
- 错误码: 6-field 模型 (per star-mcp `error.rs` + 守门 #6)

### 7.5 守门 (Quality Gate per 守门 #1)

MVP-骨架 必跑 (per §4.1 累积规 v19+):
1. `cargo check --workspace --all-targets -j 4` 0 err
2. `cargo fmt --all -- --check` 0 err
3. `cargo clippy --workspace --all-targets -- -D warnings` 0 err (advisory 模式 per 守门 #7 v3)
4. `cargo test -p star-ops --lib -j 4` 100% pass
5. frontend `npm run typecheck` 0 err

---

## §8 数据分类 (W/T/M per 守门 #13 强制分类)

### 8.1 100% 表覆盖 (6 张表)

| # | 表名 | 分类 | 派生规 | 说明 |
|---|---|---|---|---|
| 1 | `ops_helm_release_state` | **T** (业务事实) | T = 物理删除禁止 + 監査必須 + RLS 13 類必携 | F-01 集群 release 当前状态快照 (per F-01 監査) |
| 2 | `ops_cluster_action_log` | **T** (WORM) | T 同上 | F-01 灰度/回滚 动作审计 (per F-01 監査) |
| 3 | `ops_log_query_log` | **T** (WORM) | T 同上 | F-02 log 查询审计 (per F-02 監査) |
| 4 | `ops_log_entry` | **W** (短 TTL) | W = 物理删除 / タイマー失効 / 短 TTL 明示 retention | F-02 log 原始数据, retention 7 天, TTL 失効 |
| 5 | `ops_log_analysis` | **W** (短 TTL) | W 同上 | F-02 AI 分析结果, retention 30 天, 跟 log_entry 1:N |
| 6 | `ops_metrics_config` | **M** (SCD) | M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携 | F-03 运维数据配置 (采集源/告警阈值) |

**混合分類**: 0 张 (100% 干净)
**覆盖**: 6/6 = 100% (per 守门 #13 100% 表覆盖硬约束)

### 8.2 RLS 13 類必携 (T/M 通用)

T 类表 (3 张) + M 类表 (1 张) = 4 张必携 RLS 13 類 (per 守门 #13 派生规 (b)+(c)):
- tenant_id, user_id, role_id, permission_id, policy_id, workspace_id, project_id, work_item_id, agent_id, session_id, trace_id, source_module, source_kind

W 类表 (2 张) 仅带 retention_period (per 守门 #13 派生规 (d))。

### 8.3 引用基线 (per 守门 #13)

- `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1 (100 表 W/T/M 三類索引实绩)
- `docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md` v0.1 (跨项目 ルール手册 + 4 段检查清单 + 派生守门 10 条 CW-01~CW-10)

---

## §9 验收 (Acceptance Criteria, MVP 8 项)

| AC | 描述 | 验证方式 |
|---|---|---|
| AC-001 | UserMenu 右上角显示「运维」入口 (Wrench 图标) | 浏览器手动验证 + e2e test (可选) |
| AC-002 | 点击入口跳转到 `/ops` 路由 | 浏览器手动验证 |
| AC-003 | `/ops` 路由显示 4 tab (集群更新 / log AI / 运维数据 / 文档) | 浏览器手动验证 |
| AC-004 | 每个 tab 显示 "即将开放" 占位 + 真实 API 契约可见 | curl `localhost:8090/api/ops/*` 返 501 + mock data |
| AC-005 | `cargo check --workspace --all-targets -j 4` 0 err | 守门 #1 实证 |
| AC-006 | `cargo test -p star-ops --lib -j 4` 100% pass | 守门 #1 实证 |
| AC-007 | frontend `npm run typecheck` 0 err | 守门 #1 实证 |
| AC-008 | i18n 三语 (zh-CN/en/ja) 完整覆盖入口文案 + 4 tab 标题 | 手动验证 |

[后续 [M]/[L] 子项拍板后, 续 AC-009+ 实装端到端验收]

---

## §10 风险与依赖

### 10.1 风险

| # | 风险 | 等级 | 缓解 |
|---|---|---|---|
| 1 | 5 域 Lead 真人未到位, Mavis 长期代签 (守门 #14 拍板 D 维持) | 中 | 真人到位后追溯签字, 不沿用代签决策 (守门 #1 禁回溯) |
| 2 | Hybrid AI 通道切换路径未跑 e2e | 中 | [M] 子项拍板后补 e2e, 守门 #1 v3 累积规必跑 |
| 3 | K8s/Helm 实装引入 kube-rs 依赖, 编译时长 | 低 | MVP 不引入, [M] 子项评估 |
| 4 | LLM API key 误配 (环境变量泄露) | 中 | 守门 #5 v2 派生, UI 配置 + star-credential 加密 |
| 5 | 4 类功能 MVP 仅占位, 用户期望偏高 | 中 | §3.1 落档清单 + §3.2 "不做什么" 显式列, 跟 ask_user 拍板一致 |

### 10.2 依赖

| # | 依赖 | 状态 | 备注 |
|---|---|---|---|
| 1 | `star-mcp` 0.8 + axum 0.8 + 6-field 错误模型 | ✅ 存在 | 复用 error.rs 模式 |
| 2 | `star-api-rest` 鉴权 stub | ✅ 存在 | 复用 AuthLayer 模式 (Phase M+ 实装) |
| 3 | `star-context::ActorContext` | ✅ 存在 | P0-1 联动审计共享 (per 守门 #4.2 v16) |
| 4 | `star-credential` API key 加密 | ✅ 存在 | 实装阶段引入 |
| 5 | `star-telemetry` 观测 | ✅ 存在 | 实装阶段引入 |
| 6 | `domain-tenant` / `domain-permission` (守门 #1 v16) | ✅ 存在 | 复用 |
| 7 | `kube-rs` / `helm` SDK | ⏳ 待 [M] 子项评估 | MVP 不引入 |
| 8 | LLM 通道 SDK (reqwest + serde_json) | ✅ 已有依赖 | MVP stub, 实装时启用 |

### 10.3 子项估 (per 守门 #4 token-OLU + §7 质量门)

| 子项 | 估 (token) | 优先级 | 拍板后启动 |
|---|---|---|---|
| F-02 log AI 端到端 | ~800K | [M] | 拍板 Q1 触发 |
| F-01 集群更新 端到端 | ~600K | [M] | 拍板 Q1 触发 |
| F-03 运维数据 端到端 | ~400K | [M] | 拍板 Q1 触发 |
| F-04 运维文档 端到端 | ~200K | [S] | 拍板 Q1 触发 |
| **累计** | **~2.0M** | — | 4 个子项拍板后逐个 |

---

## §11 签字栏 (per AGENTS.md §3 模板)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

> 5 域 Lead 真人到位后追溯签字覆盖 (per 守门 #14 + §6.3), 不沿用代签决策 (per 守门 #1 禁回溯)

---

## §12 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 MVP-骨架 需求定义 (4 tab + 8 stub + Hybrid AI + W/T/M 6 表) | ask_user `ask_e76f2e614519fbc9eda16b53` 拍板 (4 opt 选 4 项) |
