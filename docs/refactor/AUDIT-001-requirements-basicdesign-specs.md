# AUDIT-001 — requirements.md / basic-design.md / docs/specs 一致性核查

> **触发**: 2026-09-02 用户要求核对 `docs/requirements.md`、`docs/basic-design.md` 与详细设计层(`docs/specs/domain-*-spec.md`)之间的出入
> **范围声明**: 本次核查对象是 **后端** 三层文档链(`requirements.md` → `basic-design.md` → `docs/specs/domain-*-spec.md`)。仓库里另有 `docs/internal-design.md`/`docs/external-design.md` 两份标题含"详细设计"的文档,但内容是**前端**组件级/UI-UX 详设,且已自带"上游同步"小节独立追踪一致性,不在本次范围内。
> **方法**: 结构级交叉核对(REQ-ID 双向引用、§ 引用是否存在、依赖方向是否违反硬约束),未做逐句语义审阅。**只报告,不改 `requirements.md`/`basic-design.md`**——两者是主权威文档,与 spec 的出入多数情况下是"待拍板该改哪边",不是错别字。

---

## 发现 1(高优先级)—— `basic-design.md` 自身 §2.1 与 §2.3 矛盾,并被 spec 忠实继承

- `basic-design.md` §2.1 表(Domain 列表)第 2 行:`domain-worktree` 的"关键依赖"列出 `domain-work-item, domain-scm, domain-development`。
- 同一文档 §2.3"禁线"明确写:`❌ domain-worktree → domain-work-item(状态独立,不允许反向写)`。
- `docs/specs/domain-worktree-spec.md` 附录 B"边界清单"→"上游依赖"行原样列出 `domain-work-item`,与 §2.1 一致、但违反 §2.3。

**性质**:`basic-design.md` 内部两处(§2.1 与 §2.3)对同一条边的方向描述互相矛盾,spec 只是忠实转录了 §2.1 版本。**需要 Ulysses 拍板**:要么 §2.1 表这一格是笔误(应删掉 `domain-work-item`,或改为只读投影而非"依赖"),要么 §2.3 的禁线描述需要补充例外条件(例如"仅允许只读投影,不允许写")。在拍板前,`domain-worktree-spec.md` 不应擅自改动这一行。

## 发现 2(高优先级)—— `domain-context-spec.md` 自相矛盾,且违反 `basic-design.md` §2.3 禁线

- `basic-design.md` §2.3 禁线:`❌ domain-context → domain-agent(Context 是 Agent 输入,不依赖 Agent 内部)`。
- `docs/specs/domain-context-spec.md` §1"不属于本 crate 的"明确写:`AgentSession(由 domain-agent 拥有,本 Module 输出 Context Packet 给其消费)`——即 context 只是把 Packet 输出给 agent 消费,agent 是下游。
- 但同一文件附录 B"边界清单"→"上游依赖"行却把 `domain-agent` 也列了进去(`domain-tenant, domain-work-item, domain-worktree, domain-feedback, domain-validation, domain-development (SymbolIndex), domain-agent`),"下游调用"行**又重复列了一次** `domain-agent`。
- 核对 `basic-design.md` §2.1 表第 5 行(`domain-context` 关键依赖):只有 `domain-work-item, domain-worktree, domain-feedback, domain-validation`,**没有** `domain-agent`。说明这不是从 §2.1 表继承来的,是 spec 自己附录 B 写错/写重了。

**性质**:与发现 1 不同,这处矛盾**不是**从 `basic-design.md` 继承的,是 `domain-context-spec.md` 自身笔误(同一 crate 依赖表里把 agent 同时列为上游和下游,且与本文件 §1 正文冲突)。**建议下游 AI 可直接修**:附录 B"上游依赖"行删除 `domain-agent`(与 §1 正文、与 basic-design §2.1 表对齐),"下游调用"行保留即可。这属于 spec 内部笔误修正,不涉及改动 `basic-design.md`,风险低。

## 发现 3(低优先级,非缺陷)—— 部分"下游调用"行把事件订阅标成"调用",容易引起误判

排查 §2.3 另外两条禁线(`❌ domain-scm → domain-worktree`、`❌ domain-feedback → domain-context`)时,一度以为 `domain-scm-spec.md`/`domain-feedback-spec.md` 也违反,但对照 §5"Domain Events"章节后确认:两处都是 NATS 事件发布 + 对方订阅(`domain-worktree` 订阅 `pull_request.linked`;`domain-context` 订阅 Feedback `created`),调用方向实际是订阅方主动拉取,不违反禁线。只是附录 B 用"下游调用"一词描述"谁订阅了本 crate 的事件",容易和"本 crate 主动调用谁"混淆。**不要求修改**,仅记录以免下次审计重复排查。

