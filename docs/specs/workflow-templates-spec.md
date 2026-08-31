# 工作流模版(Workflow Template)设计 spec

> **状态**: Draft v0.1(2026-08-31)
> **上游依赖**:
> - 《Requirements》§8.2(REQ-WF-001/002/003)、§8.3(DesignArtifact,DSG-001/002)、§27.4(ReviewRecord,RVW-001/002)、§27.5(Self/Cross/Agent-Assisted Review 边界)
> - 《Basic Design》§4.9(Work Management Core)、§4.1.3(Worktree 状态机,§22.2)、附录 A.2
> - 现有实现:`crates/domain-workflow/src/lib.rs`(`Guard`/`Transition`/`WorkflowInstance`)
> **下游交付**: 本 spec 通过后交给 writing-plans 生成实施计划;不直接产出代码
> **触发**: Ulysses 拍板 — "工作流模版功能要有,至少 4 组:编程 / 文学创作 / 视频创作 / 3D 美术创作,内容要符合各自 AI 自动化流程(编程走日本瀑布式),并方便标注 Worktree 自审 / 交叉审核 / 通过状态,体验要打磨到极致"

---

## 1. 目标与背景

当前 `domain-workflow` 只提供"WorkItem 默认三态 + Project 自定义"的空白画布(basic-design §4.9.3)。用户新建 Project 时必须从零拼状态机,没有对齐真实生产流程的起点。本设计新增 **4 套内置 Workflow 模版**,覆盖四类创作/工程领域的 AI 自动化流程,并让"自审(Self-Review)/ 交叉审核(Cross-Review)/ 通过状态"在 UI 上一眼可见、随手可标注。

这不是新建一套平行的模版引擎——模版本质是"预置的 `CreateWorkflowCommand` 参数集"(states + transitions + guards),用户选择后即走既有 `WorkflowCommandPort::create_workflow`,后续可自由编辑,与自定义 Workflow 无技术差异。

## 2. 范围与非目标

**范围内**:
- 4 套模版的状态机设计(States / Transitions / Guards)
- `Transition` 新增 `required_review_kind` 字段(表达"需要哪种 Review 才能放行"),`Guard` 类型本身不变
- Guard 求值语义:如何从 `WorkflowInstance` 找到对应 `ReviewRecord` 并判定放行
- 模版选择器 + Worktree 自审/交叉审核标注面板的 UX 设计
- 依赖缺口的显式声明(见 §3)

**非目标(本次不做)**:
- 不实现 `ReviewRecord` / `DesignArtifact` 的完整领域对象(见 §3,留给独立的 domain-development 扩展 spec)
- 不修改 `docs/requirements.md`、`docs/basic-design.md`(设计已在这两份文档中拍板,本 spec 只是把它们落到 Workflow 模版这一具体特性上)
- 不删除 `REQUIREMENTS-THREAD-C-HANDOFF.md`——该文件是"线程 C 需求编写"这一独立任务的完结记录(自述"本文档到此为止,不进入生产代码编写"),与本 spec 是两个不同阶段的交付物,删除会丢失需求评审的审计轨迹。本次"下游 AI 编写工作指示"改为新增 §12 的实施任务分解,不复用/不覆盖该文件。
- 不新增 `Guard` 枚举变体(§8.3 明确要求复用既有 `RequireApproval`,见 §4)

## 3. 现状与依赖缺口

`crates/domain-workflow/src/lib.rs` 现状(已实现,非本 spec 产出):

```rust
pub enum Guard {
    RequireRole(String),
    RequireValidation(String),
    RequireApproval,
}

pub struct Transition {
    pub id: TransitionId,
    pub from: WorkflowStateId,
    pub to: WorkflowStateId,
    pub trigger: TransitionTrigger,
    pub guard: Option<Guard>,
}
```

`Guard::RequireApproval` 目前的求值(`lib.rs:624` 附近)是一个 **`project_admin` 角色 stub**,尚未接到任何审批实体。

**依赖缺口(需在实施前确认,不在本 spec 内建造)**:`ReviewRecord`(requirements §27.4)与 `DesignArtifact`(§8.3)目前**只存在于 `docs/requirements.md` 的设计文字里,`crates/` 下没有任何对应结构体、状态机或 repository**。四套模版里 Programming 模版的设计批准门禁、以及全部 4 套模版的自审/交叉审核门禁,语义上都要靠这两个对象才能真正生效——在它们落地之前,`required_review_kind` 字段和相关 Guard 只能停在"数据结构已定义、求值逻辑已写好接口、但查不到真实记录"的阶段。

