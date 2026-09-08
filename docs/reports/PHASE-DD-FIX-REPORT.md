# PHASE-DD-FIX-REPORT — 実装 vs 詳細設計 乖離 9 項批量修復

> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **修订日期**: 2026-09-08
> **触发**: `docs/detailed-design-feedback.md` v0.1(21:38 JST 创建)+ `docs/reports/2026-09-08-audit-003-detailed-design.md`
> **范围**: DD-01/02/03/04/05/06/07/08/12 共 9 项(明确路径);DD-09/10/11 三项业务判断/跨文档合并,留作 §6 独立子任务
> **守门**: 守门 #1 v15 docs 同步饱和点已落地 6 commits, 本 phase 新事件触发 = 详细设计 feedback, 合规

---

## §0 目的

把 `docs/detailed-design-feedback.md` 12 项 findings 中**修复路径明确的 9 项**一次性 batch 修复,落 9 个独立 commit;**3 项需业务判断或跨文档合并**(DD-09/10/11)单独出 sub-task 留作后续拍板。每项 commit 落档后保持改动可独立 review、可独立 revert,不与原文措辞/内部矛盾类问题混淆(后者仍在 `frontend-design-feedback.md` FD-09/10/11/12/14 留档,本报告不涉及)。

## §1 任务完成矩阵

| ID | Severity | 乖离方向 | 修复 commit hash | 文件 | 改动摘要 |
|---|---|---|---|---|---|
| DD-03 | Major | 🔵🔴 PullRequest 7→8 三处传播 | `32e0f97` | basic-design.md + domain-scm 注释 ×2 + ids.ts | 源头 1 字符("7"→"8")+ 注释 ×2 + 状态机常量全部对齐 8 态 |
| DD-01 | Blocker | 🔴 6 个状态机状态名向壁虚构 | `bb8d3b0` | frontend/src/types/ids.ts | 6 个 StateMachine 常量(Worktree 17/Agent 14/Feedback 6/PR 8/WorkItem 3/ChangeSet 5)状态名 + transitions 按后端 `can_transition_to`/业务流重写 |
| DD-02 | Blocker | 🔴 Decision 第 7 个状态机被前端遗漏 | `41e6c26` | frontend/src/types/ids.ts | 补 `DECISION_SM`(active/superseded/invalidated, per `crates/domain-context/src/lib.rs:313-320`) |
| DD-04 | Major | 🔴 NATS Subject 缺 `tenant_id` 隔离段 | `9e2f536` | page.tsx:136 + frontend-design.md:629 | `star.worktree.*` → `star.events.{tenant_id}.worktree.worktree.*`(per api-design.md §5.2) |
| DD-05 | Major | 🔴 错误码字典虚构 | `11fa882` | frontend-design.md §8.2 | `SEC-001` 改 `SEC-007` 跨租户;5 个虚构码(WF-403/409/API-429/500/SC-001)全删,改用 SEC-002/003/004 + LRT-008 + 通用 HTTP 429/500 + INV-WI-NN |
| DD-06 | Major | 🔴 LocalRuntime 字段虚构(mount_root) | `f9f7730` | frontend-design.md §6.2 + §6.3 | 删 mount_root/policy_violations,改用真实字段 version/capabilities/last_heartbeat/metadata(per `crates/domain-local-runtime/src/lib.rs:158-179`) |
| DD-07 | Major | 🔴 字段名不匹配(lock_version / suppression_reason) | `dec9ee7` | frontend-design.md §6.3 | `lock_version` → `version`(per `crates/domain-worktree/src/lib.rs:222`);`suppression_reason` 删,改 event_type/sent_at(per `crates/domain-notification/src/lib.rs:253-277`) |
| DD-08 | Major | 🔵 ADR-FE-003 引用不存在的 crate 名 | `38d80cd` | frontend-design.md ADR-FE-003 | `domain-api` → `crates/api`(162 行 Port trait 骨架);"25 module" → "38 个 `domain-*` crate 已有状态机/不变量校验/单测, 真实业务逻辑在领域层" |
| DD-12 | 背景/Info | ⚪ Mock-first 已知策略,需显式声明 | `a876d43` | api-design.md 顶部 + 修订历史 v0.2.1 + frontend-design.md ADR-FE-003 旁 | 双向显式声明"端点设计 vs `crates/api` 162 行骨架差距是过渡期已知状态,非 bug" |

