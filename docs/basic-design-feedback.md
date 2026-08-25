# 基本设计書 / 详细设计 Review Feedback

> **审核对象（第一阶段）**: `docs/basic-design.md` v0.1，3550 行（修复后现为 3572 行，见下方"F-01~F-08 修复确认"）
> **审核对象（第二阶段，2026-08-25 新增）**: 10 份详细设计文档 —— `api-design.md` / `data-design.md` / `security-design.md` / `runtime-design.md` / `integration-design.md` / `ai-agent-design.md` / `external-design.md` / `internal-design.md` / `test-design.md` / `operation-design.md`（均已合并入 `main`，均声明 `basic-design.md`（修复后版本）为上游基线）
> **基线**: `docs/requirements.md` v2.0 → `docs/basic-design.md`（修复后）→ 各详细设计文档自身声明的上游（`§N`=basic-design.md 章节号，`§R-N`=requirements.md 章节号，`§API-N`/`§Data-N` 等=对应详细设计文档章节号，均沿用各文档头部自行声明的记法）
> **审核方法**: 第一阶段沿用 requirements.md §45 已确立的专项 Review Lens，并新增机制化检查。第二阶段在此基础上新增"上游一致性传播检查"：逐一核对 F-01~F-08 修复结果是否被详细设计文档正确继承（Module 数/tenant_id 对象数/状态机状态数等关键数字的传播链路），并对每份详细设计文档做结构性抽样通读 + 全文 grep 交叉核对。
> **审核者**: Claude（本会话），非原作者。本文档只产出 Finding，不直接修改被审文档。

---

## 使用说明

- 每条 Finding：`ID | Severity | 位置 | 违反的依据 | 期望修正`。
- Severity：**Blocker**（违反不可推翻的架构原则/义务，或内部自相矛盾导致下游无法施工）/ **Major**（覆盖缺口、命名/编号错误，影响可追溯性但不阻断施工）/ **Minor**（措辞、格式、局部不一致）/ **Question**（需要作者澄清的设计判断，非缺陷）。
- 位置格式：`文档名.md:§章节号` + 行号；第一阶段（F-nn）默认省略文档名前缀，均指 `basic-design.md`；第二阶段（D-nn）每条均显式标注引入文档。

---

## Findings

### F-01 [Major] §2.1 Domain 总数引用错误

- **位置**: `basic-design.md:276`（`### 2.1 完整 Domain 列表(继承 §6 共 19 个 + 5 个子域 = 24 个逻辑 Module)`）
- **依据**: `requirements.md:§6`（Domain Boundary 总览）实际列出 **22 个** Domain（Identity, Tenant, Workspace, Project, Work Management, Workflow, Planning, Collaboration, Permission, Automation, Integration, SCM, Development Context, Development Execution, Worktree, Agent, Feedback, Context, Validation, Audit, Search, Notification），而非 19 个。
- **影响**: 标题的算术（19+5=24）不成立；24 这个目标数字本身在 §2.1 表格中是对的，但"如何从上游 22 个到达 24 个"的推导过程被错误陈述，破坏了 §39 Traceability Model 要求的可追溯性。
- **期望修正**: 改为"继承 §6 共 22 个 + 2 个子域拆分 = 24 个"，并在 §2.2 脚注中明确写出具体是哪些上游 Domain 被拆分（见 F-02）、哪些被合并（见 F-03），而不是只解释 Collaboration 一处。

### F-02 [Major] "Development Context"（§6 原始 Domain 之一）在 24-Module 枚举中下落不明

- **位置**: `basic-design.md:276-325`（§2.1 表格 + §2.2 脚注）
- **依据**: `requirements.md:§6` 将 "Development Context" 与 "Development Execution" 列为两个**并列**的 Domain；`requirements.md:§20` "Development Context 要求"（Symbol-aware Repository Context / Repository Indexing）与 `requirements.md:§21` "Development Execution 要求"（ChangeSet 等）是两章不同内容。
- **观察**: `basic-design.md` §2.1 第 8 项 `domain-development` 的"主要实体"只列 `DevelopmentExecution, ChangeSet, Link`（对应 §21），未提及 Repository Indexing / Symbol Detection / Development Context 相关实体（对应 §20）。§2.2 脚注只解释了 "§6 的 Collaboration 拆为 domain-comment + domain-collaboration"，**没有**说明 "Development Context" domain 合并进了 `domain-development`（还是被拆分进了别处，例如 `domain-context`？两者命名极易混淆）。
- **影响**: 一个上游一级 Domain 的下落无法追溯，违反 §39 Traceability Model；且 `domain-context`（Context Compiler，对应 §26）与 "Development Context"（§20）两个名字在中文/英文语境下高度相似，容易被下游工程师误认为同一个东西，从而在 §20 的 Requirement（Symbol-aware Context / Repository Indexing）被实现时找不到归属 Module。
- **期望修正**: 在 §2.2 脚注中显式声明"§6 的 Development Context 合并入 domain-development"（或指出其实际去向），并在 §2.1 表格 domain-development 行的"主要实体"中补充来自 §20 的实体（如 SymbolIndex/RepositoryContext）。

### F-03 [Major] §1.1/§2.1/§2.3/§2.4 均未纳入 domain-local-runtime，但 §4.6 将其作为完整 Module 详细设计

