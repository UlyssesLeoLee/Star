// =====================================================================
// CommandBar.test.tsx — ⌘K 消费组件单测 (per DRIFT-α-020 修复守门)
// =====================================================================
// 守门: 消费组件存在 + isOpen=true 渲染 + 过滤 + 键盘 ↑↓ Enter + Esc 关闭
// =====================================================================
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { render, screen, fireEvent, within, cleanup } from "@testing-library/react";
import type { ReactNode } from "react";
import { CommandBar } from "./CommandBar";
import { useCommandBarStore } from "@/lib/commandBarStore";
import { I18nProvider } from "@/lib/i18n";

// per 2026-08-31 i18n 补缺口: CommandBar 内 useTranslation() 必须包 I18nProvider
function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

// next/navigation 路由 mock (Enter 触发 router.push 不需要真导航)
const mockPush = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: (...args: unknown[]) => mockPush(...args) }),
}));

// 简化 ALL_MODULES 引用链: 通过 onMouseEnter 验证选中变化
// 静态 import 会带入 lucide-react 图标组件, 在 jsdom 下不渲染 svg 但不影响事件

describe("CommandBar (per DRIFT-α-020 fix)", () => {
  beforeEach(() => {
    mockPush.mockReset();
    useCommandBarStore.setState({ isOpen: false, query: "", recent: [] });
  });

  afterEach(() => {
    cleanup(); // 清掉上一个 render 残留,避免 getByTestId 命中多个 inbox
  });

  it("A. isOpen=false 时不渲染 panel", () => {
    renderWithI18n(<CommandBar />);
    expect(screen.queryByTestId("command-bar-panel")).toBeNull();
    expect(screen.queryByTestId("command-bar-overlay")).toBeNull();
  });

  it("B. isOpen=true 时渲染 panel + 输入框 + filter chips + 列表项", () => {
    useCommandBarStore.getState().open();
    // 强制填充 query, 让 ALL_MODULES 显示 (没 query 时是 recent 列表)
    useCommandBarStore.getState().setQuery("in");
    renderWithI18n(<CommandBar />);
    expect(screen.getByTestId("command-bar-panel")).toBeTruthy();
    expect(screen.getByTestId("command-bar-input")).toBeTruthy();
    expect(screen.getByTestId("command-bar-filter-chips")).toBeTruthy();
    // query "in" 应该命中多个 module/task/agent (inbox 状态等)
    const list = screen.getByTestId("command-bar-list");
    const items = within(list).getAllByRole("button");
    expect(items.length).toBeGreaterThan(0);
  });

  it("C. 输入 query 过滤, 0 命中时显示空态", () => {
    useCommandBarStore.getState().open();
    useCommandBarStore.getState().setQuery("in");  // 先填 query 才能看到 hits
    renderWithI18n(<CommandBar />);
    const input = screen.getByTestId("command-bar-input");
    fireEvent.change(input, { target: { value: "zzzzz-no-match-zzzzz" } });
    const emptyHints = screen.getAllByText(/0 命中/);
expect(emptyHints.length).toBeGreaterThan(0);
  });

  it("D. 键盘 ↓ 移动 active, Enter 提交 router.push + pushRecent + close", () => {
    useCommandBarStore.getState().open();
    useCommandBarStore.getState().setQuery("in");  // 让 hits 渲染
    renderWithI18n(<CommandBar />);
    const input = screen.getByTestId("command-bar-input");
    const items = screen.getAllByTestId(/^command-bar-hit-module-/);
    // 第 0 项默认 active
    expect(items[0].getAttribute("data-active")).toBe("true");
    // ↓ 一下, 第 1 项变 active
    fireEvent.keyDown(input, { key: "ArrowDown" });
    const itemsAfter = screen.getAllByTestId(/^command-bar-hit-module-/);
    expect(itemsAfter[1].getAttribute("data-active")).toBe("true");
    // Enter 提交
    fireEvent.keyDown(input, { key: "Enter" });
    expect(mockPush).toHaveBeenCalledTimes(1);
    expect(useCommandBarStore.getState().isOpen).toBe(false);
    expect(useCommandBarStore.getState().recent.length).toBe(1);
  });

  it("E. Esc 关闭 panel", () => {
    useCommandBarStore.getState().open();
    renderWithI18n(<CommandBar />);
    expect(screen.getByTestId("command-bar-panel")).toBeTruthy();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(useCommandBarStore.getState().isOpen).toBe(false);
  });

  it("F. 点击列表项直接 commit (pushRecent + close + router.push)", () => {
    useCommandBarStore.getState().open();
    useCommandBarStore.getState().setQuery("in");  // 让 hits 渲染
    renderWithI18n(<CommandBar />);
    const inboxItem = screen.getByTestId("command-bar-hit-module-inbox");
    console.log("DEBUG inboxItem:", inboxItem.textContent, inboxItem.tagName);
    fireEvent.click(inboxItem);
    expect(mockPush).toHaveBeenCalledWith("/inbox");
    expect(useCommandBarStore.getState().isOpen).toBe(false);
    const recent = useCommandBarStore.getState().recent;
    expect(recent[0]?.id).toBe("module:inbox");
  });

  it("G. 重复点同一项, recent 去重 + 最新在前", () => {
    useCommandBarStore.getState().open();
    useCommandBarStore.getState().setQuery("in");
    const { unmount } = renderWithI18n(<CommandBar />);
    fireEvent.click(screen.getByTestId("command-bar-hit-module-inbox"));
    unmount(); // 第一次 panel 关掉, 卸载 DOM
    useCommandBarStore.getState().open();
    useCommandBarStore.getState().setQuery("in");
    renderWithI18n(<CommandBar />);
    fireEvent.click(screen.getByTestId("command-bar-hit-module-inbox"));
    const recent = useCommandBarStore.getState().recent;
    expect(recent.length).toBe(1);
    expect(recent[0]?.id).toBe("module:inbox");
  });
});
