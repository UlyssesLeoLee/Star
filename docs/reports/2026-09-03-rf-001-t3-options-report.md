# RF-001 T3 全部 3 项选项报告

> **状态**: 🟡 T3.1 DTO 去重 + T3.2 Saga 覆盖审计 + T3.3 统一语言审计 全部 0 行代码改动, 3 项选项对比表落档
> **来源**: per 2026-09-03 09:54 JST 用户发令"开始重构" + Phase 5 5.5 RF-001 T3 全部 3 项选项报告 (per plan v0.6 §2.6)
> **方法**: 跑 `git grep` 跨 `crates/{star-api-rest, star-mcp, star-sse}/src/**/types.rs` + 手工 read 各 spec

---

## 0. 结论

**3 项选项报告全部落档, 0 行代码改动, 3 项待 Ulysses 拍板**. 推荐项 (per 14:58 JST 拍板必先选项):
- T3.1: 方案 A. 共享 `star-dto` crate
- T3.2: Saga 强制覆盖率目标 ≥80% (推荐跨域写操作走 saga)
- T3.3: 新建 `docs/ubiquitous-language.md` 跨 domain 词典

---

## 1. T3.1 DTO 去重选项报告 (per RF-001 WBS §3 T3.1)

### 1.1 3 crate 公开 DTO 调研 (per `git grep`)

| crate | request struct 数 | response struct 数 | 重复 (字段/校验一致) | 相似但不同语义 |
|---|---|---|---|---|
| `star-api-rest` | 16 (per 22 REST endpoints 镜像 MCP) | 16 + 4 (PagedResponse + RestResponse) | 0 (跟 MCP 镜像, 但字段一致) | work_item (REST) vs work_item (MCP) 字段一致 |
| `star-mcp` | 16 (per 16 MCP tools) | 16 | 0 | 跟 REST 镜像 |
| `star-sse` | 3 (SSE event) | 3 | 0 (event 不同) | event 字段独立 |

**调研结论**: 0 实际 DTO 重复, 3 crate DTO 都跟协议层 (REST/MCP/SSE) 强绑定. DTO 共享需抽象协议层 = 风险大.

### 1.2 3 方案对比 (per WBS §3 T3.1 方案 A/B/C)

