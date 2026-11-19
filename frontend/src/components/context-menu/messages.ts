/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/components/context-menu/messages.ts"})
MERGE (uiLangOf:Symbol {id:"frontend/src/components/context-menu/messages.ts::uiLangOf",kind:"function"})
MERGE (CONTEXT_MENU_MESSAGES:Symbol {id:"frontend/src/components/context-menu/messages.ts::CONTEXT_MENU_MESSAGES",kind:"constant"})
MERGE (self)-[:DEFINES]->(uiLangOf)
MERGE (self)-[:DEFINES]->(CONTEXT_MENU_MESSAGES)
@endcypher
*/

/**
 * 右键菜单共享文案 (zh-CN / en / ja 三语)。
 *
 * 不碰共享 dictionary (frontend/src/lib/i18n/*): 新文案在本文件就地定义,
 * 组件按 useTranslation() 暴露的 `language` 取值 (不可得时回退 zh-CN)。
 */

export type UiLang = "zh-CN" | "en" | "ja";

/** 把任意 language 字符串归一为三语之一; 未知值回退 zh-CN。 */
export function uiLangOf(language?: string | null): UiLang {
  if (language === "en" || language === "ja") return language;
  return "zh-CN";
}

export interface ContextMenuMessages {
  /** disabled 菜单项的诚实原因 (未接后端/未实现时展示) */
  pendingBackend: string;
  copySuccess: string;
  copyFailed: string;
}

export const CONTEXT_MENU_MESSAGES: Record<UiLang, ContextMenuMessages> = {
  "zh-CN": {
    pendingBackend: "待后端接入",
    copySuccess: "已复制到剪贴板",
    copyFailed: "复制失败 (剪贴板不可用)",
  },
  en: {
    pendingBackend: "Awaiting backend integration",
    copySuccess: "Copied to clipboard",
    copyFailed: "Copy failed (clipboard unavailable)",
  },
  ja: {
    pendingBackend: "バックエンド接続待ち",
    copySuccess: "クリップボードにコピーしました",
    copyFailed: "コピーに失敗しました (クリップボード利用不可)",
  },
};
