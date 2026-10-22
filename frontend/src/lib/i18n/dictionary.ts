// =====================================================================
/*
@cypher schema=1 source_sha256=0dcbddeae3d7fc2ed3dbc2a04291a2814ec75791623c061326e0b8cbf3a778b6
MERGE (self:File {path:"frontend/src/lib/i18n/dictionary.ts"})
MERGE (category:Type {id:"frontend/src/lib/i18n/dictionary.ts::CategoryDef",kind:"interface"})
MERGE (module:Type {id:"frontend/src/lib/i18n/dictionary.ts::ModuleI18n",kind:"interface"})
MERGE (pageTitle:Type {id:"frontend/src/lib/i18n/dictionary.ts::PageTitleI18n",kind:"interface"})
MERGE (dictionary:Type {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary",kind:"interface"})
MERGE (runDirectory:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory",kind:"property"})
MERGE (sidebarOtherPages:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.otherPages",kind:"property"})
MERGE (sidebarProjectNavigation:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.projectNavigation",kind:"property"})
MERGE (sidebarOpenProjectDirectory:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.openProjectDirectory",kind:"property"})
MERGE (sidebarProjectDirectory:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.projectDirectory",kind:"property"})
MERGE (sidebarProjectDirectoryTitle:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.projectDirectoryTitle",kind:"property"})
MERGE (sidebarProjectDirectoryHint:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.projectDirectoryHint",kind:"property"})
MERGE (sidebarCloseProjectDirectory:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.closeProjectDirectory",kind:"property"})
MERGE (runDirectoryUnavailableTitle:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.directoryUnavailableTitle",kind:"property"})
MERGE (runDirectoryUnavailableDescription:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.directoryUnavailableDescription",kind:"property"})
MERGE (runDirectoryHierarchyHint:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.hierarchyHint",kind:"property"})
MERGE (runDirectoryRefresh:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.refreshDirectory",kind:"property"})
MERGE (runDirectoryIndexTitle:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.projectWorktreeIndexTitle",kind:"property"})
MERGE (runDirectoryUnknownCreate:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.unknownCreateResult",kind:"property"})
MERGE (runDirectoryExpandNode:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.expandNode",kind:"property"})
MERGE (runDirectoryCollapseNode:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.collapseNode",kind:"property"})
MERGE (runDirectoryProjectChildren:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.projectChildren",kind:"property"})
MERGE (runDirectoryWorktreeBranches:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.worktreeBranches",kind:"property"})
MERGE (runDirectoryRunChildren:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.runChildren",kind:"property"})
MERGE (runDirectorySelectNode:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.selectNode",kind:"property"})
MERGE (runDirectoryOpenRunOverview:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.openRunOverview",kind:"property"})
MERGE (runDirectoryFocusHint:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.runOverviewFocusHint",kind:"property"})
MERGE (runDirectoryRetry:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.retryAction",kind:"property"})
MERGE (runDirectoryProjectIdLabel:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.runDirectory.projectIdLabel",kind:"property"})
MERGE (self)-[:DEFINES]->(category)
MERGE (self)-[:DEFINES]->(module)
MERGE (self)-[:DEFINES]->(pageTitle)
MERGE (self)-[:DEFINES]->(dictionary)
MERGE (dictionary)-[:DEFINES]->(runDirectory)
MERGE (dictionary)-[:DEFINES]->(sidebarOtherPages)
MERGE (dictionary)-[:DEFINES]->(sidebarProjectNavigation)
MERGE (dictionary)-[:DEFINES]->(sidebarOpenProjectDirectory)
MERGE (dictionary)-[:DEFINES]->(sidebarProjectDirectory)
MERGE (dictionary)-[:DEFINES]->(sidebarProjectDirectoryTitle)
MERGE (dictionary)-[:DEFINES]->(sidebarProjectDirectoryHint)
MERGE (dictionary)-[:DEFINES]->(sidebarCloseProjectDirectory)
MERGE (dictionary)-[:DEFINES]->(runDirectoryUnavailableTitle)
MERGE (dictionary)-[:DEFINES]->(runDirectoryUnavailableDescription)
MERGE (dictionary)-[:DEFINES]->(runDirectoryHierarchyHint)
MERGE (dictionary)-[:DEFINES]->(runDirectoryRefresh)
MERGE (dictionary)-[:DEFINES]->(runDirectoryIndexTitle)
MERGE (dictionary)-[:DEFINES]->(runDirectoryUnknownCreate)
MERGE (dictionary)-[:DEFINES]->(runDirectoryExpandNode)
MERGE (dictionary)-[:DEFINES]->(runDirectoryCollapseNode)
MERGE (dictionary)-[:DEFINES]->(runDirectoryProjectChildren)
MERGE (dictionary)-[:DEFINES]->(runDirectoryWorktreeBranches)
MERGE (dictionary)-[:DEFINES]->(runDirectoryRunChildren)
MERGE (dictionary)-[:DEFINES]->(runDirectorySelectNode)
MERGE (dictionary)-[:DEFINES]->(runDirectoryOpenRunOverview)
MERGE (dictionary)-[:DEFINES]->(runDirectoryFocusHint)
MERGE (dictionary)-[:DEFINES]->(runDirectoryRetry)
MERGE (dictionary)-[:DEFINES]->(runDirectoryProjectIdLabel)
@endcypher
*/

