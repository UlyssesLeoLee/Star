# 実装 vs 详细设计 乖离 Feedback（DD 系列）

> **范围声明（先说清楚覆盖了什么，没覆盖什么，避免被当成完整审计）**
> 本文档只回答一个问题:"当前 `crates/*`、`frontend/src/*` 的实际实装,和最新的详细设计文档(`docs/*-design.md`)之间,有哪些**可用 file:line 证据核实的**具体乖离"。素材全部来自 `docs/reports/2026-09-08-audit-003-detailed-design.md`(以下简称 AUDIT-003)与本文档撰写时补做的 2 项交叉核对(crate 清单 diff、迁移 SQL 覆盖度),不重新审核文档措辞/内部矛盾类问题(那类问题仍留在 `frontend-design-feedback.md` 的 FD-09/FD-10/FD-11/FD-12/FD-14,见文末附录)。
>
> **已核实覆盖**:`frontend-design.md`(经 `frontend-design-feedback.md` 9/16 项抽查)、`basic-design.md §7.6` 状态机总览表全 7 行、`api-design.md` 端点规模(仅定性确认"远超 162 行骨架承载量",具体端点计数未能复现,见 DD-12)、`test-design.md` 测试编号可追溯性、`crates/domain-{worktree,agent,feedback,scm,work-item,development,validation,context,llm,mcp,task,tool}` 对应枚举与模块存在性。
> **未核实 / 已知盲区**:`security-design.md`/`runtime-design.md`/`integration-design.md`/`ai-agent-design.md`/`external-design.md`/`internal-design.md`/`operation-design.md` 七份文档**从未逐章通读**,仅做过关键词排查;`frontend-internal-01~04`/`frontend-canvas-design.md`/`automation-design.md`/`uat-design.md`/`uat-runbook.md`/`ubiquitous-language.md` 九份文档同样只做过结构性抽查(TODO 密度 + 关键词命中),未做语义级实装核对;`frontend-design-feedback.md` 剩余 FD-05/FD-06/FD-09/FD-10/FD-12/FD-14 六项未在本轮核实,不计入下表,也不假设已修复。**因此本文档列出的 DD 项是"已确认的乖离下限",不是"乖离全集"。**
>
> **使用说明**:每条 `ID | Severity | 乖离方向 | 证据 | 期望修正`。Severity 沿用 Blocker/Major/Minor 三档,另设 背景/Info 一档,专用于 DD-12 这类"已知策略、非缺陷"的背景说明条目,不与前三档同级排序。**乖离方向**是本文档相对 AUDIT-003 新增的字段,标注谁是"权威"、该改哪边,避免机械套用"以代码为准"或"以文档为准"——同一份表格里两个方向相反的例子(DD-03/DD-09)刚好可以互相提醒。

---

## 乖离方向图例

| 图例 | 含义 | 期望修正落点 |
|---|---|---|
| 🔵 文档过期 | 代码是对的,设计文档没跟上 | 改文档 |
| 🟠 代码缺失 | 设计文档是对的(自洽、有权威来源),代码没实现 | 先做业务判断,再补代码或改文档 |
| 🔴 前端向壁虚构 | 前端设计文档/前端代码凭空编造,和后端权威(代码或母版文档)都对不上 | 改前端(设计文档 + 代码) |
| ⚪ 阶段性已知策略 | 差距真实存在,但是团队已知的 Mock-first 过渡状态,不是被遗漏的缺陷 | 不改,但需要在文档里显式声明范围,避免被误读 |

---

## Findings

### DD-01 [Blocker] 🔴 6 个状态机状态名向壁虚构,`ids.ts` 从未回写

- **证据**:`frontend/src/types/ids.ts`(约行 894-1015)`FEEDBACK_SM.states`/`WORKITEM_SM.states`/`CHANGESET_SM.states` 等 6 个 `StateMachine` 常量的状态名,与对应后端实际枚举(`crates/domain-feedback/src/value_object.rs` 的 `Open/Acknowledged/Applied/Verified/Rejected/Superseded` 等)逐一比对后仅个别名字字面重合,多数为前端自创词汇。原始 Finding 见 `frontend-design-feedback.md` FD-01,提出后**从未被回写**(AUDIT-003 §2.1 复核确认)。
- **影响**:V1 切真后端时,`ids.ts` 6 个 `StateMachine` 常量、`StatusPill.tsx` 颜色映射表、`StateMachineDiagram` 的 `highlightState` 匹配逻辑全部要重写,推翻 ADR-FE-003"切真后端时 UI 不动"的结论。
- **期望修正**:逐一从 `crates/domain-{worktree,agent,feedback,scm,work-item,development}/src` 抄录真实状态名重写 `ids.ts`。