推荐落地位置(供后续 spec 参考,非本次决定):`ReviewRecord` 与 `DesignArtifact` 的 `Target` 都指向 `crates/domain-development` 已有的 `ChangeSet`(`domain-development/src/lib.rs:95`),同一 crate 内新增聚合根比新开一个 `domain-review` crate 更贴近"不新建平行体系"的既有原则(§27.4 行 924)。

## 4. 数据模型变更:`Transition.required_review_kind`

按 §8.3 行 263 的既定原则——"用既有 `RequireApproval` Guard 类型表达,不新增 Guard 类型"——`Guard` 保持 3 个变体不变。新增一个与 `guard` 平级的字段,只在 `guard == Some(RequireApproval)` 时有意义:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewKind {
    SelfReview,
    CrossReview,
    AgentAssistedReview,
}

pub struct Transition {
    pub id: TransitionId,
    pub from: WorkflowStateId,
    pub to: WorkflowStateId,
    pub trigger: TransitionTrigger,
    pub guard: Option<Guard>,
    /// 仅当 guard == Some(RequireApproval) 时生效。
    /// None:审批对象是 DesignArtifact(§8.3 场景,WorkItem 级设计批准门禁)。
    /// Some(kind):审批对象是最新一条 Kind == kind 的 ReviewRecord(§27.4 场景,Worktree/ChangeSet 级审核门禁)。
    pub required_review_kind: Option<ReviewKind>,
}
```

`ReviewKind` 的 3 个变体与 §27.4 `ReviewRecord.Kind` 字段一一对应,不是独立定义——将来 `ReviewRecord` 落地时两者必须共用同一枚举(建议下沉到共享 crate 或由 `domain-workflow` 依赖 `domain-development` 导出的类型,视届时的 crate 依赖方向决定,列为 Open Issue,见 §13)。

**为什么用 `None` 表示 DesignArtifact 而不是新增第 4 个变体**:DesignArtifact 批准和 Worktree 级 Review 从不出现在同一条 Transition 上(前者发生在 Worktree 创建之前,后者发生在实现完成之后),二者在时间线上互斥,靠"哪个阶段的 Transition"就能唯一确定语义,不需要额外变体来消歧。实施时会在每个模版的 Transition 定义旁写清楚注释,避免实现者猜测。

## 5. Guard 求值语义

`WorkflowInstance.work_item_id` 是求值的唯一入口:

1. `guard == RequireApproval && required_review_kind == None`(DesignArtifact 场景):查询该 WorkItem 关联的全部 `DesignArtifact`(§8.3 "0..N 个 DesignArtifact"),全部 `Status == APPROVED` 才放行;若为空集合,视为"未配置设计门禁",直接放行(该 Guard 应仅出现在启用了瀑布模式的 Project 里,不启用则不应把这个 Transition 放进 Workflow)。
2. `guard == RequireApproval && required_review_kind == Some(kind)`(ReviewRecord 场景):按 `WorkItemId == instance.work_item_id AND Kind == kind` 查询 `ReviewRecord`,取每个关联 Worktree 下 `CompletedAt` 最新的一条,要求其 `Status == APPROVED`。若该 WorkItem 下有多个非终态 Worktree,默认要求**全部**满足才放行(保守默认,防止"一个 Worktree 过审、另一个 Worktree 还没审就整体放行"的漏洞);是否允许按 Project Policy 放宽为"任一满足即可"列为 Open Issue(§13)。
3. **Agent-Assisted Review 例外(§27.5 强约束)**:`Kind == AgentAssistedReview` 的 `ReviewRecord.Decision == Reject` **不得**被 Guard 直接解释为"阻断放行"——它只是产出 Feedback/ValidationResult(§27.5:"不构成新的授权层级"),真正的放行判定仍然只看这条 ReviewRecord 本身的 `Status`。也就是说,即使 Decision=Reject,只要人工/Policy 后续把 Status 推进为 APPROVED(说明人工已经复核并接受了修复),Guard 依然放行;Guard 求值代码不需要对 Decision 做任何特判,只读 Status——这条规则本质上已经被"Guard 只读 Status,不读 Decision"的实现自动满足,写在这里是为了防止未来有人"优化"成直接读 Decision 而破坏此约束。

## 6. 四套内置工作流模版

四套模版共享同一条设计原则:**只有 Programming 模版包含 DesignArtifact 批准门禁**(呼应用户"编程要符合日本瀑布式"的明确要求,其余三套是创作类流程,不强加瀑布式设计评审);**四套模版都包含 Self-Review → Cross-Review 两级门禁**,对应用户"方便标注自审和交叉审核"的通用诉求。每套模版落地为一次 `CreateWorkflowCommand` 调用的固定参数,用户选择后可自由编辑,不锁死。

所有模版共享的收尾状态:`BLOCKED`(可从任一中间状态进出,无 Guard)、`CANCELLED`(终态,任一非终态可转入,`Guard::RequireRole("project_admin")`)。下表只列各模版的主干状态与门禁,`BLOCKED`/`CANCELLED` 不重复列出。

### 6.1 编程(Programming,日本瀑布式)

| From | To | Trigger | Guard | required_review_kind |
|---|---|---|---|---|
| TODO | DETAILED_DESIGN | UserAction | — | — |
| DETAILED_DESIGN | IN_PROGRESS | UserAction | RequireApproval | None(DesignArtifact APPROVED) |
| IN_PROGRESS | SELF_REVIEW | AgentAction / UserAction | — | — |
| SELF_REVIEW | CROSS_REVIEW | UserAction | RequireApproval | Some(SelfReview) |
| CROSS_REVIEW | READY_FOR_COMMIT | UserAction | RequireApproval | Some(CrossReview) |
| READY_FOR_COMMIT | DONE | SystemEvent | RequireValidation("build_and_tests_pass") | — |

对应 AI 自动化流程:Agent 先产出详细设计书(DesignArtifact),人工/Policy 批准后才允许 Agent 开始实现;实现完成后 Agent/用户过一遍自审 checklist,再指派人类做交叉审核(Segregation of Duties,§27.5),通过后进入构建/测试门禁,最后收尾。这是四套模版里唯一"设计先行、批准后才能动手"的强门禁流程。

### 6.2 文学创作(Literary Creation)

| From | To | Trigger | Guard | required_review_kind |
|---|---|---|---|---|
| TODO | OUTLINE | UserAction | — | — |
| OUTLINE | DRAFTING | UserAction / AgentAction | — | — |
| DRAFTING | SELF_REVIEW | AgentAction / UserAction | — | — |
| SELF_REVIEW | EDITOR_REVIEW | UserAction | RequireApproval | Some(SelfReview) |
| EDITOR_REVIEW | REVISION_NEEDED | UserAction | — | — |
| REVISION_NEEDED | DRAFTING | UserAction | — | — |
| EDITOR_REVIEW | FINALIZED | UserAction | RequireApproval | Some(CrossReview) |
| FINALIZED | PUBLISHED | UserAction | RequireRole("project_admin") | — |

对应流程:大纲确认后 Agent 生成初稿,作者自审(情节连贯性、文风一致性),再交给编辑做交叉审核;编辑可打回重写(`REVISION_NEEDED`)或批准定稿,定稿后需管理员确认才发布。不设计稿批准门禁(创作类项目通常不需要正式的"大纲评审 Gate"),Project 若需要可自行在 `OUTLINE → DRAFTING` 上追加 `RequireApproval`(复用 §8.3 机制,非模版强制)。

### 6.3 视频创作(Video Creation)

| From | To | Trigger | Guard | required_review_kind |
|---|---|---|---|---|
| TODO | STORYBOARD | UserAction | — | — |
| STORYBOARD | PRODUCTION | UserAction / AgentAction | — | — |
| PRODUCTION | SELF_REVIEW | AgentAction / UserAction | — | — |
| SELF_REVIEW | DIRECTOR_REVIEW | UserAction | RequireApproval | Some(SelfReview) |
| DIRECTOR_REVIEW | REVISION_NEEDED | UserAction | — | — |
| REVISION_NEEDED | PRODUCTION | UserAction | — | — |
| DIRECTOR_REVIEW | FINAL_CUT | UserAction | RequireApproval | Some(CrossReview) |
| FINAL_CUT | EXPORTED | SystemEvent | RequireValidation("render_success") | — |

对应流程:分镜确定后进入素材生成/剪辑(Agent 驱动),剪辑完成后先做初剪自查(`SELF_REVIEW`),再交给导演/制片人做交叉审核(`DIRECTOR_REVIEW`),通过后终剪定版,渲染导出成功才算完成。

### 6.4 3D 美术创作(3D Art Creation)

| From | To | Trigger | Guard | required_review_kind |
|---|---|---|---|---|
| TODO | CONCEPT | UserAction | — | — |
| CONCEPT | MODELING | UserAction / AgentAction | — | — |
| MODELING | SELF_REVIEW | AgentAction / UserAction | — | — |
| SELF_REVIEW | ART_DIRECTOR_REVIEW | UserAction | RequireApproval | Some(SelfReview) |
| ART_DIRECTOR_REVIEW | REVISION_NEEDED | UserAction | — | — |
| REVISION_NEEDED | MODELING | UserAction | — | — |
| ART_DIRECTOR_REVIEW | FINALIZED | UserAction | RequireApproval | Some(CrossReview) |
| FINALIZED | DELIVERED | UserAction | RequireRole("project_admin") | — |

对应流程:概念稿确认后进入建模/贴图/绑定(Agent 辅助),完成后先做拓扑/UV/贴图自查,再由美术总监交叉审核,通过后定稿交付。

## 7. UX 设计

### 7.1 模版选择器(创建 Project / 创建 Workflow 时)

4 张卡片横向排列(与既有 kanban 工具栏"add-column"改造同一套精简卡片语言,避免样式割裂):图标 + 模版名 + 一句话流程描述(如"日本瀑布式:设计批准 → 实现 → 自审 → 交叉审核 → 提交")。选中即预览该模版的状态机流程图(复用 `crates/domain-workflow/src/visualize.rs` 现有可视化能力),用户可在预览态直接"自定义"进入编辑,而不必先创建再改。

### 7.2 Worktree 自审 / 交叉审核标注面板

挂在 Worktree 详情页(现有 §22.2 状态机的 `READY_FOR_REVIEW`/`REVIEWING` 阶段附近),分两级卡片:

- **Self-Review 卡片**:展示 Project ReviewPolicy 模板给出的 Checklist(复用 §24.6 AgentPolicyTemplate 同类机制),逐项勾选;全部勾选后一键"提交自审"生成 `ReviewRecord(Kind=SelfReview, Status=APPROVED)`(Reviewer==Author,§27.4 判定规则)。
- **Cross-Review 卡片**:仅在 Self-Review APPROVED 后可指派 Reviewer(必须与 Author 不同的人类身份,§27.5 Segregation of Duties,前端在指派时过滤掉 Author 自己)。Reviewer 看到 Findings 列表 + Approve / RequestChanges / Reject 三个决策按钮,决策即写 `ReviewRecord.Decision` 并推进 `Status`。
- **Agent-Assisted Review 标注**:以独立的、视觉上弱化的小卡片呈现(区别于人类 Cross-Review 的强调色),标题带 Agent 图标;其 `Reject` 只显示"Agent 发现问题,仍需人工确认"的提示条,不出现红色阻断态,呼应 §5 第 3 点与 §27.5 的"不得自动阻断"约束。
- **状态徽标**:Worktree 列表/看板卡片上追加一枚复合徽标,例如 `Self ✓ · Cross ⏳` 或 `Self ✓ · Cross ✗ 已打回`,一眼看出当前卡在哪一级审核,不用点进详情页。

## 8. Domain Events(新增)

| Subject (NATS) | 触发条件 | Payload |
|---|---|---|
| `star.events.workflow.template.applied.v1` | 用户选择内置模版创建 Workflow | `workflow_id, project_id, template_kind`(Programming / Literary / Video / Art3D) |

其余审核相关事件(`review.record.created/approved/rejected` 等)属于 `ReviewRecord` 自身的领域事件,待 §3 依赖对象落地后在其独立 spec 中定义,不在本 spec 重复。

## 9. 接口签名变更

`domain-workflow` 现有 `WorkflowCommandPort`/`WorkflowQueryPort` 签名不变。新增一个纯查询辅助方法,供前端模版选择器拉取内置目录:

```rust
#[async_trait]
pub trait WorkflowTemplateCatalog: Send + Sync {
    /// 返回 4 套内置模版的只读定义(states + transitions),不含 tenant_id
    fn list_builtin_templates(&self) -> Vec<WorkflowTemplate>;
}

