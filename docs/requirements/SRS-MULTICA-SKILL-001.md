# SRS-MULTICA-SKILL-001

> **Multica Skill Compounding 域要件定義書 v0.1** (per ADR-0026 v0.2 §1.1 抽象, 跟 v35+ 候选对齐)
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 commit: (留空, root 统一 commit 时填)
> - 关联基本設計書: [`docs/design/BD-MULTICA-SKILL-001.md`](../design/BD-MULTICA-SKILL-001.md) (下个 turn 落档)
> - 关联 ADR: [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2
> - 关联 inventory: [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.4 (v35+ 候选)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4
> - 日期: 2026-09-11 JST
> - 受众: 詳細設計エンジニア / アーキテクト / SRE / 5 域 Lead 真人

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-MULTICA-SKILL-001 |
| 文书名 | Multica Skill Compounding 域要件定義書 (v35+ 候选对齐) |
| 版本 | v0.1 |
| 作成日 | 2026-09-11 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 ADR | ADR-0026 v0.2 §1.1 抽象表 "Skill compounding" |
| 关联 inventory | inventory §2.4 v35+ 候选 |
| 上位要件 | (无) |
| 守门合规 | 守门 #1 + #5 + #6 + #9 + #11 + #12 v21 + #14 v4 全过 |
| 模板结构 | 12 段严格按 brief §1.3 |
| 子能力 | 4 子能力 (SK-1 ~ SK-4) × 18 项 (per §6) |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档基于 ADR-0026 v0.2 §1.1 抽象 (Skill compounding 跨 subagent 复用) + inventory §2.4 v35+ 候选, 定义 STAR 平台 **Multica Skill Compounding 域** 的需求规格说明书。

**核心方向锚点 (per ADR-0026 v0.2 + 2026-09-11 20:10 JST Ulysses 拍板)**: 把 Multica 的 skill 跨 subagent 复用思路搬到 STAR —— subagent 完成的"成功方案"自动变 reusable skill, 跨 task / cross-agent 复用。**不引入** Multica server-side skill store + pgvector embedding 检索 (跟 ADR-0026 §2.2 不参考 + 过设计), 走 `docs/skills/` + `scripts/automation/` + SKILL.md 模板的本地化路径。

### 1.2 背景 (用户痛点)

STAR / Mavis 当前 root session 模型下, 3 类具体痛点:

1. **subagent 完成方案不可复用** — subagent 写完 1 段 code, 任务结束, 下一个 subagent 重新摸索
2. **`docs/skills/` 半自动** — 现有 skills 是手工写, 不从成功 task 自动产生 (per `docs/automation-design.md` §3.4 横向范式)
3. **`scripts/automation/` 复用率低** — 14 份基类已实装 (per `scripts/automation/registry.md`), 但跨 subagent 缺统一检索 / 选择机制

### 1.3 包含范围 (In-Scope)

4 子能力 (per Multica skill_create.go + skill_refresh.go + runtime_local_skills_redis_store.go 抽象):

| 子能力 | Multica 源 | 关键抽象 |
|---|---|---|
| SK-1 Skill 生命周期 | skill_create.go + skill_refresh.go | create → store → discover → reuse |
| SK-2 SKILL.md 模板 | Multica `server/pkg/skill/*.go` | name / description / when-to-use / steps / tools |
| SK-3 Local vs server skill 分离 | runtime_local_skills_redis_store.go | local = 本机, server = 共享, workspace = 项目 |
| SK-4 Skill search 跟 version | skill_search.go + skill_metadata.go | 基础 keyword match + version 30 天留存 |

### 1.4 不含范围 (Out-of-Scope, per ADR-0026 v0.2 §2.2 + 简化)

- ❌ pgvector embedding 检索 (Multica server-side) — 过设计
- ❌ Auto-create from subagent success (per Multica "skills compound" 模式) — 暂走手工 + 半自动, 后续 v39+ 候选
- ❌ Skill marketplace / cross-organization 共享 — 一人公司不需要
- ❌ Redis-backed skill store — 用文件系统 + git 即可

### 1.5 受众范围 disclaimer

- 本 SRS 跟现有 `docs/skills/` (手写) + `scripts/automation/registry.md` (脚本索引) 平行
- 落地后跟 `automation-design.md` §3.4 横向 audit log 范式协调

---

## §2 用语定义

| 用语 | 定义 |
|---|---|
| **Skill** | 一个 reusable procedure, 含 name / description / when-to-use / steps / tools 5 字段 |
| **SKILL.md** | skill 的 markdown 描述文件, 跟现有 `docs/skills/` 对齐 |
| **Local skill** | 本机用户私有, 落 `~/.mavis/skills/<name>/SKILL.md` (类比 Multica `runtime_local_skills`) |
| **Server skill** | 仓库共享, 落 `docs/skills/<name>/SKILL.md` (git tracked) |
| **Workspace skill** | 项目级, 落 `<workspace>/.mavis/skills/<name>/SKILL.md` (git ignored) |
| **Skill version** | 每次更新 +1 version, 旧 version 留 30 天可回滚 |
| **Skill match** | 触发时按 (task title, task description) keyword 匹配 skill 列表 |
| **Skill auto-load** | subagent dispatcher 接到 task 时, 自动 load 匹配 skills 进 subagent context |

---

## §3 业务背景

### 3.1 Multica 实证 (per ADR-0026 v0.2 §1.1 + §1.3)

| 源 | 关键 |
|---|---|
| `server/internal/handler/skill_create.go` | skill 创建 + metadata |
| `server/internal/handler/skill_refresh.go` | skill refresh (重读 SKILL.md) |
| `server/internal/handler/runtime_local_skills.go` | local skill store |
| `server/internal/handler/skill_search.go` | skill 检索 (本 SRS 简化) |
| `server/internal/handler/skill_metadata.go` | metadata 提取 |

**关键设计 (per Multica)**: skill 跨 subagent 复用, 完成方案自动变 skill, 跨 workspace 可用。

**本 SRS 简化**:
- skill 创建走手工 + 半自动 (不全自动) — 降低误判
- 检索走 keyword match, 不上 embedding — 一人公司够用
- 存储走文件系统 + git, 不上 Redis — 跟现有 `docs/skills/` 对齐

### 3.2 拍板来源

- 2026-09-11 20:10 JST Ulysses "multica 的核心功能我原则上都要有"
- 20:43 JST ask_user 选项 form_opt2 + inventory_opt1 + code_opt1

### 3.3 守门合规 (per AGENTS.md §4)

- 守门 #1 v15 (新事件触发)
- 守门 #5 (env 安全) — skill 不读 env
- 守门 #6 (PowerShell only)
- 守门 #11 (缺标比错标) — skill version 不删
- 守门 #12 v21 ([P] docs 同步) — 每次 skill 创建 / 更新落 log
- 守门 #14 v4 (Mavis 审核)

---

## §4 功能需求 (FR, 18 项)

### SK-1 Skill 生命周期 (FR-1 ~ FR-6)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-1 | Skill 4 状态: draft → active → deprecated → archived | P0 |
| FR-2 | Skill 创建: 手工写 SKILL.md → `automation/skill_create.py <name>` 落盘 + 写 registry | P0 |
| FR-3 | Skill 半自动从 subagent success: subagent 跑完 + Mavis review 通过 → Mavis 建议 "存为 skill?" → 人工确认 | P0 |
| FR-4 | Skill refresh: `automation/skill_refresh.py <name>` 重读 SKILL.md + 更新 version + 更新 registry | P0 |
| FR-5 | Skill deprecated: `automation/skill_deprecate.py <name>` 标 deprecated, 不出现在 match 列表, 但 30 天内可复活 | P0 |
| FR-6 | Skill archived: 30 天后自动归档, git 保留 history (per 守门 #11 不删) | P0 |

### SK-2 SKILL.md 模板 (FR-7 ~ FR-10)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-7 | SKILL.md 5 字段 frontmatter: name / description / when-to-use / steps / tools | P0 |
| FR-8 | steps 字段 markdown 列表, 跟现有 `docs/skills/*.md` 格式一致 | P0 |
| FR-9 | tools 字段 list[str], 标识 skill 依赖哪些 tool (e.g. [git, mavis_dispatcher, registry_scan]) | P0 |
| FR-10 | SKILL.md 模板由 `automation/skill_template.py` 生成, 默认 5 字段填好 | P0 |

### SK-3 Local vs server vs workspace (FR-11 ~ FR-14)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-11 | Local skill: `~/.mavis/skills/<name>/SKILL.md`, 单机用户私有, git ignored | P0 |
| FR-12 | Server skill: `docs/skills/<name>/SKILL.md`, 仓库共享, git tracked (per 现有 `docs/skills/`) | P0 |
| FR-13 | Workspace skill: `<workspace>/.mavis/skills/<name>/SKILL.md`, 项目级, git ignored | P0 |
| FR-14 | 优先级: local > workspace > server (per Multica 隐含顺序, 本地优先) | P0 |

### SK-4 Skill match + auto-load (FR-15 ~ FR-18)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-15 | Skill match: subagent dispatcher 接到 task 时, 按 (task title, task description) keyword 匹配 active skill 列表 | P0 |
| FR-16 | Match 阈值: keyword 命中率 >= 50% 视为匹配, 多个匹配取 score top 3 | P0 |
| FR-17 | Skill auto-load: 匹配 skill 列表进 subagent context (per Multica 默认行为) | P0 |
| FR-18 | Subagent 可在 result 里 "推荐存为 skill" → Mavis 收到后弹窗 "是否存为 skill" (per FR-3 半自动) | P0 |

---

## §5 非功能需求 (NFR, 5 项)

| NFR | 指标 |
|---|---|
| NFR-1 性能 | skill match < 100ms (本地 fs + keyword) |
| NFR-2 可靠性 | skill 加载失败不阻塞 subagent 启动 (fallback: 跳过该 skill, log warning) |
| NFR-3 可观测 | 每次 skill 加载 / 创建 / 更新写 audit log |
| NFR-4 易用 | `automation/skill_create.py <name>` 一行命令 |
| NFR-5 安全 | skill 不存 secret, 跟守门 #5 env 安全一致 |

---

## §6 约束 / 风险

| 约束 | 描述 |
|---|---|
| 守门 #5 env 安全 | skill 文件不读 env |
| 守门 #6 PowerShell only | subprocess.run(shell=False) |
| 守门 #11 缺标比错标 | skill version 30 天后才 archive, 期间可回滚 |
| 守门 #12 v21 [P] docs 同步 | 每次 skill 创建 / 更新落 log |

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| Subagent "推荐存为 skill" 太多, 人工 review 负担 | 中 | 中 | 阈值卡严, score < 0.7 不弹 |
| Skill match 误匹配, subagent 用错 skill | 中 | 中 | subagent 可在 result 里 "skip this skill", 反馈进 match 算法 |
| Skill 跨 subagent 复用导致上下文污染 | 低 | 中 | skill 内容是 markdown 描述, 不是 code, 污染面小 |
| Local / server / workspace 优先级冲突 | 低 | 低 | 显式优先级 local > workspace > server, 不冲突 |

---

## §7 验收条件 (AC)

| AC | 描述 |
|---|---|
| AC-1 | skill 4 状态 (draft / active / deprecated / archived) 全部跑通 |
| AC-2 | SKILL.md 5 字段 frontmatter 模板可生成 |
| AC-3 | Local / server / workspace 三级 skill 都可创建, 优先级 local > workspace > server |
| AC-4 | skill match < 100ms, 阈值 >= 50% |
| AC-5 | subagent dispatcher auto-load 匹配 skill 进 context |
| AC-6 | subagent result 里 "推荐存为 skill" → Mavis 弹窗 |
| AC-7 | skill audit log 每次创建 / 更新 +1 entry |
| AC-8 | 守门 #13 W-T-M 100% 覆盖 (skill definition = Master / skill usage = Transaction / skill match = Work) |

---

## §8 已知缺口 (per 守门 #11)

| 缺口 | 优先级 | 阻塞 | 缓解 |
|---|---|---|---|
| #1 pgvector embedding 检索不上 (per ADR-0026 §2.2 不参考) | P2 | 不阻塞 (keyword match 够用) | 后续 v39+ 评估 |
| #2 Auto-create from subagent success 仅半自动 (人工 review 确认) | P0 | 不阻塞 (降低误判) | 后续 v40+ 全自动 |
| #3 Skill version 30 天归档策略, 跟 WBS 已有 30 天 task GC 协调 | P0 | 阻塞 AC-1 | 复用 WBS GC |
| #4 Cross-organization 共享 (skill marketplace) 不做 | P0 | 不阻塞 (一人公司) | 文档加 disclaimer |
| #5 Skill match 阈值 50% 是经验值, 没实证 | P1 | 不阻塞 | 落档后看 1 周 subagent 用例, 调阈值 |
| #6 Skill 跟守门 #12 v21 [P] docs 同步: 每次 skill 创建算 [P] 吗? | P0 | 阻塞 AC-7 | 算 [P] 但小, 走 registry 同步 |

---

## §9 关联文档

| 类型 | 文档 |
|---|---|
| ADR | [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2 §1.1 抽象表 |
| Inventory | [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.4 v35+ 候选 |
| BD | [`docs/design/BD-MULTICA-SKILL-001.md`](../design/BD-MULTICA-SKILL-001.md) (下个 turn 落档) |
| 现有 skills | `docs/skills/` (现有手写 skills) |
| automation-design | [`docs/automation-design.md`](../automation-design.md) v0.1+ §3.4 横向 audit log 范式 |
| registry | [`scripts/automation/registry.md`](../../scripts/automation/registry.md) v0.1 |
| 守门 | AGENTS.md §4 + §4.1 #1 v15 / #5 / #6 / #11 / #13 |

---

## §10 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 2026-09-11 20:43 JST 拍板 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |

---

## §11 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-11 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（18 FR / 5 NFR / 6 已知缺口 + 5 角色签字栏） | 2026-09-11 20:43 JST ask_user 选项 form_opt2 |
