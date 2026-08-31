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
}
