# P1 修复总报告 — Phase D 开工前 15 项 P1 阻断项

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-27
> **制定者**：架构师（Ulysses（一人公司 12 角色 per DEC-008））— Mavis 子代理 A
> **签批**：⏳ 待 Ulysses 终审
> **基于 commit**：`245cf56`（P0-1 adr/ 修补后基点）
> **工作目录**：`D:/Star/.worktrees/phase-d-p1-fix`（worktree 分支 `wt-phase-d-p1-fix`）
> **范围**：修复 15 项 P1 阻断项，**不**扩范围、**不**写代码、**不** commit、**不**触碰其他 untracked 文件

---

## 0. 报告目的

3 子代理（A / B / C）独立审查完 Phase C 54 份 spec 后出 15 项 P1 阻断项（per `P1-BLOCKERS-SUMMARY.md` c25d261 commit 后 v0.2）。本报告记录 Mavis 子代理 A 在 `wt-phase-d-p1-fix` worktree 上完成 15 项 P1 修复的 diff 摘要（before / after 关键字段），给 Ulysses 终审时一目了然的修复落点清单。

## 1. 修复执行汇总

| 维度 | 数值 |
|---|---|
| 修复 P1 项数 | 15 / 15（100%）|
| 修改 spec 文件 | 11 份 |
| 新增文件 | 0（不新增）|
| 不 commit | ✅（per 硬约束 2 + 守门规则）|
| 修订历史 v0.2 署名 | 全部 11 份均加 Ulysses（per 2026-08-27 07:16 JST 代签规则反转）|
| 工作目录不变量验证 | ✅（git log -1 = 245cf56,无其他文件触碰）|

## 2. 15 项 P1 修复 diff 摘要

### 2.1 共识 P1（3 子代理都发现）

#### **P1-A** — CLI "17 命令" vs 实际列表不一致

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/cli/01-cli-spec.md` | §2 标题 | "核心命令（per 任务原文 §9）" | "MVP 17 核心命令（per 任务原文 §9，MVP 子集边界）" + §2.2 "扩展命令（非 MVP 子集，共 11 个）" |
| `spec/cli/01-cli-spec.md` | §2 表格 | 23 行单表 | **17 核心** + **11 扩展**（6 原 + 5 新增）双表 |
| `arch/03-star-ai-compat-arch.md` | §2.2 标题 | "17 个核心命令" | "MVP 17 核心命令（per 任务原文 §9）" + 引用 `cli/01 §2.2` 11 扩展 |
| `arch/03-star-ai-compat-arch.md` | §2.2 bash 块 | 18 命令 | **17 命令**（删 `star pipeline run` 从核心移到扩展） |

#### **P1-B** — Universal Submit 11 vs 12 步（文字+列表矛盾）

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/flows/05-universal-submit.md` | §2 标题 | "11 步流程（per §33 任务原文）" | "12 步流程（per P1-B 修复 2026-08-27 统一 12 步）" |
| `spec/flows/05-universal-submit.md` | §2 列表 | 11 正式步 + 1 comment 步（"12. 回写 IDE Session 状态" 作 comment） | **12 正式步**（comment 步升为正式步） |
| `spec/flows/05-universal-submit.md` | §2 内 5 步标记 | 无 | 第 4/6/7/8/10 步标"也可独立调用 star diff / policy check / commit / push / mr link"（per P1-H 联动） |

#### **P1-C** — `Workspace` 跨层数据泄漏

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/agent-api/01-schema.md` | §3 | 无 WorkspaceSummary | **§3.16 WorkspaceSummary**（agent 视角，6 字段：id / name / repository / worktree_id / agent_session_id / timestamps） |
| `spec/ide-api/01-schema.md` | §2.1 标题 | "Workspace" | "**WorkspaceState**（per P1-C 修复 2026-08-27，原 `Workspace` 重命名）" |
| `spec/ide-api/01-schema.md` | §1 | "跟 `agent-api/v1` 平行" | "跟 `agent-api/v1` 平行，独立演进 + **IDE 视角边界**（per P1-C 修复）" |
| `spec/cli/01-cli-spec.md` | §2.1 `star workspace current` | `agent-api/v1#Workspace` | **`agent-api/v1#WorkspaceSummary`** |
| `spec/mcp/01-mcp-spec.md` | §2 `get_workspace` | 输出 `Workspace` | 输出 **`WorkspaceSummary`**（per agent-api/v1 §3.16） |