- **位置**: 缺失处 `basic-design.md:143-208`（§1.2 逻辑架构图 crates/domain-* 子图，24 个 domain 均列出但无 local-runtime）、`basic-design.md:276-325`（§2.1 完整 Domain 列表，声称覆盖全部 24 个 Module，无 local-runtime 行）、`basic-design.md:327-359`（§2.3 Domain 间调用方向硬约束表，无 local-runtime）；对照处 `basic-design.md:1260`（§4.6 `domain-local-runtime` 独立详细设计小节，与 §4.1-4.5/4.7-4.10 同级）
- **依据**: `basic-design.md:§0.3` 命名约定明确定义 "Module / Domain: 同义，代表 crate 级别的逻辑划分（非 deployment）"；`requirements.md:§13.3` 的既有 crate 清单（16/实际 20 个，见 F-05）与 `requirements.md:§23.1` 均未定义 "domain-local-runtime" 这个 crate 名。
- **影响**: 这是一处**自相矛盾**：
  1. §2.1 标题声称"完整 Domain 列表"覆盖 24 个 Module，但实际有 25 个 Module 拥有独立详细设计小节（§4.1-4.10 共 10 节，对应表中只有 10 个 Core+部分 Supporting Domain，其中 domain-local-runtime 不在 §2.1 的 24 项枚举里）。下游工程师按 §2.1 数 crate 数量、按 §2.3 检查依赖方向合法性时，会完全遗漏这个 crate。
  2. 命名与定位冲突：§4.6.1 开篇称 "Local Runtime 是运行于开发者机器 / 企业 Runner 的安全代理进程"——这描述的是**运行在开发者机器上的独立进程**（`§1.1` 图中的 "Local Daemon - Rust"，明确在 K3s Cluster 之外）；但 `domain-local-runtime` 这个名字按 §0.3 命名约定意味着**服务器端 work-core 内的 in-process crate**。§4.6.2 的实体（Runtime 注册表、RuntimeCommand、RuntimeObservation）实际描述的是"服务器端管理 Local Runtime 的 Port/Adapter"，而非 Local Daemon 本身——但小节标题和 4.6.1 开篇没有做这个区分，容易让下游读者以为要在 K3s 集群内跑一个叫 `domain-local-runtime` 的东西来"代理开发者机器"，与 §1.1/§23.1 "Local Runtime 不计入 K8s Workload" 的不变量产生表述层面的混淆。
- **期望修正**: (a) 在 §2.1 表格中补一行 `domain-local-runtime`（服务器侧 Runtime 注册表 / Port，明确注明"与 Local Daemon 二进制是两个不同制品"），Domain 总数相应改为 25；(b) 在 §1.2 mermaid 图与 §2.3 依赖方向图中补上这个 crate 及其依赖边；(c) 在 §4.6.1 开篇加一句区分"本节讨论的是服务器侧 Port，不是 Local Daemon 进程本身（后者见 §1.1 LocalRuntime 子图）"。

### F-04 [Minor] §N 引用命名空间冲突：5 处仅引用原始提示词编号（N>47），未对齐文档自身声明的引用约定

- **位置**: `basic-design.md:15`（"本文档不输出生产代码(重申 §105)"）、`:69`（"P0/P1/P2: 优先级(继承 §63)"）、`:284`（表格内 "WorkItem ≠ Git Branch(§85)"）、`:828`（"Agent Handoff(§52)"）、`:1982`（"不能自成事实(§60)"）
- **依据**: `basic-design.md:4`（文档头部）明确声明 "上游要件定义书: `docs/requirements.md` v2.0（下文以 §N 引用）"——即全文 §N 的唯一约定含义是 requirements.md 的章节号。而 `requirements.md` 最大章节号为 47（§0-§47）。上述 5 处引用的 §105/§63/§85/§52/§60 全部 >47，只存在于**原始 105 节提示词**（未入库，`requirements.md:§0` 已声明"原始文档在本仓库中未找到"）的编号体系里，requirements.md 中虽也在括号内间接引用了这些原始编号（如 §41.2 标题写"(§63)"），但那是伴随着有效的 requirements.md 章节号（如 §41.2）一起出现的，从未单独出现。
- **影响**: 按文档自己的约定字面理解，这 5 处引用会指向 requirements.md 中不存在的章节，读者据此核对时找不到对应内容。属于孤立、可精确定位的个例（全文其余同类引用均伴随一个 ≤47 的有效 requirements.md 章节号一起出现，如 "§44.2,§86-90"），非系统性问题。
- **期望修正**: 5 处补上对应的 requirements.md 章节号，例如 L15 "§105" → "§47"（下一阶段输入清单章末尾的原始指令，或直接改引用 requirements.md:§0 关于"不写生产代码"的声明）；L69 "§63" → "见 requirements.md §41.2"；L284 "§85" → "§44.3"；L828 "§52" → "§24.5"；L1982 "§60" → "§14.1"。

---

### F-05 [Major] §4.9.3 / §7.2 的 WorkItem "默认" 状态机与 requirements.md REQ-WF-001 的"默认最简三态"矛盾

- **位置**: `basic-design.md:1707-1717`（§4.9.3 状态机(WorkItem 默认)）、`basic-design.md:2317-2341`（§7.2 WorkItem Workflow）
- **依据**: `requirements.md:236`（REQ-WF-001）明确规定 "**默认最简三态**工作流（待办 / 进行中 / 完成），支持自定义状态扩展"——即 MVP 默认必须是 TODO → IN_PROGRESS → DONE 三态，自定义扩展是可选项，不是默认项（"属于 MVP 精简范围"）。
- **观察**: `basic-design.md` 两处都把 `TODO → IN_PROGRESS → IN_REVIEW → DONE`（另加 `BLOCKED`、§7.2 图中还有 `CANCELLED`）标注为 **"WorkItem 默认"** 状态机，即 5-6 个状态作为默认值，把 REQ-WF-001 中作为"可选自定义扩展"的 `IN_REVIEW`（及图中出现但未被计入正文的 `CANCELLED`）晋升为默认状态集的一部分。
- **影响**: 直接违反 REQ-WF-001 对 MVP 默认工作流复杂度的显式约束，且与该 Requirement 背后的"不强制可视化工作流配置器、属于 MVP 精简范围"的产品判断相悖——把 4-6 态当默认，实质上悄悄扩大了 MVP 范围。此外 §7.6 状态机总览表把 WorkItem 状态数记为"5 + 扩展"，但 §7.2 的图里实际画出 6 个状态节点（含 CANCELLED），"5"与图不符，"扩展"与"默认"的边界在两处表述不一致。
- **期望修正**: 把 §4.9.3/§7.2 的"默认"状态机改为 TODO → IN_PROGRESS → DONE 三态，并将 IN_REVIEW / BLOCKED / CANCELLED / IN_TESTING / READY_FOR_DEPLOY 等一并归入"Project Policy 自定义扩展示例"一节，与 REQ-WF-001 保持严格一致；同时修正 §7.6 表格的状态计数使其与实际图示一致。