## 发现 4 —— REQ-ID 双向核对

**Spec 引用了、`requirements.md` 里查不到 `REQ-` 前缀形式,但复核后确认是命名规范不统一(已解决,非悬空引用)**:
| Spec 里的写法 | `requirements.md` 里的实际定义 |
|---|---|
| `REQ-FBK-001`(`domain-comment-spec.md`, `domain-feedback-spec.md`) | `requirements.md:1418` 表格里登记为裸编号 `FBK-001`(章节标题 `requirements.md:808` 也写 `FBK-001/002`,不带 `REQ-` 前缀) |
| `REQ-WT-001`(`domain-worktree-spec.md`) | `requirements.md:1413` 表格里登记为裸编号 `WT-001`(章节标题 `requirements.md:620` 也写 `WT-001~003`,不带 `REQ-` 前缀) |

两个 ID 本体都存在、内容对得上,只是 `requirements.md` 全篇给 Worktree/Feedback 这两类需求用了不带 `REQ-` 前缀的裸编号,而 spec 引用时加了 `REQ-` 前缀。**不是缺失定义**,是全仓 REQ-ID 命名前缀不统一(多数编号如 `REQ-WI-001` 带前缀,`WT-*`/`FBK-*` 不带),建议后续统一加前缀,但不影响本次审计结论,不再列入待拍板项。

**`requirements.md` 定义了、但从未被任何 `docs/specs/*.md` 引用**(53 个中 21 个),抽查后分三类:

1. **已被 `basic-design.md` 吸收但明确标注"未进入深度设计 Module"的 V1 候选**,spec 暂不引用是符合预期的,非缺陷:
   - `REQ-AUTO-002`(Schedule/Cron Trigger)—— `basic-design.md` §2.1 原文自己写"未进入本章 10 个深度设计 Module,先在本表与 §5.6 事件清单中登记"。

2. **`basic-design.md` 已经写了具体设计、但对应 spec 没跟上**(真实的详设遗漏,建议补):
   - `REQ-NOTIF-003`(Watcher 覆盖降噪规则)—— `basic-design.md` 有明确一句设计("用户加 Watcher 后即使不满足降噪触发条件也收关键事件"),`domain-notification-spec.md` 完全没提。
   - `REQ-WI-001`(WorkItem Labels/Components 字段)—— `requirements.md` 标注"**已实现**"并给出代码行号(`crates/domain-work-item/src/entity.rs:100,103` 等),但 `domain-work-item-spec.md` 未描述这两个字段,spec 落后于已合入代码。

3. **`basic-design.md` 自己也没提到、需要先确认是否遗漏还是 MVP 外**:
   - `REQ-DSG-001/002`(DesignArtifact + Approval Guard)—— `requirements.md` 原文自己注明"非强制瀑布……故不列入 §30.2 Must Have",大概率是有意延后,非缺陷。
   - `REQ-RT-002`(近实时状态流转可见性)—— `basic-design.md`/`docs/specs/` 均未出现。`domain-collaboration` 在 §2.1 里只是 Generic Domain,没有独立 §4.x 深度设计小节,这条要求目前没有任何设计文档承接,**建议登记为真实缺口**。
   - 其余(`REQ-COLLAB-003/004`、`REQ-DATA-001/003`、`REQ-OPS-*`、`REQ-PLAN-002/005~008`、`REQ-TST-001/002`、`REQ-TWP-001/002`)未逐条深挖,`REQ-DATA-*` 大概率在 `docs/data-design.md` 而非 domain spec 里承接,`REQ-OPS-*`/`REQ-TST-*` 可能属于运维/测试治理类、不需要 domain spec,这批建议后续单独一轮核查,不在本次结论内下判断。

## 发现 5 —— § 引用完整性核查结果:未发现问题(记录一次过程性乌龙)

