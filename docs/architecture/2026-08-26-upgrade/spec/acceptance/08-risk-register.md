# 45. Risk Register

> **状态**：🟡 草案 v0.2
> **日期**：2026-08-26

## 升级相关风险

| 风险 ID | 描述 | 概率 | 影响 | 缓解 |
|---|---|---|---|---|
| **R-001** | 2026-08-26 后 30 天 MCP 又升级 | 中 | 中 | Phase D 锁 2026-07-28 规范 + 12 个月迁移窗口 |
| **R-002** | Rust MCP SDK beta 风险 | 中 | 中 | 用 stdio transport（已稳） |
| **R-003** | OpenAPI 3.1 spec 工具链不完整 | 中 | 中 | Redocly CLI `1.25.x` (>= 1.25.0) / Stoplight Elements `0.6.0` / Swagger UI `5.18.0` 已支持 OpenAPI 3.1（per P2-6 修复 2026-08-27 补版本号实证）— 工具链由 [spec/rest/01 §5 验证](../rest/01-rest-strategy.md) 跑 `npx @redocly/cli lint` + `npx swagger-cli validate` 守门 |
| **R-004** | Unknown Agent Test 失败 | 中 | 高 | 4 步降级到 Git Only 也必须跑通（Level 1 + Level 4 两段独立验证，per [spec/acceptance/01 §3 + §4 v0.2](01-unknown-agent-test.md)） |
| **R-005** | Agent API schema 频繁 breaking | 中 | 中 | 严格 `agent-api/v1` 版本化；新 field 走 minor |
| **R-006** | Vendor 突然停止服务（per Gemini CLI 案例） | 中 | 高 | Zero Vendor Cooperation + Fallback Ladder 4 级 |
| **R-007** | GitHub/GitLab API 限速 | 中 | 中 | Provider 抽象 + cache 层落点 `crates/star-vcs/src/cache.rs`（per P2-7 修复 2026-08-27 实指落点；文件已建空 + TODO 占位）— cache 实现由 Phase D 接手填实 |
| **R-008** | 凭证泄露到 AGENTS.md | 中 | 高 | Vault 抽象（per [arch/06 §1.2 T-08 凭证 Vault 抽象](../../arch/06-threat-model-nfr.md) — **删** 原"GitGit V0 T6 task"跨项目引用 per P2-8 修复 2026-08-27；本仓 Vault 抽象落地 spec 待 Phase D 新增） |
| **R-009** | 多 Agent 文件冲突 | 中 | 中 | Worktree 物理隔离 + Phase 2 AST conflict detection |
| **R-010** | Audit trail 篡改 | 低 | 高 | HMAC chain + append-only |
| **R-011** | OLU 超 NFR-OP-010（2 SRE·周/周） | 高 | 高 | 拆 2-3 子代理并行 + 4-6 周窗口 |
| **R-012** | 缺标比错标安全被违反 | 中 | 中 | DDD Review + 必查"已知缺口"清单 |
| **R-013** | 代签新规则被滥用 | 低 | 中 | DDD Review 必查 + Ulysses 终审 |
| **R-014** | 子代理编造"per X 历史形态"叙事 | 中 | 高 | 升级前必跑 `git log -p --follow` 实证 |
| **R-015** | Phase D 闭环测试环境无 Unknown Agent | 中 | 中 | 自实现 minimal agent (per [spec/acceptance/01 §7 v0.2](01-unknown-agent-test.md)) |

## 已知缺口（DDD Review 必查）

> per 2026-08-27 校准：

| 缺口 ID | 描述 | 阻塞? | 修复路径 |
|---|---|---|---|
| GAP-R-007 | cache 层落点 `crates/star-vcs/src/cache.rs` 仅有空壳 + TODO；Phase D 须填实 cache trait + Provider integration | 否 | Phase D 任务 |
| GAP-R-008 | Vault 抽象无本仓 spec（删"GitGit V0 T6"跨项目引用后留空）；Phase D 须新增 spec | 否 | Phase D 任务 |

## 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：R-001 ~ R-015 15 项风险 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P2-6：R-003 补版本号实证（Redocly CLI `1.25.x` / Stoplight Elements `0.6.0` / Swagger UI `5.18.0`）· P2-7：R-007 cache 层落点实指 `crates/star-vcs/src/cache.rs`（空文件 + TODO 占位已建）· P2-8：R-008 删"GitGit V0 T6 task"跨项目引用，引 arch/06 T-08 凭证 Vault 抽象 · 新增 §"已知缺口" GAP-R-007 / GAP-R-008 | 8 子代理 INTERFACE-REVIEW-C P2-6 / P2-7 / P2-8 + P1-BLOCKERS-SUMMARY v0.2 |

> v0.2 fix: 2026-08-27 per C-8 (P2-6/7/8)
