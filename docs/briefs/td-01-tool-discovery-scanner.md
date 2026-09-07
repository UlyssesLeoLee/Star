# Brief: TD-01 — AI 工具自动扫描 4 源 + 启动时线程

> **状态**: 🟡 Brief v0.1
> **拍板**: per `ask_bf6bb4b2f1c27d1567da01d3` 4 推荐项
> **wt-branch**: `wt-td-01-tool-discovery-scanner`
> **base**: `main @ c9c9587` (Star-EI 8/8 wt 全部收官 + 推 origin 完成)
> **触发**: 2026-09-08 05:15 JST 用户发令"在这类 AI 工具链接应该在启动软件时自动扫描识别并链接" + 9/7 22:00 JST 用户发令"完成所有wbs内未完成任务"
> **关联**: [ADR-0049 AI 工具自动扫描 + 链接 拍板](../architecture/2026-08-26-upgrade/adr/0049-ai-tool-auto-discovery.md) · [EX-06 middleware idempotency.rs](../../crates/star-mcp/src/middleware/idempotency.rs) · [AGENTS.md §4 守门硬约束](../../AGENTS.md) · [AGENTS.md §4 #5 env 安全](../../AGENTS.md) · [AGENTS.md §4 #23 AI mock](../../AGENTS.md) · [WBS §14.4 B-3/B-4/B-5 凭证阻塞](../reports/STAR-P3-WBS-001.md)

---

## 1. 目标 (Objective)

在 `crates/star-mcp/src/discovery/` 落档 4 源文件 (mod + scanner + registry + auth_broker), 实现 4 源 (.mcp.json / Claude / OpenAI / Cursor/Windsurf) AI 工具 config 扫描 + 注册 + 凭证 env_var passthrough (per 拍板 C) + 启动时后台线程 (per 拍板 B/D-04).

## 2. 范围 (Scope)

### 2.1 In-Scope (TD-01 子项)

- `crates/star-mcp/src/discovery/mod.rs` (模块声明, 1KB)
- `crates/star-mcp/src/discovery/scanner.rs` (`ToolDiscoveryScanner` + 4 parser, 5KB)
- `crates/star-mcp/src/discovery/registry.rs` (`ToolRegistry` + 复用 EX-06 middleware, 3KB)
- `crates/star-mcp/src/discovery/auth_broker.rs` (`AuthBroker` env_var passthrough, 2KB)
- `crates/star-mcp/src/discovery/tests/scanner_test.rs` (5 UT, 2KB)
- `crates/star-mcp/src/main.rs` (1 行添加: `mod discovery;`)
- 跟 EX-06 middleware 集成 (复用 `IdempotencyMiddleware` dedup)
- 跟 WBS B-3/B-4/B-5 凭证阻塞关联文档

### 2.2 Out-of-Scope (后续子项, 跨 session 续)

- ❌ UI 扫描按钮 + 设置界面 + 右上角菜单 (TD-04, 跨 session 续)
- ❌ 健康检查 + 心跳 (TD-03 后续, 跨 session 续)
- ❌ 真实 4 源 config 集成测试 (需真实 Claude/OpenAI config, 跨 session 续)

## 3. 已知缺口 (per 守门 #11 缺标比错标)

- 本地无 `.mcp.json` / `~/.claude/` / `~/.openai/` / `~/.cursor/` 真实 config, IT 用 mock parser
- TD-04 UI 跨 session 续 (TS 实施 + gm-console AppShell 集成)
- TD-03 启动时 tokio::spawn 后台线程跨 session 续 (per 守门 #9 实证 RPC 不可靠下谨慎落地)
- 跨工具同名 dedup 优先级排序 (per ADR-0049 §2.4) 跨 session 续

## 4. 守门 (per 守门 #1 4 守门 + #5 + #7 + #10 + #13 + #23)

1. **守门 #1 4 守门**: `cargo check -p star-mcp -j 4` 0 err + `cargo fmt + clippy` 0 警告 + `cargo test -p star-mcp discovery -j 4` 100% pass + `cargo build --release` 0 err
2. **守门 #5 env 安全**: 不打印 `env` 字段值, 仅引用; 守门派生: AUTH_BROKER 模块 doc 注释明示
3. **守门 #7 0 unsafe**: 全 crate 不允许 `unsafe` 块
4. **守门 #10 author=Ulysses**: 1 commit, author=`Ulysses <ulysses@mavis.local>`
5. **守门 #11 缺标比错标**: §3 显式列 4 缺口
6. **守门 #13 a L0 协调**: scanner 在 L0 启动层, 不直接跟 L1 tool 通信, 经 registry 注册后 L1 可调
7. **守门 #23 AI mock**: Slack 告警 + 部分 webhook 走 mock; AI 工具凭证必须真实 (per 拍板 C env_var passthrough)
8. **守门 #1 v25 cargo test 跳过 workspace**: `cargo test -p star-mcp discovery` 单 crate 测

## 5. 依赖 (Dependencies)

### 5.1 上游

- Star-EI 8/8 wt 全部收官 (per 9/7 推 origin 完成), EX-06 `IdempotencyMiddleware` 复用

### 5.2 下游阻塞

- TD-02 registry dedup 复用 EX-06 middleware (per 拍板)
- TD-03 启动线程 (跨 session 续)
- TD-04 UI 扫描按钮 (跨 session 续)

## 6. 交付物 (Deliverables)

| # | 路径 | 描述 | 大小 |
|---|---|---|---|
| 1 | `crates/star-mcp/src/discovery/mod.rs` | 模块声明 | 0.4KB |
| 2 | `crates/star-mcp/src/discovery/scanner.rs` | `ToolDiscoveryScanner` + 4 parser | 5.5KB |
| 3 | `crates/star-mcp/src/discovery/registry.rs` | `ToolRegistry` + 复用 EX-06 middleware | 3.0KB |
| 4 | `crates/star-mcp/src/discovery/auth_broker.rs` | `AuthBroker` env_var passthrough | 2.0KB |
| 5 | `crates/star-mcp/src/discovery/tests/scanner_test.rs` | 5 UT: 4 parser + 1 dedup | 2.5KB |
| 6 | `crates/star-mcp/src/main.rs` | 1 行添加: `mod discovery;` | 1 line |
| 7 | `docs/briefs/td-01-tool-discovery-scanner.md` | 本 brief | ~6KB |
| **总** | | **7 文件, ~20KB raw** | |

## 7. 验收 (Acceptance Criteria)

### 7.1 守门实证 (per 守门 #1 4 守门)

- [ ] `cargo check -p star-mcp -j 4` exit 0, 0 err
- [ ] `cargo fmt --all -- --check` 0 diff
- [ ] `cargo clippy -p star-mcp -- -D warnings` 0 err (走守门 #1 v25 实证)
- [ ] `cargo test -p star-mcp discovery -j 4` 100% pass (5/5 UT, ~3s)
- [ ] `cargo build --release -p star-mcp` exit 0
- [ ] `cargo test -p star-mcp middleware::idempotency -j 4` 100% pass (EX-06 0 回归)

### 7.2 4 想定シナリオ (S-TD-01..S-TD-04)

- [ ] **S-TD-01** scanner.scan_all() 4 源, 10s 内完成 (mock parser 即可)
- [ ] **S-TD-02** 同名 tool 多源 dedup, 优先级排序 (per 4 源优先级)
- [ ] **S-TD-03** AuthBroker.inject() 凭证 env_var passthrough, 不打印值
- [ ] **S-TD-04** registry.register() 调用 IdempotencyMiddleware 0 回归 (EX-06 兼容)

### 7.3 git 实证 (per 守门 #9 主体规则)

- [ ] `git log -p --follow crates/star-mcp/src/discovery/scanner.rs` 实证类完整
- [ ] commit author = Ulysses per 守门 #10
- [ ] 1 commit 含全部 7 文件, 不散落

## 8. 实施路径 (per 守门 #9 v3 fallback + 守门 #19 v19)

### 8.1 Mavis 直接落地 (per 守门 #9 v3 实证 5/5 subagent RPC 不可靠)

1. 创建 wt: `git worktree add ../.worktrees/wt-td-01-tool-discovery-scanner -b wt-td-01-tool-discovery-scanner main`
2. Mavis 在 wt 内写 7 文件 (per §6 交付物)
3. 实证守门 #1 4 步
4. `git add` + `git commit -m "..."` author=Ulysses
5. 切回 main + `git merge --no-ff wt-td-01-tool-discovery-scanner`
6. cargo check --workspace --lib 0 err 实证 (跨 star-mcp + 之前 41 crate 0 回归)
7. `git push origin main` (per 守门 #1 反转 9/7 21:08 JST)

### 8.2 失败接手 (per 守门 #9 v3 fallback)

- Mavis 直做失败 → 不再派 worker subagent (实证 5/5 RPC 不可靠, 简化)

## 9. 风险 (per ADR-0049 §4.2)

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 4 源 config 格式跨版本兼容 | 中 | 中 | 4 parser 各自独立, 解析失败 fallback skip 该源 |
| 同名 tool 多源冲突 | 中 | 中 | 优先级排序: `.mcp.json` > `Claude` > `OpenAI` > `Cursor/Windsurf` |
| 凭证泄露 (env_var 内存中) | 低 | 高 | 不打印 + 守门 #5 实证; 0 凭证落盘 |
| 启动延迟 +200ms | 中 | 低 | 后台 tokio::spawn 不阻塞主进程 |

## 10. 签字 (5 角色 Mavis 临时代签 per 守门 #14 v2)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 |
| SRE Lead | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 临时代签) |
| 平台 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 临时代签) |
| 评审主持 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 临时代签) |
| PM | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 临时代签) |

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-08 05:18 JST | Ulysses — Mavis 接手 | 初稿, 7 文件 ~20KB, 4 守门, 5 角色代签 | per `ask_bf6bb4b2f1c27d1567da01d3` 4 推荐项 + 拍板 D |