// Dictionary — 字典 interface (宽 string 类型, 允许 3 语言字面量差异)
// =====================================================================
// 注意: 此处用 interface 而非 typeof as const, 否则 en/ja 字面量会被
// 强制推断成 zh-CN 的字面量值, 导致跨语言赋值 TS2322 报错.
//
// v0.2 (2026-08-31 补缺口) 新增: sidebar / appMatrix / commandBar /
//   pageHeader / common / modules — 覆盖高曝光面 Sidebar / AppMatrix /
//   CommandBar / 28 模块名 / 共用按钮文案.
//
// v0.6 (2026-09-05 拍板 C 全 i18n 接管) 新增 5 域横展 (per 守门 #13
//   W/T/M 同 shape 严格):
//     - navModules:   33 module label/categoryLabel/description
//     - pageTitles:   28 page title/subtitle (按 route)
//     - ariaLabels:   通用 aria-label 兜底 (key = 业务语义)
//     - placeholders: input placeholder 兜底
//     - categoryNames: 5 域 (core/work/agent/integration/system) 名称
//   缺标策略: 开发环境 console.warn + 渲染 "[key]" 路径;
//            生产 fallback en (per 守门 缺标比错标安全)
// =====================================================================

export interface CategoryDef {
  label: string;
  tag: string;
}

export interface ModuleI18n {
  label: string;
  description: string;
  categoryLabel: string;
}

export interface PageTitleI18n {
  title: string;
  subtitle?: string;
}

