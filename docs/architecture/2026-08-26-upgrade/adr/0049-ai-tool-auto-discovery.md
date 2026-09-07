# ADR-0049: Star AI 工具自动扫描 + 链接 (Tool Auto-Discovery)

> **状态**: 🟢 Accepted v1.0 (per 2026-09-08 05:18 JST `ask_bf6bb4b2f1c27d1567da01d3` 4 推荐项拍板)
> **生效**: 2026-09-08
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签字**: 🟢 架构师 (Mavis 接手 agent per DEC-008) (per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签" + 守门 #10 author=Ulysses)
> **母文档**: [Star 排他与幂等架构 view 03-detailed-design.md §1.1 M-15](../2026-09-07-exclusion-idempotency/03-detailed-design.md) (跟 EX-06 middleware 集成)
> **关联**: [ADR-0048 排他与幂等架构 view 拍板](0048-exclusion-idempotency-design.md) · [ADR-0032 MCP Transport stdio](0032-mcp-transport-stdio.md) · [AGENTS.md §4 守门硬约束](../../../AGENTS.md) · [AGENTS.md §4 #5 env 安全](../../../AGENTS.md) · [AGENTS.md §4 #23 AI mock](../../../AGENTS.md)
> **下游**: [docs/briefs/td-01-tool-discovery-scanner.md](../../briefs/td-01-tool-discovery-scanner.md) (brief 落档) · `crates/star-mcp/src/discovery/` 4 源文件实施 · UI 扫描按钮 spec

---

## 1. 背景与问题

### 1.1 业务背景 (per 2026-09-08 05:15 JST 用户发令原话)

Ulysses 在 2026-09-08 05:15 JST 明确发令:

> **"在这类 AI 工具链接应该在启动软件时自动扫描识别并链接"**

**关键 3 维**:
1. **多 AI 工具生态** — 同一个开发者用 Claude Desktop + Claude Code + Cursor + Windsurf + OpenAI Codex + VS Code 等多种 AI 工具,每个工具各自维护自己的 config
2. **手动配置痛点** — 当前 B-3 (B.5 OpenClaw 真实凭证) / B-4 (B.6 Hermes 真实凭证) / B-5 (E.4 KMS 凭证) 都是 WBS 阻塞项,需要 Ulysses 手动提供
3. **跨工具凭证复用** — 同一份 API key 已经在 Claude / OpenAI 等配置中存在, Star 启动时应自动识别 + 复用, 不需要用户重复配

### 1.2 现状缺口

| 已落地 (v0.x) | 缺口 (本 ADR 补) |
|---|---|
| `crates/star-mcp` 16 tool 落地 (per EX-06) | 16 tool 静态注册, 不支持外部 AI 工具自动发现 |
| `AGENTS.md §4 #5` env 安全 | AI 工具 config 扫描 + 凭证注入机制缺位 |
| `AGENTS.md §4 #23` AI mock | 真实 AI 工具 config 跟 Star 隔离, 无自动 link |
| WBS §14.4 B-3 / B-4 / B-5 真实凭证 (mock 备选已落地) | 真实凭证自动复用, 减 B-3/B-4/B-5 阻塞 |
| `gm-console` AppShell 5-tab + 设置界面 (per AGENTS.md §7 #15 v0.15) | 设置界面没有"AI 工具扫描"按钮, 无 UI 触发 |

**关键缺口**:
- **多源 config 扫描**: 4 个源 (`.mcp.json` / `~/.claude/settings.json` / `~/.config/openai/` / `~/.cursor/mcp.json`) 当前全无扫描机制
- **凭证自动注入**: env_var passthrough 守门 #5 已建立, 但缺自动发现 → 注入链路
- **UI 触发**: 设置界面 + 扫描按钮缺位
- **跨工具 dedup**: 同名工具 (e.g. `create_worktree`) 在多源同时注册时, 需 dedup 策略 (per EX-06 idempotency middleware 复用)

### 1.3 架构冲突 (守门 #13 a + #19 v19 + #5 + #23 派生规)

per [AGENTS.md §4 #13 a](../../../AGENTS.md): **L1↔L1 禁止通信**. AI 工具扫描由 L0 (star-mcp 启动) 统一协调, 扫描结果注册到 star-mcp 内部 registry, 16 tool 调外部 AI 工具经 L0 派发.

per [AGENTS.md §4 #19 v19](../../../AGENTS.md): **agent 交互 Python 化**. 本 ADR 实施不涉及 subagent dispatch, 守门 #19 不适用.

per [AGENTS.md §4 #5](../../../AGENTS.md): **环境变量安全**. 凭证通过 `$env:VAR` 引用, 不打印.

per [AGENTS.md §4 #23](../../../AGENTS.md): **AI mock**. Slack 告警 + 部分 webhook 走 mock, 不开真实 API. 但 AI 工具凭证必须真实 (per 拍板 C).

---

## 2. 决策

**加到现有 `crates/star-mcp` 16 tool (per 拍板 A), 实施 AI 工具自动扫描 + 链接, 包括 4 源扫描 (`.mcp.json` / Claude / OpenAI / Cursor/Windsurf) + env_var passthrough 凭证注入 (per 拍板 C) + 启动时自动 + 设置界面手动扫描触发 (per拍板 B).**

### 2.1 4 拍板项 (per `ask_bf6bb4b2f1c27d1567da01d3`)

| # | 决策轴 | 拍板 | 理由 |
|---|---|---|---|
| **D-01** | 范围 | **加到现有 star-mcp 16 tool** | 跟 EX-06 middleware 集成, 用 idempotency dedup 防止重复注册; 1-2 周实施 |
| **D-02** | 扫描源 | **全 4 个** (.mcp.json / Claude / OpenAI / Cursor/Windsurf) | 覆盖主流 AI 工具生态; 减 WBS B-3/B-4/B-5 凭证阻塞 |
| **D-03** | 凭证 | **env_var passthrough** | per 守门 #5 不打印 + 守门 #23 走 mock webhook 但凭证必须真实; 0 凭证存储 |
| **D-04** | 触发 | **启动时自动 + 设置界面手动** (per 拍板 B) | 启动时后台扫不阻塞 (估 +200ms); 设置界面右上角菜单放"扫描"按钮, 设置界面本身没有则新建 |

### 2.2 4 源扫描实现

| 源 | 路径 (Windows) | 格式 | 凭证字段 |
|---|---|---|---|
| **`.mcp.json`** (Anthropic MCP) | 项目根 `.mcp.json` + `~/.mcp.json` | JSON `{mcpServers: {name: {command, args, env}}}` | `env` 字段 |
| **Claude / Claude Code** | `~/.claude/settings.json` + `~/.claude.json` + `.claude/settings.local.json` | JSON `{mcpServers: ...}` 或 `{apiKey: "..."}` | `apiKey` + `env` |
| **OpenAI / Codex** | `~/.config/openai/` + `~/.openai/` | TOML / JSON `{api_key, base_url, tools: [...]}` | `api_key` + env |
| **Cursor / Windsurf** | `~/.cursor/mcp.json` + `~/.windsurf/mcp.json` | JSON `{mcpServers: ...}` | `env` 字段 |

### 2.3 6 组件 (在 `crates/star-mcp/src/discovery/`)

| # | 组件 | 职责 |
|---|---|---|
| 1 | `mod.rs` | 模块声明 |
| 2 | `scanner.rs` | `ToolDiscoveryScanner` - 扫 4 源 + 并行 + 10s 内完成 |
| 3 | `registry.rs` | `ToolRegistry` - 注册 + dedup (per EX-06 idempotency) + 心跳 |
| 4 | `auth_broker.rs` | `AuthBroker` - env_var passthrough (per 守门 #5) |
| 5 | `health.rs` | `ToolHealth` - 定期检查 + 失败告警 (per F-12) |
| 6 | `settings_ui.rs` | 设置界面 spec (TS 调用入口) + 扫描按钮触发 |

### 2.4 触发链路 (per 拍板 D-04)

```
启动 → 后台线程 ToolDiscoveryScanner.scan_all() (10s)
            ↓
       4 源扫描 → ToolRegistry.register(tool) (dedup)
            ↓
       AuthBroker.inject(tool.env) (env_var passthrough)
            ↓
       16 tool 调外部 tool (经 L0 派发, 守门 #13 a)

用户手动: 设置界面右上角菜单 → "扫描" 按钮
            ↓
       ToolDiscoveryScanner.scan_all() (强制重扫)
```

### 2.5 跟 WBS 阻塞项的关联

| 阻塞项 | 本 ADR 影响 |
|---|---|
| B-3 B.5 OpenClaw 真实凭证 | 自动从 `.mcp.json` / `~/.openai/` 扫, 凭证可自动注入 |
| B-4 B.6 Hermes 真实凭证 | 同上 |
| B-5 E.4 KMS 凭证 | 部分缓解 (env_var passthrough 不替代 KMS, 仅降低 B-5 优先级) |
| B-2 5 域 Lead 真人 | 无影响, 5 域 Lead 跟 AI 工具配置无关 |

---

## 3. 拒绝方案 (3 备选 + 理由)

### 3.1 范围备选 (D-01)

| 备选 | 拒绝理由 |
|---|---|
| 新建独立架构 view (跟 Star-EI 平行) | 跟 EX-06 middleware 集成紧密, 拆 view 跨层调用复杂; 1-2 周 vs 3-4 周 |
| 跨项目持久规则 (跟 DB W/T/M 一样) | 抽象层级太早, 应先在 star-mcp 实证, 跨项目推广等 H2 阶段 |

### 3.2 凭证备选 (D-03)

| 备选 | 拒绝理由 |
|---|---|
| KMS 加密存储 (per V2-1 crates/star-credential) | 多一层加密开销, 跟 EX-06 idempotency middleware 解耦; 跨 session 凭证复用不便 |
| 懒加载 on-demand (OAuth 弹窗) | 用户每次启动需输入, UX 差; 减 WBS B-3/B-4 阻塞效果弱 |

### 3.3 触发备选 (D-04)

| 备选 | 拒绝理由 |
|---|---|
| 仅启动时自动扫 | 用户改 AI 工具 config 后, 需重启 Star 才能生效, UX 差 |
| 仅按需 (首次访问时扫) | 首次访问有 200-500ms 延迟, 影响 UX; 启动时扫是更稳的选择 |
| 启动扫 + 文件系统 watch (持续同步) | 多 watch 资源占用; 暂不需要实时同步, 设置界面手动重触发足够 |

---

## 4. 后果

### 4.1 正面

1. **WBS 阻塞缓解**: B-3 / B-4 / B-5 真实凭证自动复用, 部分阻塞降级
2. **UX 提升**: 启动即用, 无需手动配 AI 工具
3. **跨工具一致**: 同一份凭证在 Claude / OpenAI / Cursor / Star 共享
4. **去重**: 跟 EX-06 idempotency middleware 集成, 同名 tool 自动 dedup
5. **可观测**: 工具发现 + 注册 + 心跳全链路 metric 暴露

### 4.2 负面 / 风险

1. **启动延迟**: +200ms (估) 启动时扫描; 通过后台线程不阻塞主进程缓解
2. **凭证泄露风险**: env_var passthrough 引用, 不打印 (per 守门 #5), 但内存中存在
3. **格式兼容性**: 4 源各自格式不同, 需维护 4 个 parser; 跨版本兼容需测试
4. **冲突解决**: 同名工具多源同时注册, 需 dedup 策略 (优先级排序)

### 4.3 5 域 Lead RACI

| 域 | RACI | 拍板权 |
|---|---|---|
| admin 域 | R+A (Mavis 临时代签) | AI 工具凭证管理 (per 守门 #14 v2 拍板 D) |
| 其他 4 域 | I 通知 | 无影响 |

---

## 5. 实施计划 (1-2 周)

| # | 子项 | 内容 | 估时 |
|---|---|---|---|
| **TD-01** | 4 源 scanner + 1 UT | scanner.rs + 4 parser (mcp.json / claude / openai / cursor) | 0.5 周 |
| **TD-02** | registry + dedup | registry.rs + 复用 EX-06 idempotency middleware | 0.3 周 |
| **TD-03** | auth_broker + 启动时线程 | auth_broker.rs + tokio 异步启动扫描 | 0.3 周 |
| **TD-04** | UI 扫描按钮 + 设置界面 | settings_ui.ts + 右上角菜单 + 设置界面 (新建) | 0.4 周 |
| **TD-05** | 集成测试 + 文档 | 1 IT + 1 文档 + commit + 推 origin | 0.2 周 |
| **总** | | | **1.7 周** |

---

## 6. 验证

### 6.1 守门实证

- (1) `cargo check -p star-mcp -j 4` 0 err
- (2) `cargo test -p star-mcp discovery -j 4` 100% pass
- (3) `cargo test -p star-mcp middleware::idempotency -j 4` 100% pass (EX-06 0 回归)
- (4) `cargo build --release -p star-mcp` 0 err

### 6.2 4 想定シナリオ (S-TD-01..S-TD-04)

- S-TD-01: 启动时自动扫 4 源, 10s 内完成
- S-TD-02: 同名 tool 多源 dedup, 取优先级最高源
- S-TD-03: 凭证从 `.mcp.json` env 字段自动注入, 不打印
- S-TD-04: 设置界面"扫描"按钮强制重扫, 5s 内完成

---

## 7. 签字

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 |
| SRE Lead | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 拍板 D 临时代签) |
| 平台 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 拍板 D 临时代签) |
| 评审主持 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 拍板 D 临时代签) |
| PM | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 拍板 D 临时代签) |

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-08 05:18 JST | Ulysses — Mavis 接手 | 初稿, 4 拍板项 + 6 组件 + 5 子项计划 | per `ask_bf6bb4b2f1c27d1567da01d3` 4 推荐项 |
| v1.0 | 2026-09-08 05:18 JST | Ulysses — Mavis 接手 | Accepted v1.0 拍板落地, 同步 brief + 实施 | 拍板 + 实施 |
