# ADR-0051: Star 凭证 UX 分类拍板 (AI Agent 界面 vs 设置界面)

> **状态**: 🟢 Accepted v1.0 (per 2026-09-08 05:30 JST 用户发令"这些凭证ai相关的允许用户在agent界面自己填,其他放在设置界面填")
> **生效**: 2026-09-08
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签字**: 🟢 架构师 (Mavis 接手 agent per DEC-008) (per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签" + 守门 #10 author=Ulysses)
> **关联**: [ADR-0049 AI 工具自动扫描 + 链接 (D-03 env_var passthrough)](0049-ai-tool-auto-discovery.md) · [V2-1 crates/star-credential (KMS 加密)](../../../crates/star-credential/) · [WBS §14.4 B-3/B-4/B-5 凭证阻塞](../../reports/STAR-P3-WBS-001.md) · [AGENTS.md §4 守门硬约束](../../../AGENTS.md)
> **下游**: 跨 view 集成 (TD-01 / V2-1 / 设置界面 / Agent 界面) + 推 origin (per 守门 #1 反转 9/7 21:08 JST)

---

## 1. 背景与问题

### 1.1 业务背景 (per 2026-09-08 05:30 JST 用户发令原话)

Ulysses 在 2026-09-08 05:30 JST 明确发令:

> **"这些凭证 ai 相关的允许用户在 agent 界面自己填, 其他放在设置界面填"**

**关键 3 维**:
1. **凭证分类**: AI 相关凭证 (OpenAI / Claude / Anthropic / GitHub Copilot / Cursor / Codex) 跟其他凭证 (KMS / Vault / 内部 secret / DB 凭证) UX 路径分离
2. **per-agent vs per-tenant**: AI 凭证 per-agent (per 9 SA + SA-10), 其他凭证 per-tenant (组织级)
3. **UX 减负**: 开发者日常最频繁的"AI 工具"凭证放在 agent 界面, 减少"开设置界面"操作

### 1.2 现状缺口

| 已落地 (v0.x) | 缺口 (本 ADR 补) |
|---|---|
| `crates/star-credential` (per V2-1) KMS 加密存储 per-tenant 凭证 | UI 端 UX 路径未定义, 用户不知道在哪填 AI 凭证 |
| `TD-01` AI 工具自动扫描 env_var passthrough (per ADR-0049 D-03) | 漏用户自己配 AI 凭证的入口 (per-agent) |
| gm-console 设置界面 (per AGENTS.md §7 #15) | 5 tab 命名拍板完成 (per ADR-0050), 但**凭证 UX 分类**未定 |
| WBS §14.4 B-3/B-4/B-5 凭证阻塞 (mock 备选) | 真实凭证 UX 入口未拍板 |

**关键缺口**:
- **per-agent AI 凭证**: SA-01..SA-09 + SA-10 各自用 OpenAI/Claude key, 需 per-agent 持久化
- **per-tenant 其他凭证**: KMS / Vault / DB 凭证, 集中管理
- **env_var passthrough 优先**: TD-01 已有, 但用户漏配时需 agent 界面 fallback

### 1.3 架构冲突 (守门 #5 + #13 c + #14 v2 + #23)

per [AGENTS.md §4 #5](../../../AGENTS.md): **环境变量安全**. 凭证通过 `$env:VAR` 引用, 不打印.

per [AGENTS.md §4 #13 c](../../../AGENTS.md): **RLS 13 类必携**. per-agent 凭证 (per-agent 隔离) + per-tenant 凭证 (per-tenant 隔离) 双层.

per [AGENTS.md §4 #14 v2](../../../AGENTS.md): **5 域 Lead 临时代签**. Mavis 临时代签 admin 域凭证管理决策, 真人到位后追溯签字.

per [AGENTS.md §4 #23](../../../AGENTS.md): **AI mock**. 凭证走真实 (per 拍板 C env_var passthrough), 但显示层走 mock.

---

## 2. 决策

**凭证 UX 分类拍板: AI 相关凭证 (per-agent) 走 agent 界面, 其他凭证 (per-tenant) 走设置界面. 跟现有 `crates/star-credential` (KMS 加密) + TD-01 (env_var passthrough) 集成.**

### 2.1 凭证分类总表 (per 拍板)

| 类别 | 子类 | 凭证 | UX 入口 | 持久化 | 隔离粒度 |
|---|---|---|---|---|---|
| **AI 凭证** (per-agent) | LLM API | OpenAI API key / Claude API key / Anthropic API key / Google Gemini API key / Mistral API key | **Agent 界面** (per-agent 配置) | KMS 加密 (per V2-1) | per-agent (per SA) |
| **AI 凭证** (per-agent) | Code AI | GitHub Copilot PAT / Cursor API key / OpenAI Codex API key / Claude Code token / Cline API key | **Agent 界面** | KMS 加密 | per-agent |
| **AI 凭证** (per-agent) | Search API | Tavily API key / Brave Search API key / SerpAPI key | **Agent 界面** | KMS 加密 | per-agent |
| **AI 凭证** (per-agent) | Embedding | OpenAI Embedding / Cohere Embedding / Voyage AI | **Agent 界面** | KMS 加密 | per-agent |
| **AI 凭证** (per-agent) | Custom | 用户自配的 AI 工具凭证 (per agent 私有) | **Agent 界面** | KMS 加密 | per-agent |
| **其他凭证** (per-tenant) | KMS | Vault token / AWS IAM role / GCP KMS service account / Azure Key Vault | **设置界面** (admin 域) | KMS 加密 | per-tenant |
| **其他凭证** (per-tenant) | DB | PostgreSQL DSN / Redis URL / Kafka SASL | **设置界面** | KMS 加密 | per-tenant |
| **其他凭证** (per-tenant) | Webhook | Slack webhook / GitHub App secret / Generic webhook secret | **设置界面** | KMS 加密 | per-tenant |
| **其他凭证** (per-tenant) | Org-level GitHub | GitHub Org PAT (per-org 而非 per-agent) | **设置界面** | KMS 加密 | per-tenant |
| **其他凭证** (per-tenant) | 内部 secret | JWT signing key / OAuth client secret | **设置界面** | KMS 加密 | per-tenant |

### 2.2 拍板理由 (per 用户发令)

- **UX 减负**: AI 凭证开发者日常最常用, 放在 agent 界面"1 click 即可填"
- **职责清晰**: AI 凭证 per-agent (谁用谁配), 其他 per-tenant (admin 集中)
- **跟现有架构兼容**:
  - `crates/star-credential` (KMS 加密) 已落地, 复用
  - TD-01 env_var passthrough 优先 (用户已有 AI 工具 config 自动复用)
  - Agent 界面 / 设置界面 fallback (用户漏配时手动填)
- **守门 #5 + #13 c 合规**: KMS 加密 + RLS 13 类隔离 (per-agent / per-tenant 双层)

### 2.3 UX 流程

```
[启动] TD-01 自动扫描 4 源 (.mcp.json / Claude / OpenAI / Cursor/Windsurf)
    ↓
[命中] env_var passthrough (per 守门 #5 不打印) → 直接用
    ↓
[未命中] 用户在 agent 界面 (per-agent 配置) 或设置界面 (per-tenant) 手动填
    ↓
[KMS 加密] 存 star_credential 表 (per V2-1) + RLS 13 类 (per-agent / per-tenant 隔离)
    ↓
[运行时] 解密 + 注入 AI tool / KMS 调用
```

### 2.4 跟现有 view 集成

| 现有 view | 集成点 |
|---|---|
| **TD-01** (per ADR-0049) | 4 源扫描 + env_var passthrough 优先, 未命中时 fallback agent 界面 / 设置界面 |
| **V2-1 crates/star-credential** | KMS 加密存储, 复用 (per-agent / per-tenant 双层 RLS) |
| **gm-console 设置界面** (per ADR-0050) | 设置界面 (per-tenant admin 域) 集中管理其他凭证 |
| **gm-console Agent 界面** | 9 SA + SA-10 各自 agent 配置 tab (per-agent 凭证) |
| **守门 #5 env 安全** | agent 界面 + 设置界面都不打印 value, 仅引用 |
| **守门 #13 c RLS 13 类** | per-agent / per-tenant 双层隔离 |
| **守门 #23 AI mock** | 显示层 mock, 凭证真实 (per 拍板 C) |

---

## 3. 拒绝方案 (3 备选 + 理由)

### 3.1 UX 路径备选

| 备选 | 拒绝理由 |
|---|---|
| **A** 所有凭证统一设置界面 | 用户高频操作 (AI 凭证) 路径长, UX 差; 违反"per-agent 凭证归 agent"原则 |
| **B** 所有凭证 agent 界面 | per-tenant 凭证 (KMS / DB) 暴露给所有 agent, 安全风险 |
| **C** ✅ AI 凭证 agent 界面 + 其他凭证设置界面 (本拍板) | 跟 per-agent / per-tenant 隔离一致, UX 减负, 跟现有架构兼容 |

### 3.2 持久化备选

| 备选 | 拒绝理由 |
|---|---|
| 明文存 DB | 违反安全最佳实践, 跟 V2-1 KMS 冲突 |
| **KMS 加密 (per V2-1 crates/star-credential)** | ✅ 已落地, 复用 |
| 浏览器 localStorage | 跨设备失效, 不安全 |
| OS keychain | 跨用户失效 (per-user 唯一) |

### 3.3 隔离粒度备选

| 备选 | 拒绝理由 |
|---|---|
| per-user 隔离 | 同一 user 多 agent 共享凭证, 不必要 |
| **per-agent (AI 凭证) + per-tenant (其他凭证)** | ✅ 跟职责匹配 (per-agent 谁用谁配, per-tenant admin 集中) |
| 全局共享 | 安全风险 |

---

## 4. 后果

### 4.1 正面

1. **UX 优化**: 开发者日常高频 AI 凭证 1 click 填, 减少"开设置界面"操作
2. **职责清晰**: AI 凭证 per-agent, 其他 per-tenant, 边界清楚
3. **跟现有架构兼容**: V2-1 KMS 加密 + TD-01 env_var passthrough 复用, 0 重写
4. **安全强化**: KMS 加密 + RLS 13 类双层隔离 (per-agent / per-tenant)
5. **WBS B-3/B-4/B-5 阻塞部分缓解**: 凭证 UX 路径拍板, 减少凭证阻塞

### 4.2 负面 / 风险

1. **UI 改造**: agent 界面新增"凭证"tab, 设置界面 (admin 域) 新增"凭证"section, 跨 session 续
2. **DDD Review 真人到位后追溯签字**: 守门 #14 v2 拍板 D 要求"真人到位后追溯签字", admin 域凭证管理真人到位
3. **跨 session 续**: agent 界面 + 设置界面 UI 实施跨 session 续
4. **TD-04 UI 扫描按钮跟本 ADR 集成**: 跨 session 续

### 4.3 5 域 Lead RACI (per 守门 #14 v2)

| 域 | RACI | 拍板权 |
|---|---|---|
| admin 域 | R+A (Mavis 临时代签) | 设置界面凭证管理 (per-tenant) |
| 其他 4 域 | I 通知 | agent 界面凭证管理 (per-agent) |

---

## 5. 实施计划 (跨 session 续)

| # | 子项 | 内容 | 估时 |
|---|---|---|---|
| **CR-01** | Agent 界面凭证 tab | 9 SA + SA-10 各自 agent 配置页加 "凭证" tab (per-agent 持久化) | 0.3 周 |
| **CR-02** | 设置界面凭证 section | 设置界面 admin 域加 "凭证" section (per-tenant 持久化) | 0.3 周 |
| **CR-03** | 跟 TD-01 集成 | 4 源扫描未命中时, agent 界面 / 设置界面 fallback 入口 | 0.2 周 |
| **CR-04** | 跟 V2-1 集成 | KMS 加密 + RLS 13 类双层隔离 (per-agent / per-tenant) | 0.2 周 |
| **总** | | | **1.0 周** |

---

## 6. 验证 (per 守门 #1 + #5 + #13 c)

### 6.1 守门实证

- (1) 守门 #10 author=Ulysses
- (2) 守门 #14 v2 5 域 Lead 临时代签 (admin 域凭证管理)
- (3) 守门 #15 饱和: 用户发令=新事件, 符合
- (4) 0 文档改动 (per-agent / per-tenant 双层 UX 跟现有 `crates/star-credential` 兼容)

### 6.2 4 想定シナリオ (S-CR-01..S-CR-04)

- S-CR-01: Agent 界面填 OpenAI key, per-agent 持久化 + KMS 加密
- S-CR-02: 设置界面填 Vault token, per-tenant 持久化 + KMS 加密
- S-CR-03: TD-01 扫描命中 → env_var passthrough; 未命中 → 引导用户到 agent 界面 / 设置界面
- S-CR-04: RLS 13 类隔离, tenant A 不能读 tenant B 的 KMS token

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
| v0.1 | 2026-09-08 05:30 JST | Ulysses — Mavis 接手 | 初稿, 10 凭证分类 + 2 UX 入口 + 跟 TD-01/V2-1/守门 集成 | per 2026-09-08 05:30 JST 用户发令"这些凭证ai相关的允许用户在agent界面自己填,其他放在设置界面填" |
| v1.0 | 2026-09-08 05:30 JST | Ulysses — Mavis 接手 | Accepted v1.0 拍板落地, 0 文档改动, 跨 session 续 CR-01..CR-04 实施 | 拍板 + 跟现有架构兼容 |