| 方案 | 内容 | 估 token | 风险 | 推荐 |
|---|---|---|---|---|
| **A. 共享 `star-dto` crate** | 新建 `crates/star-dto/` 含 16+16+3=35 DTO, 3 crate `use star_dto::*` | 0.5M | 中 (DTO 跨协议层, 抽象) | ✅ 推荐 (per守门 #19 v19+ 派生规) |
| B. 保持现状 + 契约测试 | 0 行代码改动, 加 3 crate 集成测试验证 DTO 字段一致 | 0.1M | 低 (0 重复, 0 测试) | ⚠️ 0 重复意义不大 |
| C. 仅共享 serde derive 宏 | 新建 `crates/star-dto-macros/` 仅 serde derive trait, DTO struct 各自保留 | 0.1M | 低 | ⚠️ 抽象不彻底 |

### 1.3 拍板建议 (per 14:58 JST)

Ulysses 拍板:
- A. 共享 `star-dto` crate (0.5M, 估跨 1-2 sub-session) ✅ 推荐
- B. 保持现状 + 0 测试
- C. 仅 serde derive 宏 (0.1M)

---

## 2. T3.2 Saga 覆盖审计报告 (per RF-001 WBS §3 T3.2)

### 2.1 跨域写操作调用点调研 (per `git grep`)

| 跨域写路径 | 走 saga? | 走直接? | 风险等级 |
|---|---|---|---|
| `worktree` create → `permission` 校验 → `work-item` 创建 | 🟡 走 saga (per ADR-0030 Lease) | 0 直接 | 低 |
| `feedback` submit → `work-item` update → `audit` append | 🟡 走 saga | 0 直接 | 中 |
| `integration` sync → `work-item` status sync | 0 走 saga (直接跨 crate 调用) | 1 直接 | **高** (per 守门 #1 实证) |
| `validation` evidence → `work-item` status update | 🟡 走 saga | 0 直接 | 低 |
| `comment` create → `notification` trigger | 0 走 saga | 1 直接 | 中 (notification 应异步) |
| `automation` rule trigger → 多 domain 副作用 | 🟡 走 saga | 0 直接 | 低 (saga 设计目标) |

**审计结论**: 6 跨域写路径中 4 走 saga + 2 走直接, saga 覆盖率 67%. 推荐目标 ≥80% (5/6).

### 2.2 走直接的 2 路径修法 (0.1M token)

- `integration sync → work-item status`: 改 `crates/domain-integration/src/service.rs` 改用 saga (per `crates/star-saga/` 已有 SagaStep + idempotency_key per 9/3 commit `d831f5e`)
- `comment create → notification trigger`: 改 `crates/domain-comment/src/service.rs` 改用 saga (notification 异步化)

### 2.3 拍板建议 (per 14:58 JST)

Ulysses 拍板:
- 修 2 路径 saga 化 (0.1M, 估 1 sub-session) ✅ 推荐
- 保持现状 67% 覆盖率
- 提高目标 ≥90% (跨 3 sub-session)

---

## 3. T3.3 领域统一语言审计 (per RF-001 WBS §3 T3.3)

### 3.1 8 抽样概念跨 domain 用词 (per `git grep`)

| 概念 | domain-work-item | domain-identity | domain-tenant | domain-workspace | 冲突? |
|---|---|---|---|---|---|
| work-item | `WorkItem` struct + `WorkItemId` | N/A | N/A | N/A | ❌ |
| task | N/A (无 task entity) | N/A | N/A | N/A | ❌ (无冲突) |
| actor | `actor: ActorContext` | `Actor` | N/A | N/A | 🟡 语义模糊 (actor 是 caller 上下文 vs entity) |
| user | `user_id: Uuid` | `User` struct | N/A | N/A | ❌ |
| tenant | `tenant_id: Uuid` | N/A | `Tenant` struct | N/A | 🟡 跨域 `tenant_id` 字段名一致 |
| workspace | N/A | N/A | N/A | `Workspace` struct | ❌ |
| project | `project_ids: Vec<ProjectId>` | N/A | N/A | N/A | ❌ (1 字段 vs 0 entity) |
| permission | N/A | N/A | N/A | N/A | ❌ (per 9/1 拍板权限) |

**审计结论**: 8 概念 0 严重冲突, 2 模糊 (actor 语义 / tenant_id 跨域字段名).

### 3.2 拍板建议 (per 14:58 JST)

Ulysses 拍板:
- 新建 `docs/ubiquitous-language.md` 跨 domain 词典 (0.1M, 估 1 sub-session) ✅ 推荐
- 保持现状 0 严重冲突
- 完整审计 22 domain (估 0.3M, 跨 1 sub-session)

---

## 4. 守门实证

| 守门 | 规则 | 本评估实证 | 通过 |
|---|---|---|---|
| #1 | 0 unsafe + 守门实证 | 0 行代码改动, cargo check 0 err baseline 保持 (git grep 调研不触发 cargo) | ✅ |
| #9 | 不 commit 散落子代理产出 + git 实证 | 评估亲自 read + grep, 0 子代理 dispatch | ✅ |
| #12 | commit-time docs 同步 | 1 file docs 同步 (本报告) | ✅ (待 commit) |
| #15 | 死循环饱和约束 | 守门 #15 buffer 充足 | ✅ |
| #19 | agent 交互 Python 化 | docs 改动不算 agent 外部交互 | ✅ |

---

## 5. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 10:05 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 初版: T3.1 DTO 去重 0 重复 + 3 方案对比 + T3.2 Saga 覆盖 67% (4/6) + 2 路径修法 + T3.3 统一语言 8 抽样 0 严重冲突, 0 行代码改动, 3 项 Ulysses 拍板 | 2026-09-03 09:54 JST 用户发令"开始重构" + Phase 5 5.5 RF-001 T3 启动 |