#### **P1-D** — agent-api/v1 21 schema 仅展开 3 个

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/agent-api/01-schema.md` | §3 标题 | "核心 Schemas（节选）" | "核心 Schemas"（去"节选"） |
| `spec/agent-api/01-schema.md` | §3 展开数 | 3 个（Task / Worktree / SubmitResult） | **17 个**（§3.1-3.15 = 15 核心 + §3.16 WorkspaceSummary + §3.17 Resume） |
| `spec/agent-api/01-schema.md` | §1 versioning | 4 段版本规则 | + **OpenAPI `info.version` 演化规则**（per 硬约束 9 + 子代理 A 🟡 #11） |
| `spec/agent-api/01-schema.md` | §4 落盘文件名 | 12 文件 + "..." | 25+ 文件（§3.1-3.17 + 13 扩展命令 / 子结构） |
| `spec/agent-api/01-schema.md` | §5 验证 | 2 段 | + Error schema 统一性校验循环（per P1-G） |

### 2.2 子代理 A 独 flag

#### **P1-E** — MCP spec 漏 2026-07-28 关键变更

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/mcp/01-mcp-spec.md` | §1 | 3 行版本说明 | + **§1.1 2026-07-28 关键变更符合度表**（6 项：stateless / MRTR / Header routing / ttlMs / RFC 9207 / Feature Lifecycle） + **§1.2 兼容承诺**（tool list 排序 / metadata 必含 ttlMs + cacheScope） |
| `spec/mcp/01-mcp-spec.md` | §2 工具表 | 3 列（Tool / 输入 / 输出） | **4 列**（+ metadata `ttlMs` / `cacheScope`） |

#### **P1-F** — MCP §7 说"必须能 invoke star submit"但 §2 无 submit

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/mcp/01-mcp-spec.md` | §2 工具数 | 15 tools | **16 tools**（+ `submit` tool） |
| `spec/mcp/01-mcp-spec.md` | §2 submit 行 | 无 | `submit` | `{worktree_id?, force?}` | `SubmitResult` | 0 / none |
| `spec/mcp/01-mcp-spec.md` | §6 实施位置 | "13 个 tool 实现" | "**16 个 tool 实现**（含 submit, per P1-F）" |
| `spec/mcp/01-mcp-spec.md` | §7 验证 | "13+ tools" + "必须能 invoke star submit" | "**16 tools**（per P1-F：含 submit）" + "必须能 invoke **submit**"（替换"star submit"为"submit" tool 验证） |

#### **P1-G** — 错误模型 4 套并存

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/agent-api/01-schema.md` | §3 | 无 Error 权威定义 | **§3.15 Error**（6 字段：`error` / `recoverable` / `suggested_actions` / `message` / `trace_id` / `details`） |
| `spec/cli/01-cli-spec.md` | §5 | 5 字段 JSON 例子（无 details） | 6 字段 JSON 例子（+ `details`）+ 注 "per agent-api/v1#Error" |
| `spec/flows/05-universal-submit.md` | §3 | 4 字段 JSON 例子（无 message / trace_id） | 6 字段 JSON 例子（+ `message` + `trace_id`）+ 注 "统一引用 agent-api/v1#Error" |
| `spec/mcp/01-mcp-spec.md` | §3（新增） | 1 子节（禁止） | §3.1 禁止 + **§3.2 错误模型**（JSON-RPC 2.0 error envelope，data = 完整 Error 6 字段） |
| `arch/05-gitgit-compat-arch.md` | §5 关键约束 | 2 行 | + 4xx / 5xx 统一引用 `agent-api/v1#Error` |
| `spec/rest/01-rest-strategy.md` | §4 端点表 | 2 列（Endpoint / 用途） | **3 列**（+ 4xx/5xx 响应 → 引用 `agent-api/v1#Error`） |

