# PHASE-PRODUCT-NAMING-AUDIT-20261106 — Star 产品命名审计

**状态**　源码与本地 Cargo 元数据审计完成，整改与发布验收待执行
**基线**　`dev@17943a750877e887cb6808c662af65831f8de39b`
**任务简报**　`D:/Star/docs/briefs/PHASE-PRODUCT-NAMING-AUDIT-20261106.md`

## §0 目的

以 [AGENTS.md §0](../../AGENTS.md#0-一句话硬约束) 为准，核对 Star 内置能力、错误与空状态、帮助、宣传、Cargo 包说明与对外候选文档。对真实连接器，仅在其专属设置和连接信息中如实显示服务商身份。把旧输入、配置和持久化标识与显示文案分开，把调查原文与现行产品定义分开。

审计工作树为 `C:/Users/leo19/.codex/worktrees/product-naming-audit-20261106/Star`。下表链接为完整仓库路径，行号固定于基线。范围包括 Web（含语言字典和样例）、Rust CLI、REST 与数据边界、Rust 桌面源码、根入口文档、专题文档与可独立打开的交付页。命中项按正向证据记录，发布面的归属按现有证据区分。

## §1 命中矩阵与最小整改拆分

### A. 用户可见文案及包元数据

| ID / 优先级 | 证据与可见范围 | 整改与验收 |
|---|---|---|
| A1 / P2 | [D:/Star/frontend/src/components/gantt/GanttChart.tsx:297](../../frontend/src/components/gantt/GanttChart.tsx#L297) 将 `linkCountTitle` 渲染为 `title`，由 [D:/Star/frontend/src/app/(app)/projects/ProjectsClient.tsx:578](../../frontend/src/app/%28app%29/projects/ProjectsClient.tsx#L578) 和 [D:/Star/frontend/src/app/(app)/analytics/page.tsx:254](../../frontend/src/app/%28app%29/analytics/page.tsx#L254) 调用。[en.ts:271](../../frontend/src/lib/i18n/en.ts#L271)、[ja.ts:272](../../frontend/src/lib/i18n/ja.ts#L272)、[zh-CN.ts:282](../../frontend/src/lib/i18n/zh-CN.ts#L282) 的提示均含 `MS Project`。 | 三语言保留链接计数与插值，移除品牌与对标语。验收两个入口在含依赖链接时的提示、计数与切换语言行为。 |
| A2 / P2 | [D:/Star/frontend/src/app/integration/page.tsx:18](../../frontend/src/app/integration/page.tsx#L18) 读取本地 store，[第 40–45 行](../../frontend/src/app/integration/page.tsx#L40) 显示连接器名称与类型。[D:/Star/frontend/src/lib/store.ts:374](../../frontend/src/lib/store.ts#L374) 注入 seed，[D:/Star/frontend/src/lib/seed.ts:323](../../frontend/src/lib/seed.ts#L323) 的 `Linear Mirror` 是标为断路状态的样例条目。 | 生产入口用授权连接记录或明确的未接入状态。样例隔离为开发 fixture。验收未接入、真实连接和读取失败状态，真实连接详情仍可显示服务商。 |
| A3 / P2 | [D:/Star/crates/star-cli/src/main.rs:43](../../crates/star-cli/src/main.rs#L43)、[同文件:68](../../crates/star-cli/src/main.rs#L68) 与 [D:/Star/crates/star-cli/src/commands/worktree.rs:41](../../crates/star-cli/src/commands/worktree.rs#L41)、[同文件:49](../../crates/star-cli/src/commands/worktree.rs#L49)、[同文件:60](../../crates/star-cli/src/commands/worktree.rs#L60) 的 Clap `Subcommand` / `Args` 文档注释会进入帮助，含 `FR-ORCA`。 | 改为面向用户的操作描述，内部条目放普通注释或追踪文档。运行 `star --help`、`star worktree --help`、`star worktree start-from-picker --help`，核对描述、参数和退出状态。 |
| A4 / P2 | [D:/Star/crates/star-cli/src/skill_registry.rs:162](../../crates/star-cli/src/skill_registry.rs#L162) 默认注入包括 `multica` 在内的兼容列表，同文件 [第 300–309 行](../../crates/star-cli/src/skill_registry.rs#L300) 将新注册对象序列化到 CLI JSON。该默认值也参与 [兼容筛选:201](../../crates/star-cli/src/skill_registry.rs#L201)。 | 区分显式注册的真实 adapter 与默认声明。验收未指定、显式指定、按旧 ID 查询及输出 schema，先确定默认能力契约再变更。 |
| A5 / P2 | [D:/Star/demo/index.html:1075](../../demo/index.html#L1075) 的视图卡和 [表头:1172](../../demo/index.html#L1172) 以 Jira、Miro、MS Project 标注 Star 内置视图。[提交卡:1243](../../demo/index.html#L1243) 与 [倍率:1263](../../demo/index.html#L1263) 还展示对标文案与倍率。 | 若继续对外展示，视图改用任务工作流、协作画布和项目调度，展示可重放的绝对指标。若仅供归档，把原页面排除在公开入口与发布制品之外。验收所有 Tab、表头、动态行和可见倍率。 |
| A6 / P2 | [D:/Star/deliverables/kanban-vmodel-jp/index.html:6](../../deliverables/kanban-vmodel-jp/index.html#L6) 的浏览器标题使用 JIRA。[D:/Star/deliverables/kanban-vmodel-jp/app.js:832](../../deliverables/kanban-vmodel-jp/app.js#L832)、[904](../../deliverables/kanban-vmodel-jp/app.js#L904)、[1071](../../deliverables/kanban-vmodel-jp/app.js#L1071)、[1085](../../deliverables/kanban-vmodel-jp/app.js#L1085) 的确认框与 Sprint 计划提示以 Jira 为设计依据。[交付 README:59](../../deliverables/kanban-vmodel-jp/README.md#L59) 重复同一表述。 | 继续交付时同步改日文标题、提示、确认和 README，并将 [D:/Star/scripts/automation/kanban_sprint_gen.py:80](../../scripts/automation/kanban_sprint_gen.py#L80) 的品牌正向断言改为验证 Backlog 行为与提示。验收添加、完成、中止、删除的约束与任务回流。 |
| A7 / P3 | Cargo 元数据中有 7 个匹配包，每个清单第 13 行提供说明，第 5 行 `publish = false`。[D:/Star/crates/agent-bridge/Cargo.toml](../../crates/agent-bridge/Cargo.toml#L13)、[D:/Star/crates/worktree-shared-dir/Cargo.toml](../../crates/worktree-shared-dir/Cargo.toml#L13)、[D:/Star/crates/star-registry/Cargo.toml](../../crates/star-registry/Cargo.toml#L13)、[D:/Star/crates/star-task/Cargo.toml](../../crates/star-task/Cargo.toml#L13) 含 Multica 名称或路径。[D:/Star/crates/ai-vault/Cargo.toml](../../crates/ai-vault/Cargo.toml#L13)、[D:/Star/crates/terminal-stack/Cargo.toml](../../crates/terminal-stack/Cargo.toml#L13)、[D:/Star/crates/terminal-protocol/Cargo.toml](../../crates/terminal-protocol/Cargo.toml#L13) 含 `FR-ORCA` 追踪号。 | 只修改说明字段。将来源引用留在内部追踪文档，保留原包名、依赖、feature、协议与发布设置。重跑元数据扫描并核对 7 个包的 `publish` 字段。 |
| A8 / 待定 | [D:/Star/crates/star-workflow/src/lib.rs:171](../../crates/star-workflow/src/lib.rs#L171) 的公开工厂构造描述 `Default Jira-like workflow`。仓内 `default_workflow()` 调用命中均位于同文件测试，第 [615 行](../../crates/star-workflow/src/lib.rs#L615) 和 [622 行](../../crates/star-workflow/src/lib.rs#L622)。 | 中性化默认描述，保留状态、迁移与规则。包外消费者、既有持久化对象和 API 可见性另验。 |

### B. 兼容标识与错误边界

| ID | 证据与处置 |
|---|---|
| B1 | [D:/Star/frontend/src/app/worktree/[id]/group/page.tsx:600](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L600) 接受旧 `jira` / `multica` 查询参数，映射到 `backlog` / `task-flow`。同文件 [625–632](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L625) 定义中性标签，[828–834](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L828) 替换旧 URL。保留输入兼容，在授权与未授权路径测试旧链接、URL 归一化和新链接的生成。 |
| B2 | [D:/Star/crates/worktree-shared-dir/src/shared_dir_resolver.rs:55](../../crates/worktree-shared-dir/src/shared_dir_resolver.rs#L55) 用 `multica-config` 读取已有 workspace 配置，属于文件适配契约。[218](../../crates/worktree-shared-dir/src/shared_dir_resolver.rs#L218)、[227](../../crates/worktree-shared-dir/src/shared_dir_resolver.rs#L227) 的严格 IO/解析错误包含该路径，[294–305](../../crates/worktree-shared-dir/src/shared_dir_resolver.rs#L294) 的普通实现记录诊断并回落空配置。保留读取兼容，验证失败时完整路径只在受限诊断可见。当前未证实该错误文本接入外部路由。 |
| B3 | [D:/Star/db/migrations/2026-09-30-worktree-task-execution-run.sql:89](../../db/migrations/2026-09-30-worktree-task-execution-run.sql#L89) 的 `multica.task_execution_run` 与 [D:/Star/db/migrations/2026-10-07-run-owner-infrastructure-capacity-result.sql:163](../../db/migrations/2026-10-07-run-owner-infrastructure-capacity-result.sql#L163) 的 `star_multica_run_owner` 是物理表和授权角色。保留在 DB 边界。显示改名不触碰 SQL、迁移身份、授权或事件契约。 |
| B4 | [D:/Star/frontend/src/types/ids.ts:655](../../frontend/src/types/ids.ts#L655) 有 `jira` / `linear` IntegrationKind，[D:/Star/crates/domain-scm/src/multi_provider_identity.rs:82](../../crates/domain-scm/src/multi_provider_identity.rs#L82) 将它们列为此身份模块的扩展占位。保留需要兼容的类型值，外部连接声明另以实际 provider 接通为依据。[D:/Star/frontend/src/components/AppHeader.tsx:38](../../frontend/src/components/AppHeader.tsx#L38) 的旧环境变量名是输入 fallback，[84](../../frontend/src/components/AppHeader.tsx#L84) 显示 Worktree ID 的值。验收两类输入时核查品牌字样是否来自 Star 的预设值。 |
| B5 | [D:/Star/frontend/src/lib/group/worktreeGroupApi.ts:1708](../../frontend/src/lib/group/worktreeGroupApi.ts#L1708) 接收服务端 `error.message`，[D:/Star/frontend/src/app/worktree/[id]/group/page.tsx:967](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L967)、[1416](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L1416)、[2511](../../frontend/src/app/worktree/%5Bid%5D/group/page.tsx#L2511) 可显示上游错误文本。这是动态透传点。静态命中未证实这里已有固定竞品错误或空状态文案。后续用失权、读取失败和上游带品牌/内部路径的响应验收公共提示与受限诊断分离。 |

### C. 现行文档与历史证据

| ID | 证据与处置 |
|---|---|
| C1 | [D:/Star/docs/external-design.md:1478](../external-design.md#L1478) 的快捷键描述以 Gmail / Jira 举例。[D:/Star/docs/requirements/SRS-CANVAS-001.md:165](../requirements/SRS-CANVAS-001.md#L165)、[181](../requirements/SRS-CANVAS-001.md#L181)、[658](../requirements/SRS-CANVAS-001.md#L658) 仍用品牌定义现行应用入口和验收。[D:/Star/docs/design/BD-CANVAS-WORKFLOW-001.md:189](../design/BD-CANVAS-WORKFLOW-001.md#L189)、[D:/Star/docs/design/DD-CANVAS-WORKFLOW-001.md:2474](../design/DD-CANVAS-WORKFLOW-001.md#L2474)、[2575](../design/DD-CANVAS-WORKFLOW-001.md#L2575) 也有相关命中。现行产品说明、拓扑图和验收用 Star 名称，并按当前 Project → Engineering Run → Worktree → Branch 规范同步三层文档与既有 AC ID。[D:/Star/docs/requirements.md:3079](../requirements.md#L3079) 已区分稳定专题文件 ID 与产品名，可据此保留链接。 |
| C2 | [D:/Star/docs/wiki/docswiki/nodes/View-AgentView.md:58](../wiki/docswiki/nodes/View-AgentView.md#L58) 有面向架构读者的 Miro 风格描述。根 [D:/Star/README.md:33](../../README.md#L33) 指向架构与 Wiki 集合，进一步提供了读者发现文档的入口。随公开文档导出的现行摘要须中性化，归档内容单独标明用途。 |
| C3 | [D:/Star/docs/ecosystem-survey/orca-design-survey.md:1](../ecosystem-survey/orca-design-survey.md#L1) 是具名调研，`git log -1 -p --follow` 见 `c0d47a5e`；[D:/Star/docs/architecture/2026-08-26-upgrade/adr/0034-jira-ification.md:1](../architecture/2026-08-26-upgrade/adr/0034-jira-ification.md#L1) 的原始记录见 `d03798d6`，Wiki 有 [对应 ADR 镜像](../wiki/pgwiki/30-architecture/adr/adr-0034-jira-ification.md#L2)。保留归档原文、来源、稳定 ID 与修订表证据，将仍需执行的设计提炼到中性的现行文档。 |

后续可独立拆为四组。① Web 展示与 connector 样例（A1、A2、B1、B4、B5）；② CLI 帮助、默认能力与元数据（A3、A4、A7、A8、B2）；③ 静态演示与对外出口（A5、A6）；④ 现行文档、索引与归档（C1–C3）。② 的配置兼容和 B3 的 DB 身份无需加入文案修改；④ 使用 ①–③ 验收后的名称与状态。③ 可以先确定每项是继续交付还是仅供归档，再改或隔离相应入口。

## §2 验证摘要

以下 PowerShell 命令以固定基线的仓库根目录为工作目录。用固定基线复跑可排除审计新增文件。正则是基于当前产品域的发现词表，`linear` 的数学/CSS 用法、`cursor` 的协议用途与 `VectorCAST` 子串误报均需人工分类。

```powershell
git rev-parse HEAD dev
git status --short --branch
$names = 'multica|\bjira\b|atlassian|\bmiro\b|\borca\b|\blinear\b|\bnotion\b|trello|asana|clickup|monday\.com|figjam|height\.app|plane\.so|shortcut\.com|ms project|microsoft project|star cursor|cursor[- ](style|like)'
rg -n -i -g '!pnpm-lock.yaml' -g '!package-lock.json' -- $names frontend/src frontend/public crates/star-desktop bff/src demo deliverables/kanban-vmodel-jp
rg -n -i -g Cargo.toml -- ('^\s*description\s*=.*(' + $names + ')') .
rg -n -i -- $names README.md CHANGELOG.md NOTICE.md frontend/README.md crates/star-desktop/README.md docs/external-design.md docs/requirements.md docs/basic-design.md docs/wiki docs/requirements docs/design
rg -n -i -- 'linkCountTitle|Default Jira-like|Linear Mirror|NEXT_PUBLIC_MULTICA_ISSUE_ID|CLI_CONFIG_DIR_NAME|FR-ORCA' frontend/src crates/star-cli/src crates/star-workflow/src crates/worktree-shared-dir/src
rg -n -i -- 'deploy-pages|upload-pages-artifact|gh-pages|cargo publish|generate_release_notes|demo/index\.html|kanban-vmodel-jp' .github/workflows .dockerignore README.md frontend/README.md
cargo metadata --no-deps --offline --locked --format-version 1 |
  ConvertFrom-Json |
  ForEach-Object {
    "Workspace packages: $($_.workspace_members.Count)"
    $_.packages |
      Where-Object { $_.description -match $names } |
      Select-Object name, description, publish |
      ConvertTo-Json -Depth 4
  }
git log -1 --format='%h %ad %s' --date=short -p --follow -- docs/architecture/2026-08-26-upgrade/adr/0034-jira-ification.md
git log -1 --format='%h %ad %s' --date=short -p --follow -- docs/ecosystem-survey/orca-design-survey.md
git diff --cached --check
```

| 检查 | 结果 |
|---|---|
| 工作树与工具 | 取证时工作树和本地 `dev` 同为 `17943a75`，`rg 15.2.0`。审计工作树的 `.codegraph/` 只有跟踪的 `.gitignore`，`codegraph status` 和 MCP 均返回未初始化。Codebase Memory 覆盖检查工具未暴露，本工作树无 `graphify-out`。文案以本地源码和清单为证。 |
| Cargo 元数据 | Rust / Cargo 1.99.1 下离线、锁定、无依赖解析模式成功退出，工作区 109 个包，7 个说明命中，`publish` 均为 `[]`。这是包元数据证据。 |
| UI、帮助与错误 | 源码核对了渲染、Clap derive、JSON 输出与旧别名的入口。浏览器渲染、CLI 二进制、真实连接和注入失败场景待验证。 |
| 文档与发布 | 核对了根文档入口、当前专题正文、静态交付页与选定历史记录的 Git 差异。工作流关键词扫描未定位到这两个演示的发布配置。没有查看远端实际页面、公开状态或成品。 |
| 自审 | 对照 §1 每个证据路径、行号、可见范围与验收要求，检查报告和暂存差异。报告为 Markdown，代码审查服务未调用。 |

## §3 已知缺口

- 浏览器布局和生成产物未验收。须按实际发布清单检查 Web、桌面安装包、CLI 帮助、静态演示、公开文档、图标、图像、截图与自动生成页面。静态匹配无法完整覆盖动态字符串和外部插件清单。
- 未核实部署端路由、旧直链重定向、选定真实连接器、服务端失败文本及生产配置。B2、B4、B5 的运行可见性仍待验证。
- 未核实公开站点索引与下载入口。A5、A6 有可打开的本地页面，C1、C2 有外部阅读候选材料。是否已有公网曝光需依据发布证据。
- 词表由仓库正向命中扩展，报告对新品牌、拼写变体和编码/图像内容不作覆盖率声明。工作树图谱缺失，所有代码路径结论均限制在已查源码。
- 本次只有审计报告。分组整改、测试、目标制品验收及三层设计同步为后续工作。

## §4 子代理失败接手清单

本 lane 由 root 调度到独立工作树。未再次委派，无子代理失败待接手。图谱缺失后直接核对源码、清单和 Git 证据。

## §5 守门规则

| # | 门禁 | 结果 |
|---|---|---|
| 1 | 基线固定到本地 `dev` | 已核对 SHA |
| 2 | 独立工作树与报告范围 | 报告 lane，业务文件只读 |
| 3 | 产品 UI、可访问名称与 tooltip | 源码核对 A1、A2，渲染待验 |
| 4 | 语言字典一致 | 已定位 en、ja、zh-CN，修复待验 |
| 5 | 错误、空状态与失败透传 | 已定位 B2、B5，故障注入待验 |
| 6 | CLI 帮助 | 已定位 Clap 入口，输出待验 |
| 7 | CLI JSON 与旧 provider 输入 | 已定位 A4，兼容验收待验 |
| 8 | 连接器与样例隔离 | 已定位 A2、B4，授权记录与空状态待验 |
| 9 | 旧 Group URL 兼容 | 已定位映射与替换，回归待验 |
| 10 | 配置、SQL、RLS 和事件 ID | 区分显示与契约，业务文件只读 |
| 11 | 包说明与发布标志 | Cargo 元数据核对成功，说明整改待验 |
| 12 | 静态演示与宣传 | 源码证据 A5、A6，发布状态待验 |
| 13 | 需求、基本设计、详细设计 | 已定位 C1，改名后同步与 AC 回归待验 |
| 14 | 历史来源与归档边界 | 两个原始文件的 Git 差异已核对 |
| 15 | 成品发布范围 | 待核对公开入口、最终文档与制品 |
| 16 | 差异质量与提交范围 | 暂存差异检查与文件清单核对 |

## §6 签字栏

| 角色 | 审计状态 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008)，按现行规则复核分类 |
| SRE Lead | Mavis 接手审核，未运行部署或发布门 |
| 平台 | Mavis 接手审核，已核对本地元数据和兼容标识示例 |
| 评审主持 | Mavis 接手审核，核对报告证据和已知缺口 |
| PM | Mavis 接手审核，记录整改与验收顺序 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-06（JST） | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立固定基线的产品命名矩阵、可复跑证据和后续验收 | 用户要求产品不得显示竞争产品名称 |
