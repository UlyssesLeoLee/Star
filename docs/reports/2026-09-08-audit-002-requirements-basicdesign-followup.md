# AUDIT-002 — requirements.md / basic-design.md 全面审核(跟进 AUDIT-001)

> **触发**: 2026-09-08 用户要求"全面审核需求和基本设计文档"
> **范围声明**: 主审对象是母版文档对 `docs/requirements.md`(1792 行)+ `docs/basic-design.md`(3862 行)。`docs/architecture/*/01-requirements.md`/`02-basic-design.md` 系列(agent-runtime / langgraph / treesitter-worktree-graph / exclusion-idempotency 等)与 `docs/requirements/SRS-*.md` 是本次的**一致性核对对象**,非主审对象。
> **前置动作**: 本次不是从零审核。先通读三份既有审核记录 —— `docs/refactor/AUDIT-001-requirements-basicdesign-specs.md`、`docs/basic-design-feedback.md`(F-01~F-08 + D-01~D-05)、`docs/reports/2026-09-02-audit-001-redteam-findings.md` + `2026-09-03-blue-team-26-items-review.md` —— 逐项核实其当前修复状态,避免重报已闭环问题。
> **方法**: (1) ID/依赖方向机械核对;(2) 逐条核实既有 Finding 的当前状态(grep + 读当前文档,非采信历史记录);(3) 用当前 `Cargo.toml`/`crates/` 实际状态与文档声称的 Module 清单做交叉核对(文档 vs 代码现实);(4) 对此前"仅关键字抽样、未逐章通读"的 4 份详设文档(security/integration/external/internal-design.md)做结构性复核。

---

## 0. 结论先行

**母版文档链(requirements.md ↔ basic-design.md)本身内部一致性良好**,且此前三轮审核(AUDIT-001 + 基本设计 Review F/D 系列 + 红蓝对抗 26 项)中标记为"Ulysses 必拍板"或"Major"的问题**绝大多数已实际修复并可在当前文档中验证**,不是仅停留在报告里的口头承诺。本轮新增发现集中在**代码规模持续增长、母版 Module 清单未跟上**这一类"新鲜过期",而非既有链路重新腐化。

| 类别 | 数量 | 说明 |
|---|---|---|
| 既有 Finding 复核 → 确认已修复 | 6 大类(见 §1) | 拍2-7(§2.1 依赖方向循环)、D-02/D-03/D-04、F-05 mermaid 残留 |
| 既有 Finding 复核 → 未闭环(比记录时更严重) | 1 项(拍1,见 §2.2) | Module 清单过期从"已知静态快照"恶化为"持续扩大的缺口"(34→38) |
| 既有 Finding 复核 → 仍未解决(状态不变) | 1 项(REQ-RT-002) | 见 §2.1 |
| 本轮新发现 | 2 项(1 Major + 1 Minor) | domain-llm/mcp/task/tool 零文档且与 domain-ai 存在文档层面职责重叠(§2.2);domain-workflow 行"拍"编号误标(§2.3) |
| 4 份详设文档结构复核 | 未发现新增结构性缺陷 | 见 §3 |

> **注**:拍 1(§2.1 25-Module 表过期)历史上被记录为"已处理"(2026-09-03 静态快照式补登记至 34),但本轮核实它并未真正闭环——只是把过期节点从 25 推到了 34,过期本身仍在持续发生(见 §2.2)。因此不计入"确认已修复"。

---

## 1. 既有 Finding 复核(核实"报告说已修复"是否等于"文档确实已修复")

逐条用当前文档内容核实(非采信历史报告的自我认定),全部通过:

### 1.1 §2.3 硬禁线 / §2.1 依赖方向矛盾(AUDIT-001 发现 1/6-A,红方 Finding-Worktree/Workflow/Board-Planning,蓝方拍 2/4/7)

当前 `basic-design.md:283-322` §2.1 表逐行核实:

