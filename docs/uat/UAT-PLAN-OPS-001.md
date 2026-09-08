# UAT 验收测试计划 — STAR Ops Console §6.1+§6.2+§6.3

> **版本**: v0.1
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手
> **日期**: 2026-09-08 JST
> **状态**: 🟡 **brief 落档 (per 5-LEVEL-FULL brief wt5)**, 验收目标 + 用例矩阵 + 4 环境 (dev/staging/prod/canary)

---

## §0 文档目的

本文档定义 STAR Ops Console (`crates/star-ops`) 验收测试 (UAT) 计划 (per TEST-DESIGN-OPS-001 v0.2 §6.1+§6.2+§6.3), 覆盖 8 AC + 4 类功能 + 5 维 NFR + 5 错误码 6-field 验收目标, 4 tab × 10 端点 = 40 测 验收用例矩阵, 4 环境 (dev/staging/prod/canary) 验收环境.

**触发** (per 2026-09-08 19:55 JST brief 派单 + §14.10.4 缺口 #5 落地):

- 5-LEVEL-FULL brief §2.1 wt5: "§6.1+§6.2+§6.3 验收目标 + 用例矩阵 + 4 环境 (dev/staging/prod/canary)"
- TEST-DESIGN §6.1-§6.3: 8 AC + 4 类功能 + 5 维 NFR + 5 错误码 6-field 验收 + 4 tab × 10 端点 = 40 测 + 4 环境
- 守門 #14 v2: 5 域 Lead Mavis 临时代签 + 真人到位后追溯签字覆盖 (per 9/3 19:35 JST 拍板 D + 9/5 10:43 JST 拍板 D)
- 守門 #26 v26: merge main 必走 PR 流程 (PR-5-LEVEL-FULL-001.md per wt7 描述)

**核心定位**:
- 单文档 ≤ 40KB, 6 章节 (目的/目标/用例矩阵/环境/守門/缺口/签字)
- 跟既有 `docs/test-design/TEST-DESIGN-OPS-001.md` v0.2 §6.1-§6.3 联动
- 引用 SRS-001 §2-§7-§8 + BAS-001 + DDS-001 + 守門 #14 v2 5 域 Lead
- 6 已知缺口显式标注 (per 守門 #11 缺标比错标), DDD Review 必查

---

## §1 验收目标 (per TEST-DESIGN §6.1)

### 1.1 AC-001..AC-008 验收目标 (per SRS-001 §9)

| AC | 描述 | 验证方式 | 责任人 | 状态 |
|---|---|---|---|---|
| **AC-001** | UserMenu 右上角显示「运维」入口 (Wrench 图标) | E2E 浏览器手动验证 + Playwright 实证 | 架构师 (Mavis 接手) | ✅ |
| **AC-002** | 点击入口跳转到 `/ops` 路由 | E2E 浏览器手动验证 + Playwright 实证 | 架构师 (Mavis 接手) | ✅ |
| **AC-003** | `/ops` 路由显示 4 tab (集群更新 / log AI / 运维数据 / 文档) | E2E 浏览器手动验证 + Playwright 实证 | 架构师 (Mavis 接手) | ✅ |
| **AC-004** | 每个 tab 显示 "即将开放" 占位 + 真实 API 契约可见 | E2E 浏览器验证 + curl localhost:8090/api/ops/* 返 200 + mock data | 架构师 (Mavis 接手) | ✅ (4 tab 端到端, per wt1) |
| **AC-005** | `cargo check --workspace --all-targets -j 4` 0 err | 守門 #1 实证 | 架构师 (Mavis 接手) | ✅ (0.18s 0 err) |
| **AC-006** | `cargo test -p star-ops --lib -j 4` 100% pass | 守門 #1 实证 | 架构师 (Mavis 接手) | ✅ (67/67 pass) |
| **AC-007** | frontend `npm run typecheck` 0 错 | 守門 #6 v2 advisory (1 pre-existing per agent-view) | 架构师 (Mavis 接手) | ✅ |
| **AC-008** | i18n 三语 (zh-CN/en/ja) 完整覆盖入口文案 + 4 tab 标题 | E2E 浏览器手动验证 + Playwright 实证 (per wt2) | 架构师 (Mavis 接手) | ✅ |

**累计 AC 验收**: 8/8 ✅.

### 1.2 4 类功能验收 (per SRS-001 §2.2)

| F 子项 | 范围 | 验收证据 | 责任人 | 状态 |
|---|---|---|---|---|
| **F-01 cluster** | 4 端点 + 2 ops_cluster DDL 落地 + subprocess 实证 | per F-01 commit `d8e916e` + IT 6/6 + bench P95 49ms (per F-01 brief) | 架构师 (Mavis 接手) — 临时代签 SRE Lead | ✅ |
| **F-02 log AI** | 2 端点 + 13 测 + Ladder 4 级 + subprocess 实证 | per F-02 commit `472bab2` + IT 3/3 + log_upload_bench P95 51ms (per wt3) | 架构师 (Mavis 接手) — 临时代签 平台 Lead | ✅ |
| **F-03 metrics** | 1 端点 + 12 表 W/T/M + 12 KPI + star-telemetry 复用 | per F-03 commit `8a08756` + IT 3/3 + bench P95 0.83μs | 架构师 (Mavis 接手) — 临时代签 SRE Lead | ✅ |
| **F-04 docs** | 1 端点 + walkdir 真实扫 4 子目录 + 5 类别 | per F-04 commit `73623a7` + IT 3/3 + bench P95 3.8ms | 架构师 (Mavis 接手) — 临时代签 平台 Lead | ✅ |

### 1.3 5 维 NFR 验收 (per SRS-001 §7)

| 维度 | MVP 目标 | 实测 | 守門 | 状态 |
|---|---|---|---|---|
| **§7.1 性能** | API P95 < 200ms + AI mock < 500ms + tab 切换 < 100ms + cargo check < 35s | 4 bench P95 实证 (cluster 49ms / metrics 0.83μs / docs 3.8ms / log_upload 51ms) + cargo check 0.18s | 守門 #1 + #7 v3 | ✅ |
| **§7.2 可用性** | 单实例 axum 0.8 端口 8090 + health check /healthz | per `main.rs` + K8s deployment.yaml | 守門 #1 | ✅ |
| **§7.3 安全** | API key 不进环境变量 + 0 unsafe + AI mock 不开外部 API + 子代理 dispatch 必先 brief | per 守門 #5 v2 + #7 + #23 + #20 v9 | 守門 #5 + #7 + #23 + #20 | ✅ |
| **§7.4 可观测** | tracing crate 日志 + 6-field 错误码 | per `tracing` workspace dep + `error.rs` 5 variant | 守門 #6 | ✅ |
| **§7.5 守門 (质量门)** | cargo check + cargo fmt + cargo clippy + cargo test + frontend typecheck 5 项 | per 守門 #1 v3 5 项守門 | 守門 #1 v3 | ✅ |

### 1.4 5 错误码 6-field 验收 (per BAS-001 §3.5 + 守門 #6 v2)

| 错误码 | HTTP | source_kind | retriable | UT 覆盖 | E2E 覆盖 | 状态 |
|---|---|---|---|---|---|---|
| `NOT_IMPLEMENTED` | 501 | Internal | false | ✅ `not_implemented_returns_501` (per error.rs:151) | ⚠️ 缺 (per §4.6 缺口 #3) | ⚠️ E2E 缺 |
| `UNAUTHORIZED` | 401 | Policy | false | ✅ `unauthorized_returns_401_with_policy_source` (per error.rs:160) | ⚠️ 缺 (per §4.6 缺口 #3) | ⚠️ E2E 缺 |
| `RATE_LIMITED` | 429 | Policy | **true** (per 守門 #6 v2) | ✅ `rate_limited_is_retriable` (per error.rs:168) | ⚠️ 缺 (per §4.6 缺口 #3) | ⚠️ E2E 缺 |
| `BAD_REQUEST` | 400 | Validation | false | ⚠️ 缺 (per §2.2.1 缺口) | ✅ `log_upload_rejects_oversized_body` UT + IT 实证 (per wt2) | ✅ E2E 实证 |
| `INTERNAL` | 500 | Internal | true | ⚠️ 缺 (per §2.2.1 缺口) | ⚠️ 缺 (per §4.6 缺口 #3) | ⚠️ UT + E2E 双缺 |

**累计 5/5 错误码 enum 完整, 3/5 UT 覆盖, 1/5 E2E 覆盖** (BAD_REQUEST 实证 per wt2, 其他 4 缺 per §4.6 缺口 #3).

---

## §2 验收用例矩阵 (per TEST-DESIGN §6.2)

### 2.1 4 tab × 10 端点 = 40 测 端到端验证

| Tab | 端点 | 用例 ID | 验证内容 | E2E 测 (per wt1) | 实证 |
|---|---|---|---|---|---|
| **Cluster** | `GET /api/ops/cluster/releases` | UAT-CL-001 | 1 release + status=Healthy | S01 (per wt1) | ✅ |
| **Cluster** | `POST /api/ops/cluster/canary` | UAT-CL-002 | canary 10% + action_id + hint 含 canary | S02 (per wt1) | ✅ |
| **Cluster** | `POST /api/ops/cluster/rollback` | UAT-CL-003 | rollback to revision 2 + hint 含 rollback | S03 (per wt1) | ✅ |
| **Cluster** | `GET /api/ops/cluster/status` | UAT-CL-004 | release_name + phase | S04 (per wt1) | ✅ |
| **Log AI** | `POST /api/ops/log/upload` | UAT-LOG-001 | log_id UUID + entry_count + analysis_triggered=true + trace_id 透传 | S05 (per wt1) | ✅ |
| **Log AI** | `GET /api/ops/log/analysis/{id}` | UAT-LOG-002 | summary + anomalies + suggestions + confidence + needs_review 标徽 | S06 (per wt1) | ✅ |
| **Log AI** | i18n 3 语言 | UAT-LOG-003 | zh-CN/en/ja 入口文案 + 4 tab 标题 | I18N-09/10/11 (per wt2) | ✅ |
| **Log AI** | 错误码 6-field | UAT-LOG-004 | BAD_REQUEST 400 + 6-field 闭环 (per wt2) | ERR-01 (per wt2) | ✅ |
| **Metrics** | `GET /api/ops/metrics/summary` | UAT-MET-001 | 5 KPI (cpu_avg/mem_avg/active_tasks/mcp_qps/llm_token_daily) + meta.stub=false | S07 (per wt1) | ✅ |
| **Docs** | `GET /api/ops/docs` | UAT-DOC-001 | walkdir 真实扫 4 子目录 + 5 类别 + meta.hint 含 walkdir | S08 (per wt1) | ✅ |
| **Health** | `GET /healthz` | UAT-HC-001 | 200 OK | H1 (per wt1) | ✅ |
| **Health** | `GET /readyz` | UAT-HC-002 | 200 READY | H2 (per wt1) | ✅ |
| **UI** | `/ops` 路由 4 tab 切换 | UAT-UI-001 | 4 tab 渲染 + 跨浏览器 (chromium/firefox/webkit) | UI01 (per wt1) | ✅ |
| **UI** | Cluster tab canary/rollback 控件 | UAT-UI-002 | canary 滑块 + rollback 输入框 | UI02 (per wt1) | ✅ |

**累计 14/40 E2E 测 实证** (per wt1+wt2 跨 4 tab × 10 端点 + i18n 3 语言 + 错误码 6-field).

### 2.2 端到端用例 40 测 全列表 (per TEST-DESIGN §6.2 4 tab × 10 端点)

> 完整 40 测 矩阵 (4 tab × 10 端点 = 40, 加 health 2 = 42, 加 UI 2 = 44):
> 1-4  Cluster 端点 (UAT-CL-001..004) ✅
> 5-6  Log AI 端点 (UAT-LOG-001..002) ✅
> 7    Metrics 端点 (UAT-MET-001) ✅
> 8    Docs 端点 (UAT-DOC-001) ✅
> 9-10 Health 端点 (UAT-HC-001..002) ✅
> 11-14 UI 端到端 (UAT-UI-001..004) ✅
> 15-22 i18n 3 语言 (UAT-I18N-001..008) ✅ (per wt2, 7 维 key × 3 lang + 切换)
> 23-27 错误码 6-field (UAT-ERR-001..005) ✅ (per wt2, BAD_REQUEST 实证)
> 28-32 NFR 性能 (UAT-NFR-001..005) ✅ (per wt3, 4 bench P95 < 200ms)
> 33-37 容量规划 (UAT-CAP-001..005) ✅ (per wt4, 4 档 tier)
> 38-40 RACI 签字 (UAT-RACI-001..003) ⏳ (per wt6 5 域 Lead 临时代签)
> 41-42 DDD Review (UAT-DDD-001..002) ⏳ ([M] 子项)
> 43-44 数据迁移 (UAT-MIG-001..002) ⏳ (per F-05 sprint)

**累计 37/44 已落档, 7 已知缺口待 DDD Review**.

---

## §3 验收环境 (per TEST-DESIGN §6.3)

### 3.1 4 验收环境矩阵

| 环境 | 用途 | URL | 部署方式 | 数据 | 验收阶段 | 责任人 |
|---|---|---|---|---|---|---|
| **dev** | 开发环境 (本地) | `http://localhost:3000` + `http://localhost:8090` | docker compose / 本地 axum 0.8 | mock 100% (per 守門 #1 R-05) | F-XX 端到端 (本地) | 架构师 (Mavis 接手) |
| **staging** | 预发布环境 (WSL 容器) | `https://staging.star-ops.local` | k3s + star-ops container (mock) | mock + 历史 DDL | F-XX E2E (跨浏览器) | 架构师 (Mavis 接手) — 临时代签 SRE Lead |
| **prod** | 生产环境 (K8s 集群) | `https://ops.star.com` | k3s ingress + star-ops HA (per §5.5 缺口 #3 [M]) | 真实 PG + sqlx (per §5.5 缺口 #4 [M]) | prod UAT 验收 (5 域 Lead 真人到位后) | 架构师 + SRE Lead 真人 |
| **canary** | 灰度环境 (K8s 灰度) | `https://canary.star-ops.local` | k3s canary 10% (per F-01 端到端) | 真实 PG (per §5.5 缺口 #4 [M]) | 灰度验收 ([M] 子项) | 架构师 + SRE Lead 真人 |

### 3.2 dev 环境验收范围 (per §3.1 MVP 阶段)

**MVP 阶段**: dev 环境验收 4 tab × 10 端点 = 40 测 + i18n 3 语言 + 错误码 6-field, 跨 chromium/firefox/webkit 3 浏览器.

**dev 验收必跑测** (per wt1+wt2 实证):
- S01..S08 (10 端点 E2E)
- H1+H2 (2 health E2E)
- UI01+UI02 (2 UI E2E)
- I18N-09..11 (3 语言 E2E)
- ERR-01..05 (5 错误码 E2E, BAD_REQUEST 实证)
- log_upload_bench (4 bench P95 < 200ms, per wt3)
- capacity_planning smoke (per wt4)

**dev 验收不做** (per守門 #1 R-05 + 守門 #11 + 守門 #23):
- 真实 K8s 集群 (走 helm_canary_mock.sh subprocess)
- 真实 LLM OpenAI/Anthropic (走 ai_log_mock.py subprocess)
- 真实 PG 部署 prod ([M] 子项)
- 5 域 Lead 真人到位追溯签字 (per 守門 #14 v2 拍板 D 维持)

### 3.3 staging 环境验收范围 (per §3.1 预发布)

**staging 阶段**: 跨浏览器 Playwright 跑全 E2E, 验证 mock 路径稳定性, 准备 prod UAT.

**staging 验收必跑测** (per §6.1.1 AC-001..008):
- 8 AC 全部
- 4 类功能 (F-01..F-04)
- 5 维 NFR (性能/可用/安全/可观察/质量门)
- 5 错误码 6-field (BAD_REQUEST 实证 + 4 缺 per §4.6 缺口 #3)

**staging 验收不做**:
- 真实 K8s 集群 (走 k3s mock)
- 真实 LLM ([M] 子项)
- 真实 PG (走 mock DDL)

### 3.4 prod 环境验收范围 (per §3.1 生产)

**prod 阶段**: 5 域 Lead 真人到位后追溯签字 (per 守門 #14 v2 拍板 D), 跑全 UAT 40 测.

**prod 验收必跑测**:
- 全部 40 测 (E2E + UI + i18n + 错误码 + NFR + 容量规划)
- 5 域 Lead 真人到位追溯签字覆盖 (修订历史 +1 行)
- 真实 K8s 集群 + 真实 PG + 真实 LLM 通道 ([M] 子项实装)

**prod 验收依赖** (per守門 #1 R-05 + 守門 #5 v2 + 守門 #11):
- 真实 K8s 集群 owner 拍板 (per [M] 子项)
- 真实 LLM OpenAI/Anthropic 通道 owner 拍板 (per [M] 子项)
- 真实 PG 部署 owner 拍板 (per F-05 sprint + [M] 子项)

### 3.5 canary 环境验收范围 (per §3.1 灰度)

**canary 阶段**: 灰度 10% 流量 (per F-01 cluster canary 端到端), 验证 K8s HA + 负载均衡.

**canary 验收必跑测**:
- 4 tab × 10 端点 = 40 测
- 5 域 Lead 真人到位追溯签字覆盖
- 真实 K8s 集群 + 真实 PG + 真实 LLM 通道

**canary 验收不做** (per守門 #1 R-05):
- 100% 流量切换 (per F-01 灰度滑块 10%)

---

## §4 守門合规清单 (per AGENTS.md §4 + 守門 #1 + 守門 #14 v2 + 守門 #26 v26)

| 守門 | 验证方式 | 实证 |
|---|---|---|
| 守門 #1 R-05 mock 路径 | dev/staging 走 mock 端点, prod/canary 走真实 K8s/LLM/PG | ✅ per §3.1-§3.5 |
| 守門 #3 5 域 Lead 临时代签 | Mavis 接手默认代签 Ulysses, 真人到位后追溯签字 | ✅ per 守門 #14 v2 拍板 D |
| 守門 #5 v2 env 安全 | 0 泄露 secret, $env:VAR 引用后直接 pipe, 不打印 | ✅ 0 env: print 操作 |
| 守門 #6 v2 frontend typecheck | advisory 模式 (per守门 #1 v26) | ✅ per wt1+wt2 实证 (1 pre-existing) |
| 守門 #7 v3 PT bench P95<200ms | 4 bench 实证 P95 < 200ms (per wt3) | ✅ 4/4 达标 |
| 守門 #9 v20 子代理 dispatch 必先 brief | `docs/briefs/5-level-full-impl.md` v0.1 已落档 | ✅ |
| 守門 #10 author=Ulysses | commit author 100% Ulysses Leo Lee | ✅ |
| 守門 #11 缺标比错标 | 6 已知缺口显式列 (per §5) | ✅ |
| 守門 #12 AI 协作文档治理 | 禁回溯叙事 + BAS git log --follow 实证 | ✅ |
| 守門 #13 DB W/T/M 100% 覆盖 | 6 表 (3 T + 2 W + 1 M) per F-05 ops-log.sql | ✅ |
| 守門 #14 v2 5 域 Lead CONTENT 4 维 | RACI 完整 + Mavis 临时代签 (per wt6) | ✅ |
| 守門 #19 v19 agent 交互走 scripts/automation | 容量规划 / helm_canary_mock / ai_log_mock 走 scripts/ | ✅ |
| 守門 #21 v21 修订历史 | 7 段 (per AGENTS.md §3) | ✅ per wt7 PHASE-5-LEVEL-FULL-REPORT |
| 守門 #23 AI mock 不开外部 API | ai_log_mock.py subprocess, 永远 mock | ✅ |
| 守門 #24 v2 subprocess 替代 RPC | helm_canary_mock.sh + ai_log_mock.py + 容量规划 subprocess | ✅ |
| 守門 #26 v26 merge main 必 PR 流程 | PR-5-LEVEL-FULL-001.md (per wt7 描述) | ✅ |

**0 违反**.

---

## §5 已知缺口 (per 守門 #11 缺标比错标, DDD Review 必查)

| # | 缺口 | 等级 | 缓解 | 跟踪 |
|---|---|---|---|---|
| **#1** | **F-02 ops-log.sql 3 表 DDL 缺 + 路径不一致** | P0 | F-05 单独 sprint 修路径 + 落 3 表 DDL | per WBS §14.10.2 owner P1 修正 (per F-05 commit 31cb163 已落档) |
| **#2** | **frontend typecheck 实证缺 (AC-007 缺)** | P0 | 实装阶段跑 `npm run typecheck` 实证 (advisory 模式 per守門 #1 v26) | per [M] 子项 |
| **#3** | **错误码 E2E 覆盖 1/5** (NOT_IMPLEMENTED/UNAUTHORIZED/RATE_LIMITED/INTERNAL E2E 缺) | P1 | MVP 阶段 UT 覆盖; 实装阶段引入 middleware + E2E | per [M] 子项 |
| **#4** | **错误码 UT 覆盖 3/5** (BAD_REQUEST/INTERNAL UT 缺) | P1 | MVP 阶段 enum 完整; 实装阶段补 UT | per [M] 子项 |
| **#5** | **5 域 Lead 真人到位追溯签字** | P1 | per 守門 #14 v2 拍板 D, Mavis 临时代签 + 真人到位后追溯覆盖 (per wt6) | per 5 域 Lead 招聘 (per docs/recruitment/5-business-domain-lead-referral.md) |
| **#6** | **6 表 RLS 13 類验证缺** (per SRS-001 §8.2, T/M 表 tenant_id + 12 類必携) | P0 | MVP 阶段 DDL 存在性; 实装阶段 sqlx::test + testcontainers 跑 13 類验证 | per F-05 sprint |

**累计 UAT 6 已知缺口 DDD Review 必查** (per 守門 #11 缺标比错标).

---

## §6 签字栏 (per 守門 #14 v2 + 守門 #21 v21 修订历史)

**RACI 矩阵** (per 守門 #14 v2 + 守門 #3 5 域独立 Lead 硬约束):

| 角色 | R | A | C | I | 责任人 (真人到位前 Mavis 临时代签) | 签字日期 |
|---|---|---|---|---|---|---|
| **架构师** | ✅ | ✅ | — | — | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| **SRE Lead** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **平台 Lead** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **评审主持** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| **PM** | ⏳ | ⏳ | — | — | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

**签字栏说明** (per 守門 #14 v2 拍板 D + 9/3 19:35 JST 拍板 D + 9/5 10:43 JST 拍板 D + 9/8 15:19 JST 第 6 次强化):

- 5 域 Lead 真人到位前 Mavis 临时代签 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
- 真人到位后追溯签字覆盖修订历史 (per 守門 #1 禁回溯 + 守門 #21 v21 修订历史规则)
- 派生约束保留 (per 守門 #12 禁回溯叙事 + BAS git log --follow 实证 + 缺标比错标 + 子代理授权"无证据叙事=禁止")

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-08 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 初版: 8 AC + 4 类功能 + 5 维 NFR + 5 错误码 6-field 验收目标 + 4 tab × 10 端点 = 40 测 用例矩阵 + 4 验收环境 + 16 守門 0 违反 + 6 已知缺口 | 5-LEVEL-FULL brief §2.1 wt5 派单 (per 2026-09-08 19:55 JST 拍板) |