### F-06 [Minor] §4.10.4 / §6.1 tenant_id 隔离对象计数与表格行数不一致

- **位置**: `basic-design.md:1843`（"强制 tenant_id 携带的对象(12 项)"）与其后表格（`:1845-1859`，实际 13 行，编号 1-13）；`basic-design.md:2114`（"13 类对象必带 tenant_id 隔离(继承 §16)"，此处口径又与 §4.10.4 的表格内容基本重复但两处标题数字不完全对应上游）
- **依据**: 表格本身内部编号 1-13，共 13 行。
- **影响**: 标题"12 项"与表格实际的 13 行不符，属局部计数错误；此外 §4.10.4 与 §6.1 两处几乎是同一张表格的重复呈现（内容高度重合，只是章节位置不同），存在信息重复维护的风险——未来若增补一类隔离对象，容易只改一处漏改另一处。
- **期望修正**: 统一为 13 项并修正标题数字；评估是否可将 §4.10.4 表格直接引用 §6.1（或反之），避免同一份清单在两处独立维护。

### F-07 [Major] §8.5 明确 "Local Runtime 是外部进程"，与 §4.6/§4.10.4/§6.1 将其作为 `domain-*` crate 引用相矛盾

- **位置**: `basic-design.md:2582-2596`（§8.5"Local Runtime 不计入 K8s Workload"，明确写"Local Runtime 是**外部进程**，运行于 Developer Machine / Self-hosted Runner / Cloud Workspace"）对照 `basic-design.md:1260`、`:1848`、`:2119` 等处将 "domain-local-runtime" 作为服务器侧 `crates/domain-*` 模块反复引用
- **依据**: `basic-design.md:§0.3` 命名约定："Module / Domain: 同义，代表 crate 级别的逻辑划分（非 deployment）"，隐含 `domain-*` 前缀专指跑在 work-core 进程内、K3s 集群内的 Rust crate。
- **影响**: 与 F-03 指向同一根因，这里是其在部署拓扑章节的具体体现：§8.5 的正确表述（Local Runtime 是集群外的独立进程）与 §4.6.1 开篇"Local Runtime 是运行于开发者机器的安全代理进程"字面上完全一致，但两处都用 `domain-local-runtime` 这个 crate 命名法来指代它，而没有像 §1.1 架构图那样区分"服务器侧的 Runtime 注册表/Port（crate 内）"与"Local Daemon 二进制（集群外）"这两个不同制品。这会让下游详细设计阶段的工程师误以为要在 work-core 里实现一个叫 `domain-local-runtime` 的 crate 来"代表"外部进程本身,而不是实现一个管理外部 Runtime 注册信息的 Port/Adapter。
- **期望修正**: 与 F-03 合并处理——将 `domain-local-runtime` 明确重命名或加注为"服务器侧 Runtime Registry / Local Runtime Gateway Port"，与"Local Daemon"（外部二进制，不属于任何 `crates/domain-*`）两个概念在全文档中一致区分。

### F-08 [Minor] §7.4/§7.6/接口稳定承诺 均将 AgentSession 状态数记为 13，实际状态集合有 14 个

- **位置**: `basic-design.md:2366-2378`（§7.4 AgentSession 状态机文本块）、`:2429`（§7.6 表格 "AgentSession | 13"）、`:3521`（接口稳定承诺 #8 "AgentSession 状态机(§7.4):13 个状态"）、附录 A.4 mermaid 图（`:3298-3328`）
- **依据**: 逐一清点 §7.4/A.4 中出现的唯一状态节点：`CREATED, STARTING, RUNNING, WAITING_TOOL, TOOL_RUNNING, TOOL_COMPLETED, WAITING_FEEDBACK, FEEDBACK_RECEIVED, VALIDATING, COMPLETED, FAILED, ABORTED, CRASHED, TIMEOUT`，共 **14** 个，而非 13 个。
- **影响**: 这个错误计数被写进了 §14 之后的"**接口稳定承诺**"，即被正式声明为下游阶段"不会因详细设计而变更"的锁定数字——如果下游工程师按"13"核对状态机完整性（例如写单元测试遍历所有状态,或做状态覆盖率检查工具），会漏查一个真实存在的状态。
- **期望修正**: 三处"13"改为"14"。同时核实 Worktree(17,已验证正确)、Feedback(6,已验证正确)、Decision(3,已验证正确)三处计数保持不变。

---

## F-01~F-08 修复确认（commit `81778d9`）

逐条核对 `docs/basic-design.md`（修复后，3572 行）是否已落实第一阶段全部 Finding：