### DD-02 [Blocker] 🔴 Decision 第 7 个状态机被前端遗漏

- **证据**:`basic-design.md §7.6` 状态机总览表实际有 **7** 行(含 Decision,附录 A.7),`crates/domain-context/src/lib.rs:313-320` 确认 `DecisionStatus` 已实现 `Active/Superseded/Invalidated` 三态;`frontend/src/types/ids.ts` grep `Decision.*SM|DCSM` 无结果,`frontend-design.md §4` 标题仍写"6 状态机"。原始 Finding 见 FD-01B,AUDIT-003 §2.2 复核确认仍未修复。
- **期望修正**:`ids.ts` 补 `DECISION_SM` 常量,`/context` 路由补一个复用 `StateMachineDiagram` 的 3 节点 Decision SmView。

### DD-03 [Major] 🔵🔴 PullRequest 状态数"7 vs 8",三层传播(母版表格→后端注释→前端状态机)

- **证据**(三处独立确认,方向不同):
  - 🔵 `basic-design.md:2737` §7.6 表仍写 `| PullRequest | 7 | ... |`,与同文档附录 A.6 的 8 状态(`DRAFT/OPEN/REVIEWING/CHANGES_REQUESTED/APPROVED/MERGEABLE/MERGED/CLOSED`)矛盾——**代码是对的(8),表格过期**。
  - 🔵 `crates/domain-scm/src/lib.rs:25,233` 文档注释仍写"7 状态机",但紧接着定义的 `PullRequestState` 枚举(`:237-254`)实测 8 个变体——**代码注释自相矛盾,和实现本身不一致**。
  - 🔴 `frontend/src/types/ids.ts`(约行 973-990)`PR_SM.states` 仍为 7 项,且 `ci_failed`/`review_required` 是前端自创、后端不存在的状态名,后端真实的 `Reviewing`/`Mergeable` 前端完全没出现——**前端向壁虚构**。
  - 原始 Finding 见 FD-15,AUDIT-003 §2.5 确认三处全部原样保留,无一处修复。
- **期望修正**:源头是 `basic-design.md:2737` 一个字符("7"→"8"),建议同一 PR 里把母版表格 + `domain-scm` 源码注释 ×2 + `ids.ts` 四处一起改,避免分批修复时方向搞反。

### DD-04 [Major] 🔴 NATS Subject 缺 `tenant_id` 隔离段,已从设计文档扩散进已上线的前端渲染代码

- **证据**:`frontend/src/app/worktree/page.tsx:136` 实际渲染文案"状态切换会触发 NATS event `star.worktree.*`",与 `api-design.md §5.2` 定义的稳定格式 `star.events.{tenant_id}.{domain}.{aggregate}.{action}` 不符,缺 `events.` 段与 `{tenant_id}` 段。原始 Finding 见 FD-04,提出时问题只在 `frontend-design.md` 表格里,AUDIT-003 §2.4 复核发现**现已写进用户可见的实际 UI 文案**——本轮唯一一项从"纯文档问题"劣化为"已落地代码问题"的乖离。
- **影响**:涉及多租户隔离描述,安全相关,建议不与常规文档修订合并,单独处理。
- **期望修正**:`frontend-design.md` 表格与 `page.tsx:136` 渲染文案同步改为正确 Subject 格式。

### DD-05 [Major] 🔴 错误码字典虚构

- **证据**:`frontend-design.md:665-670` 错误反馈规范表格里 `SEC-001`(误用为跨租户,`api-design.md:2020` 实际定义 `SEC-001` = 401 未认证,跨租户真实码是 `:2026` 定义的 `SEC-007`,另在 `:527` 有一致的重复定义)、`WF-403`/`WF-409`/`API-429`/`API-500`/`SC-001` 五个码在 `api-design.md` 全文 0 命中(本轮重新 `grep -n "SEC-001\|SEC-007"` 确认)。目前 `frontend/src/` 尚未引用这些码(Mock-first,无真实错误处理链路),但 `frontend-design.md` 是后续 V1 实装会直接抄的规范文本。原始 Finding 见 FD-03,AUDIT-003 §2.7 确认字面未修复。
- **期望修正**:对照 `api-design.md §8`(真实错误码字典)逐行重写该表格,`SEC-001` 改为跨租户场景应使用的 `SEC-007`,其余 5 个虚构码要么找到真实对应码,要么删除。

