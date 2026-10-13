/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/components/editor/MonacoEditor.tsx",type:"file",language:"typescript"}),
  (props:Class {name:"MonacoEditorProps",type:"interface"}),
  (component:Function {name:"MonacoEditor",type:"function"}),
  (language:Function {name:"inferLanguageFromPath",type:"function"}),
  (file)-[:CONTAINS]->(props),(file)-[:CONTAINS]->(component),(file)-[:CONTAINS]->(language);
*/

// =====================================================================
// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/editor/MonacoEditor.tsx — ULYS-98-W1.1
// =====================================================================
// W1 (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 1.1 frontend 接入 monaco editor")
// wraps `@monaco-editor/react` with a thin Star-friendly surface:
// - Props: `value` / `language` / `onChange` / `path` (path → language fallback)
// - 0 AI 补全 (W2 实装)
// - 0 keyboard AI shortcut (W2 实装)
// - SSR-safe via dynamic loading (Monaco 是 browser-only)
// =====================================================================
"use client";

import dynamic from "next/dynamic";
import { useCallback, useMemo } from "react";
import type * as Monaco from "monaco-editor";

// Lazy-load the official @monaco-editor/react wrapper — Monaco ships Web Workers
// that must NOT execute on the Next.js server side.
const MonacoEditorClient = dynamic(
  () => import("@monaco-editor/react").then((m) => m.Editor),
  {
    ssr: false,
    loading: () => (
      <div
        className="flex h-full w-full items-center justify-center text-sm text-gray-500"
        data-testid="monaco-loading"
      >
        Loading editor…
      </div>
    ),
  },
);

// =====================================================================
// Props
// =====================================================================

export interface MonacoEditorProps {
  /** 当前文件内容 (controlled). */
  value: string;
  /**
   * 显式语言 (`"typescript"` / `"rust"` / `"json"` / …). 如果省略,
   * 从 `path` 推断. Monaco 不识别的语言会回退到 `"plaintext"`.
   */
  language?: string;
  /**
   * 编辑回调. Monaco 每输入一字符都会调一次, 上层负责 debounce
   * (W1: 传透不处理; W2 实装 autosave).
   */
  onChange?: (next: string) => void;
  /**
   * 文件路径 (e.g. `"src/lib/store.ts"`). 仅用于 (1) 在标题栏显示
   * (2) 当 `language` 缺省时推断 language. W2 接 file-system 时
   * 还会用来 resolve 真实文件.
   */
  path?: string;
  /** 编辑器只读开关 (W1: 默认 false). */
  readOnly?: boolean;
  /** 自定义高度 (默认 100%). */
  height?: number | string;
  /** 自定义类名. */
  className?: string;
  /** `data-testid` 透传. */
  testId?: string;
  /**
   * sakura 级交互 (per 2026-10-12 用户拍板"确保至少可以替代 sakura editor"):
   * 逐个 feature flag, 缺省全开 = 与 sakura 同级或更佳; 关掉即退回 W1 基线。
   */
  /** 行号 + 当前行高亮 + 列指示 (sakura 编辑器核心可读性). */
  showLineNumbers?: boolean;
  /** 大纲/符号折叠 + 缩进参考线 (代码结构导航). */
  showFolding?: boolean;
  /** 匹配括号 / 括号着色 (sakura 语法高亮对位项). */
  bracketPairColorization?: boolean;
  /** 查找替换面板 (Ctrl+F / Ctrl+H). */
  findWidget?: boolean;
  /** 多光标 (Alt+Click / Ctrl+D) — sakura 并行编辑能力. */
  multiCursor?: boolean;
  /** 光标处文档概览 (sakura minimap 的等价替代). */
  overviewRuler?: boolean;
  /** 自动补全 (Monaco 内置 provider). sakura 补全是纯键盘驱动, 必须开. */
  quickSuggestions?: boolean;
  /** 行内 AI 建议 (ghost text, Tab 接受) — ULYS-98-W2.3 能力, 需 BFF. */
  inlineSuggest?: boolean;
  /** 格式 / 缩进建议 (Formatter). */
  formatOnPaste?: boolean;
  /** 主题: `"star-dark"` (默认, 跟随 Star 霓虹配色) / `"vs-dark"` / `"light"` / `"high-contrast"`. */
  theme?: string;
  /** 字号 (默认 13). */
  fontSize?: number;
  /** Tab 插入空格数 (默认 2). */
  tabSize?: number;
  /** 暴露底层 Monaco editor 实例, 供 CmdKAction / FileSession CodeLens 挂载. */
  onEditorReady?: (editor: Monaco.editor.IStandaloneCodeEditor, monaco: typeof Monaco) => void;
}