pub struct WorkflowTemplate {
    pub kind: WorkflowTemplateKind, // Programming | Literary | Video | Art3D
    pub display_name: String,
    pub description: String,
    pub states: Vec<WorkflowState>,
    pub transitions: Vec<Transition>,
    pub default_initial_state: WorkflowStateId,
}
```

`list_builtin_templates` 是纯内存 seed data(参照现有 `WorkflowRepository` 之外的只读来源,类比 system_default 三态的处理方式),不落库、不带 `tenant_id`;用户选择后照常走 `create_workflow` 落成一条真实的、属于该 tenant/project 的 `Workflow`。

## 10. 鉴权

复用 `domain-workflow` 既有鉴权模型(`workflow:create` 权限即可创建/应用模版,无需新增权限字符串)。Self-Review 提交权限 = ReviewRecord 未来落地时按 Author 身份天然限定(自己给自己提交,不需要额外权限检查);Cross-Review 指派/决策权限待 `ReviewRecord` 落地时随其 spec 一并定义。

## 11. 验收标准(AC)

```gherkin
Feature: 工作流模版

  Scenario: 应用 Programming 模版
    Given 用户是 project_admin,选择 "Programming(日本瀑布式)" 模版
    When POST /v1/workflows { project_id, from_template: "programming" }
    Then 201 Created,Workflow 含 6 个主干状态 + BLOCKED + CANCELLED
    And  DETAILED_DESIGN → IN_PROGRESS 的 Transition.guard == RequireApproval 且 required_review_kind == None

  Scenario: 未启用瀑布门禁的 Project 里 DesignArtifact 为空集合
    Given WorkItem 未关联任何 DesignArtifact
    When 求值 Guard::RequireApproval(required_review_kind=None)
    Then 放行(§5 第 1 点:空集合视为未配置门禁)

  Scenario: Self-Review 未过,Cross-Review 门禁不放行
    Given WorkItem 关联的 ReviewRecord(Kind=SelfReview) Status=IN_PROGRESS
    When 求值 SELF_REVIEW → CROSS_REVIEW 的 Guard
    Then 拒绝放行

  Scenario: Agent-Assisted Review 的 Reject 不阻断
    Given 一条 ReviewRecord(Kind=AgentAssistedReview, Decision=Reject, Status=APPROVED,即人工已复核接受)
    When 求值任何依赖该 Kind 的 Guard(若模版配置了 AgentAssistedReview 门禁)
    Then 放行(§5 第 3 点:Guard 只读 Status,不读 Decision)