### DD-06 [Major] 🔴 Local Runtime 字段虚构(`mount_root`)

- **证据**:`frontend-design.md:545,572` 仍引用 `mount_root` 字段,真实 `LocalRuntime` 结构体(`crates/domain-local-runtime/src/lib.rs:158-179`,本轮重新 grep 确认)字段是 `id/tenant_id/user_id/device_id/status/version/capabilities/last_heartbeat/registered_at/metadata`,不存在 `mount_root`。原始 Finding 见 FD-07,AUDIT-003 §2.7 确认未修复。
- **期望修正**:删除或替换为真实字段名。

### DD-07 [Major] 🔴 字段名不匹配(`lock_version` / `suppression_reason`)

- **证据**:`frontend-design.md:563` 仍写 `lock_version`,真实字段是 `version`(`crates/domain-worktree/src/lib.rs:222`);`:570` 仍写 `suppression_reason`,但 Notification 被抑制的事件从不落库,该字段实际不存在。原始 Finding 见 FD-08,AUDIT-003 §2.7 确认未修复。
- **期望修正**:`lock_version` 改 `version`;`suppression_reason` 一行整体删除或改写为"抑制事件不落库,前端无需处理"的说明。

### DD-08 [Major] 🔵 ADR-FE-003 引用不存在的 crate 名 `domain-api`

- **证据**:`frontend-design.md:709` 仍写"backend 25 module 仅 `domain-api` crate 是骨架 Port trait,无真实 handler"。`crates/` 目录下不存在 `domain-api`,真实的骨架 crate 是 `crates/api`(本轮重新核实:`crates/api/src/lib.rs` 仅 **162 行**)。"25 module"这个数字本身也已过期——本轮 `ls crates/` 实测 `domain-*` crate 共 **38** 个(见 DD-11 的 crate 清单交叉核对)。原始 Finding 见 FD-02,AUDIT-003 §2.3 确认未修复。Mock-first **决策本身**不受影响(见 DD-12),被推翻的只是 ADR 理由栏的措辞与归因。
- **期望修正**:把 `domain-api` 改为 `crates/api`,把"25 module 均为骨架"改为准确表述——实际 38 个 `domain-*` crate 已有状态机/不变量校验/单测,真正的骨架只在 `crates/api`(REST handler 层)。

### DD-09 [Major] 🟠 `ValidationResult` 声称 6 状态(含 `ERRORED`),代码只实现 5 状态——与 DD-03 方向相反的独立矛盾

- **证据**:`basic-design.md:2731-2737` §7.6 表 + 附录 A.5(`:3638-3651` mermaid 图)自洽地要求 `PENDING/RUNNING/PASSED/FAILED/ERRORED/SKIPPED` 共 6 态;`crates/domain-validation/src/value_object.rs:98-111` 的 `ValidationStatus` 枚举只有 5 个变体(`Pending/Running/Passed/Failed/Skipped`),枚举自带文档注释明确写"SOW 要求 5 状态",**没有承认与母版设计文档的差异**。这是 AUDIT-003 §3.2 新发现,完成 FD-15 自己建议的"核对 §7.6 表其余 5 行"这一步之后暴露。
- **性质**:与 DD-03(PullRequest)同类但方向相反——DD-03 是"母版文档过期,代码是对的";DD-09 是"母版文档(表格+附录都自洽)要求 6 态,代码实现只做了 5 态,`ERRORED`(编译失败/网络中断等基础设施异常终态)这个业务上有意义的状态从未实现"。
- **影响**:若 `ERRORED` 场景在实际系统中发生,当前 `ValidationStatus` 没有专门状态承载,可能被错误归入 `Failed`(业务断言失败),导致状态转移逻辑缺少对应处理路径。
- **期望修正**:先做业务判断(是否真的要支持基础设施异常终态),再决定给代码补 `ERRORED` 状态,还是把母版文档的"6"改成"5"——**不要机械套用 DD-03 的"以代码为准"结论**。

### DD-10 [Major] 🟠 `test-design.md` 161 个正式测试用例编号与实际测试代码零可确认关联