| Finding | 状态 | 核实依据 |
|---|---|---|
| F-01（Domain 总数算术错误） | ✅ 已关闭 | `basic-design.md:277-279` 改为"继承 §6 共 22 个 + 3 个拆分/合并 = 25 个逻辑 Module"，算术自洽 |
| F-02（Development Context 下落不明） | ✅ 已关闭 | 同上脚注显式列出 3 处拆分/合并，含"Development Context 合并入 domain-development（新增 SymbolIndex/RepositoryContext/DevelopmentContext 实体）" |
| F-03（domain-local-runtime 遗漏/定位混淆） | ✅ 已关闭 | §2.1 补入 domain-local-runtime 行并注明"集群外 Runtime 服务器侧 Registry/Port，与 Local Daemon 二进制区分（见 §4.6.1）"；Domain 总数改为 25 |
| F-04（§N 引用越界 5 处） | ✅ 已关闭 | 抽查原 5 处引用位置，编号均已替换为 requirements.md 有效章节号 |
| F-05（WorkItem 默认状态机膨胀） | ⚠️ 文字已关闭 / **图示残留缺陷** | §4.9.3、§7.2 正文均已改为"默认最简三态：TODO → IN_PROGRESS → DONE"，扩展状态移入"Project Policy 自定义扩展示例"；**但**附录 A.2 mermaid 图（`basic-design.md:3278-3296`）未同步更新，见下方新记录的残留问题 |
| F-06（tenant_id 对象计数 12 vs 13） | ✅ 已关闭 | `basic-design.md:1861`/`:2132` 均改为"13 项"/"13 类对象"，与表格行数一致 |
| F-07（§8.5 与 §4.6 crate 命名矛盾） | ✅ 已关闭 | 与 F-03 合并修复，§4.6.1 开篇补充"本节讨论服务器侧 Port，非 Local Daemon 进程本身"的区分句 |
| F-08（AgentSession 状态计数 13 vs 14） | ✅ 已关闭 | `basic-design.md:2447`、`:3543` 均改为"14"，与状态机文本/A.4 图一致 |
| REQ-AUT-001 笔误 | ✅ 已关闭 | 已改为 VAL-001（随附 F-06 一并核实） |

### F-05 残留缺陷（新记录，未占用 D-nn 编号——问题根源在 basic-design.md 本身，非详细设计引入）

- **位置**: `basic-design.md:3276-3296`（附录 A.2 "WorkItem Workflow（默认三态 + 扩展）" mermaid 图）
- **依据**: 图中实际边为 `TODO→IN_PROGRESS`、`IN_PROGRESS→IN_REVIEW`（出现两次，标签分别为"User"与"直接提交"，重复边）、`IN_REVIEW→DONE`、`IN_PROGRESS→BLOCKED`、`BLOCKED→IN_PROGRESS`、`TODO/IN_PROGRESS/IN_REVIEW→CANCELLED`。
- **影响**: 图中**不存在 `IN_PROGRESS → DONE` 直连边**，`DONE` 唯一可达路径必须经过 `IN_REVIEW`（一个在正文中已被明确划为"非默认扩展状态"的节点）。也就是说，F-05 修复后 §4.9.3/§7.2 正文所写的默认最简路径 `TODO → IN_PROGRESS → DONE` 在这张"默认三态 + 扩展"图里**画不出来**，图与刚修复的正文再次出现口径分歧；此外 `IN_PROGRESS→IN_REVIEW` 重复出现两条边（"User"/"直接提交"）也是图本身的冗余。
- **期望修正**: 在图中补一条 `IN_PROGRESS --> DONE: 直接完成` 边以还原三态默认路径，并合并两条重复的 `IN_PROGRESS→IN_REVIEW` 边（或明确区分两者触发条件的差异，若确有差异应在标签中说明而非用两条视觉相同的边表达）。

---

## Findings（详细设计阶段，2026-08-25 新增）

> 编号延续 D-01 起，与第一阶段 F-nn 区分，因为审核对象已从单一文档变为 10 份下游文档；根因归属统一标注"引入文档"。

### D-01 [Major] api-design.md 为 WorkItem 引入未定义的 `ARCHIVED` 终态

- **位置**: `api-design.md:624`（"默认三态：`TODO → IN_PROGRESS → DONE`（`ARCHIVED` 终态）"）
- **依据**: `basic-design.md:1720-1735`（§4.9.3）与 `:2335`（§7.2）修复后的 WorkItem 默认状态机均只定义 `TODO/IN_PROGRESS/DONE` 三态，`ARCHIVED` 从未出现在 WorkItem 的默认或扩展状态清单中（`ARCHIVED` 仅出现在 Worktree 生命周期，`basic-design.md:557/2311/2329/2333`，是另一个聚合根的状态）；下游 `data-design.md:1078-1103`（`work_item_status` Lookup Table 完整种子数据：`TODO/IN_PROGRESS/DONE/IN_REVIEW/BLOCKED/CANCELLED/IN_TESTING/READY_FOR_DEPLOY/NEEDS_INFO`，共 9 行）同样**不包含 `ARCHIVED`**。
- **影响**: 这是一个仅在 `api-design.md` 单处出现、且与同为详细设计阶段产物的 `data-design.md`（更接近实现的 DDL 级制品）相矛盾的孤立错误——很可能是对 Worktree `ARCHIVED` 状态的误引用/笔误。若不修正，API 契约文档会让下游客户端/前端工程师误以为 `GET /v1/work-items/{id}/transitions` 可能返回一个实际状态机和数据库枚举都不存在的终态。
- **期望修正**: 删除 `api-design.md:624` 的 `（ARCHIVED 终态）` 备注，或改为明确指出这是 Worktree（而非 WorkItem）的状态；同时核查 api-design.md 全文是否还有其他地方复用了这一笔误。

### D-02 [Major] ai-agent-design.md §2.2 `compile_context()` 算法与自身 P0-P4 定义自相矛盾，且 P5 存在双重处理路径