最初用 grep 抽取 `《Basic Design》§X` 时,因为正则在逗号处截断,误判"所有 domain spec 都只引用了 §2.1、没引用自己的 §4.x 详设小节"。用不截断的方式逐文件重新核对后确认**这是假阳性**:§2.1 表覆盖的 25 个 domain 里,只有 `domain-audit`、`domain-automation`、`domain-collaboration`、`domain-comment`、`domain-notification`、`domain-search`、`domain-workspace` 这 7 个 spec 完全没有引用任何 `§4.x`(均只引用 `§2.1`/`§5.7`/`§3.2.1` 等)。用 `grep -n "^### 4\." docs/basic-design.md` 核对 `basic-design.md` 全文后确认:全文只有 §4.1~§4.13 共 13 个 `§4.x` 深度设计小节(worktree、agent、feedback、context、validation、local-runtime、scm、development、work-item+workflow+board+planning、permission&security、跨域协作、event bus、realtime),这 7 个 domain 本来就**没有**对应的 §4.x 小节——`basic-design.md:335` 自己也说明 §2.1 表里部分条目"未进入本章 10 个深度设计 Module"。所以这 7 个 spec 不引用 §4.x 是文档本身的覆盖范围决定的,**不是遗漏**。其余全部正确引用了自己的 §4.x 章节(如 worktree→§4.1、agent→§4.2、context→§4.4、validation→§4.5、**workflow→§4.9.2**、`planning→§4.9.2/§4.9.4`、`relation→§4.9.4`……),头部引用链完整,不需要修复(`domain-workflow-spec.md:6` 明确写了 `§4.9.2`,复核时一度怀疑它缺失,核对原文后确认是复核误判,已排除;`domain-planning`/`domain-relation` 此前一度被误列为"无 §4.x 引用"的例外,复核 `basic-design-citations.txt` 第 20/22 行后确认它们分别引用了 `§4.9.2/§4.9.4`、`§4.9.4`,并非例外,已更正)。

另有 `domain-ai/cli/dashboard/form/kms/report/theme-spec.md` 共 7 个文件完全没有"Basic Design"引用行——核实后确认这 7 个是 basic-design §6"22 logical domain + 7 supporting crate"里的 **supporting crate**,不在 §2.1 表(只收录 22 个 logical domain)覆盖范围内,走的是另一套引用体系(引用 `basic-design.md` §6,而非 §2.1/§4.x),**不属于遗漏,是设计使然**,不计入本发现的核查范围。

记录此项是为了避免下次审计重复走这条弯路。

## 发现 6 —— `basic-design.md` §2.1 表"关键依赖"列 与 25 个核心 domain spec 附录 B"上游依赖"行系统性交叉核对

**方法**:此前发现 1/2/3 都是抽查 §2.3 禁线涉及的 4~5 条边时顺带发现的,没有做全表核对。这里把 §2.1 表(25 行,`basic-design.md:285-319`)每行"关键依赖"列,与对应 spec 附录 B"上游依赖"行逐一比对(供应链/技术设施类的 `domain-tenant` 几乎全表统一省略,视为约定俗成的隐式依赖,不计入不一致)。**列方向已核实无歧义**:用 `domain-tenant`(第 18 行,关键依赖 = 无)、`domain-workspace`(第 19 行,= `domain-tenant`)、`domain-project`(第 20 行,= `domain-tenant, domain-workspace`)这条已知的底层依赖链做基线,确认"关键依赖"列在表内统一读作"本行 domain 依赖列出的 domain"(正向),不存在反向读法;§2.3 硬约束图里的 `A ← B` 用同一条基线链解码,同样统一表示"B 依赖 A"。因此下面列出的表内矛盾是真实的数据冲突,不是列语义问题。

**A. 高置信度 —— 表自身内部循环依赖,且两侧 spec 各自独立否认**

- §2.1 表第 10 行(`basic-design.md:299`,`domain-board`)"关键依赖"列出 `domain-work-item, domain-planning`;第 11 行(`basic-design.md:300`,`domain-planning`)"关键依赖"又列出 `domain-work-item, domain-board`——**表内部 board↔planning 循环依赖**。
  - `domain-board-spec.md:244` 附录 B 写的上游依赖是 `domain-project`, `domain-workflow`(state 引用)——**完全不提** `domain-planning`。
  - `domain-planning-spec.md:318` 附录 B 写的上游依赖是 `domain-tenant`, `domain-project`, `domain-work-item`——**完全不提** `domain-board`。
  - §2.3 硬约束图里 `domain-board` 与 `domain-planning` 都是 `domain-work-item` 的独立分支(`↘ domain-board`、`↘ domain-planning`),图上完全没有画出 board↔planning 这条边。
  - **三份独立文档(两侧 spec 附录 B + §2.3 图)都不支持表里声称的这条边**,§2.1 表是唯一断言它存在的地方,证据链最干净,建议优先拍板。