```

## 12. 实施任务分解(供 writing-plans 参考)

| 任务 | 描述 | 依赖 | 估算 |
|---|---|---|---|
| T1 | `domain-workflow`:新增 `ReviewKind` 枚举 + `Transition.required_review_kind` 字段 + 序列化/反序列化兼容(现有数据无此字段,需默认 `None`) | 无 | 40K tokens |
| T2 | `domain-workflow`:`WorkflowTemplateCatalog` trait + 4 套模版 seed data(§6 表格逐条落成 `Transition`) | T1 | 120K tokens |
| T3 | Guard 求值逻辑改造(`lib.rs:624` 附近):按 §5 三条规则求值,**阻塞依赖**——DesignArtifact/ReviewRecord 查询接口不存在前,此任务只能先落一个显式返回 `Err(NotImplemented)` 或 `false` 的占位分支,真正生效需等 §3 依赖对象落地后回填 | T1, §3 依赖对象 | 100K tokens(不含依赖对象本身) |
| T4 | 前端:模版选择器 4 卡片 + 流程图预览(复用 `visualize.rs`) | T2 | 100K tokens |
| T5 | 前端:Worktree 详情页 Self-Review/Cross-Review 面板 + 状态徽标 | T3(可先接占位分支联调 UI,不阻塞) | 150K tokens |
| T6 | 单元测试:4 套模版状态机可达性、Guard 求值三条规则(用 mock ReviewRecord/DesignArtifact 数据源) | T1-T3 | 100K tokens |

**合计估算**:~610K tokens(不含 `ReviewRecord`/`DesignArtifact` 领域对象本身的实施成本,那部分需要独立 spec 与任务分解)。

## 13. Open Issues

- J-WT-01:一个 WorkItem 下多个非终态 Worktree 时,Review Guard 是"全部满足"还是"任一满足"放行?本 spec 默认"全部满足"(§5 第 2 点),需 Project Policy 层面确认是否要做成可配置项。
- J-WT-02:`ReviewKind` 枚举将来应定义在哪个 crate,由谁依赖谁(`domain-workflow` 依赖 `domain-development`,还是下沉到共享 crate)?待 `ReviewRecord` 落地 spec 时一并决定。
- J-WT-03:模版选择后是否允许"切换模版"(而不是在已选模版基础上手动编辑)?本 spec 未设计切换语义,默认视为"重新创建一次 Workflow",不支持原地切换合并历史 Transition。
- J-WT-04:文学/视频/3D 模版是否也应该像 Programming 一样提供"可选的前置设计门禁"开关?本 spec 建议交给 Project 自行用 §8.3 既有机制追加,不作为模版默认项(§6.2 已注明),避免四套模版复杂度不对等。