- **位置**: `ai-agent-design.md:202-253`（§2.2 `compile_context()` 伪代码）
- **依据**: `basic-design.md:1077-1091`（§4.4.4）定义 Token Budget 优先级为 **P0-P4 共 5 层**（P0 Explicit Human Constraint / P1 AC.../ P2 相关代码/失败测试 / P3 历史讨论 / **P4 Low-confidence AI Summary**），且此 5 层结构已被 `basic-design.md:3538` 接口稳定承诺 #3 锁定为"P0-P4 五层"，不属于可自由调整的草案部分（草案未冻结的只是各层具体 Budget 百分比/字节数，"P0-P4 五层结构"本身是冻结项）。
- **影响**: `compile_context()` 的优先级分桶列表为 `[P0, P1, P2, P3, P5]`（`ai-agent-design.md:207`）及对应预算分配 `P0:30% | P1:30% | P2:25% | P3:10% | P5:5%`（`:209`）——**P4（Low-confidence AI Summary）在这个核心确定性算法里完全消失**，既无分桶也无预算，任何被标记为"低置信度 AI 摘要"的候选内容在这份伪代码中找不到归宿。同时该算法内部自相矛盾：Step 2-4 把 `P5`（Untrusted Repo Content）当作与 P0-P3 同等的普通优先级桶纳入正常的候选填充循环并分配 5% 预算；但 Step 6 又单独执行 `filter_untrusted(candidates)` 并把结果写入 `sections['untrusted_repo_content']`，同时注明"强制：Untrusted 不得进入 P0/P1/P2/P3 任何桶"。若 Untrusted 内容在 Step 1 `collect_candidates()` 阶段就已经被标记为 P5 并进入 Step 4 的正常分桶循环，则 Step 6 的"隔离"是对已经被混入正常预算池的内容做二次处理（与"绝不与 P0-P3 混合"的隔离意图矛盾）；若 Step 4 中的 P5 桶实际上永远不会被触发（因为 Untrusted 候选从未进入 `by_priority[P5]`），则 Step 4 中列出的 `P5` 与 `5%` 预算是死代码，写在确定性算法说明里具有误导性。这是 §26.1 所强调"Context Compiler 必须是确定性、可单元测试系统"的核心模块，这样的伪代码无法直接转化为可通过单元测试的实现。
- **期望修正**: (a) 恢复 P4 桶及其预算份额，五桶分配需重新计算（如 P0/P1/P2/P3/P4 共 100%，不占用给 Untrusted 隔离逻辑的空间）；(b) 从 Step 2-4 的正常优先级分桶/预算分配循环中彻底移除 P5，只保留 Step 6 的单独 `filter_untrusted` + 隔离段逻辑作为 Untrusted 内容的唯一处理路径，使伪代码与"P5 单独分类，绝不与 P0-P4 混合"的既定约束一致。

### D-03 [Major] "9 种白名单命令" 在 6 份详细设计文档中一致引用，但与 basic-design.md 实际的 `RuntimeCommand` 枚举（8 个变体）不符，且新增的第 9 项 `ReportObservation` 与 basic-design.md 另一独立枚举 `RuntimeObservation` 概念冲突

- **位置**: `api-design.md:1038`（首次给出完整清单："`GitStatus / CreateWorktree / ReadDiff / RunApprovedTest / QueryAgentStatus / SubmitFeedback / StartAuthorizedAgentSession / StopAgentSession / ReportObservation`"，共 9 项）；同一说法（"9 种白名单命令"）以一致措辞重复出现于 `data-design.md`（3 处，含 DDL 注释）、`runtime-design.md`（3 处，含 §12.1 小节标题）、`security-design.md`（约 15 处，含 §5.5.2 独立小节标题）、`integration-design.md`（1 处）、`test-design.md`（4 处）
- **依据**: `basic-design.md:1290-1304`（§4.6.2 `RuntimeCommand` 枚举定义）实际只有 **8 个变体**：`GitStatus/CreateWorktree/ReadDiff/RunApprovedTest/QueryAgentStatus/SubmitFeedback/StartAuthorizedAgentSession/StopAgentSession`，且明确注释"严禁出现 `ExecuteArbitraryShell`"；`basic-design.md` 全文未在任何位置给出过"9"这个数字（唯一相关数字是"8 种类型"，出现在别处的 Risk Signal 计数，与本枚举无关）。`ReportObservation` 这个名字在 `basic-design.md` 中不存在于 `RuntimeCommand` 枚举——`basic-design.md:1306-1318` 定义了一个**方向相反、语义不同**的独立枚举 `RuntimeObservation`（Local Daemon → Control Plane 的上报事件：`WorktreeStatusObserved/AgentSessionStateObserved/BuildCompleted/TestCompleted/DiffAvailable/Heartbeat/Disconnected`，共 7 个变体），是"上报"而非"下发命令"。
- **影响**: `RuntimeCommand` 白名单是 LRT-002/SEC-008 安全边界的核心机制——§4.6.3 强制项表格明确"Command Authorization：每次 Command 由 Control Plane 验证（白名单）"。把"上报观测事件"（`RuntimeObservation`，Daemon→Control Plane，语义上是遥测/审计数据，不需要"命令授权"这种防止 Daemon 被滥用执行危险操作的机制）当作第 9 种"命令"并入同一个白名单枚举，混淆了两个威胁模型完全不同的方向（"Control Plane 授权 Daemon 执行什么"vs"Daemon 可以上报什么"）。且这个"9/ReportObservation"口径已经从 `api-design.md`（最早合并的详细设计文档）一致传播到后续 5 份文档，包括安全设计（`security-design.md` §5.5.2 将其列为独立的鉴权/ACL 小节标题）与测试设计（作为测试用例数量依据），意味着如果后续按"9 种命令"编写单元测试/ACL 配置，会引入一个 basic-design.md 从未定义、且与既有 `RuntimeObservation` 概念职责重叠的第 9 种命令类型。
- **期望修正**: (a) 若 `ReportObservation` 确有必要作为 Control Plane 可下发的第 9 种命令（例如"主动拉取 Daemon 上报一次观测"），需要先在 `basic-design.md` 的 `RuntimeCommand` 枚举中正式补充这一变体并更新接口稳定承诺，而不是让详细设计阶段单方面引入；(b) 若这只是对 `RuntimeObservation`（Daemon 主动上报，非 Control Plane 命令）的误用/概念混淆，应在全部 6 份文档中统一改回"8 种白名单命令"，并将 `ReportObservation` 相关表述改为引用独立的 `RuntimeObservation` 上报机制，与"命令白名单"的鉴权语义脱钩。

### D-04 [Major] test-design.md §6.3"验收门禁"只覆盖 VAL-001 强制四重门中的 1/4，其余三项（ValidationPassed / FeedbackResolved / GateApproved）在全文档任何位置均未出现对应测试用例