- §2.1 表第 1 行(`basic-design.md:285`,`domain-work-item`)"关键依赖"列出 `domain-workflow`;第 9 行(`basic-design.md:298`,`domain-workflow`)"关键依赖"又列出 `domain-work-item`——**表自己同时声称两个方向都存在核心依赖**。三方证据并不一致,不能简单判定某一行是笔误:
  - 表第 1 行 + `domain-work-item-spec.md:365` 附录 B(依赖 `domain-workflow`,WorkflowDefinition 查询)——两者一致,支持 work-item → workflow。
  - 表第 9 行 + §2.3 硬约束图(`domain-work-item ← domain-workflow`,用上面验证过的基线解码为"workflow 依赖 work-item")——两者一致,支持 workflow → work-item。
  - `domain-workflow-spec.md:301` 附录 B 明确写"无核心依赖(system_default 由本 crate seed)"——**否认**表第 9 行 + §2.3 图共同支持的方向。
  - 换句话说,work-item→workflow 有表+work-item-spec 两处支持,workflow→work-item 有表+§2.3 图两处支持,只有 `domain-workflow-spec.md` 自己的附录 B 唱反调。一个可能的调和方向是两者本质不同类:work-item 对 workflow 是只读查询(WorkflowDefinition),workflow 对 work-item 可能是事件驱动(与发现 3 里"下游调用"混淆"事件订阅"是同一类问题),但这需要 Ulysses 核实,不下结论。

这两组都落在 basic-design.md §4.9 集群(work-item + workflow + board + planning)内,建议 Ulysses 一并核实这次表格编写是否在这个集群上系统性出错。

**B. 中低置信度 —— 表与 spec 单向不一致,列出供后续确认,本次不下结论**

| Domain | 表(`关键依赖`) | Spec 附录 B(`上游依赖`) | 差异 |
|---|---|---|---|
| `domain-agent`(表第 3 行) | `domain-worktree, domain-feedback, domain-validation` | `domain-agent-spec.md:337`:`domain-tenant, domain-worktree, domain-work-item, domain-permission` | 表有 feedback/validation,spec 不提;spec 有 work-item/permission,表不提 |
| `domain-scm`(表第 7 行) | `domain-work-item, domain-worktree` | `domain-scm-spec.md:321`:`domain-tenant, domain-project, domain-permission` | 完全不重叠(与发现 3 已记录的"worktree 是事件订阅方"互相印证,但这里差异更大,work-item 边也无 spec 佐证) |
| `domain-worktree`(表第 2 行) | 含 `domain-development` | `domain-worktree-spec.md:336` 未提 `domain-development` | 表多出的一条边,叠加发现 1 已指出的 `domain-work-item` 反向边问题 |
| `domain-collaboration`(表第 24 行) | `domain-work-item, domain-worktree` | `domain-collaboration-spec.md:283`:"无核心依赖(订阅所有 NATS Event)" | 疑似与发现 3 同类——表把"订阅事件"误记成"核心依赖" |
| `domain-work-item`(表第 1 行) | 含 `domain-permission` | `domain-work-item-spec.md:365` 未提 `domain-permission` | 表多出的一条边,spec 不认 |

**性质**:A 组证据链完整(表自相矛盾 + 两侧 spec 各自反证),建议优先拍板处理;B 组只是单向不一致,可能是 spec 遗漏也可能是表过时,证据不足以下结论,列出供后续单独核实,不在本次报告下判断。

## 发现 7 —— §2.3 禁线补测:`domain-audit` 只追加不可读

补测此前遗漏的第 5 条禁线(`❌ domain-audit 读其他 domain,只追加,不可读`):`domain-audit-spec.md:290` 附录 B"上游依赖"写"无核心依赖(本 Module 是横切)",与 §2.1 表第 15 行(`basic-design.md:304`)"关键依赖"列"所有 domain-*(Append-only)"(即别的 domain 依赖 audit 做归档,而非 audit 依赖别的 domain)方向一致。**结论:未违反,通过。**

---

## 建议处理顺序

1. **发现 2**(`domain-context-spec.md` 附录 B 删掉重复/自相矛盾的 `domain-agent` 上游依赖)—— spec 内部笔误,风险低,可直接改。
2. **发现 4-类 2**(`REQ-NOTIF-003`/`REQ-WI-001` 补进对应 spec)—— 只是给已有设计/已有代码补文档引用,不改设计结论,风险低。
3. **发现 1**(`domain-worktree-spec.md` 的 `domain-work-item` 依赖方向)、**发现 6-A**(work-item↔workflow、board↔planning 两组表内循环依赖)与 **`REQ-RT-002` 缺口**—— 需要 Ulysses 拍板或补充调研,不要下游 AI 自行判断改哪边。
4. **发现 6-B**(5 条中低置信度的表/spec 单向不一致)—— 证据不足以下结论,建议单独排一轮核查,不在本次处理顺序里下判断。
