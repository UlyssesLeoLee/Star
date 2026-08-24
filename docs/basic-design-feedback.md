# 基本设计書 Review Feedback

> **审核对象**: `docs/basic-design.md` v0.1 (2026-08-25)，3550 行
> **基线**: `docs/requirements.md` v2.0（本文档头部声明的唯一上游，"下文以 §N 引用"即指该文档的章节号）
> **审核方法**: 沿用 requirements.md §45 已确立的专项 Review Lens（Product / Context Engineering / Agent Security / Development Runtime），并新增机制化检查（引用可解析性、Module 枚举完备性、Non-Goals 违反扫描、P0/ARCH-OBL/MVP 覆盖矩阵）。逐章通读 + 全文 grep 交叉核对，边读边记录，未在末尾一次性回忆。
> **审核者**: Claude（本会话），非 basic-design.md 原作者。本文档只产出 Finding，不直接修改 `basic-design.md`。

---

## 使用说明

- 每条 Finding：`ID | Severity | 位置 | 违反的依据 | 期望修正`。
- Severity：**Blocker**（违反不可推翻的架构原则/义务，或内部自相矛盾导致下游无法施工）/ **Major**（覆盖缺口、命名/编号错误，影响可追溯性但不阻断施工）/ **Minor**（措辞、格式、局部不一致）/ **Question**（需要作者澄清的设计判断，非缺陷）。
- 位置格式：`basic-design.md:§章节号` + 行号。

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

## 已完成的补充检查（未发现新增问题，记录以避免重复审查）

- **crate 总数口径**：requirements.md §13.3 自身即存在"声称 16 crates,实际列出 17 domain crate + 3 支撑 crate = 20"的计数错误（这是 requirements.md 自身的既有缺陷,非 basic-design.md 引入）。basic-design.md 没有重复"16"这个错误数字，而是重新给出 24(见 F-01/F-03,该数字本身仍有 domain-local-runtime 未计入的问题)，即 basic-design.md 并未原样照抄 requirements.md 的错误计数，值得肯定。
- **K3s 部署角色**：§8.1/§8.2 反复重申 `gateway / identity / work-core / worker` 四角色 + `realtime`(条件性)的边界，§8.2 明确逐一列出 7 个被禁止的独立 Deployment 名称并逐一说明其收敛去向，未发现任何隐藏的额外常驻 Deployment。符合 §44.2 K8s Tax 纪律。
- **ch7 状态机文本表 与 附录 A mermaid 图**：内容互补（前者列"触发者/迁移条件"表格，后者是可视化状态图），未发现状态集合本身相互矛盾（除 F-05/F-08 指出的计数错误外，两处对同一状态机的描述是一致的）。不构成结构性重复。
- **P0 Requirement / ARCH-OBL-DEV-001~006 覆盖**：WT-001~003、AGT-001/002、FBK-001/002、CTX-001/002、VAL-001、SCM-001、LRT-001/002、SEC-xxx 及全部 6 条 ARCH-OBL-DEV 均在文中至少出现一次（部分以 `WT-001~003` 等区间记法出现，非遗漏）。**唯一例外**：§2.1 表格第 6 行引用 "REQ-AUT-001"（`basic-design.md:289`），既不匹配 requirements.md §41.1 定义的任何前缀，也不是该行实际对应的 P0 ID（该行描述"AI 自我报告不构成完成"，按 §41.2 应为 **VAL-001**）。判断为笔误：应改为 `VAL-001`，与同页 §4.5.6 的正确引用保持一致。（Minor，并入本条不单独编号）
- **MVP Must Have 映射完整性**：§13.1 表格 21 行与 requirements.md §30.2 原文 21 个条目逐一对应，**无遗漏、无多余**，是全文交叉引用质量最高的一节。
- **DDL / 生产代码越界扫描**：全文未出现 `CREATE TABLE`/`ALTER TABLE` 等 DDL，Rust 代码块全部止步于 `trait`/`struct`/`enum` 签名，未出现函数体实现，符合 §0.1 声明的"本文档不输出生产代码"自我约束。
- **Non-Goals 违反扫描**：全文出现的 Graph Database / Vector Database / OpenSearch / Service Mesh / Event Sourcing / CQRS / Database per Domain 均以"明确排除"的方式出现（ADR 选项 C 被拒绝、§13.5 强化排除清单等），未发现将其作为实际设计选择采纳的情况。