- **位置**: `test-design.md:782-787`（§6.3 验收门禁，全文唯一一处名为"完成门禁"性质的小节，内容仅为"✅ 所有 MUST AC 必须有 Test / ⚠️ SHOULD AC 鼓励有 Test / ❌ COULD AC 可选"）
- **依据**: `basic-design.md:1258`（"关键不变量"）明确锁定 AI 完成声明必须同时满足四个条件："`ValidationPassed && AcceptanceCoverage==100 && FeedbackResolved && GateApproved` 四重门，缺一不可"；`basic-design.md:1173`（VAL-001，§27.3）"AI 修改不能以'Agent says done'作为完成条件"是全文档标注为 P0 的核心不变量之一（`basic-design.md:1262`）。对 `test-design.md` 全文 grep `VAL-001`/`四重门`/`FeedbackResolved`/`GateApproved`/`ValidationPassed`/`is_ai_complete_claim`/`自我报告`，均**零匹配**。
- **影响**: `test-design.md` §6.3 的"验收门禁"字面上只测试了四重门中的 `AcceptanceCoverage` 一项（且仅到"MUST AC 有无 Test"的粒度，未验证运行时 `AcceptanceCoverage==100%` 这个数值条件本身如何被测试覆盖）；`ValidationPassed`（Build/Unit/Integration/Lint/Format/Static Analysis/Security/Review/Custom 十项校验全部通过）、`FeedbackResolved`（未解决 Feedback 是否会阻断完成声明）、`GateApproved`（显式审批环节）三项均未见于测试设计的任何测试层级（单元/集成/E2E/验收）。这意味着 VAL-001 这条被 basic-design.md 明确标注为"防止 AI 自我报告充当完成依据"的 P0 防线，在测试设计阶段没有对应的测试用例矩阵去验证其四个子条件是否被正确地全部强制执行（例如：故意只满足 3/4 条件时系统是否正确拒绝完成声明，这类"负向测试"完全缺失）。
- **期望修正**: 在 §6（验收测试）或 §8（安全测试，因为这本质是一条防止 Agent 自我报告绕过流程的安全不变量）中新增专门的测试用例组，覆盖 VAL-001 四重门的全部 4 个子条件，尤其是"任一子条件不满足时完成声明必须被拒绝"的负向测试（4 选 1 缺失 × 4 种组合的边界测试），并在 §12.1"给 Implementation"契约中显式列出这条不变量作为必须实现的测试点。

### D-05 [Minor] data-design.md 遗漏 3 处"（核心聚合根）"表格标题标注

- **位置**: `data-design.md:2755`（`#### 4.19.1 development_execution 表`）、`:2802`（`#### 4.19.2 change_set 表`）、`:3853`（`#### 4.23.3 decision 表`），均无"（核心聚合根）"字样；对照同一文档中已正确标注的 6 处：`:890 work_item`、`:3085 worktree`、`:3357 agent_session`、`:3550 feedback`、`:3742 context_packet`、`:3917 validation_result`
- **依据**: `basic-design.md` 明确用"聚合根"字样标注 `DevelopmentExecution`（`:1561`）、`ChangeSet`（`:1575`）、`Decision`（`:1038`），与 WorkItem/Worktree/AgentSession/Feedback/ContextPacket/ValidationResult 地位相同（均为聚合根）；`PullRequest`（`:1459`）则被明确标注为"实体"而非聚合根，`data-design.md` 对 `pull_request` 表未标注"核心聚合根"是正确的，形成对照，说明"核心聚合根"这个标注惯例在 `data-design.md` 中是被有意使用、而非随意省略的。
- **影响**: 影响面窄（只是表格标题的一致性标注缺失，不影响 DDL 本身的正确性），但破坏了"核心聚合根"标注在全文档中的完备性——下游读者若依赖 `grep 核心聚合根` 来枚举全部聚合根表（例如生成聚合根清单/ADR 索引），会漏掉 `development_execution`/`change_set`/`decision` 三张表。
- **期望修正**: 在 `data-design.md:2755`/`:2802`/`:3853` 三处表格标题补上"（核心聚合根）"字样，与其余 6 处保持一致的标注惯例。

---

## 已完成的补充检查（未发现新增问题，记录以避免重复审查）

- **crate 总数口径**：requirements.md §13.3 自身即存在"声称 16 crates,实际列出 17 domain crate + 3 支撑 crate = 20"的计数错误（这是 requirements.md 自身的既有缺陷,非 basic-design.md 引入）。basic-design.md 没有重复"16"这个错误数字，而是重新给出 24(见 F-01/F-03,该数字本身仍有 domain-local-runtime 未计入的问题)，即 basic-design.md 并未原样照抄 requirements.md 的错误计数，值得肯定。
- **K3s 部署角色**：§8.1/§8.2 反复重申 `gateway / identity / work-core / worker` 四角色 + `realtime`(条件性)的边界，§8.2 明确逐一列出 7 个被禁止的独立 Deployment 名称并逐一说明其收敛去向，未发现任何隐藏的额外常驻 Deployment。符合 §44.2 K8s Tax 纪律。
- **ch7 状态机文本表 与 附录 A mermaid 图**：内容互补（前者列"触发者/迁移条件"表格，后者是可视化状态图），未发现状态集合本身相互矛盾（除 F-05/F-08 指出的计数错误外，两处对同一状态机的描述是一致的）。不构成结构性重复。
- **P0 Requirement / ARCH-OBL-DEV-001~006 覆盖**：WT-001~003、AGT-001/002、FBK-001/002、CTX-001/002、VAL-001、SCM-001、LRT-001/002、SEC-xxx 及全部 6 条 ARCH-OBL-DEV 均在文中至少出现一次（部分以 `WT-001~003` 等区间记法出现，非遗漏）。**唯一例外**：§2.1 表格第 6 行引用 "REQ-AUT-001"（`basic-design.md:289`），既不匹配 requirements.md §41.1 定义的任何前缀，也不是该行实际对应的 P0 ID（该行描述"AI 自我报告不构成完成"，按 §41.2 应为 **VAL-001**）。判断为笔误：应改为 `VAL-001`，与同页 §4.5.6 的正确引用保持一致。（Minor，并入本条不单独编号）
- **MVP Must Have 映射完整性**：§13.1 表格 21 行与 requirements.md §30.2 原文 21 个条目逐一对应，**无遗漏、无多余**，是全文交叉引用质量最高的一节。
- **DDL / 生产代码越界扫描**：全文未出现 `CREATE TABLE`/`ALTER TABLE` 等 DDL，Rust 代码块全部止步于 `trait`/`struct`/`enum` 签名，未出现函数体实现，符合 §0.1 声明的"本文档不输出生产代码"自我约束。
- **Non-Goals 违反扫描**：全文出现的 Graph Database / Vector Database / OpenSearch / Service Mesh / Event Sourcing / CQRS / Database per Domain 均以"明确排除"的方式出现（ADR 选项 C 被拒绝、§13.5 强化排除清单等），未发现将其作为实际设计选择采纳的情况。