// =====================================================================
// 路径 → language 推断表 (per 守门 #11 缺标比错标 — 显式映射, 不 silently fallback)
// =====================================================================

const EXT_LANGUAGE_MAP: Record<string, string> = {
  ts: "typescript",
  tsx: "typescript",
  js: "javascript",
  jsx: "javascript",
  mjs: "javascript",
  cjs: "javascript",
  json: "json",
  jsonc: "json",
  css: "css",
  scss: "scss",
  less: "less",
  html: "html",
  htm: "html",
  md: "markdown",
  mdx: "markdown",
  yaml: "yaml",
  yml: "yaml",
  toml: "ini",
  rs: "rust",
  py: "python",
  go: "go",
  java: "java",
  kt: "kotlin",
  swift: "swift",
  c: "c",
  h: "c",
  cpp: "cpp",
  cxx: "cpp",
  hpp: "cpp",
  sh: "shell",
  bash: "shell",
  zsh: "shell",
  sql: "sql",
  xml: "xml",
  svg: "xml",
  php: "php",
  rb: "ruby",
  lua: "lua",
  dockerfile: "dockerfile",
};

const FILENAME_LANGUAGE_MAP: Record<string, string> = {
  Dockerfile: "dockerfile",
  Makefile: "makefile",
  ".bashrc": "shell",
  ".zshrc": "shell",
  ".gitignore": "ini",
};

/** 推断 `path` 的语言 (per brief §1.1 `从 worktree file path 推断 language`). */
export function inferLanguageFromPath(path: string | undefined): string {
  if (!path) return "plaintext";
  const filename = path.split("/").pop() ?? path;
  // Exact filename match first (e.g. Dockerfile).
  if (FILENAME_LANGUAGE_MAP[filename]) return FILENAME_LANGUAGE_MAP[filename];

  const dot = filename.lastIndexOf(".");
  if (dot <= 0) return "plaintext";
  const ext = filename.slice(dot + 1).toLowerCase();
  return EXT_LANGUAGE_MAP[ext] ?? "plaintext";
}

// =====================================================================
// Star 暗色主题 (per Star 霓虹赛博配色: cyan #00f0ff / crimson #ff184c)
// =====================================================================

/** Star 霓虹暗色主题名. 关键色板与 globals.css 的 --cel-* token 对齐. */
export const STAR_DARK_THEME = "star-dark";

const STAR_DARK_COLORS: Record<string, string> = {
  "editor.background": "#0b0d10",
  "editor.foreground": "#d1fae5",
  "editorLineNumber.foreground": "#3f4a55",
  "editorLineNumber.activeForeground": "#00f0ff",
  "editorCursor.foreground": "#00f0ff",
  "editor.selectionBackground": "#00f0ff33",
  "editor.inactiveSelectionBackground": "#00f0ff1a",
  "editor.lineHighlightBackground": "#ffffff0a",
  "editorIndentGuide.background1": "#ffffff12",
  "editorIndentGuide.activeBackground1": "#00f0ff55",
  "editorBracketMatch.background": "#00f0ff22",
  "editorBracketMatch.border": "#00f0ff88",
  "editorWhitespace.foreground": "#ffffff18",
  "editorGutter.background": "#0b0d10",
  "editorWidget.background": "#11151c",
  "editorWidget.border": "#00f0ff55",
  "editorSuggestWidget.background": "#11151c",
  "editorSuggestWidget.border": "#00f0ff55",
  "editorSuggestWidget.selectedBackground": "#00f0ff22",
  "editorHoverWidget.background": "#11151c",
  "editorHoverWidget.border": "#00f0ff55",
  "scrollbarSlider.background": "#00f0ff22",
  "scrollbarSlider.hoverBackground": "#00f0ff44",
  "scrollbarSlider.activeBackground": "#00f0ff66",
  "editorOverviewRuler.findMatchForeground": "#ffcf0033",
  "editorStickyScroll.background": "#0b0d10",
};

