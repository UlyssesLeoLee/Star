// =====================================================================
// zh-CN Dictionary — 简体中文 (默认语言)
// =====================================================================
// 命名规则: 按功能域分组 (userMenu / appHeader / common / nav / theme)
// =====================================================================

import type { Dictionary } from "./dictionary";

export const zhCN: Dictionary = {
  common: {
    appName: "Star",
    appTagline: "Vibe Coding Work Management",
  },
  userMenu: {
    menuLabel: "用户菜单: {name} ({role})",
    themeEngine: "主题引擎",
    themeDark: "深色 · 暗夜黑曜",
    themeLight: "浅色 · 机械白",
    themeDarkShort: "Neo-Tokyo",
    themeLightShort: "Mecha Lab",
    toolsAndWorkspaces: "工具与工作区",
    agentWindows: "Agent Windows 任务窗口",
    agentWindowsStatus: "在线",
    cliProfiles: "CLI Profiles",
    cliProfilesCount: "{count} 内置",
    apiKeys: "API Key 凭据管理",
    apiKeysMode: "双模式",
    profile: "个人中心",
    settings: "全局设置",
    signOut: "退出登录",
    language: "语言",
    languageSwitchHint: "切换界面语言",
  },
  appHeader: {
    workspaceSwitcher: "切换工作区",
    allApps: "ALL APPS",
    appsCount: "25+",
    tacticalJump: "战术跳转...",
    notifications: "通知 ({count} 未读)",
    realtimeOnline: "实时同步: 在线",
    synced: "SYNCED",
    addMoreTabs: "添加更多标签到顶栏",
    removeFromHeader: "从顶栏移除 {label}",
  },
  languageSwitcher: {
    current: "当前: {name}",
    title: "界面语言",
  },
};