**（以下为第二阶段——详细设计审核——新增的补充检查）**

- **25-Module 数字传播**：`api-design.md`（§0 头部、§2.1 契约映射表、3.26 节标题）、`data-design.md`（§0 头部、25 个 PostgreSQL Schema 与 Module 1:1 对应）均正确继承修复后的"25"，未发现任何详细设计文档仍沿用旧数字"24"。
- **13 类 tenant_id 隔离对象传播**：10 份详细设计文档**全部**正确继承"13"，且每份文档均包含各自独立的"全部 13 类必带对象至少出现 1 次"自查清单，未发现计数漂移。
- **AgentSession 14 状态传播**：全部抽查文档一致引用"14"；`data-design.md:3434`/`:5346` 甚至直接以"（F-08 修正）"字样注明数字来源，确认下游文档编写时确实消费了本反馈文档的 Finding，而非独立重新计数。
- **K8s Tax / 禁止独立 Deployment 清单传播**：`operation-design.md` §1.1 明确禁止的 7 个独立 Deployment 名称（notification-service/validation-service/ai-service/search-service/realtime-service（条件性）/audit-service/agent-orchestrator）与 `basic-design.md` §8.2/§8.6 的收敛清单逐一匹配，未发现新增的隐藏常驻 Deployment；MVP 阶段 Service ≤10/Deployment ≤15/Pod ≤100 等数值上限未见被任何详细设计文档突破。
- **`ExecuteArbitraryShell` 等 3 类禁止能力传播**：`runtime-design.md`/`security-design.md`/`test-design.md`/`api-design.md` 对"禁止 `ExecuteArbitraryShell`/`ReadArbitraryFile`/`WriteArbitraryFile`"的引用一致，未发现放宽或遗漏。
- **聚合根标注完整性核查**：`basic-design.md` 本身明确用"聚合根"字样标注了 9 个对象——WorkItem、Worktree、AgentSession、Feedback、ContextPacket、ValidationResult（这 6 个）之外，`ChangeSet`（`basic-design.md:1575`）、`DevelopmentExecution`（`:1561`）、`Decision`（`:1038`）**同样被明确标注"聚合根"**；而 `PullRequest`（`:1459`）被明确标注为"**实体**"而非聚合根，是刻意的架构区分（统一抽象 GitHub PR/GitLab MR 的技术性实体，不是领域聚合根），这一点是正确的、不构成问题。但 `data-design.md` 的表格标题只给 6 个表（`4.4.1 work_item`/`4.20.1 worktree`/`4.21.2 agent_session`/`4.22.1 feedback`/`4.23.1 context_packet`/`4.24.1 validation_result`）标注了"（核心聚合根）"字样，`4.19.1 development_execution`、`4.19.2 change_set`、`4.23.3 decision` 三张表虽然内容完整但表格标题**未**标注"（核心聚合根）"——见下方 D-05。

---

## 未审核 / 超出本次范围

- §11/§12（PoC / Risk 清单）中每条"成功标准"的数值合理性未逐条验证（如"100 Worktree 渲染 < 500ms"是否现实），这类工程可行性判断超出"文档一致性 Review"范畴，建议在 RFC 阶段由实现团队复核。
- 未对 requirements.md 本身做二次审查（如 §13.3 的"16 crates"计数错误、§6 与 §41.1 之间的既有缺陷）——这些已在本会话更早的 `docs/要件定義書.md` 自审中部分覆盖，但未与当前仓库的 `requirements.md` 逐字核对是否完全一致（`requirements.md` 是否为原始要件定義書的精确副本尚未做 diff）。
- mermaid 图语法本身是否可渲染（如 `RT{Realtime (Optional)}` 这类含括号/空格的节点标签是否会导致 mermaid 解析错误）未做渲染测试。
- 附录 C 数据所有权矩阵、§9.4 追踪链查询 API 是否越界进入"外部设计"阶段：经核对，两者均止步于 §0.1 允许范围内的"接口契约签名"与"数据所有权矩阵"，未发现 REST 路径 / 状态码 / DDL 等外部设计产物，判断为**未越界**，不构成 Finding。

**（以下为第二阶段——详细设计审核——新增记录）**

- **审核深度不均**：`api-design.md`、`data-design.md`、`ai-agent-design.md`、`runtime-design.md`、`operation-design.md`、`test-design.md` 六份文档做了较深入的分段通读 + 针对性 grep 交叉核对；`security-design.md`、`integration-design.md`、`external-design.md`、`internal-design.md` 四份文档**仅做了全文 grep 关键词抽样**（tenant_id 13 项、9/8 种白名单命令、AgentSession 14 状态等数字传播链路），未逐章通读，可能遗漏这四份文档内部的结构性问题（例如它们各自的"接口稳定承诺"与"Open Issues"章节是否与上游及彼此一致，未做逐条核对）。
- **各详细设计文档自身"接口稳定承诺"之间的横向一致性**（例如 api-design.md 与 security-design.md 是否对同一 SEC-xxx 错误码给出完全一致的语义）未做逐条 diff，只做了 D-03 涉及的"9 种白名单命令"这一条线索的横向核对。
- **各文档"Open Issues"章节（继承自 basic-design.md §15 J.1-J.15 + 各自新增编号）与上游 J.x 的对应关系**未逐条验证是否有遗漏或重复关闭。
- **mermaid 图语法可渲染性**（第一阶段遗留问题）与 **PoC/Risk 数值可行性判断**（第一阶段遗留问题）在本轮详细设计审核中同样未纳入范围。