---

## 未审核 / 超出本次范围

- §11/§12（PoC / Risk 清单）中每条"成功标准"的数值合理性未逐条验证（如"100 Worktree 渲染 < 500ms"是否现实），这类工程可行性判断超出"文档一致性 Review"范畴，建议在 RFC 阶段由实现团队复核。
- 未对 requirements.md 本身做二次审查（如 §13.3 的"16 crates"计数错误、§6 与 §41.1 之间的既有缺陷）——这些已在本会话更早的 `docs/要件定義書.md` 自审中部分覆盖，但未与当前仓库的 `requirements.md` 逐字核对是否完全一致（`requirements.md` 是否为原始要件定義書的精确副本尚未做 diff）。
- mermaid 图语法本身是否可渲染（如 `RT{Realtime (Optional)}` 这类含括号/空格的节点标签是否会导致 mermaid 解析错误）未做渲染测试。
- 附录 C 数据所有权矩阵、§9.4 追踪链查询 API 是否越界进入"外部设计"阶段：经核对，两者均止步于 §0.1 允许范围内的"接口契约签名"与"数据所有权矩阵"，未发现 REST 路径 / 状态码 / DDL 等外部设计产物，判断为**未越界**，不构成 Finding。

---

## 总体结论

**Finding 统计**：共 8 条（Major 5：F-01/F-02/F-03/F-05/F-07；Minor 3：F-04/F-06/F-08），另有 1 处笔误（REQ-AUT-001，并入补充检查未单独编号）。无 Blocker，无 Question。

**根因归类**：8 条 Finding 可归为 3 类根本原因，而非 8 个独立问题：

1. **Module 枚举与详细设计范围不同步**（F-03、F-07，及 F-01/F-02 间接相关）——`domain-local-runtime` 在"总览类"章节（§1.2/§2.1/§2.3、附录 B、附录 C）中被系统性遗漏，却在"详细设计类"章节（§4.6、§4.10.4、§6.1、§8.5、接口稳定承诺 #1）中被当作一等 Module 使用。这是本次审核发现的**最高优先级问题**：它不是某一处的笔误，而是贯穿全文 6+ 处的结构性不一致，且已被"接口稳定承诺"锁定为 24（应为 25），若不在下一阶段之前修正，后续 RFC/详细设计会继承错误的 crate 边界。
2. **状态机"默认值"与 requirements.md 强约束的口径漂移**（F-05，及 F-08 的计数错误）——WorkItem 默认状态机把 REQ-WF-001 明确要求"默认三态、扩展可选"的边界移动了，AgentSession 状态计数少算一个。两者都已被写入"接口稳定承诺"，建议**在基本设计冻结前必须修正**，优先级仅次于第 1 类。
3. **局部计数 / 引用笔误**（F-04、F-06、REQ-AUT-001）——影响面小、定位精确、修改成本低，可在下一轮编辑中顺带修正，不阻断评审通过。

**优势与值得肯定之处**（详见"已完成的补充检查"）：MVP Must Have 映射（§13.1）与 requirements.md §30.2 逐条对应、无遗漏无多余；未出现生产代码/DDL 越界；未采纳任何 Non-Goals 中明确排除的技术方案（Graph DB/Vector DB/Service Mesh 等）；K3s 部署角色严格控制在 4+1 个，符合 §44.2 K8s Tax 纪律；Open Issues（J.1-J.15）主动暴露了作者自知的缺口，而非掩盖。整体上 basic-design.md 在**范围裁剪、K8s 纪律、MVP 可追溯性**三个维度执行得比较严谨，问题集中在**Module 边界的自洽性**与**状态机默认值的口径**两点。

**是否可以进入下一设计阶段（详细设计/RFC）**：**有条件通过（Conditional Pass）**。建议先修复 F-01/F-02/F-03/F-07（同一根因，建议一次性合并修复：明确 domain-local-runtime 定位、把 Domain 总数统一为 25、补齐 §1.2/§2.1/§2.3/附录 B/附录 C 中缺失的行）与 F-05（WorkItem 默认状态机改回三态），这两组问题会直接影响下游 crate 划分和 Workflow 实现，返工成本远高于现在修复的成本；F-04/F-06/F-08 及 REQ-AUT-001 笔误可与下一版本修订一并处理，不必阻塞评审通过。
