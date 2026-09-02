# ADR-0044: phase2-3-mock-full — Phase 2 backend + Phase 3 memgraph 全 MSW 落地

> **ステータス**: Draft v0.1
> **日付**: 2026-09-02
> **改訂人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-02 自审
> **触发**: per 2026-09-02 09:35 JST Ulysses 4 拍板 (scope opt4 / barrier opt3 / session opt1 / datainput opt3)
> **依据**: [ADR-0043 v0.1 audit-onboarding-failed](../.git) + [commit 62bc032 audit 拍板](../.git) + [commit 742d377 arch-graph MSW 13 节点 fixture](../.git)

> **dual-use 提醒 (per AGENTS.md §5)**: 本 ADR 涉及 全 frontend mock 层 落地, backend / memgraph 全 MSW 模拟, **不接 真 backend / memgraph 实例**. 25 domain / RGS 5 域 マッピング非該当 (arch-graph 数据 是 22 domain-* 投影, mock fixture).

---

## §0 目的

Phase 2 (backend 真接) + Phase 3 (memgraph 真接) 全部 **MSW 端点 + 试响应** 落地. **不接真 backend / 真 memgraph**. 0 凭证 依赖, 0 P3-B 拍板 依赖. 5 增量 commit 走完.

---

## §1 5 增量 commit 計画 (per 09:35 JST 拍板)