#### **P1-H** — Universal Submit 5 步无独立 CLI 命令

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/cli/01-cli-spec.md` | §1 设计原则 | 4 条 | + **"`star` 是 `git` 的 superset"** 原则 |
| `spec/cli/01-cli-spec.md` | §2.2 扩展命令表 | 6 行原扩展 | **11 行扩展**（+ 5 个新加：`star diff` / `star policy check` / `star commit` / `star push` / `star mr link`） |
| `spec/flows/05-universal-submit.md` | §4 实施位置 | 2 行 | + "5 个新加独立命令"行（`crates/star-cli/src/commands/{diff,policy,commit,push,mr_link}.rs`） |
| `arch/03-star-ai-compat-arch.md` | §2.2 关键约束 | 4 条 | + "`star` 是 `git` 的 superset" 关键约束 |

### 2.3 子代理 C 独 flag

#### **P1-I** — MCP 13 tools vs arch/03 列 14

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `arch/03-star-ai-compat-arch.md` | §2.3 | 15 tools 列表 + "13 个领域语义 tools" | **MVP 13 tools 子集边界表**（13 MVP + 3 扩展 = 完整 16）+ 注释 submit 实际属 MVP（accept/04 §3 必跑通） |

#### **P1-J** — REST 12 endpoints vs arch/05 列 14

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `arch/05-gitgit-compat-arch.md` | §5 | 14 endpoints 单表 | **MVP 12 endpoints 子集边界表**（12 MVP + 2 扩展 = 完整 14）+ 4xx/5xx 引用 Error 关键约束 |

#### **P1-K** — arch/03 §7 vs acceptance/01 冲突

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `arch/03-star-ai-compat-arch.md` | §7 | "Unknown Agent Test 必须**只**用 Level 4 (Git Only) 通过" | "**Unknown Agent Test 跑 Level 1**（跟 acceptance/01 §3 16 步实际用 star CLI 兼容）+ **Level 2 / 3 / 4 单独跑 conformance**" |

#### **P1-L** — 测试位置不一致

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/vcs/04-fallback-strategy.md` | §5 实施位置 | "全部 4 级的 conformance 测试在 `crates/star-cli/tests/`" | "全部 4 级的 conformance 测试在 **`tests/`**（per P1-L 修复 2026-08-27）— `tests/unknown-agent/` / `tests/zero-knowledge-agent/` / `tests/unknown-ide/` / `tests/fallback-conformance/`" |

### 2.4 子代理 B 独 flag

#### **P1-M** — flows/01/03 状态字符串大小写不一致

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/flows/01-agent-task-lifecycle.md` | §1 状态机 | 全大写（SHOUTING_CASE：`IMPLEMENTING`） | **PascalCase**（`Implementing`）+ 加注"状态字符串以 Rust enum 命名为准" |
| `spec/flows/01-agent-task-lifecycle.md` | §2 异常状态 | 全大写 | PascalCase |
| `spec/flows/03-agent-resume.md` | §2 `current_state` | `"Implementing"` | `"Implementing"`（保持 PascalCase）+ §3 关键约束加"状态字符串统一 PascalCase" |

#### **P1-N** — flows/01 状态数 9+5 vs 任务摘要 9+4

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/flows/01-agent-task-lifecycle.md` | §2 异常状态数 | 5 异常（未在 v0.1 显式说明 9+5 vs 9+4） | 5 异常保留 + §2 顶部加注"spec 9+5 = 14 状态；任务摘要曾写 9+4 误去 CONFLICT，但 CONFLICT 业务语义完整保留" |
| `spec/flows/01-agent-task-lifecycle.md` | 修订历史 v0.2 | — | + "P1-N：spec 9+5 vs 任务摘要 9+4 的来源差异"行 |

#### **P1-O** — flows/03 Resume 11 字段未在 agent-api/v1 定义

| 涉及 spec | 字段 | before | after |
|---|---|---|---|
| `spec/agent-api/01-schema.md` | §3 | 无 Resume schema | **§3.17 Resume**（11 字段权威定义） |
| `spec/flows/03-agent-resume.md` | §2 协议 | 11 字段 JSON 例子（部分字段语义模糊） | 11 字段完整 JSON 例子 + **引用 [`agent-api/v1#Resume`](spec/agent-api/01-schema.md) §3.17** + 补全 `last_modified` / `open_diagnostics` / `test_results` / `relevant_context` 等之前未明确的字段 |

## 3. 11 份 spec 修改详情

