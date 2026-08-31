// =====================================================================
// Dictionary — 字典 interface (宽 string 类型, 允许 3 语言字面量差异)
// =====================================================================
// 注意: 此处用 interface 而非 typeof as const, 否则 en/ja 字面量会被
// 强制推断成 zh-CN 的字面量值, 导致跨语言赋值 TS2322 报错.
// =====================================================================

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
}