/**
 * 幂等注册 Star 主题。
 *
 * 注意: `monaco.editor` 上**没有** `getTheme()` (只有 `setTheme`),
 * 所以幂等靠模块级 flag, 而不是查询 API ——
 * strict mode 下 `beforeMount` 会跑两次, 重名 `defineTheme` 不抛错但会
 * 无谓地重建主题对象。
 */
let starDarkThemeRegistered = false;
function ensureStarDarkTheme(monaco: typeof Monaco): void {
  if (starDarkThemeRegistered) return;
  monaco.editor.defineTheme(STAR_DARK_THEME, {
    base: "vs-dark",
    inherit: true,
    rules: [],
    colors: STAR_DARK_COLORS,
  });
  starDarkThemeRegistered = true;
}

// =====================================================================
// MonacoEditor (default export)
// =====================================================================

/**
 * **MonacoEditor** — Star frontend Monaco wrapper (W1 基线 + sakura 级交互)。
 *
 * W1 只做 controlled editor + path → language 推断 + change 回调。
 * 2026-10-12 用户拍板"确保至少可以替代 sakura editor" 后,
 * 缺省打开行号/折叠/括号配色/查找/多光标/概览标尺/自动补全/格式化,
 * 并新增 `star-dark` 霓虹主题 + `onEditorReady` 逃生舱
 * (供 CmdKAction 行内 AI 与 FileSession CodeLens 挂载)。
 */