| # | commit | 内容 | 文件 | token 估 |
|---|---|---|---|---|
| 1 | `feat(api-mock)` | api handler stub — 5 endpoint 試応答 (POST /api/api-keys, DELETE /api/api-keys/:id, POST /api/graph/ensure-fresh, POST /api/graph/cypher, GET /api/graph/health) + P3-B B.5/B.6 备选路径 | 1 改 + 1 新 | 0.20M |
| 2 | `feat(llm-fetch-mock)` | 真 fetch MSW 试响应 — 4 必备 LLM provider (openai/claude/gemini/minimax) 试端点 + 3 状态 (200 / 401 / 10s timeout) + retry.ts 5 重试 走真 fetch 路径 | 1 改 + 1 新 | 0.20M |
| 3 | `feat(kms-mock)` | KMS mock — POST /api/kms/unlock /api/kms/lock 试 + frontend useKms hook stub | 1 改 + 2 新 | 0.15M |
| 4 | `feat(memgraph-1hop-mock)` | memgraph 1-hop MSW 扩 — arch-graph fixture 13 节点 → 17 节点 (per Phase 3 1-hop 仕様, per ADR-0041 §2.1) | 1 改 | 0.15M |
| 5 | `feat(memgraph-2hop-codeside)` | 2-hop code-side MSW — 4 节点 (2 cratemodule + 2 symbol) + 2 边 (REFERENCES / LIVES_IN) + hop_level=2 样式区别 | 1 改 | 0.10M |
| 6 | `docs(agents)` | AGENTS.md v0.34 修订历史 (5 commit 实证 + 守门 #12 cascade) + push origin 6 commits | 1 改 | 0.10M |
| **計** | | | | **0.90M** |

---

## §2 Phase 2 backend (5 endpoint 試応答, commit 1)

| Method | Path | 用途 | 試応答 |
|---|---|---|---|
| POST | `/api/api-keys` | Add API key (per commit cb2475e AgentSettingsModal) | 201 + `{ id, provider, label, mode, preview, createdAt }` |
| DELETE | `/api/api-keys/:id` | Delete key | 200 + `{ deleted: true }` |
| POST | `/api/graph/ensure-fresh` | Arch-graph 1-hop ensure-fresh (per commit 742d377) | 200 fresh / 202 running |
| POST | `/api/graph/cypher` | Arch-graph 1-hop query | 200 GraphPayload |
| GET | `/api/graph/health` | Health check | 200 / 503 |

**P3-B B.5/B.6 备选路径** (per 29692a7 wiremock 备选):
- B.5 OpenClaw: 已有 `cli.ts` handler, 不动
- B.6 Hermes: 已有 `cli.ts` handler, 不动
- **本 commit 关键**: 5 endpoint 试応答 增分 (P3-B 7/9 收官 维持)

---

## §3 真 fetch MSW (4 必备 LLM, commit 2)

| Provider | Test URL | Headers (Build) | 试响应 |
|---|---|---|---|
| openai | `https://api.openai.com/v1/models` | `Authorization: Bearer {key}` | 200 + `{ data: [...] }` / 401 `{ error: "invalid_api_key" }` / timeout 10s |
| claude | `https://api.anthropic.com/v1/messages` | `x-api-key: {key}, anthropic-version: 2023-06-01` | 200 / 401 / timeout |
| gemini | `https://generativelanguage.googleapis.com/v1beta/models` | `x-goog-api-key: {key}` | 200 / 401 / timeout |
| minimax | `https://api.minimax.chat/v1/models` | `Authorization: Bearer {key}` | 200 / 401 / timeout |

**retry.ts 真接** (per commit f14ef0f 5 重试):
- attempt 0: 试端点 试响应
- 失败 → 3s wait → attempt 1
- 失败 → 6s wait → attempt 2
- 失败 → 12s wait → attempt 3
- 失败 → 24s wait → attempt 4
- 失败 → 48s wait → attempt 5 → 失败 (audit log 写入)

---

## §4 KMS mock (commit 3)

| Method | Path | 用途 | 試応答 |
|---|---|---|---|
| POST | `/api/kms/unlock` | Unlock API key from encrypted_rust | 200 + `{ session_token, expires_in: 3600 }` |
| POST | `/api/kms/lock` | Lock (clear session token) | 200 + `{ locked: true }` |

**frontend useKms hook stub**:
- 状态: `locked` | `unlocked` + `session_token`
- mock 永远 `unlocked` (Phase 2 简化)

---

## §5 memgraph 1-hop MSW (commit 4)

per ADR-0041 §2.1 节点 kind union (25 节点) + commit 742d377 arch-graph 现有 13 节点 fixture. 扩到 **17 节点** (1-hop + 2-hop code-side):

| 类型 | 现 (commit 742d377) | 本 commit (扩) | 总 |
|---|---|---|---|
| work_item | 1 | 0 | 1 |
| project | 1 | 0 | 1 |
| identity | 2 | 0 | 2 |
| worktree | 1 | 0 | 1 |
| agent_session | 1 | 0 | 1 |
| change_set | 1 | 0 | 1 |
| scm_repository | 1 | 0 | 1 |
| pull_request | 1 | 0 | 1 |
| validation_case | 2 | 0 | 2 |
| feedback | 1 | 0 | 1 |
| comment | 1 | 0 | 1 |
| cratemodule | 0 | **+2** (domain-physics-core, domain-physics-rigid-body) | 2 |
| symbol | 0 | **+2** (RigidBody::apply_radial_impulse, PhysicsCore::step) | 2 |
| **計** | **13** | **+4** | **17** |

---

## §6 memgraph 2-hop code-side (commit 5)

- 2 cratemodule + 2 symbol + 4 边 (REFERENCES / LIVES_IN x 2 / DEPENDS_ON)
- hop_level=2 样式: opacity 0.2 (per ADR-0041 §2.3.3)
- ArchGraphModal 现有 style 已支持, 不改组件

---

## §7 守门对齐 (per AGENTS.md §4)

- **#1 禁回溯叙事**: 全部 commit message 引用 09:35 JST 4 拍板 + 既存 commit 短码 (62bc032 / 742d377 / f14ef0f), 无回溯
- **#5 環境変数安全**: 全 mock, 无 .env 依赖, Mavis 接手 不読任何 secret
- **#9 子代理実証**: 0 子代理调用, root 直 5 commit
- **#10 代签規則**: author = Ulysses, Mavis 接手代签
- **#11 缺标比错标**: 5 缺口显式列 (per commit 1-5), 不藏
- **#12 文档治理**: 5 commit 跨引用闭环, AGENTS.md v0.34 + 本 ADR + spec 全部一致
- **#13 DB 三類**: Phase 3 memgraph 是 graph DB (類外), audit_audit_event 沿用 T 類 (Phase 2 既存)

---

## §8 已知缺口 (per 缺标比错标, 守門 #11)

| # | 缺口 | 原因 |
|---|---|---|
| 1 | 真 backend 不接 | 全 mock 拍板, 等 P3-B 拍板 |
| 2 | 真 memgraph 不接 | 全 mock 拍板, 等部署拍板 |
| 3 | 真 KMS 不接 | 全 mock 拍板, 等 P3-B 拍板 |
| 4 | 25 节点 全部 (現 17, 缺 8) | Phase 1 mock 限定, Phase 3 扩 |
| 5 | 4 必备 LLM 真凭证 | 全 mock 拍板, 等 P3-B 拍板 |

---

## §9 签字栏

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-02 | Mavis 接手代签 (per 19:39/20:56/21:59 JST) |
| SRE Lead | ⏳ 待签 | - | DDD Review 阶段补 |
| 平台 | ⏳ 待签 | - | 同上 |
| 评审主持 | ⏳ 待签 | - | 同上 |
| PM | ⏳ 待签 | - | 同上 |

---

## §10 修订历史

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 初版: 5 增量 commit 計画 (api handler / 真 fetch MSW / KMS / memgraph 1-hop / 2-hop code-side) + 6 文档 commit = 估 0.9M token | 2026-09-02 09:35 JST Ulysses 4 拍板 (scope opt4 + barrier opt3 + session opt1 + datainput opt3) |
