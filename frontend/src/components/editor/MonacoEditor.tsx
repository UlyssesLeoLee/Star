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
import { useMemo } from "react";

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
// MonacoEditor (default export)
// =====================================================================

/**
 * **MonacoEditor** — Star frontend Monaco wrapper (W1).
 *
 * W1 阶段只做 controlled editor + path → language 推断 + change 回调;
 * 不接 AI 补全 / 不接 inline completion (W2 实装 per brief §W2).
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
}: MonacoEditorProps) {
  const resolvedLanguage = useMemo(
    () => language ?? inferLanguageFromPath(path),
    [language, path],
  );

  return (
    <div
      className={className}
      data-testid={testId}
      data-monaco-path={path ?? ""}
      data-monaco-language={resolvedLanguage}
      style={{ height, width: "100%" }}
    >
      <MonacoEditorClient
        height={height}
        language={resolvedLanguage}
        value={value}
        theme="vs-dark"
        options={{
          readOnly,
          minimap: { enabled: false },
          fontSize: 13,
          wordWrap: "on",
          automaticLayout: true,
          scrollBeyondLastLine: false,
          // W1: 0 AI 补全. W2 在这里再 openInlineSuggest + registerCompletionItemProvider.
          inlineSuggest: { enabled: false },
          quickSuggestions: false,
        }}
        onMount={(_editor, monaco) => {
          // Force plaintext model → language, 避免语言未注册时 Monaco 抛错.
          monaco.editor.getModels().forEach((model) => {
            if (model.getLanguageId() !== resolvedLanguage) {
              monaco.editor.setModelLanguage(model, resolvedLanguage);
            }
          });
        }}
        onChange={(next) => {
          if (onChange) onChange(next ?? "");
        }}
      />
    </div>
  );
}