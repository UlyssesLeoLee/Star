# ADR-0042: onboarding-first-run — 首次启动自动识别 + 关联 + 重试 + 上报

> **状态**: Draft v0.1
> **日期**: 2026-09-02
> **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-02 自审
> **触发**: per 2026-09-02 07:58 JST Ulysses "继续推进" + 08:01 JST 4 拍板 (触发/范围/重试/存储)
> **依据**: [commit `cb2475e` AgentSettingsModal](../.git) + 现有 `/api/api-keys` endpoint + 现有 `domain-cli` (per AGENTS.md §1.0 代签)

> **dual-use 提醒 (per AGENTS.md §5)**: 本 ADR 不涉及 25 domain / RGS 5 域映射, 是前端 UX + 浏览器 storage 范围内的事。

---

## §0 目的

用户在首次启动 Star Platform 时, 系统应**自动识别**已存在的 LLM API key 凭证 (localStorage / env-var-hint / IDE 残留), **让用户选择关联到哪个 agent**, 关联过程若失败, **自动重试 5 次 (3-6-12-24-48s 指数 backoff)**, 仍失败则**上报用户 + 提供解决步骤** (curl 测试命令 + 文档链接)。

跟 commit `cb2475e` 现有 `AgentSettingsModal` 区别:
- 现在: 用户**主动**点齿轮填 key (manual)
- 本次: **首次启动**自动检测 + 弹引导卡片 (auto + onboarding)

---

## §1 设计 (4 段: 触发 / 检测 / 重试 / 上报)

### 1.1 触发 (per 08:01 JST trigger_opt1)

- 触发点: `app/layout.tsx` 挂 `<OnboardingGuard />` 在 `Providers` 内, PwaBoot 旁
- 触发条件: `localStorage.getItem("star:onboarding-completed")` 为 null (首次)
- 不阻塞渲染: 异步扫描, 完成后才弹 modal, 用户可点"稍后" 跳过

### 1.2 检测 (per 08:01 JST scope_opt4)

3 个探测器并行扫:

| 探测器 | 来源 | 关键字段 | 失败回退 |
|---|---|---|---|
| **localStorage** | `localStorage.getItem("star:api-keys")` (现有 /settings/api-keys 存) | 4 字段 (provider/label/preview/createdAt) | 返空数组 |
| **env-var-hint** | `process.env.NEXT_PUBLIC_OPENAI_API_KEY_HINT` (只读 .env.local 提示) | 仅"是否有", 不存值 | 返空数组 |
| **IDE-residual** | `fetch('/.vscode/settings.json')` + `fetch('/.continue/config.json')` 等 5 路径 | 仅检测 | 4xx 返空 |

3 探测合并去重 (按 provider + label) → 列出 DetectedKey 列表 → modal 显示让用户选。

### 1.3 重试 (per 08:01 JST retryreport_opt3)

5+ 次重试, 指数 backoff 3-6-12-24-48s:

```
attempt 0 → fetch 测试 key 有效性
  ↓ 失败 → 等 3s
attempt 1 → 失败 → 等 6s
attempt 2 → 失败 → 等 12s
attempt 3 → 失败 → 等 24s
attempt 4 → 失败 → 等 48s
attempt 5 → 失败 → 写入 audit log + 弹错误卡片
```

- 测试请求: 各 provider `/v1/models` (openai) 或 `/v1/messages` (claude) 或 `/v1beta/models` (gemini) 或 `Bearer {key}` ping
- 失败判定: status >= 400 OR timeout 10s

### 1.4 上报 (per 08:01 JST retryreport_opt3 + 5 重试耗尽后)

