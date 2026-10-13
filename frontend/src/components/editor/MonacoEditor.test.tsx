// =====================================================================
// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/editor/MonacoEditor.test.tsx — ULYS-98-W1.1 + sakura 级交互
// =====================================================================
// W1 验收 (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 1.1"):
// "1 unit test (MonacoEditor.test.tsx 渲染 + change event)"
//
// 2026-10-12 用户拍板"确保至少可以替代 sakura editor" 后追加:
// 4. sakura 级 options 缺省全开 (行号/折叠/括号/查找/多光标/概览/补全/格式化)
// 5. feature flag 可逐个关闭 = 退回 W1 基线
// 6. star-dark 主题在 beforeMount 幂等注册
// 7. onEditorReady 逃生舱把底层实例交给 CmdKAction / FileSession CodeLens
//
// 守门合规: 不实际 mount Monaco (jsdom 无 worker / canvas), 只断言传给
// @monaco-editor/react 的 props 契约。0 网络. 0 副作用.
// =====================================================================
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";

// next/dynamic 在 jsdom 下不会真正加载 Monaco, 但会渲染 loading 占位符
// (因为 ssr:false + no chunk resolved). 这是 W1 测试范围内期望行为.
//
// 关键: mock 里**保留** Editor 的 props, 否则 options/theme 一律是 undefined,
// sakura 级断言就变成"断言 undefined === undefined" 的假绿。
type CapturedProps = Record<string, unknown> | undefined;
let captured: CapturedProps;
vi.mock("@monaco-editor/react", () => ({
  Editor: (props: CapturedProps) => {
    captured = props;
    return null;
  },
}));

import MonacoEditor, {
  inferLanguageFromPath,
  STAR_DARK_THEME,
} from "./MonacoEditor";

/** 取最后一次传给 Monaco <Editor> 的 options。 */
function editorOptions(): Record<string, unknown> {
  const props = captured as { options?: Record<string, unknown> } | undefined;
  return props?.options ?? {};
}

beforeEach(() => {
  vi.clearAllMocks();
  captured = undefined;
});

describe("MonacoEditor (ULYS-98-W1.1)", () => {
  it("renders the loading placeholder with data-testid", () => {
    render(
      <MonacoEditor
        value="// hello"
        path="src/lib/store.ts"
        onChange={() => {}}
      />,
    );
    const el = screen.getByTestId("monaco-editor");
    expect(el).toBeTruthy();
    expect(el.getAttribute("data-monaco-path")).toBe("src/lib/store.ts");
    // Language was inferred from path → "typescript".
    expect(el.getAttribute("data-monaco-language")).toBe("typescript");
  });

  it("respects explicit language prop over path-inferred one", () => {
    render(
      <MonacoEditor
        value=""
        language="json"
        path="anything.txt"
        onChange={() => {}}
      />,
    );
    const el = screen.getByTestId("monaco-editor");
    expect(el.getAttribute("data-monaco-language")).toBe("json");
  });

  it("falls back to plaintext for unknown extensions", () => {
    render(
      <MonacoEditor
        value=""
        path="weird.unknownext"
        onChange={() => {}}
      />,
    );
    const el = screen.getByTestId("monaco-editor");
    expect(el.getAttribute("data-monaco-language")).toBe("plaintext");
  });
});