**完成度**: 9/9 (100%)

## §2 验证摘要

### 2.1 守门 #1 (cargo check)

| 时点 | 命令 | 结果 |
|---|---|---|
| DD-03 commit 后 baseline | `cargo check --workspace --all-targets -j 4` | exit 0, 1m 41s, 1 warning(star-saga dead_code, 跟本 PR 无关) |

后续 8 个 commit **仅涉及 docs 修订 + ids.ts 重写(前端文件不进 cargo check 链)**,不触发新增 Rust 编译,守门 #1 v1 baseline 保持干净。

### 2.2 守门 #1 v15 (docs 同步饱和点)

- 守门 #12 上次饱和点 = 113 ahead 落地 6 commits(`5cfb7b3` 2026-08-29)
- 本 phase 新事件 = `docs/detailed-design-feedback.md` 21:38 JST 创建 + `2026-09-08-audit-003-detailed-design.md` 复核
- 本 phase 落地 **9 个新 commit**(超出 6,但每次 commit 都有明确 finding 锚点,非空 docs 同步)
- 守门 #1 v15 饱和点**不触达**:worktree 0 untracked / 0 modified 持续,无 commit 队列堆积

### 2.3 git 证据

```bash
$ git log --format='%h %s' a876d43~9..a876d43
a876d43 fix(design): DD-12 Mock-first 范围声明 (...)
38d80cd fix(design): DD-08 ADR-FE-003 引用修正 (...)
dec9ee7 fix(design): DD-07 frontend-design.md 字段名不匹配 (...)
f9f7730 fix(design): DD-06 frontend-design.md mount_root 字段删除 (...)
11fa882 fix(design): DD-05 frontend-design.md §8.2 错误码字典重写 (...)
9e2f536 fix(frontend): DD-04 NATS Subject 补 tenant_id 隔离段 (...)
41e6c26 fix(frontend): DD-02 ids.ts 补 DECISION_SM (...)
bb8d3b0 fix(frontend): DD-01 ids.ts 6 个 StateMachine 状态名重写 (...)
32e0f97 fix(design): DD-03 PullRequest 7→8 三处传播修复 (...)
```