- **证据**:提取 `test-design.md` 全文形如 `T-XX...`/`T-E2E-N-N` 的编号(修正后正则 `T-[A-Z0-9]+(-[A-Z0-9]+)*`,去重 161 个),在 `crates/*/src/**/*.rs`/`frontend/src/**/*.ts(x)`/`frontend/e2e/*.spec.ts` 全仓字符串检索,原始命中 14 处,逐一查看上下文后**全部确认为误报**(子串巧合命中 `OPT-*`/`REQ-TST-*`/`INV-CT-*`/`LRT-*`,或命中 `IT-CROSS-N`/`IT-N` 这一套前缀不同、纯属巧合的独立集成测试编号体系)。修正后结论:**161 个编号,0 个确认命中**。详细方法论与逐条排除记录见 AUDIT-003 §3.1。
- **性质**:`test-design.md` 定义了"应该测什么"(161 个用例),代码写了"实际测了什么",两者之间没有任何显式双向可追溯锚点——无法从文档判断某用例是否已被实现验证,也无法从一个失败的真实测试反查对应验收标准。
- **期望修正**:不要求一次性覆盖 161 项;建议先在 P0 用例(如 `VAL-001` 四重门相关的 `T-VL` 系列、跨租户安全相关的 `T-CROSS` 系列)对应的真实测试函数上补 `// per test-design.md T-VL4` 类锚点注释,并在 `test-design.md` 补一列"实现状态/对应测试文件路径"。

### DD-11 [Major] 🟠 `domain-llm`/`domain-mcp`/`domain-task`/`domain-tool` 在全部 10 份详细设计文档 + `basic-design.md` 中零文档

- **证据**:对 10 份详设文档(`api/data/security/runtime/integration/ai-agent/external/internal/test/operation-design.md`)逐一 grep 4 个 crate 名,**全部零命中**(AUDIT-003 §3.3)。本轮补做的 crate 清单交叉核对进一步确认:`ls crates/` 实际 38 个 `domain-*` crate,与 `basic-design.md` 全文出现的 crate 名做 `comm` diff,**唯一"代码里有、母版文档提都没提"的 4 个 crate 正是这组**(其余候选差异经核实均为误报:`*-spec` 后缀命中的是专题 spec 文档名而非 crate 名;`domain-trace` 在 `basic-design.md:2980` 被明确描述为"嵌于 application 的子模块",不是独立 crate,不构成矛盾)。
- **性质**:不是"母版表格没更新"这类局部问题——这 4 个 crate 事实上从未被纳入过正式的 V-model 详设评审流程,唯一设计线索来自游离在主链之外的 `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` 等专题文档。
- **期望修正**:与既有的 AUDIT-002 §2.2 合并处理,补齐这 4 个 crate 在 `basic-design.md` + 相关详设文档里的正式章节,而非继续依赖专题文档。

### DD-12 [背景/Info,⚪ 阶段性已知策略——非独立缺陷,但需要显式声明] `api-design.md` 端点设计规模 vs `crates/api` 162 行骨架 + 前端 100% Mock-first(Zustand)