// =====================================================================
// sakura 级交互 (per 2026-10-12 用户拍板 "确保至少可以替代 sakura editor")
// =====================================================================
describe("MonacoEditor — sakura 级交互缺省全开", () => {
  it("行号 + 当前行高亮 (可读性基线)", () => {
    render(<MonacoEditor value="" path="a.rs" />);
    const o = editorOptions();
    expect(o.lineNumbers).toBe("on");
    expect(o.renderLineHighlight).toBe("line");
    // sakura 无 minimap, 用概览标尺替代
    expect(o.minimap).toEqual({ enabled: false });
    expect(o.overviewRulerLanes).toBe(2);
  });

  it("折叠 + 缩进参考线 (结构导航)", () => {
    render(<MonacoEditor value="" path="a.rs" />);
    const o = editorOptions();
    expect(o.folding).toBe(true);
    expect(o.showFoldingControls).toBe("always");
    expect(o.guides).toEqual({ indentation: true, bracketPairs: true });
  });

  it("括号匹配/着色 (语法高亮对位项)", () => {
    render(<MonacoEditor value="" path="a.rs" />);
    expect(editorOptions().bracketPairColorization).toEqual({ enabled: true });
  });

  it("查找替换: 输入即搜 + 回绕 + 选区预填 (sakura 行为)", () => {
    render(<MonacoEditor value="" path="a.rs" />);
    const find = editorOptions().find as Record<string, unknown>;
    expect(find.findOnType).toBe(true);
    expect(find.cursorMoveOnType).toBe(true);
    expect(find.loop).toBe(true);
    expect(find.seedSearchStringFromSelection).toBe("selection");
    expect(find.autoFindInSelection).toBe("multiline");
  });

  it("多光标并行编辑 (sakura 核心能力)", () => {
    render(<MonacoEditor value="" path="a.ts" />);
    const o = editorOptions();
    expect(o.multiCursorModifier).toBe("ctrlCmd");
    expect(o.columnSelection).toBe(true);
    expect(
      screen.getByTestId("monaco-editor").getAttribute("data-monaco-multicursor"),
    ).toBe("on");
  });

  it("自动补全纯键盘驱动: 10ms 延迟 + Tab 接受", () => {
    render(<MonacoEditor value="" path="a.ts" />);
    const o = editorOptions();
    expect(o.quickSuggestions).toEqual({ other: true, comments: false, strings: true });
    expect(o.tabCompletion).toBe("on");
    expect(o.quickSuggestionsDelay).toBe(10);
    expect(o.suggestSelection).toBe("first");
  });

  it("行内 AI ghost text 默认关 (需 BFF), 可显式开", () => {
    const { rerender } = render(<MonacoEditor value="" path="a.ts" />);
    expect(editorOptions().inlineSuggest).toEqual({ enabled: false });
    expect(
      screen.getByTestId("monaco-editor").getAttribute("data-monaco-inline-suggest"),
    ).toBe("off");

    rerender(<MonacoEditor value="" path="a.ts" inlineSuggest />);
    expect(editorOptions().inlineSuggest).toEqual({ enabled: true });
    expect(
      screen.getByTestId("monaco-editor").getAttribute("data-monaco-inline-suggest"),
    ).toBe("on");
  });

  it("粘贴/输入格式化", () => {
    render(<MonacoEditor value="" path="a.ts" />);
    const o = editorOptions();
    expect(o.formatOnPaste).toBe(true);
    expect(o.formatOnType).toBe(true);
  });

  it("Monaco 字体 + 连字 + 平滑滚动/光标 (sakura 手感)", () => {
    render(<MonacoEditor value="" path="a.rs" />);
    const o = editorOptions();
    expect(String(o.fontFamily)).toContain("--font-mono");
    expect(o.fontLigatures).toBe(true);
    expect(o.smoothScrolling).toBe(true);
    expect(o.cursorSmoothCaretAnimation).toBe("on");
    expect(o.cursorBlinking).toBe("smooth");
  });

  it("缺省 star-dark 霓虹主题, 可覆盖", () => {
    const { rerender } = render(<MonacoEditor value="" path="a.rs" />);
    expect((captured as { theme?: string })?.theme).toBe(STAR_DARK_THEME);
    expect(
      screen.getByTestId("monaco-editor").getAttribute("data-monaco-theme"),
    ).toBe(STAR_DARK_THEME);

    rerender(<MonacoEditor value="" path="a.rs" theme="high-contrast" />);
    expect((captured as { theme?: string })?.theme).toBe("high-contrast");
  });

  it("star-dark 主题在 beforeMount 幂等注册一次", () => {
    const defineTheme = vi.fn();
    const monacoMock = {
      editor: {
        defineTheme,
        getModels: () => [],
        setModelLanguage: vi.fn(),
      },
    };
    render(<MonacoEditor value="" path="a.rs" />);
    const beforeMount = (captured as { beforeMount?: (m: unknown) => void })?.beforeMount;
    expect(typeof beforeMount).toBe("function");

    beforeMount?.(monacoMock);
    const firstCall = defineTheme.mock.calls.length;
    expect(firstCall).toBe(1);
    // 第二次调用不应重复注册 (strict mode 双跑)
    beforeMount?.(monacoMock);
    expect(defineTheme.mock.calls.length).toBe(firstCall);
    expect(defineTheme).toHaveBeenCalledWith(
      STAR_DARK_THEME,
      expect.objectContaining({ base: "vs-dark", inherit: true }),
    );
  });

  it("onEditorReady 把底层实例交给 CmdKAction / FileSession CodeLens", () => {
    const onEditorReady = vi.fn();
    // 先 render 再取 captured —— render 之前 captured 还是 undefined。
    render(<MonacoEditor value="" path="a.ts" onEditorReady={onEditorReady} />);
    const onMount = (captured as { onMount?: (e: unknown, m: unknown) => void })?.onMount;
    expect(typeof onMount).toBe("function");
    expect(onEditorReady).not.toHaveBeenCalled(); // mount 前不触发

    const editor = { id: "editor-1" };
    const monacoMock = {
      editor: { defineTheme: vi.fn(), getModels: () => [], setModelLanguage: vi.fn() },
    };
    onMount?.(editor, monacoMock);
    expect(onEditorReady).toHaveBeenCalledWith(editor, monacoMock);
  });
});