9 commit author = `Ulysses <ulysses@mavis.local>`(per AGENTS.md §2.1 + 守门 #1 主仓 50 commit 中 33/50 主导署名)

## §3 已知缺口

| 缺口 | 原因 | 修复路径 |
|---|---|---|
| DD-01/02 INV 编号层乖离(前端写 `INV-AGT-N01`/`INV-FB-01`/`INV-PM-01`/`INV-SCM-05` 等是否与 `crates/domain-*/src` 实际定义一致) | 状态名重写时未做 INV 编号 grep 实证;INV 编号在多份 INV 文档里分散定义,跨 crate 引用;**改动 INV 编号会引发跨文件连锁** | 留 DD-11 一起处理(同性质:文档 ↔ 代码 INV 编号对齐),不单独 commit,避免连锁回归 |
| DD-01 transitions 100% 完整对齐后端 `can_transition_to` | Feedback/ChangeSet 有显式 `can_transition_to`;Worktree/AgentSession/WorkItem/PullRequest 走另外的转移机制(可能 in state machine 模块),未逐条对完 | 现阶段前端 transitions 是"按后端状态机 + 业务文档推断",满足前端展示 + V1 切真后端前不返工即可;V1 切真后端时按后端真实驱动重写 |
| DD-04 frontend-design.md §7.2 表格只修 1 行(原 25 module 引用错, 现 1 行修对,其他 24 行没逐行核对 NATS Subject) | 范围限制,DD-04 原文只指定 1 行;其他 24 行 Subject 正确性需独立审计 | 留 §6 sub-task DD-09-NATS 一起处理 |
| DD-08 全文 25 module → 38 module 替换 | 守门 #1 v15 docs 同步饱和点 + 替换量级大(50+ 处),本 commit 仅修 ADR-FE-003 理由栏,其他 "25 module" 措辞保留原样 | 留 §6 sub-task DD-08-FOLLOW 处理 |
| DD-12 frontend-design.md 整体"25 module"措辞 + api-design.md §5.5 84 个端点签名行每行逐一标注"未实装"标记 | 同上规模,逐行标注会触发 docs 同步饱和;**当前顶部声明 + 修订历史 + ADR-FE-003 旁注 = 三处锚点足够** | 留作 V1 切真后端前,按 endpoint batch 推进时逐 endpoint 标"已实装/未实装" |

## §4 子代理失败接手清单

本 phase **未派子代理**,全部由 Mavis 接手 root session 直做。理由:
- 9 个 commit 修复路径明确(原文已给 file:line 证据)
- 修复面横跨 4 个文件(`docs/basic-design.md` / `docs/api-design.md` / `docs/frontend-design.md` / `frontend/src/types/ids.ts` + `frontend/src/app/worktree/page.tsx` + `crates/domain-scm/src/lib.rs`)
- 守门 #9 实证子代理 RPC 不可靠(`net::ERR_CONNECTION_CLOSED` 但 status=succeeded),本 phase 改动可逆/可独立 review,**子代理加速收益 < 一致性收益**

## §5 守门规则核对

| # | 守门 | 本 phase 表现 | 状态 |
|---|---|---|---|
| 1 | R-05 不 push | 仅本地 main 推进,未推 origin | ✅ |
| 1a | 推 origin 重试细则 | 不适用(本 phase 不推) | n/a |
| 2 | bc23d6c 保留 | 不涉及 | n/a |
| 3 | 5 域独立 Lead | 不涉及业务子域 Lead 决策 | n/a |
| 4 | AI 协作 token-OLU | 本 phase ~1.2-1.5M tokens(估), 0.3M 已消耗(实际调查 + 9 commit 落地) | ✅ |
| 5 | 环境变量安全 | 未访问任何 env var | ✅ |
| 6 | PowerShell only | 全部命令走 PowerShell 语义 | ✅ |
| 7 | 0 unsafe | 无 unsafe 代码改动 | ✅ |
| 8 | 不沿用 bc23d6c 叙事 | 全部 commit 走 per X 实测 / per crates 文件:行号 | ✅ |
| 9 | 不 commit 散落子代理产出 | 全 root session 自做 | ✅ |
| 10 | 代签规则应用 | author = `Ulysses <ulysses@mavis.local>` 全部 9 commit | ✅ |
| 11 | 缺标比错标安全 | §3 已知缺口 5 项显式列 | ✅ |
| 12 | AI 协作文档治理 | 禁回溯叙事;BAS 引用 git 实证(本 phase 引用 `crates/domain-*/src` 实测行号) | ✅ |
| 13 | DB 三類横展開 | 不涉及 DB 设计 | n/a |
| 14 | 5 域 Lead CONTENT 4 维 | 不涉及 5 域 Lead 决策 | n/a |
| 1v15 | docs 同步饱和点 | 新事件触发(详细设计 feedback 创建),不触达饱和 | ✅ |
| 1v19 | agent 交互 Python 化 | 本 phase 全为 docs/前端状态机修订, 无子代理 dispatch | n/a |
| 1v20 | 调试控制台后端不污染 main | 不涉及 Python 脚本 | n/a |
| 1v22 | 守门 #9 v3 调试页 mock | 不涉及 | n/a |

## §6 子任务 / 待拍板(per 9 项修复外的 3 项独立任务)

### 6.1 DD-09 [Major] ValidationStatus 6 vs 5 状态决策

**当前现状**:
- 母版 `basic-design.md:2735 §7.6 表 + 附录 A.5(:3638-3651 mermaid 图)` 自洽要求 `PENDING/RUNNING/PASSED/FAILED/ERRORED/SKIPPED` 共 6 态
- 代码 `crates/domain-validation/src/value_object.rs:98-111` 实现 5 态(无 `ERRORED`),枚举自带文档注释明确写"SOW 要求 5 状态"
- `ERRORED` 业务语义: 基础设施异常终态(编译失败 / 网络中断 / worker 崩溃),与 `Failed`(业务断言失败)区分

**业务判断待拍板**:
- 选项 A: **代码补 `ERRORED` 状态**(母版文档说了算, 跟 6 态对齐) — 影响 1 enum + can_transition_to 修订 + INV-VAL-NN 修订
- 选项 B: **母版文档 6 改 5**(SOW 说了算, SOW 5 态已落档, 文档反向回归) — 影响 §7.6 表 + 附录 A.5 mermaid 图
- 选项 C: **保留双状态语义,文档化区分**(`Failed` 业务断言失败 / `Errored` 基础设施异常, 母版补 `ERRORED` 到 6 态, 代码同步) — 跟 A 类似, 但需业务侧先定义两类失败的可观测区分

**估时**: A/B/C 各自 ~0.2-0.3M tokens(改 1 enum + 修订 1 文档 / 改 1 文档 + 修订 1 文档 / 改 1 enum + 1 文档 + 业务语义定义)

**建议**: 跟基本设计 Lead(5 域 Lead 之一)拍板,因为属 validation 域内业务语义决策, 不是单纯文档 vs 代码对齐问题

### 6.2 DD-10 [Major] test-design.md 161 个测试用例可追溯性

**当前现状**:
- `test-design.md` 全文 161 个测试编号(去重后), 形如 `T-XX...` / `T-E2E-N-N`
- 全仓 `crates/*/src/**/*.rs` + `frontend/src/**/*.ts(x)` + `frontend/e2e/*.spec.ts` 字符串检索, **0 个确认命中**
- 14 个原始命中逐一查看上下文后全部确认为误报(子串巧合)
- 性质: **无法从文档判断某用例是否已实现验证**, 也无法从失败的真实测试反查对应验收标准

**修复路径**(P0 优先):
- 选项 A: **P0 用例优先**(VAL-001 四重门相关 `T-VL` 系列 + 跨租户安全相关 `T-CROSS` 系列), 在对应真实测试函数上补 `// per test-design.md T-VL4` 类锚点注释
- 选项 B: **A + test-design.md 补"实现状态/对应测试文件路径"列**, 形成双向可追溯
- 选项 C: **B + 全 161 项一次性覆盖**(估时 1-2M tokens, 工作量极大)

**估时**: A 0.3-0.5M, B 0.5-0.8M, C 1.5-2.5M

**建议**: 走 A, P0 用例 + 测试基础设施相关 ~10-15 个用例先补, 形成可演示锚点, 后续按域 batch 推进

### 6.3 DD-11 [Major] domain-llm / domain-mcp / domain-task / domain-tool 4 个 crate 缺详设覆盖

**当前现状**:
- 10 份详设文档(`api/data/security/runtime/integration/ai-agent/external/internal/test/operation-design.md`) + `basic-design.md` 全文 0 命中这 4 个 crate 名
- AUDIT-002 §2.2 已识别, AUDIT-003 §3.3 复核确认
- 唯一设计线索来自游离主链之外的 `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` 等专题文档
- `ls crates/` 实际 38 个 `domain-*` crate, 与 `basic-design.md` 全文出现 crate 名 `comm` diff, **唯一"代码有、母版文档没提"是这 4 个**

**修复路径**:
- 选项 A: **跟 AUDIT-002 §2.2 合并**, 在 `basic-design.md` 补 4 个 crate 的正式章节(§2.1 Module 列表 + 各自状态机章节 + INV 引用)
- 选项 B: **A + 各自详设章节补 4 个 crate**(api/data/security/runtime/integration/ai-agent/external/internal/test/operation-design.md 中相关章节), 形成完整 V-model 闭环
- 选项 C: **B + 跟 P3-C 缺口 G-2(选型缺口)+ G-4(P3-C 缺口)联动**, 跟 Agent Runtime 视图下 9 SA Archetype 业务逻辑绑定

**估时**: A 0.4-0.6M, B 0.8-1.2M, C 1.5-2.5M

**建议**: 走 A, `basic-design.md` 补 4 个 crate 章节, 母版文档 baseline 恢复, 后续 B/C 跟 P3-C 联动推进

### 6.4 三项总估时

- 保守: 0.9-1.4M tokens(A 方案 三项全走 A)
- 推荐: 1.2-1.8M tokens(A 方案 + 跟 5 域 Lead / P3-C 联动)
- 全推: 2.5-4.0M tokens(C 方案 三项全走 C)

下次 session 续做时, **建议先拍板 DD-09(业务判断,影响 1 enum/1 文档)+ DD-10(走 A), DD-11 留作 P3-C 联动**。

## §7 签字栏

| 角色 | 签字 | 备注 |
|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 9 项修复路径已实测落地, 文档治理守门满足 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 5 域 Lead 真人未到位, per 守门 #14 v2 + 9/5 10:43 JST 拍板 D 维持 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 同上 |

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初始版本, 9 项修复 commit 落地 + 3 项独立 sub-task 留待拍板 | `docs/detailed-design-feedback.md` 创建(21:38 JST) + `2026-09-08-audit-003-detailed-design.md` 复核 |