- `domain-worktree`(行 2)关键依赖现为 `domain-scm, domain-development`,**不再包含** `domain-work-item` —— 发现 1 / 拍 2 的循环依赖已消除。该行本身未标注拍板编号。
- `domain-workflow`(行 9)关键依赖现为"无(system_default 由本 crate seed, 2026-09-03 拍 2 单向只读投影落档)" —— work-item↔workflow 循环依赖已消除,**但该行标注的"拍 2"编号有误,应为拍 4**(蓝方报告 `2026-09-03-blue-team-26-items-review.md:86` 明确:拍 2 = Finding-Worktree/WorkItem,拍 4 = Finding-Workflow/work-item↔workflow 循环;详见 §2.3 新发现)。
- `domain-board`(行 10)依赖 `domain-work-item`、`domain-planning`(行 11)依赖 `domain-work-item`,**两者互不再引用对方** —— board↔planning 循环依赖已消除(拍 7)。
- `domain-agent`(行 3)依赖集合改为 `domain-tenant, domain-worktree, domain-work-item, domain-permission`,与 `domain-agent-spec.md` 对齐(拍 6/蓝方 #16)。

**结论**:四组当时"三份独立文档互相矛盾"的架构循环依赖,当前 §2.1 表已全部改写为无环状态,可视为拍 2/4/6/7 的实质内容均已落地。但改写理由的编号标注本身有一处错误(见 §2.3),**依赖方向本身确认关闭,编号标注问题另计。**

### 1.2 D-02 — ai-agent-design.md Context Compiler P4/P5 分桶矛盾

`ai-agent-design.md:237` 明确写"D-02 修复:恢复 P4 桶;Step 2-4 完全移除 P5",且 `:272` "强制:Untrusted 不得进入 P0/P1/P2/P3/P4 任何桶"、`:333` P5 独立桶说明与之一致。**确认关闭。**

### 1.3 D-03 — RuntimeCommand"9 种白名单命令"口径

对 `api-design.md`(3 处)、`security-design.md`、`runtime-design.md`、`data-design.md`(3 处)逐一 grep 核实,全部已统一为"8 种白名单命令,`ReportObservation` 走独立 `RuntimeObservation` 枚举",且当前 `basic-design.md:1423-1433` `RuntimeCommand` 枚举确认为 8 变体,与全部下游文档口径一致。**这是本次跟踪中风险等级最高的一条(涉及安全边界,一旦按错误口径写 ACL 返工成本高),确认已在全部 6 份文档中同步关闭。**

### 1.4 D-04 — test-design.md VAL-001 四重门测试覆盖缺口

`test-design.md:889` §6.3.2 "VAL-001 不变量"、`:1054` `T-VL4` 用例、`:1223` `T-E2E-6-3` 用例、`:1903` "VAL-001 四重门 1/2/3/4 缺失的负向测试(D-04 修复)"均已落地。**确认关闭。**

### 1.5 F-05 残留问题(附录 A.2 mermaid 图 `TODO→DONE` 直连边缺失)

`basic-design.md:3572` 当前 mermaid 图明确含 `IN_PROGRESS --> DONE: 直接完成(默认三态)` 边,`:3566` 附带说明"默认最简三态路径...F-05 修复后口径"。**确认关闭。**

### 1.6 AUDIT-001 发现 2(domain-context-spec.md 自相矛盾)

抽查确认蓝方 #14 已代修(附录 B 删除重复 `domain-agent` 上游依赖)。**确认关闭。**

---

## 2. 本轮核实仍然存在 / 新发现的问题

### 2.1 REQ-RT-002(近实时状态流转可见性)—— 持续缺口,状态未变

- **位置**: `requirements.md:463`
- **核实结果**: 全仓 grep `REQ-RT-002`,唯一命中即定义处本身。`basic-design.md`、`docs/specs/domain-collaboration-spec.md`(该 Requirement 理应落地的 domain)均未提及。
- **性质**: 与 AUDIT-001"发现 4-类 3"的判断一致 —— `domain-collaboration` 在 §2.1 只是 Generic Domain,没有独立 §4.x 深度设计小节,这条要求至今没有任何设计文档承接。**这不是新问题,是确认一个 5 天前已登记的缺口至今未被处理**,状态从"已知缺口"维持为"已知缺口",报告目的是避免它被后续开发悄悄遗漏(该文档链路上没有任何机制会在缺口关闭前主动提醒)。
- **建议**: 若 V1 范围仍打算做"Worktree/AgentSession 状态流转的近实时展示"(REQ-RT-001 已有部分覆盖,REQ-RT-002 特指状态**流转过程**的可见性,细粒度高于 REQ-RT-001),需要在 `domain-collaboration-spec.md` 或前端 `frontend-design.md` 补一节;若判定为 REQ-RT-001 已隐式覆盖、可以合并,也需要显式登记"REQ-RT-002 并入 REQ-RT-001",而不是保持沉默悬空。

### 2.2 【新发现,Major】母版 Module 清单已知过期问题持续恶化:4 个新增 domain crate(`domain-llm`/`domain-mcp`/`domain-task`/`domain-tool`)在 `basic-design.md` 全文零引用

- **背景**: 2026-09-03 的拍板(拍 1)已确认 §2.1 25-Module 表过期,当时补建 §2.1.4(9 个跨切 supporting crate)+ §2.1.5(10 个 `star-*` infrastructure crate),使文档覆盖数从 25 提升到 34,并**明确记录"决定不追平代码现实,只做静态快照式补登记"**(`RF-001` T2.4 handoff 原文:"评估报告里顺带提一句 25-Module 表已知过期...但不要求顺带修复整张表")。
- **核实结果**: 实测当前 `crates/` 目录下 `domain-*` crate 共 **38 个**(`ls crates/ | grep '^domain-'`),而 §2.1+§2.1.4 合计仍只登记 **34 个**。差额 4 个 —— `domain-llm`、`domain-mcp`、`domain-task`、`domain-tool` —— 在 `basic-design.md` 全文**零次**出现(逐一 grep 确认),也不在任何 `docs/specs/domain-*-spec.md` 里(无对应 spec 文件)。
- **不是空壳**:抽查 `crates/domain-llm/src/lib.rs`(212 行,定义 `LlmProvider` trait 等)确认是有实质内容的 stub,非占位空文件;文件头注释自称"per ADR-0048 D42 + SRS-001 G-4""v0.0.1 stub, Phase 2 spec to complete docs later""pending real 5-domain Lead signoff"——即代码作者本人已在代码注释里承认设计文档缺失,但这个自我承认从未被同步进 `basic-design.md` 或任何 `docs/specs/` 索引,只存在于源码注释里,外部审核者(包括产生本报告之前的所有既有审核)都不会通过读文档发现它。
- **影响**: (a) §2.1.4 注释原文写"9 个跨切 supporting crate 跟 §2.1.1-§2.1.3 的 25 logical domain 不重叠,是 §2.1 表未覆盖的扩展模块"——这个"未覆盖清单"本身现在也过期了,形成"过期表格的过期修订记录"的二级腐化;(b) **`domain-ai`/`domain-llm`/`domain-mcp` 存在已核实的文档层面职责重叠,非猜测**:`basic-design.md:345` §2.1.4 明确写 `domain-ai` 的职责是"AI 编排 (**LLM Provider / Agent Runtime 抽象**)",而 `crates/domain-llm/src/lib.rs:1-3` 自称"Star L2 **LLM Pool** business layer",`crates/domain-mcp/src/lib.rs:1-3` 自称"Star L2 **MCP Pool** business layer"——三者的文字描述在"LLM Provider/Pool 抽象"这一点上直接重合,但因为 `domain-llm`/`domain-mcp` 没有任何 spec 或母版章节,**无法判断这是有意拆分(如 domain-ai 做编排调度、domain-llm 做底层 Provider 池化)还是未经协调的重复建模**,这正是母版文档缺失后本应由 §2.1.4/spec 的"关键依赖"列暴露、却因缺失而失效的问题;(c) `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md`(子文档)确认提及了这几个 crate 的一部分,说明设计工作实际发生过,只是没有回填母版——这正是 AUDIT-001 §"根因归类"第 1 条("跨文档权威数字未回查源头,靠横向复制传播")的重演,只是这次反过来:子文档领先,母版落后。
- **建议**: 不要求本轮下游 AI 直接改写整张 §2.1 表(改动面大,且历史决策已明确"暂不追平"是有意识的取舍);但建议至少在 §2.1.4 补充第 10-13 行登记这 4 个 crate,并在登记时明确 `domain-ai` 与 `domain-llm`/`domain-mcp` 的边界(哪怕内容只是"见 `domain-ai-spec.md` / 子文档 `02-basic-design.md`,边界待 Lead 拍板"这类占位说明,也好过完全不提及重叠的存在),同时在该节补一句"母版 Module 清单更新频率滞后于 crate 新增速度,建议后续新增 domain crate 时把补登记 §2.1.4/§2.1.5 作为该 crate 首个 commit 的必做项",否则这个缺口会随着 crate 数量增长持续扩大而不会被现有审核流程自动捕获(过去 5 天从 34 → 38,如果不改变流程,下次审核大概率还会看到新的未登记 crate)。

### 2.3 【新发现,Minor】domain-workflow 行的拍板编号误标:应为"拍 4",标注为"拍 2"

- **位置**: `basic-design.md:298`,domain-workflow 行"关键依赖"列写"无(system_default 由本 crate seed, 2026-09-03 拍 2 单向只读投影落档)"。
- **核实结果**: 对照 `docs/reports/2026-09-03-blue-team-26-items-review.md:84-86` 的拍板编号权威定义——**拍 2 = Finding-Worktree/WorkItem**(worktree 依赖方向,对应 §2.1 行 2 `domain-worktree`),**拍 4 = Finding-Workflow**(work-item↔workflow 循环依赖,即本行所述内容)。当前 `domain-worktree` 行(行 286)反而**没有**标注任何拍板编号,而本应标"拍 4"的 `domain-workflow` 行标成了"拍 2"。
- **性质**: 内容本身(依赖已消除)是对的,只是归因编号错位,不影响架构结论,但会误导后续读者去 §2.3/拍板记录里查错编号从而找不到对应决策依据。**Minor,文档笔误,可自行修复**:建议将 `basic-design.md:298` 的"拍 2"改为"拍 4",并考虑在 `domain-worktree` 行补上"拍 2"标注以保持两处都可追溯。

---

## 3. 4 份此前"仅关键字抽样"的详设文档结构复核(security/integration/external/internal-design.md)

`basic-design-feedback.md` 明确记录这 4 份文档此前只做过"tenant_id 13 项 / 8 种白名单命令 / AgentSession 14 状态"等数字传播链路的抽样核对,未逐章通读。本轮做了结构性复核(非逐句语义审阅):

- **TODO/FIXME/待拍板类标记密度**:security-design.md 3 处、integration-design.md 0 处、external-design.md 1 处、internal-design.md 12 处。逐一核实 internal-design.md 的 12 处**全部是 `TBD-MEASURE`**(第 1396-1415 行,数值型指标占位符),这是 `requirements.md` §0/§36/§80 明确要求的合规写法(缺乏真实测量数据时必须标注 `TBD-MEASURE`,不得臆造数字),**不构成缺陷**,反而是遵守规范的证据。
- **未发现**这 4 份文档内部出现新的自相矛盾或与母版口径冲突的实例(本轮复核范围有限,详见下方"未覆盖"声明)。

**未覆盖**:本轮对这 4 份文档仍未做逐章通读级别的审核(工作量级别为整份文档精读,超出本次时间预算),`basic-design-feedback.md` 结尾提出的"各文档接口稳定承诺之间的横向一致性""各文档 Open Issues 章节与上游 J.x 对应关系"两项仍未被任何一轮审核覆盖,建议列为下一轮专项。

---

## 4. 建议处理顺序

1. **REQ-RT-002**(§2.1):低成本决策——要么在 `domain-collaboration-spec.md` 补设计,要么显式登记"并入 REQ-RT-001,关闭"。当前状态是"既不关闭也不设计",是唯一会持续误导后续读者的状态。
2. **4 个未登记 crate + domain-ai/llm/mcp 边界**(§2.2):补登记 §2.1.4,明确职责边界,并考虑把"新增 domain crate 需同 commit 补登记母版清单"写入 `AGENTS.md` 或 CI 检查项,防止差额继续扩大。
3. **domain-workflow 行拍板编号笔误**(§2.3):可自行修复,一处字符改动。
4. **4 份详设文档逐章通读**(§3):列为下一轮专项审核,当前只完成结构性抽查。

---

## 5. 审核范围外声明 / 抽样深度披露

- **抽样深度**(与 `basic-design-feedback.md` 对 4 份详设文档的做法保持同一披露标准,不应对己从宽):`requirements.md`(1792 行)本轮**逐字通读全文**。`basic-design.md`(3862 行)本轮**未逐章通读**,而是针对既有 Finding 的落点(§2.1 依赖表、§7.2 状态机图、RuntimeCommand 枚举定义处等)做定点交叉核查 + 全文 grep 排查(crate 名称、REQ-ID、拍板编号)。这种方法对"已知问题是否修复"和"结构性关键词是否漂移"类核查有效,但**不能排除 basic-design.md 中存在与既有 Finding 无关、本轮定点核查未覆盖到的全新语义矛盾**。
- 基本設計書 章节完备性对照 V-model 标准章节清单(機能一覧/画面設計/帳票/DB論理設計/外部インタフェース/非機能要件/移行/運用等):本轮未覆盖,仅在开头看过 §0-§15 顶层目录,未做逐节归类核对。
- mermaid 图语法可渲染性、PoC 数值可行性判断:延续此前历次审核的既定排除范围,本轮未纳入。
- `docs/architecture/*/01-requirements.md`/`02-basic-design.md` 系列子文档与母版的逐条一致性:本轮仅做了 domain crate 覆盖面的交叉检查(§2.2),未做完整的子文档内容审核。