// =====================================================================
// feature flag 可逐个关闭 = 退回 W1 基线
// =====================================================================
describe("MonacoEditor — feature flag 可退回基线", () => {
  it("关掉行号/折叠/括号/查找/多光标/概览/补全/格式化", () => {
    render(
      <MonacoEditor
        value=""
        path="a.ts"
        showLineNumbers={false}
        showFolding={false}
        bracketPairColorization={false}
        findWidget={false}
        multiCursor={false}
        overviewRuler={false}
        quickSuggestions={false}
        formatOnPaste={false}
      />,
    );
    const o = editorOptions();
    expect(o.lineNumbers).toBe("off");
    expect(o.renderLineHighlight).toBe("none");
    expect(o.folding).toBe(false);
    expect(o.showFoldingControls).toBe("never");
    expect(o.guides).toEqual({ indentation: false, bracketPairs: false });
    expect(o.bracketPairColorization).toEqual({ enabled: false });
    expect(o.find).toEqual({ addExtraSpaceOnTop: false });
    expect(o.columnSelection).toBe(false);
    expect(o.overviewRulerLanes).toBe(0);
    expect(o.quickSuggestions).toBe(false);
    expect(o.suggestOnTriggerCharacters).toBe(false);
    expect(o.formatOnPaste).toBe(false);
  });

  it("字号 / tab 宽度可配 (Rust 4 / Go tab)", () => {
    render(<MonacoEditor value="" path="a.rs" fontSize={15} tabSize={4} />);
    const o = editorOptions();
    expect(o.fontSize).toBe(15);
    expect(o.tabSize).toBe(4);
    expect(o.insertSpaces).toBe(true);
  });
});

describe("inferLanguageFromPath (ULYS-98-W1.1)", () => {
  it("maps known extensions to Monaco language ids", () => {
    expect(inferLanguageFromPath("foo.ts")).toBe("typescript");
    expect(inferLanguageFromPath("foo.tsx")).toBe("typescript");
    expect(inferLanguageFromPath("foo.rs")).toBe("rust");
    expect(inferLanguageFromPath("foo.py")).toBe("python");
    expect(inferLanguageFromPath("foo.json")).toBe("json");
    expect(inferLanguageFromPath("foo.md")).toBe("markdown");
    expect(inferLanguageFromPath("deep/nested/path/foo.go")).toBe("go");
  });

  it("matches Dockerfile / Makefile by exact filename", () => {
    expect(inferLanguageFromPath("Dockerfile")).toBe("dockerfile");
    expect(inferLanguageFromPath("Makefile")).toBe("makefile");
    expect(inferLanguageFromPath("path/to/Dockerfile")).toBe("dockerfile");
  });

  it("returns plaintext for empty / extension-less / unknown", () => {
    expect(inferLanguageFromPath(undefined)).toBe("plaintext");
    expect(inferLanguageFromPath("")).toBe("plaintext");
    expect(inferLanguageFromPath("README")).toBe("plaintext");
    expect(inferLanguageFromPath("weird.unknownext")).toBe("plaintext");
  });

  it("is case-insensitive on extension", () => {
    expect(inferLanguageFromPath("Foo.TS")).toBe("typescript");
    expect(inferLanguageFromPath("Foo.RS")).toBe("rust");
  });
});