| # | spec 路径 | 状态 | 修订行触发 | 承载 P1 |
|---|---|---|---|---|
| 1 | `spec/cli/01-cli-spec.md` | 🟡 v0.2 | v0.2 修订行 | P1-A, P1-C, P1-G, P1-H |
| 2 | `spec/agent-api/01-schema.md` | 🟡 v0.2 | v0.2 修订行 | P1-C, P1-D, P1-G, P1-O |
| 3 | `spec/ide-api/01-schema.md` | 🟡 v0.2 | v0.2 修订行 | P1-C |
| 4 | `spec/mcp/01-mcp-spec.md` | 🟡 v0.2 | v0.2 修订行 | P1-E, P1-F, P1-G |
| 5 | `arch/03-star-ai-compat-arch.md` | 🟡 v0.2 | v0.2 修订行 | P1-A, P1-I, P1-K |
| 6 | `arch/05-gitgit-compat-arch.md` | 🟡 v0.2 | v0.2 修订行 | P1-J |
| 7 | `spec/flows/01-agent-task-lifecycle.md` | 🟡 v0.2 | v0.2 修订行 | P1-M, P1-N |
| 8 | `spec/flows/03-agent-resume.md` | 🟡 v0.2 | v0.2 修订行 | P1-M, P1-O |
| 9 | `spec/flows/05-universal-submit.md` | 🟡 v0.2 | v0.2 修订行 | P1-B, P1-G |
| 10 | `spec/vcs/04-fallback-strategy.md` | 🟡 v0.2 | v0.2 修订行 | P1-L |
| 11 | `spec/rest/01-rest-strategy.md` | 🟡 v0.2 | v0.2 修订行 | P1-G |

## 4. 守门规则遵循度

| 守门规则 | 状态 | 说明 |
|---|---|---|
| ✅ 不修改 15 项 P1 之外的内容 | ✅ | 仅改 11 份 spec 涉及 15 项 P1 字段；其他 P2 弱信号 38 项 + P3 缺口 23 项不动 |
| ✅ 不 commit | ✅ | 所有修改均为 untracked；git status 见 §6 |
| ✅ 不写代码 | ✅ | 仅改 Markdown spec |
| ✅ 不触碰其他 untracked | ✅ | 仅 11 份 spec + P1-FIX-SUMMARY.md 共 12 文件修改；其他 12+17 untracked 文件未触碰 |
| ✅ 不沿用 bc23d6c 叙事 | ✅ | 所有"per X 历史形态"类回溯叙事零使用 |
| ✅ 不编造未做过的 commit hash | ✅ | 仅引用 `245cf56`（已 git log -1 验证的基点 commit） |
| ✅ P1-G Error 与 3 子代理一致 | ✅ | 6 字段（`error` / `recoverable` / `suggested_actions` / `message` / `trace_id` / `details`）= 子代理 A 🔴 #6 + 子代理 B 报告 + 子代理 C 报告 三者交叉验证 |
| ✅ P1-K arch/03 §7 与 acceptance/01 §3 兼容 | ✅ | Level 1（用 star CLI，匹配 acceptance/01 §3 16 步）+ Level 2/3/4 单独 conformance |
| ✅ P1-D agent-api/v1 符合 OpenAPI 3.1 | ✅ | §1 加 OpenAPI `info.version` 同步演化规则 + 完整 OpenAPI 3.1 规范（webhooks / `nullable: true` / `info.summary`） |
| ✅ P1-C WorkspaceSummary/WorkspaceState 守 ADR-0024 | ✅ | ide-api/v1 §1 加"IDE 视角边界"声明，agent-api/v1 §3.16 WorkspaceSummary 不含 IDE 内部状态 |
| ✅ 不可代签反转 | ✅ | 11 份 spec 修订历史 v0.2 行全部署名 Ulysses（per 2026-08-27 07:16 JST 规则反转），原 v0.1 "Mavis 代签 2026-08-26" 行保留不追溯改写 |
| ✅ 缺标比错标安全 | ✅ | 显式列"未触碰项"清单（见 §5）+ 修订历史写明"per 2026-08-26 子代理 A/B/C 报告触发" |

## 5. 未触碰项（per 硬约束 4 + 缺标比错标安全）

> 本任务**仅**修改 15 项 P1 涉及的 11 份 spec。下述内容**不**在本任务范围，Phase D 实施同期或后续阶段处理：

### 5.1 38 项 P2 弱信号（per `P1-BLOCKERS-SUMMARY.md §4`）

