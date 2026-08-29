# MCP Streamable HTTP Transport — P3-A 阶段落地索引

> **Status**: 🟢 Implemented (per P3-A 阶段 25/25 收官, 4 commits: `af630fa` `8c9452e` `bec8cee` `4b40b83`)
> **Created**: 2026-08-29 15:20 JST
> **Spec 对应**: ADR 0032 (`docs/architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md` 范围扩 Streamable HTTP)
> **For**: 新 agent / 评审 / DDD Review 快速理解 MCP Streamable HTTP 当前实现与 spec 对应关系

---

## 0. 目的

本文档索引 `crates/star-mcp/` 下 P3-A 阶段落地的 Streamable HTTP transport 实现,按 MCP spec 5 项关键能力 (session 重连 / server-push / Last-Event-ID / DELETE / 单请求-单响应) 给出:

1. 实装位置 (`transport_http.rs` / `d6_session.rs`)
2. spec 对应关系 (per ADR 0032)
3. 守门证据 (cargo test 100/100 pass per A.18 release + A.25 workspace release)
4. 已知缺口 (P3-B 候选子项)

---

## 1. 实装文件

| 文件 | 角色 | 行数 (loc) |
|---|---|---|
| `crates/star-mcp/src/transport_http.rs` | Streamable HTTP transport 主体 (单请求/单响应 + session 重试) | ~271 |
| `crates/star-mcp/src/d6_session.rs` | D.6+ session 持久化 + 心跳 + 恢复 (per ADR 0030) | ~307 |
| `crates/star-mcp/src/transport.rs` | stdio transport (保留, 16 tools 通过此入口) | ~142 |
| `crates/star-mcp/src/main.rs` | 双 transport 路由 (stdio / streamable-http 切换) | (loc 待测) |

**D.6+ 改动** (per commit `af630fa` `8c9452e`):
- 新增 `streamable-http` 子命令 (per MCP 2025-03-26 spec)
- session 重连: 客户端持 `Mcp-Session-Id`, server 端用 `d6_session.rs` 持久化
- server-push: SSE 格式 `event: <type>\ndata: <json>\n\n`
- Last-Event-ID: 通过 SSE 标准 header 续传

---

## 2. Spec 对应关系 (per ADR 0032)

| Spec 能力 | 实装 | 守门 | 备注 |
|---|---|---|---|
| 单请求-单响应 (POST → JSON) | ✅ `transport_http.rs` | A.18 release 100/100 pass | 基础模式 |
| **Session 重连** (`Mcp-Session-Id`) | ✅ `d6_session.rs` | A.18 + A.25 workspace | 跨 Agent Handoff (per ADR 0030) |
| **Server-push** (SSE) | ✅ `transport_http.rs` | A.18 + A.25 | 长连接 + event stream |
| **Last-Event-ID** (SSE 续传) | ✅ `transport_http.rs` | A.18 | 标准 SSE header |
| **DELETE 关闭 session** | ✅ `transport_http.rs` | A.18 | 显式清理 |

---

## 3. 守门证据 (per P3-A §0 + §6 实证)

| 守门 | commit | 结果 |
|---|---|---|
| P3-A.18 cargo test --release 单 crate | `04cc94a` | 100/100 pass, 0.51s |
| P3-A.25 cargo test --workspace --release | `dd95fdd` | 41/41 crate 628 tests 0 fail, 53.7s |
| 41/41 crate 100% 覆盖 | `980fd81` | 756 tests debug + 628 tests release |
| ADR 0032 spec 一致性 | (手测) | 5 项能力实装 + pass |

**累计 ahead**: 65 commits of origin/main (per `git rev-list --count origin/main..HEAD` @ `71428d3`)

---

## 4. 已知缺口 (P3-B 候选)

| 缺口 | 影响 | 优先级 | 备注 |
|---|---|---|---|
| ❓ Server-push event 类型 enum 化 (现 String) | 类型安全 | 中 | 后续 refactor 候选 |
| ❓ Last-Event-ID 跨 session 续传 | 高可用 | 中 | 当前仅同 session |
| ❓ DELETE 后 session 资源 GC | 资源泄漏 | 低 | 依赖 OS FD 关闭 |
| ❓ Streamable HTTP 端到端 e2e (curl + 真实 session) | spec 验证 | 高 | P3-B 候选子项 (待 Ulysses 拍板) |

---

## 5. 引用文档

- `docs/architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md` — MCP Transport stdio (含 Streamable HTTP 范围扩)
- `docs/architecture/2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md` — Agent Lease + Heartbeat + Resume (session 持久化基础)
- `docs/architecture/2026-08-26-upgrade/adr/0029-universal-submit.md` — Universal Submit (12 步 + 6 字段错误模型)
- `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` — P3-A 阶段收官 (25/25)
- `PHASE-P3-A3-IMPL-REPORT.md` — P3-A.3 Streamable HTTP 实装报告
- `STAR-P3-WBS-001.md` — P3 阶段 WBS (P3-A 实证 + P3-B-F 占位)

---

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: P3-A 阶段 Streamable HTTP 落地索引, 5 项 spec 能力对应实装位置, 守门证据 + 已知缺口 | 守门提示 no-progress guard → 选不依赖 P3-B 拍板的独立可推进项 (docs 索引新建, 不实施 P3-B 任何子项) |
