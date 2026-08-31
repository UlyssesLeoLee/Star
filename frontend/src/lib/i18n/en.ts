// =====================================================================
// en Dictionary — English
// =====================================================================
// Mirror zh-CN shape — keep keys 1:1 for type safety
// =====================================================================

import type { Dictionary } from "./dictionary";

export const en: Dictionary = {
  common: {
    appName: "Star",
    appTagline: "Vibe Coding Work Management",
  },
  userMenu: {
    menuLabel: "User menu: {name} ({role})",
    themeEngine: "THEME ENGINE",
    themeDark: "Dark · Obsidian",
    themeLight: "Light · Ceramic",
    themeDarkShort: "Neo-Tokyo",
    themeLightShort: "Mecha Lab",
    toolsAndWorkspaces: "Tools & Workspaces",
    agentWindows: "Agent Windows",
    agentWindowsStatus: "LIVE",
    cliProfiles: "CLI Profiles",
    cliProfilesCount: "{count} built-in",
    apiKeys: "API Key Vault",
    apiKeysMode: "Dual mode",
    profile: "Profile",
    settings: "Settings",
    signOut: "Sign Out",
    language: "Language",
    languageSwitchHint: "Switch interface language",
  },
  appHeader: {
    workspaceSwitcher: "Switch workspace",
    allApps: "ALL APPS",
    appsCount: "25+",
    tacticalJump: "Tactical Jump...",
    notifications: "Notifications ({count} unread)",
    realtimeOnline: "Realtime status: online",
    synced: "SYNCED",
    addMoreTabs: "Add more tabs to header",
    removeFromHeader: "Remove {label} from header",
  },
  languageSwitcher: {
    current: "Current: {name}",
    title: "Interface Language",
  },
};
