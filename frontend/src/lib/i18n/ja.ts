// =====================================================================
// ja Dictionary — 日本語
// =====================================================================
// Mirror zh-CN shape — keep keys 1:1 for type safety
// =====================================================================

import type { Dictionary } from "./dictionary";

export const ja: Dictionary = {
  common: {
    appName: "Star",
    appTagline: "Vibe Coding Work Management",
  },
  userMenu: {
    menuLabel: "ユーザーメニュー: {name} ({role})",
    themeEngine: "テーマエンジン",
    themeDark: "ダーク · オブシディアン",
    themeLight: "ライト · セラミック",
    themeDarkShort: "Neo-Tokyo",
    themeLightShort: "Mecha Lab",
    toolsAndWorkspaces: "ツールとワークスペース",
    agentWindows: "Agent Windows",
    agentWindowsStatus: "稼働中",
    cliProfiles: "CLI プロファイル",
    cliProfilesCount: "{count} 件組み込み",
    apiKeys: "API キー管理",
    apiKeysMode: "デュアルモード",
    profile: "プロフィール",
    settings: "設定",
    signOut: "サインアウト",
    language: "言語",
    languageSwitchHint: "表示言語を切り替える",
  },
  appHeader: {
    workspaceSwitcher: "ワークスペース切替",
    allApps: "ALL APPS",
    appsCount: "25+",
    tacticalJump: "タクティカルジャンプ...",
    notifications: "通知 ({count} 件未読)",
    realtimeOnline: "リアルタイム状態: オンライン",
    synced: "同期中",
    addMoreTabs: "ヘッダーにタブを追加",
    removeFromHeader: "{label} をヘッダーから削除",
  },
  languageSwitcher: {
    current: "現在: {name}",
    title: "表示言語",
  },
};
