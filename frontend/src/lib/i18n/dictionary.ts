// =====================================================================
// Dictionary — 字典 interface (宽 string 类型, 允许 3 语言字面量差异)
// =====================================================================
// 注意: 此处用 interface 而非 typeof as const, 否则 en/ja 字面量会被
// 强制推断成 zh-CN 的字面量值, 导致跨语言赋值 TS2322 报错.
//
// v0.2 (2026-08-31 补缺口) 新增: sidebar / appMatrix / commandBar /
//   pageHeader / common / modules — 覆盖高曝光面 Sidebar / AppMatrix /
//   CommandBar / 28 模块名 / 共用按钮文案.
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

export interface Dictionary {
  common: {
    appName: string;
    appTagline: string;
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
    profile: string;
    settings: string;
    signOut: string;
    language: string;
    languageSwitchHint: string;
  };
  appHeader: {
    workspaceSwitcher: string;
    allApps: string;
    appsCount: string;
    tacticalJump: string;
    notifications: string;
    realtimeOnline: string;
    synced: string;
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
    groupWorkspaces: string;       // "Workspaces"
    groupTactical: string;         // "Tactical Views"
    pinned: string;                // "PINNED" badge
    customAdd: string;             // "+ 定制添加模块"
    footerStatus: string;          // "SYS // TACTICAL"
    footerStatusAllGreen: string;  // "ALL GREEN"
    footerNode: string;            // "NERV-01 // VIBE CODING NODE"
    removeFromSidebar: string;     // "从左侧移除 {label}"
    removeFromPinned: string;      // "从视图移除 {label}"
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
    placeholder: string;           // "搜索 25+ 模块..."
    emptyHint: string;             // "0 命中 — 按 Esc 退出"
    hintNavigate: string;          // "navigate"
    hintOpen: string;              // "open"
    hintClose: string;             // "Esc close"
    modulesCounter: string;        // "{filtered} / {total} modules"
    recentCounter: string;         // "· {count} recent"
    ariaLabel: string;             // "Command bar"
    closeAria: string;             // "Close command bar (Esc)"
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
    subtitle: string;                 // "分批次对已完成任务做重构 · Jira 风格 todo→done"
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
}