---

## 总体结论

### 第一阶段（基本设计）回顾

**Finding 统计**：共 8 条（Major 5：F-01/F-02/F-03/F-05/F-07；Minor 3：F-04/F-06/F-08），另有 1 处笔误（REQ-AUT-001）。**全部已于 commit `81778d9` 修复并逐条核实关闭**（详见"F-01~F-08 修复确认"），仅 F-05 在附录 A.2 mermaid 图留有一处残留缺陷（`IN_PROGRESS→DONE` 直连边缺失）未随正文一并修正。第一阶段的整体结论（K3s 4+1 角色纪律严谨、MVP Must Have 映射无遗漏无多余、未采纳 Non-Goals 排除技术方案）依然成立，不再重复。

### 第二阶段（详细设计,10 份文档）新结论

**Finding 统计**：新增 5 条（D-01~D-05），全部 Major/Minor，无 Blocker：

| Finding | Severity | 引入文档 | 一句话 |
|---|---|---|---|
| D-01 | Major | api-design.md | WorkItem 被引入未定义的 `ARCHIVED` 终态，且与同阶段的 data-design.md 数据字典矛盾 |
| D-02 | Major | ai-agent-design.md | Context Compiler 核心算法遗漏 P4 分桶，且 P5 隔离逻辑自相矛盾 |
| D-03 | Major | api-design.md（源头）→ 传播至另 5 份文档 | "9 种白名单命令"与 basic-design.md 实际的 8 变体枚举不符，混入了另一个独立枚举的概念 |
| D-04 | Major | test-design.md | VAL-001 强制四重门只有 1/4 有对应测试用例，其余 3 项（含防 AI 自我报告的核心防线）零覆盖 |
| D-05 | Minor | data-design.md | 3 张聚合根表遗漏"（核心聚合根）"标题标注 |

**根因归类**：

1. **跨文档"权威数字"未回查源头，靠横向复制传播**（D-03，影响面最广）——`api-design.md` 最先给出"9 种白名单命令 + ReportObservation"的表述，此后 5 份文档（含安全设计、测试设计）原样复制，没有一份文档回头核对 basic-design.md 实际的 `RuntimeCommand` 枚举只有 8 个变体、`ReportObservation` 其实属于另一个独立枚举 `RuntimeObservation`。这是本轮审核发现的**最高优先级问题**：LRT-002/SEC-008 是安全边界核心机制，一旦按"9 种"实现 ACL/单元测试，会引入一个从未在 basic-design.md 中定义过的命令类型。
2. **确定性算法的伪代码与其自身声明的数据结构不同步**（D-02）——ai-agent-design.md 一边在文中/表格里正确使用 P0-P4/P5 taxonomy，一边在最核心的 `compile_context()` 伪代码里悄悄丢掉 P4、把 P5 处理逻辑写重复，属于"写文档时没有对着自己刚定义完的表格核对伪代码"的典型疏漏。
3. **测试设计对 P0 不变量的覆盖存在盲区**（D-04）——VAL-001 是 basic-design.md 明确标注的核心不变量（"防止 Agent 自我报告冒充完成"），但落到 test-design.md 时被简化成了单一维度的 AC 覆盖率检查，四重门里最关键的"负向测试"（条件不满足时必须拒绝）完全缺失。
4. **局部孤立错误 / 标注遗漏**（D-01、D-05）——影响面小、定位精确、可在下一轮编辑中顺带修正。

**优势与值得肯定之处**：跨 10 份文档核对 Module 数(25)、tenant_id 隔离对象数(13)、AgentSession 状态数(14) 三条第一阶段修复的关键数字,传播链路**全部正确、无一遗漏**,`data-design.md` 甚至直接以"F-08 修正"字样注明数字来源,说明下游文档编写时确实基于本反馈文档的 Finding 做了回查,而不是各自独立重算;K8s Tax 纪律(禁止 7 个独立 Deployment、Service/Deployment/Pod 数值上限)在 `operation-design.md` 中被完整继承;`RuntimeCommand` 白名单中"禁止 `ExecuteArbitraryShell`/`ReadArbitraryFile`/`WriteArbitraryFile`"这条核心安全约束本身(区别于 D-03 指出的计数问题)在全部涉及文档中表述一致,未被削弱。

**是否可以进入下一设计阶段（实现/RFC）**：**有条件通过（Conditional Pass）**。建议按以下优先级处理：
1. **必须在实现启动前修复**：D-03（RuntimeCommand 白名单口径,涉及安全边界,一旦按错误口径写 ACL/单元测试返工成本很高,需要先确定是"回填 basic-design.md 补第 9 种命令"还是"6 份详细设计文档改回 8 种"）、D-02（Context Compiler 是 §26.1 要求"确定性、可单元测试"的核心模块,伪代码缺陷会直接传导成实现缺陷）、D-04（VAL-001 是显式 P0 不变量,测试盲区应在写测试代码之前补齐用例设计,而不是实现完再补测试）。
2. **可与常规修订一并处理,不阻塞**：D-01（单点孤立错误）、D-05（标注一致性）、F-05 残留的 A.2 图示缺陷（basic-design.md 自身的小缺陷,建议随下一次 basic-design.md 修订一并处理）。
3. 未深入审核的 4 份文档（security-design.md/integration-design.md/external-design.md/internal-design.md）目前只做了关键字级别抽样,建议后续单独安排一轮通读级别的审核,不建议现在就据此认定它们"合格"。