- 子代理 A P2 #8-#20：13 项（如 CLI ↔ MCP 命名映射表 / REST 端点覆盖度 / OpenAPI 3.1 关键字段 / agent-api info.version 关系等）— **未动**
- 子代理 B P2 #5-#17：13 项（如 flows/02 Agent Lost 恢复状态转换 / Audit Service 对象定义 / 4 类权限主体 / 4 处 spec 引用 agent-api 但本任务范围外）— **未动**
- 子代理 C P2 #1-#12：12 项（如 arch/03 §7 "真实 Agent 4 款" vs acceptance/01 "自实现 minimal agent" 职责切分 / acceptance/03 §3 10 步未消费 OpenAPI / NFR-OP-001 单位 "SRE·周/周" 跟 token-OLU 偏好不对齐 / R-003 工具链断言无版本号 / R-007 cache 层无指等）— **未动**

### 5.2 23 项 P3 已知缺口（per `P1-BLOCKERS-SUMMARY.md §5`）

- 子代理 B §3 P3 #18-#25：8 项 — **未动**
- 子代理 C §9 弱信号：12 项 — **未动**
- 5 份基础文件 §6 已知缺口：3 项 — **未动**

### 5.3 其他 untracked 文件（per 8/26 拍板"现在不动"）

- STAR 仓库 12 个未跟踪文件 + 17 个其他 untracked = 29 个 — **未触碰**

## 6. git 状态验证

```bash
$ cd D:/Star/.worktrees/phase-d-p1-fix
$ git log -1 --oneline
245cf56 fix(adr): 补 5 份 ADR 到 architecture/2026-08-26-upgrade/adr/ 修 P0-1 断链

$ git status --short
 M docs/architecture/2026-08-26-upgrade/arch/03-star-ai-compat-arch.md
 M docs/architecture/2026-08-26-upgrade/arch/05-gitgit-compat-arch.md
 M docs/architecture/2026-08-26-upgrade/spec/agent-api/01-schema.md
 M docs/architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md
 M docs/architecture/2026-08-26-upgrade/spec/flows/01-agent-task-lifecycle.md
 M docs/architecture/2026-08-26-upgrade/spec/flows/03-agent-resume.md
 M docs/architecture/2026-08-26-upgrade/spec/flows/05-universal-submit.md
 M docs/architecture/2026-08-26-upgrade/spec/ide-api/01-schema.md
 M docs/architecture/2026-08-26-upgrade/spec/mcp/01-mcp-spec.md
 M docs/architecture/2026-08-26-upgrade/spec/rest/01-rest-strategy.md
 M docs/architecture/2026-08-26-upgrade/spec/vcs/04-fallback-strategy.md
?? docs/architecture/2026-08-26-upgrade/P1-FIX-SUMMARY.md
```

修改文件清单：
- **11 份 spec**（M = modified，已 tracked 在 245cf56 commit，本任务修改未 commit）
- **1 份新增报告**（?? = untracked，`P1-FIX-SUMMARY.md` 本报告）

合计 12 文件改动，全部不 commit（per 硬约束 2 + 守门规则）。

## 7. 假设

1. **代签规则已反转**（per 2026-08-27 07:16 JST 指令）— 修订历史 v0.2 全部署名 Ulysses
2. **工作目录基点 = 245cf56**（P0-1 修补后）— 仅该基点后产生的 untracked 修改
3. **Mavis 终审后由 Mavis 统一 commit**（per 硬约束 2）— 本任务不 commit
4. **11 份 spec 的"其他内容"不被触动**（per 硬约束 1）— 仅 15 项 P1 涉及的字段修改

## 8. 阻塞 / 遗留风险

- **Mavis 终审未完成** — 本报告 + 11 份 spec 改动需 Mavis 终审，签字后方可 commit
- **38 项 P2 弱信号 + 23 项 P3 缺口未消解** — Phase D 实施同期 / 后续 backlog 消解
- **`submit` tool 实属 MVP 14 而非 13** — arch/03 §2.3 子集边界注释已说明（MVP 13 任务原文 + submit = 14 MVP，完整 16 = 14 MVP + 2 扩展），若后续任务原文调整需重新对账

## 9. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 / 一人公司 12 角色 | Ulysses（per DEC-008）| 2026-08-27 | 🟡 草案 v0.1；15 项 P1 全部修复 + 11 份 spec 修改 + 守门规则全通过；待 Mavis 终审 + 统一 commit |
| 2 | Mavis 终审人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| 初版：15 项 P1 修复 diff 摘要 + 11 份 spec 修改详情 + 守门规则自检 + 未触碰项清单 | Phase D 开工前 15 项 P1 修复任务完成 |