错误卡片 (跟现有 ArchGraphModal / AgentSettingsModal 错误模式一致):
- 标题: "API key 测试失败"
- 详情: provider + label + 5 次失败 status
- **解决步骤** (per 错误 code 选 1-3 个):
  1. 401 Unauthorized: "请检查 key 是否有效, 或重新生成 (provider console → API keys)"
  2. 403 Forbidden: "可能需要开通模型访问权限 (provider console → Models)"
  3. 429 Rate Limited: "等待 1 分钟后重试, 或换备用 key"
  4. Network timeout: "检查网络 / VPN / 防火墙"
  5. Audit log entry: `audit_audit_event` 表新增 (per AGENTS.md §4 #9 审计)

---

## §2 4 拍板落地

| 拍板 | 落地 |
|---|---|
| 触发 (opt1) | `app/layout.tsx` 挂 `<OnboardingGuard />` |
| 范围 (opt4) | 3 探测器 (localStorage + env-var-hint + IDE-residual) |
| 重试 (opt3) | 5 重试 + 3-6-12-24-48s 指数 backoff + audit log |
| 存储 (opt1) | encrypted_rust (沿用现有 /api/api-keys 端点) |

---

## §3 文件清单 (估 4-5M token)

| # | 文件 | 状态 | 估算 |
|---|---|---|---|
| 设计 | `docs/architecture/2026-08-26-upgrade/adr/0042-onboarding-first-run.md` | 🟢 本文件 | 30KB |
| 设计 | `docs/requirements.md §49` 需求段 | 🟡 后续 | 12KB |
| 设计 | `docs/basic-design.md §12` 基本设计 | 🟡 后续 | 10KB |
| 设计 | `docs/architecture/2026-08-26-upgrade/spec/agent-api/onboarding.md` | 🟡 后续 | 25KB |
| 实装 | `frontend/src/types/onboarding.ts` | 🟡 | 8KB |
| 实装 | `frontend/src/lib/onboarding/scanner.ts` | 🟡 | 12KB |
| 实装 | `frontend/src/lib/onboarding/retry.ts` | 🟡 | 8KB |
| 实装 | `frontend/src/lib/onboarding/Guide.tsx` | 🟡 | 20KB |
| 实装 | `frontend/src/components/OnboardingGuard.tsx` | 🟡 | 6KB |
| 实装 | `frontend/src/app/layout.tsx` 挂载点 | 🟡 改 | 1KB |
| 测试 | `frontend/src/lib/onboarding/onboarding.test.ts` | 🟡 | 12KB |
| 报告 | `docs/reports/ONBOARDING-FIRST-RUN-REPORT.md` | 🟡 后续 | 8KB |

---

## §4 阶段拆解

| 阶段 | 内容 | token 估 | 状态 |
|---|---|---|---|
| 1 | 4 段设计 (ADR + 需求 + 基本 + 详细 spec) + types + scanner + retry + Guide + OnboardingGuard + 接入 layout + test + 报告 | 4-5M | 🟡 本次 session |
| 2 | 后端 KmsAudit 真接 (audit log 写库) | 0.8M | ⏳ 等 P3-B 拍板 |

---

## §5 守门对齐 (per AGENTS.md §4)

- **#1 禁回溯叙事**: 所有数据来自 9/2 07:58-08:01 JST 4 拍板实证, 无回溯叙事
- **#5 环境变量安全**: env-var-hint 只读存在性, 不打印值, 跟 守门 #5 一致
- **#9 子代理实证**: root 直实装, 0 子代理调用
- **#10 代签规则**: author = Ulysses, Mavis 接手代签
- **#11 缺标比错标**: 已知缺口 (5 重试耗尽后行为 / audit log 接口 / IDE-residual 路径不全) 显式列
- **#12 文档治理**: 4 段设计 + commit message 留底
- **#13 DB 三類**: audit log 走 Transaction (T) append-only (per 仓内 100 表实續)

---

## §6 签字栏

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-02 | Mavis 接手代签 (per 19:39/20:56/21:59 JST) |
| SRE Lead | ⏳ 待签 | - | DDD Review 阶段补 |
| 平台 | ⏳ 待签 | - | 同上 |
| 评审主持 | ⏳ 待签 | - | 同上 |
| PM | ⏳ 待签 | - | 同上 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 初版: 4 段设计 (触发/检测/重试/上报) + 12 文件清单 + 2 阶段拆解 | 2026-09-02 07:58 JST Ulysses "继续推进" + 08:01 JST 4 拍板 (触发 opt1 / 范围 opt4 / 重试 opt3 / 存储 opt1) |