export default function MonacoEditor({
  value,
  language,
  onChange,
  path,
  readOnly = false,
  height = "100%",
  className,
  testId = "monaco-editor",
  // sakura 级交互 — 缺省全开
  showLineNumbers = true,
  showFolding = true,
  bracketPairColorization = true,
  findWidget = true,
  multiCursor = true,
  overviewRuler = true,
  quickSuggestions = true,
  inlineSuggest = false,
  formatOnPaste = true,
  theme = STAR_DARK_THEME,
  fontSize = 13,
  tabSize = 2,
  onEditorReady,
}: MonacoEditorProps) {
  const resolvedLanguage = useMemo(
    () => language ?? inferLanguageFromPath(path),
    [language, path],
  );

  // 主题必须在 editor 挂载前注册, 否则首帧闪一下 vs-dark。
  const handleBeforeMount = useCallback((monaco: typeof Monaco) => {
    if (theme === STAR_DARK_THEME) ensureStarDarkTheme(monaco);
  }, [theme]);

  return (
    <div
      className={className}
      data-testid={testId}
      data-monaco-path={path ?? ""}
      data-monaco-language={resolvedLanguage}
      data-monaco-theme={theme}
      data-monaco-inline-suggest={inlineSuggest ? "on" : "off"}
      data-monaco-multicursor={multiCursor ? "on" : "off"}
      style={{ height, width: "100%" }}
    >
      <MonacoEditorClient
        height={height}
        language={resolvedLanguage}
        value={value}
        theme={theme}
        beforeMount={handleBeforeMount}
        options={{
          readOnly,
          // ── sakura 对位: 概览标尺代替 minimap。sakura 无 minimap,
          //    右侧细标尺指示选区/搜索命中/错误, 更省横向空间。
          //
          // 注意: 编辑器构造选项里**没有** `overviewRuler` 这个键 ——
          // 它是 model decoration 的属性。构造期只有 `overviewRulerBorder`
          // (画不画那条竖分隔线) 与 `overviewRulerLanes` (占几条 lane 宽度)。
          minimap: { enabled: false },
          overviewRulerBorder: false,
          overviewRulerLanes: overviewRuler ? 2 : 0,
          // ── sakura 对位: 行号 + 当前行号高亮
          lineNumbers: showLineNumbers ? "on" : "off",
          glyphMargin: true,
          renderLineHighlight: showLineNumbers ? "line" : "none",
          // ── sakura 对位: 结构折叠 + 缩进参考线
          folding: showFolding,
          showFoldingControls: showFolding ? "always" : "never",
          guides: { indentation: showFolding, bracketPairs: bracketPairColorization },
          // ── sakura 对位: 括号匹配 / 着色
          bracketPairColorization: { enabled: bracketPairColorization },
          // ── sakura 对位: 查找替换 (Ctrl+F / Ctrl+H)
          // 不用 matchCase — IEditorFindOptions 没有这个键 (大小写在 find API 调用时传参)。
          // seedSearchStringFromSelection="selection": 打开查找即用当前选区预填,
          // 与 sakura 选中词后 Ctrl+F 的行为一致。
          find: findWidget
            ? {
                addExtraSpaceOnTop: false,
                findOnType: true,
                cursorMoveOnType: true,
                seedSearchStringFromSelection: "selection",
                // 到底后回绕到开头 — sakura 查找的行为, 不是停住。
                loop: true,
                closeOnResult: false,
                // 选区跨行时自动限定在选区内查找 (sakura 的 "仅在选中范围查找")。
                // 注意取值只有 "never" | "always" | "multiline" —— "multiline"
                // 即"选区跨多行时"生效, 单行选区不限定, 这正是 sakura 的行为。
                autoFindInSelection: "multiline",
              }
            : { addExtraSpaceOnTop: false },
          // ── sakura 对位: 多光标并行编辑
          // multiCursorModifier 只接受 "ctrlCmd" | "alt"; Monaco 自己在 mac 上
          // 自动把 ctrlCmd 映射成 Cmd, 所以不需要运行时判平台。
          multiCursorModifier: multiCursor ? "ctrlCmd" : "alt",
          columnSelection: multiCursor,
          // ── sakura 对位: 自动补全 (纯键盘驱动)
          quickSuggestions: quickSuggestions
            ? { other: true, comments: false, strings: true }
            : false,
          suggestOnTriggerCharacters: quickSuggestions,
          acceptSuggestionOnCommitCharacter: quickSuggestions,
          tabCompletion: "on",
          // ── sakura 对位: 行内 AI ghost text (ULYS-98-W2.3, 需 BFF)
          inlineSuggest: { enabled: inlineSuggest },
          // ── sakura 对位: 粘贴/输入时格式化
          formatOnPaste,
          formatOnType: true,
          // ── 基础观感
          fontSize,
          tabSize,
          insertSpaces: true,
          fontFamily:
            'var(--font-mono), ui-monospace, "JetBrains Mono", Menlo, monospace',
          fontLigatures: true,
          wordWrap: "on",
          automaticLayout: true,
          scrollBeyondLastLine: false,
          renderWhitespace: "selection",
          smoothScrolling: true,
          cursorBlinking: "smooth",
          cursorSmoothCaretAnimation: "on",
          padding: { top: 12, bottom: 12 },
          suggestSelection: "first",
          // 注意拼写是 quickSuggestionsDelay (复数 Suggestions), 不是 quickSuggestionDelay。
          // 10ms = 接近瞬时; sakura 补全是纯键盘驱动, 延迟感会打断手感。
          quickSuggestionsDelay: 10,
        }}
        onMount={(editor: Monaco.editor.IStandaloneCodeEditor, monaco: typeof Monaco) => {
          // Force plaintext model → language, 避免语言未注册时 Monaco 抛错.
          monaco.editor.getModels().forEach((model) => {
            if (model.getLanguageId() !== resolvedLanguage) {
              monaco.editor.setModelLanguage(model, resolvedLanguage);
            }
          });
          // sakura 对位: 暴露实例给 CmdKAction / FileSession CodeLens
          if (onEditorReady) onEditorReady(editor, monaco);
        }}
        onChange={(next) => {
          if (onChange) onChange(next ?? "");
        }}
      />
    </div>
  );
}
