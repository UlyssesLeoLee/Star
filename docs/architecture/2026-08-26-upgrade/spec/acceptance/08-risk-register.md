# 45. Risk Register

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26

## 升级相关风险

| 风险 ID | 描述 | 概率 | 影响 | 缓解 |
|---|---|---|---|---|
| **R-001** | 2026-08-26 后 30 天 MCP 又升级 | 中 | 中 | Phase D 锁 2026-07-28 规范 + 12 个月迁移窗口 |
| **R-002** | Rust MCP SDK beta 风险 | 中 | 中 | 用 stdio transport（已稳） |
| **R-003** | OpenAPI 3.1 spec 工具链不完整 | 中 | 中 | Redocly CLI / Stoplight / Swagger UI 5.x 已支持 |
| **R-004** | Unknown Agent Test 失败 | 中 | 高 | 4 步降级到 Git Only 也必须跑通 |
| **R-005** | Agent API schema 频繁 breaking | 中 | 中 | 严格 `agent-api/v1` 版本化；新 field 走 minor |
| **R-006** | Vendor 突然停止服务（per Gemini CLI 案例） | 中 | 高 | Zero Vendor Cooperation + Fallback Ladder 4 级 |
| **R-007** | GitHub/GitLab API 限速 | 中 | 中 | Provider 抽象 + cache |
| **R-008** | 凭证泄露到 AGENTS.md | 中 | 高 | Vault 抽象（GitGit V0 T6 task） |
| **R-009** | 多 Agent 文件冲突 | 中 | 中 | Worktree 物理隔离 + Phase 2 AST conflict detection |
| **R-010** | Audit trail 篡改 | 低 | 高 | HMAC chain + append-only |
| **R-011** | OLU 超 NFR-OP-010（2 SRE·周/周） | 高 | 高 | 拆 2-3 子代理并行 + 4-6 周窗口 |
| **R-012** | 缺标比错标安全被违反 | 中 | 中 | DDD Review + 必查"已知缺口"清单 |
| **R-013** | 代签新规则被滥用 | 低 | 中 | DDD Review 必查 + Ulysses 终审 |
| **R-014** | 子代理编造"per X 历史形态"叙事 | 中 | 高 | 升级前必跑 `git log -p --follow` 实证 |
| **R-015** | Phase D 闭环测试环境无 Unknown Agent | 中 | 中 | 自实现 minimal agent (per [spec/acceptance/01 §6](01-unknown-agent-test.md)) |

## 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