export interface Dictionary {
  common: {
    appName: string;
    appTagline: string;
  };
  runDirectory: {
    newRun: string;
    newRunTitle: string;
    runName: string;
    runNamePlaceholder: string;
    create: string;
    creating: string;
    cancel: string;
    hostIndependentHint: string;
    invalidRunName: string;
    projectWorktreeIndexTitle: string;
    directoryUnavailableTitle: string;
    directoryUnavailableDescription: string;
    hierarchyHint: string;
    refreshDirectory: string;
    unknownCreateResult: string;
    expandNode: string;
    collapseNode: string;
    projectChildren: string;
    worktreeBranches: string;
    runChildren: string;
    selectNode: string;
    openRunOverview: string;
    runOverviewFocusHint: string;
    retryAction: string;
    projectIdLabel: string;
    localWorktree: string;
    humanState: string;
    machineState: string;
    humanStates: {
      running: string; waiting: string; ready: string; diverged: string; conflict: string; merged: string; stale: string;
    };
    machineStates: {
      agent_executing: string; git_clean: string; git_dirty: string; merge_in_progress: string; rebase_in_progress: string;
      ci_running: string; awaiting_human: string; awaiting_feedback: string; awaiting_tool: string; merged_clean: string;
      archived: string; deleted: string;
    };
    unknownState: string;
    agentAssigned: string;
    agentUnassigned: string;
    sessionBound: string;
    sessionUnbound: string;
    runtimeBound: string;
    runtimeUnbound: string;
    archived: string;
    notArchived: string;
    runBindingVersion: string;
    projectBindingVersion: string;
    lockUnknown: string;
    dirtyDetailsUnknown: string;
    observedAtUnknown: string;
    branchFreshnessUnknown: string;
    branchRecord: string;
    branchUnavailable: string;
  };
  userMenu: {
    menuLabel: string;
    themeEngine: string;
    themeDark: string;
    themeLight: string;
    themeDarkShort: string;
    themeLightShort: string;
    toolsAndWorkspaces: string;
    agentWindows: string;
    agentWindowsStatus: string;
    cliProfiles: string;
    cliProfilesCount: string;
    apiKeys: string;
    apiKeysMode: string;
    /** v0.7 (2026-09-08 Phase OPS-INTRY) 新增: 运维入口 */
    ops: string;
    profile: string;
    settings: string;
    signOut: string;
    language: string;
    languageSwitchHint: string;
  };
  appHeader: {
    workspaceSwitcher: string;
    notifications: string;
    realtimeOnline: string;
    addMoreTabs: string;
    removeFromHeader: string;
  };
  languageSwitcher: {
    current: string;
    title: string;
  };
  // ── v0.2 新增 5 域 ──
  sidebar: {
    brandTagline: string;          // "VIBE CONTROL PLANE"
    vibeControlPlane: string;      // alias for brandTagline
    otherPages: string;            // secondary navigation disclosure
    projectNavigation: string;
    openProjectDirectory: string;
    projectDirectory: string;
    projectDirectoryTitle: string;
    projectDirectoryHint: string;
    closeProjectDirectory: string;
    groupWorkspaces: string;       // pinned workspace links
    groupTactical: string;         // "Tactical Views"
    pinned: string;                // "PINNED" badge
    customAdd: string;             // "+ 定制添加模块"
    removeFromSidebar: string;     // "从左侧移除 {label}"
    removeFromPinned: string;      // "从视图移除 {label}"
    // 折叠 / scope 切换 (per 2026-09-03 12:36 JST 拍板)
    fold: {
      collapse: string;            // "折叠侧栏 (Ctrl+B)"
      expand: string;              // "展开侧栏 (Ctrl+B)"
      collapseShort: string;       // "折叠"
      expandShort: string;         // "展开"
    };
    scope: {
      main: string;                // "主导航"
      project: string;             // "项目专属"
      mainHint: string;            // "切换到全局核心模块"
      projectHint: string;         // "切换到当前选中项目的视图"
      projectDisabledHint: string; // "项目专属导航仅在 /projects 路径下可用"
    };
    activeHint: string;            // "当前: {scope}"
  };
  appMatrix: {
    title: string;                 // "APP MATRIX // ..."
    subtitle: string;              // "默认仅展示核心功能..."
    capabilities: string;          // "25+ CAPABILITIES"
    searchPlaceholder: string;     // "快速检索 25+ 业务模块..."
    resetDefault: string;          // "重置默认"
    resetDefaultTitle: string;     // "恢复系统默认导航"
    done: string;                  // "完成定制 (Done)"
    pinToSidebar: string;          // "钉选到左侧导航"
    unpinFromSidebar: string;      // "从左侧导航移除"
    pinToHeader: string;           // "钉选到顶部标签"
    unpinFromHeader: string;       // "从顶部标签移除"
    openNow: string;               // "立即打开该模块"
    sidebarLabel: string;          // "左侧"
    headerLabel: string;           // "顶部"
    footerPinnedSidebar: string;   // "左侧已钉选:"
    footerPinnedHeader: string;    // "顶部已钉选:"
    categories: {
      all: CategoryDef;
      core: CategoryDef;
      work: CategoryDef;
      agent: CategoryDef;
      integration: CategoryDef;
      system: CategoryDef;
    };
  };
  commandBar: {
    placeholder: string;           // 搜索框 placeholder
    emptyHint: string;             // 0 命中提示
    hitSuffix: string;             // "hits" / "命中" / "件"
    hintNavigate: string;          // "navigate"
    hintOpen: string;              // "open"
    hintClose: string;             // "Esc close"
    modulesCounter: string;        // "{filtered} / {total} modules"
    recentCounter: string;         // "· {count} recent"
    ariaLabel: string;             // "Command bar"
    closeAria: string;             // "Close command bar (Esc)"
    filterAll: string;             // "全部"
  };
  pageHeader: {
    trackPill: string;             // "Track {track}"
    telemetryTag: string;          // "// TELEMETRY"
  };
  modules: Record<string, ModuleI18n>;
  // ── v0.3 新增 (per 2026-08-31 补缺口 v2): Board / Gantt / Calendar + 状态枚举 ──
  status: {
    /** WorkItemStatus: todo / in_progress / review / done / blocked / wontfix */
    workItem: Record<string, string>;
    /** SprintStatus: active / planned / completed */
    sprint: Record<string, string>;
    /** WorkItemKind: bug / feature / task / story / epic (subset used in StatusPill) */
    workItemKind: Record<string, string>;
    /** RefactorStatus: todo / doing / testing / review / done (5 态, per 2026-09-02 10:41 JST 拍板) */
    refactor: Record<string, string>;
  };
  board: {
    columnsActive: string;            // "COLUMNS // 4 ACTIVE"
    columnReorderHint: string;        // "· 拖动 ⋮⋮ 重排列, 点击列名重命名"
    addColumn: string;                // "+ Add Column"
    addColumnTitle: string;           // "添加新看板列"
    dragCardsHere: string;            // "拖卡片到此"
    addTask: string;                  // "Add task"
    addTaskTitle: string;             // "新增任务卡 (弹抽屉)"
    wipExceeded: string;              // "WIP 超过限制"
    clickToRename: string;            // "点击改列名"
    removeColumn: string;             // "删除列"
    fallbackColumnProtected: string;  // "兜底列不可删除 — 删除时其他列的任务会回到此列, 是数据兜底"
    fallbackColumnNotRemovable: string; // "兜底列 {name} 不可删除"
    dragToReorder: string;            // "拖动重排列"
    reorderColumn: string;            // "重排列 {name}"
  };
  gantt: {
    zoom: string;                     // "Zoom"
    zoomUnit: string;                 // "{totalDays}d · {pxPerDay}px/day · {totalWidth}px"
    linkCount: string;                // "🔗 {count} link{s}"
    linkCountTitle: string;           // "任务依赖链接数: {count} 条 (per MS Project task link)"
    linkCountSingular: string;        // ""
    sprintsHeader: string;            // "Sprints"
    milestonesHeader: string;         // "Milestones"
    conflictPredecessor: string;      // "依赖冲突: predecessor {key} ({name}) 结束于 {date}, 当前任务不能早于此"
    expandModal: string;              // "⛶"
    expandTitle: string;              // "展开为浮动窗口 (双击图表空白处也可打开)"
    criticalPath: string;             // "critical path"
    linkBlocks: string;               // "blocks"
    linkDuplicates: string;           // "duplicates"
    linkRelatesTo: string;            // "relates_to"
  };
  calendar: {
    today: string;                    // "Today"
    previous: string;                 // "Previous"
    next: string;                     // "Next"
    timezoneTitle: string;            // "Timezone: {tz}"
    timezoneDisplay: string;          // "UTC · {tz}"
    month: string;                    // "Month"
    week: string;                     // "Week"
    /** 12 月份名 (0-indexed 数组) */
    monthNames: [string, string, string, string, string, string, string, string, string, string, string, string];
    /** 7 星期名 (0=Sun ... 6=Sat) */
    weekdayNames: [string, string, string, string, string, string, string];
    weekOf: string;                   // "Week of {month} {day}, {year}"
    legendHeader: string;             // "Legend:"
    legendActiveSprint: string;       // "active sprint"
    legendPlannedSprint: string;      // "planned sprint"
    legendMilestone: string;          // "milestone due"
    legendMilestoneKind: string;      // "Milestone"
    legendP0: string;                 // "P0 item"
    legendP0Hint: string;             // "high priority"
    legendP1: string;                 // "P1 item"
    legendP1Hint: string;             // "medium"
    legendP2P3: string;               // "P2/P3"
    legendP2P3Hint: string;           // "low"
  };
  workItem: {
    storyPointsUnit: string;          // "SP"
    priorityP0: string;               // "P0"
    priorityP1: string;               // "P1"
    priorityP2: string;               // "P2"
    priorityP3: string;               // "P3"
  };
  // ── v0.5 新增 (per 2026-09-02 11:06 JST): Tooltip 漫画气泡的 i18n 配套 ──
  // Tooltip 组件本身接受 ReactNode, 默认场景需要 3 语言兜底文案.
  tooltip: {
    moreInfo: string;                 // "更多信息"
    closeAria: string;                // "关闭 (Esc)"
  };
  // ── v0.4 新增 (per 2026-09-02 10:41 JST 拍板): Refactor Sweep 重构专项页 ──
  // 范围: 5 状态 todo/doing/testing/review/done + 通用 UI 文案 + 列自定义
  //   (RefactorStatus 翻译走 status.refactor, 跟 workItem.kind / sprint 同 shape)
  // 设计依据: docs/frontend/design/refactor-sweep-design.md
  refactor: {
    /** 重构专项页主标题 / 副标题 */
    title: string;                    // "Refactor Sweep 重构专项"
    subtitle: string;                 // "分批次重构已完成任务 · 待办→完成循环"
    /** 顶部 KPI 4 卡 (待办/进行中/测试中/评审中/已完成 计数) */
    kpiTodo: string;                  // "待办"
    kpiDoing: string;                 // "进行中"
    kpiTesting: string;               // "测试中"
    kpiReview: string;                // "评审中"
    kpiDone: string;                  // "已完成"
    /** Round / Batch 标签 */
    roundLabel: string;               // "Round #"
    batchLabel: string;               // "Batch"
    currentBatch: string;             // "当前批次"
    totalCards: string;               // "总卡数"
    finishedCards: string;            // "已完成"
    /** 操作按钮 */
    openNextRound: string;            // "开启 Round #N+1"
    openNextRoundConfirm: string;     // "确认开启下一轮?"
    openNextRoundHint: string;        // "所有卡状态将重置为 todo, round + 1"
    pullNextBatch: string;            // "拉下一批"
    pullNextBatchHint: string;        // "把剩余 todo 卡补满当前批次"
    addCards: string;                 // "添加任务"
    addCardsTitle: string;            // "从项目已完成任务中挑选加入重构"
    noDoneWorkItems: string;          // "当前项目暂无 status=done 的任务可重构"
    /** 列自定义 — 跟 Kanban 行为 1:1 对齐 (per 2026-09-02 拍板) */
    addColumn: string;                // "+ Add Column"
    addColumnTitle: string;           // "添加新重构列 (状态名)"
    removeColumn: string;             // "删除列"
    renameColumn: string;             // "点击改列名"
    dragToReorder: string;            // "拖动 ⋮⋮ 重排"
    fallbackProtected: string;        // "兜底列 {name} 不可删除 — 删其他列时任务回到此列, 数据兜底"
    fallbackNotRemovable: string;     // "{name} 不可删除"
    columnsCustomizeHint: string;     // "· 拖 ⋮⋮ 重排, 点击列名改, ✕ 删"
    /** 看板列标题 + 空状态 */
    emptyColumn: string;              // "暂无可重构任务"
    dropCardHere: string;             // "拖卡到此"
    wipExceeded: string;              // "WIP 超过限制"
    /** 历史轮次 */
    historyTitle: string;             // "历史重构轮次"
    historyEmpty: string;             // "尚无历史轮次"
    historyRound: string;             // "Round #"
    historyStarted: string;           // "开始于"
    historyClosed: string;            // "结束于"
    historyProgress: string;          // "{done} / {total}"
    historyActive: string;            // "进行中"
    /** 设置抽屉 */
    batchSizeLabel: string;           // "每批卡数"
    batchSizeHint: string;            // "默认 5, 推荐 3-8"
    resetColumns: string;             // "重置为默认 5 列"
    resetColumnsTitle: string;        // "重置后丢失自定义列与命名, 不可恢复"
    resetColumnsConfirm: string;      // "确认重置?"
    /** 卡片状态徽章 */
    refactorRoundBadge: string;       // "第 N 次重构"
    refactorMovedAt: string;          // "更新于 {time}"
    /** 提示 */
    roundComplete: string;            // "本轮全部完成, 可开启下一轮"
    batchComplete: string;            // "本批全部完成"
    nothingInProgress: string;        // "本批无进行中任务"
    /** Merge 按钮 (per 2026-09-02 10:50 JST 拍板: done 列加 Merge 按钮, 触发 worktree + PR merge) */
    merge: string;                    // "Merge"
    mergeTitle: string;               // "合并到 main: 把 worktree 状态 → merged, PR → merged"
    merged: string;                   // "已合并"
    mergedAt: string;                 // "合并于 {time}"
    mergeNoWorktree: string;          // "无 worktree, 仅标记已合并"
    mergeConfirm: string;             // "确认合并?"
    mergeInProgress: string;          // "合并中..."
  };
  // ── v0.6 (per 2026-09-05 23:09 JST 拍板 C) 新增 5 域横展 ──
  // 跟守门 #13 W/T/M 同 shape 严格: 100% 表覆盖, 禁止"混在"一括列举.
  navModules: Record<string, ModuleI18n>;
  pageTitles: Record<string, PageTitleI18n>;
  ariaLabels: Record<string, string>;
  placeholders: Record<string, string>;
  categoryNames: Record<"core" | "work" | "agent" | "integration" | "system", string>;
  /** v0.7 (2026-09-08 Phase OPS-INTRY) 新增: Ops Console 4 tab 翻译 */
  opsConsole: {
    pageTitle: string;
    pageSubtitle: string;
    tabCluster: string;
    tabLogAI: string;
    tabMetrics: string;
    tabDocs: string;
    comingSoon: string;
    /** F-01 Cluster */
    clusterTitle: string;
    clusterReleases: string;
    clusterCanary: string;
    clusterRollback: string;
    clusterStatus: string;
    /** F-02 Log AI (F-02 端到端实装) */
    logAITitle: string;
    logAIUpload: string;
    logAIAnalysis: string;
    logAIChannelMock: string;
    logAINeedsReview: string;
    /** F-02 LogAITab 端到端 i18n (2026-09-08 F-02 端到端拍板) */
    logAIUploadTitle: string;
    logAISourceLabel: string;
    logAILevelFilterLabel: string;
    logAIContentLabel: string;
    logAIUploadBtn: string;
    logAIUploading: string;
    logAIUploadOk: string;
    logAIErrorPrefix: string;
    logAIAnalysisTitle: string;
    logAIConfidenceLabel: string;
    logAIAnomaliesLabel: string;
    logAISuggestionsLabel: string;
    /** F-03 Metrics */
    metricsTitle: string;
    metricsKPI: string;
    metricsLoading: string;
    metricsCardCpu: string;
    metricsCardMem: string;
    metricsCardTasks: string;
    metricsCardMcp: string;
    metricsCardLlm: string;
    metricsTrendTitle: string;
    metricsTrendHint: string;
    /** F-04 Docs */
    docsTitle: string;
    docsCategory: string;
    /** F-04 DocsTab 端到端 i18n (2026-09-08 F-04 端到端拍板) */
    docsSubtitle: string;
    docsCategorySrs: string;
    docsCategoryBas: string;
    docsCategoryDet: string;
    docsCategoryReport: string;
    docsCategoryOther: string;
    docsEmpty: string;
  };
}