- **证据**:真实承载 REST handler 的 `crates/api/src/lib.rs` 目前仅 **162 行**(本轮重新核实,与 AUDIT-003 §4 一致)。`api-design.md` 定义的端点规模,AUDIT-003 §4 给出的数字是"约 279",但本轮用 `grep -oE "(GET|POST|PUT|PATCH|DELETE) /..."` 独立重新统计,得到全文 HTTP 动词+路径声明 **142** 处、去重后 **84** 个唯一端点签名——均未能复现 279,统计口径(是否含请求/响应示例块里的重复出现、是否含已废弃端点等)待澄清,此处不采用未经复现的数字,仅以"远超 162 行骨架能承载的规模"这一定性结论为准。对应地,`frontend-internal-01-architecture.md:389-393`(ADR-FE-003)确认前端当前**完全**跑在内存态 Mock 之上——`frontend/src/lib/seed.ts`(约 50 KB)覆盖 25 域 + 6 状态机的全量 mock 数据,通过 Zustand store 驱动,没有一个页面调用真实 REST API。("25 域"是前端文档自身反复使用的领域模型计数,`frontend-design.md`/`frontend-internal-01-architecture.md` 4 处一致引用,与 DD-08/DD-11 的后端 38 个 `domain-*` crate 计数是两个不同维度的数字,不构成同一数字的重复错误,此处未做交叉核实,仅如实转述。)
- **为什么单独列出而不算入上面几条 Blocker/Major**:这不是"遗漏"或"向壁虚构",团队通过 ADR-FE-002/003/004 显式决策了 Mock-first 路线,`crates/api` 骨架化是**当前阶段的既定状态**,不是 bug。但它是全仓库当前规模最大的一处"实装 vs 详设"规模级差距,且骨架侧几乎不含真实业务逻辑,如果不显式声明,容易被后续审核或新加入者误读为"api-design.md 已经过时/无人维护"。
- **期望修正**:不是改代码或改设计文档本身,而是在 `api-design.md` 顶部或 `frontend-design.md` ADR-FE-003 旁补一句显式范围声明,例如"当前处于 Mock-first 过渡期,`crates/api` 尚未实现 api-design.md 定义的端点,预计 V1 阶段补齐,进度追踪见 <链接>",避免这个已知差距被反复当成新发现的缺陷重复提出。
- **背景数据补充**(本轮新核实,AUDIT-003 未覆盖):`db/migrations/`(3 个文件)+ `docs/migrations/`(3 个文件)合计仅 14 条 `CREATE TABLE`,且均是运维/审计类局部 SQL(`ops-cluster`/`ops-log`/`ops-metrics`/`audit-trigger`/`exclusion-rls`),不构成 `data-design.md`(约 94 处表/章节标记)的完整 DDL 权威来源——这意味着 `data-design.md` 的逐表结构目前**无法**通过实际 DDL 做机械交叉核对,这也是 AUDIT-002/AUDIT-003 两轮都未能覆盖 `data-design.md` 语义级核实的直接原因(见文首范围声明)。

---

## 附录:不计入本文档的"文档内部矛盾"类问题(非乖离,是文档自身缺陷)

以下问题在 `frontend-design-feedback.md` 中已有记录,但性质是"设计文档内部前后不一致"或"文档措辞与自身另一处矛盾",不涉及实际代码/实装,因此**不属于"实装 vs 详设乖离"**,不计入上表,仍建议按原文档的处理顺序修复:

- **FD-11**(`frontend-design.md:320-321`"三选一"标签下实际列出 4 个/6 个选项,标签数字与列表条目数不符)——纯文档措辞问题。
- **FD-09/FD-10/FD-12/FD-14**(RSC vs Client-Component 表述、组件复用率表格、组件目录树、`INV-SCM` 范围表述)——AUDIT-003 §2.7 未在本轮核实,状态未知,且均属于文档内部一致性问题,不属于本文档范围。

---

## 处理优先级建议(沿用 AUDIT-003 §5,按本文档 ID 重新标注)

1. **DD-03**(PullRequest "7"→"8" 源头一个字符,三处已传播)
2. **DD-01 / DD-02**(状态机状态名重写,V1 切真后端前的阻塞项)
3. **DD-04**(NATS Subject 安全相关,已进入前端可见代码)
4. **DD-05 / DD-06 / DD-07 / DD-08**(错误码/字段名/crate 名对齐,修复成本低,建议合并进同一批前端文档修订)
5. **DD-09**(先做业务判断,再决定改代码或改文档)
6. **DD-10**(从 P0 用例开始补可追溯锚点)
7. **DD-11**(与 AUDIT-002 §2.2 合并处理)
8. **DD-12**(背景/Info,非缺陷——补一句显式范围声明,防止被误读为遗漏)

---

**产出方式说明**:本文档基于 `docs/reports/2026-09-08-audit-003-detailed-design.md` 已核实证据整理而成,DD-01~DD-04、DD-07 直接复用该报告的 file:line 证据(未重新验证);DD-05 的 `SEC-001`/`SEC-007` 行号、DD-06 的 `LocalRuntime` 字段范围均已在本轮重新 grep 核实;DD-08 的 `crates/api` 162 行为本轮重新核实,"25→38"crate 数量修正复用 DD-11 的交叉核对结果;DD-09 沿用该报告 §3.2 的核实结果;DD-10 直接复用该报告 §3.1;DD-11 的 4-crate 缺口沿用该报告 §3.3,crate 清单 `comm` diff(38 个 `domain-*`)为本文档撰写时新做的补充核实;DD-12 的 `crates/api` 162 行、migrations 目录 14 条 `CREATE TABLE` 统计为本文档撰写时新做的补充核实,"279"端点数未能复现(本轮独立统计为 142 处声明/84 个唯一签名),不作为已核实数字使用。
