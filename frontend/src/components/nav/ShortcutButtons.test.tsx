// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"frontend/src/components/nav/ShortcutButtons.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/nav/ShortcutButtons.test.tsx::ShortcutButtons tests",kind:"suite"})
MERGE (self)-[:DEFINES]->(suite)
@endcypher
*/
//! 自定义快捷按钮渲染 + tab 感知跳转（per G-6 关闭条件）。
//!
//! 背景：`shortcutButtons` 此前只存在于 navStore，**无任何渲染处** ——
//! 用户添加的快捷按钮不会出现在 UI 上。G-6 的关闭条件是：
//! 「点击后打开的是那个页面的那个选项卡，而不是那个页面的默认选项卡」。
//!
//! mock 的是路由与链接边界（next/navigation / next/link），不是业务数据。

import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ShortcutButtons } from "@/components/nav/ShortcutButtons";
import { useNavStore } from "@/lib/nav/navStore";

const navigation = vi.hoisted(() => ({ push: vi.fn() }));

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: navigation.push }),
  usePathname: () => "/projects",
  useSearchParams: () => new URLSearchParams(window.location.search),
}));

vi.mock("next/link", () => ({
  default: ({ href, target, children, onNavigate, onClick, ...props }: any) => (
    <a
      href="#"
      data-destination={String(href)}
      target={target}
      {...props}
      onClick={(event) => {
        onClick?.(event);
        if (
          event.defaultPrevented ||
          event.metaKey ||
          event.ctrlKey ||
          event.shiftKey ||
          event.altKey ||
          target === "_blank"
        )
          return;
        const destination = new URL(String(href), window.location.origin);
        if (destination.origin !== window.location.origin) return;
        event.preventDefault();
        let blocked = false;
        onNavigate?.({
          preventDefault: () => {
            blocked = true;
            event.preventDefault();
          },
        });
        if (!blocked) navigation.push(String(href));
      }}
    >
      {children}
    </a>
  ),
}));

function seedButtons() {
  useNavStore.setState({
    shortcutButtons: [
      {
        id: "sc-kanban",
        label: "看板",
        icon: "KanbanSquare",
        href: "/projects",
        targetTab: "kanban",
        category: "custom",
        order: 1,
        createdAt: "2026-11-25T00:00:00Z",
      },
      {
        id: "sc-plain",
        label: "缺陷页",
        icon: "Bug",
        href: "/defect",
        category: "custom",
        order: 2,
        createdAt: "2026-11-25T00:00:01Z",
      },
    ],
  });
}

describe("ShortcutButtons (G-6)", () => {
  beforeEach(() => {
    navigation.push.mockReset();
    window.history.replaceState({}, "", "/projects");
    seedButtons();
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("渲染 store 里的自定义快捷按钮", () => {
    render(<ShortcutButtons />);
    expect(screen.getByTestId("shortcut-button-sc-kanban")).toBeTruthy();
    expect(screen.getByTestId("shortcut-button-sc-plain")).toBeTruthy();
    expect(screen.getByText("看板")).toBeTruthy();
  });

  it("带 targetTab 的按钮跳转时带上 ?tab=<targetTab>（不是默认选项卡）", () => {
    render(<ShortcutButtons />);
    fireEvent.click(screen.getByTestId("shortcut-button-sc-kanban"));
    expect(navigation.push).toHaveBeenCalledTimes(1);
    const dest = navigation.push.mock.calls[0][0] as string;
    expect(dest).toContain("tab=kanban");
  });

  it("无 targetTab 的按钮跳转不带 tab 参数", () => {
    render(<ShortcutButtons />);
    fireEvent.click(screen.getByTestId("shortcut-button-sc-plain"));
    expect(navigation.push).toHaveBeenCalledTimes(1);
    const dest = navigation.push.mock.calls[0][0] as string;
    expect(dest).toContain("/defect");
    expect(dest).not.toContain("tab=");
  });

  it("空列表时不渲染任何东西", () => {
    useNavStore.setState({ shortcutButtons: [] });
    render(<ShortcutButtons />);
    expect(screen.queryByTestId("shortcut-buttons")).toBeNull();
  });
});
