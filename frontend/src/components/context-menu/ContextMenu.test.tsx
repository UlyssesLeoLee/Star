/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/components/context-menu/ContextMenu.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/context-menu/ContextMenu.test.tsx::ContextMenuSuite",kind:"suite"})
MERGE (menu:File {path:"frontend/src/components/context-menu/ContextMenu.tsx"})
MERGE (ContextMenu:Symbol {id:"frontend/src/components/context-menu/ContextMenu.tsx::ContextMenu",kind:"component"})
MERGE (self)-[:IMPORTS]->(menu)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(ContextMenu)
@endcypher
*/
import { describe, expect, it, vi, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { ContextMenu, type ContextMenuItem, type ContextMenuState } from "./ContextMenu";

const noop = () => undefined;

function makeState(items: ContextMenuItem[], x = 20, y = 30): ContextMenuState {
  return { open: true, x, y, items };
}

const BASIC_ITEMS: ContextMenuItem[] = [
  { id: "edit", label: "编辑", onSelect: vi.fn() },
  { id: "delete", label: "删除", danger: true, onSelect: vi.fn() },
  { id: "sep", label: "", separator: true },
  { id: "pending", label: "待接入", disabled: true, disabledReason: "待后端接入", onSelect: vi.fn() },
];

describe("ContextMenu 共享右键菜单基建", () => {
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it("createPortal 到 document.body, 稳定 testid", () => {
    const { container } = render(<ContextMenu state={makeState(BASIC_ITEMS)} onClose={noop} />);
    expect(container.querySelector('[data-testid="context-menu"]')).toBeNull(); // 不在挂载容器内
    expect(document.body.querySelector('[data-testid="context-menu"]')).toBeTruthy();
    cleanup();
  });

  it("每个 item 渲染 role=menuitem + testid context-menu-item-<id>", () => {
    render(<ContextMenu state={makeState(BASIC_ITEMS)} onClose={noop} />);
    const menu = document.body.querySelector('[data-testid="context-menu"]') as HTMLElement;
    expect(menu.getAttribute("role")).toBe("menu");
    expect(screen.getByTestId("context-menu-item-edit").getAttribute("role")).toBe("menuitem");
    expect(screen.getByTestId("context-menu-item-delete")).toBeTruthy();
    expect(screen.getByTestId("context-menu-item-pending")).toBeTruthy();
  });

  it("separator 渲染 role=separator, 不渲染按钮", () => {
    render(<ContextMenu state={makeState(BASIC_ITEMS)} onClose={noop} />);
    const sep = document.body.querySelector('[role="separator"]');
    expect(sep).toBeTruthy();
    expect(screen.queryByTestId("context-menu-item-sep")).toBeNull();
  });

  it("点击 item 触发 onSelect 并关闭菜单", () => {
    const onClose = vi.fn();
    const editFn = vi.fn();
    const items: ContextMenuItem[] = [
      { id: "edit", label: "编辑", onSelect: editFn },
    ];
    render(<ContextMenu state={makeState(items)} onClose={onClose} />);
    fireEvent.click(screen.getByTestId("context-menu-item-edit"));
    expect(editFn).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("disabled item 点击不触发 onSelect", () => {
    const onClose = vi.fn();
    const pendingFn = vi.fn();
    const items: ContextMenuItem[] = [
      { id: "pending", label: "待接入", disabled: true, disabledReason: "待后端接入", onSelect: pendingFn },
    ];
    render(<ContextMenu state={makeState(items)} onClose={onClose} />);
    const btn = screen.getByTestId("context-menu-item-pending");
    expect((btn as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(btn);
    expect(pendingFn).not.toHaveBeenCalled();
    expect(btn.getAttribute("title")).toBe("待后端接入");
  });

  it("danger item 带危险色样式", () => {
    render(<ContextMenu state={makeState(BASIC_ITEMS)} onClose={noop} />);
    expect(screen.getByTestId("context-menu-item-delete").className).toContain("text-err");
  });

  it("Esc 关闭菜单", () => {
    const onClose = vi.fn();
    render(<ContextMenu state={makeState(BASIC_ITEMS)} onClose={onClose} />);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("点击外部关闭菜单", () => {
    const onClose = vi.fn();
    render(<ContextMenu state={makeState(BASIC_ITEMS)} onClose={onClose} />);
    fireEvent.pointerDown(document.body, { bubbles: true });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("键盘 ArrowDown + Enter 选中第二项并触发 onSelect", () => {
    const onClose = vi.fn();
    const editFn = vi.fn();
    const deleteFn = vi.fn();
    const items: ContextMenuItem[] = [
      { id: "edit", label: "编辑", onSelect: editFn },
      { id: "delete", label: "删除", danger: true, onSelect: deleteFn },
    ];
    render(<ContextMenu state={makeState(items)} onClose={onClose} />);
    fireEvent.keyDown(window, { key: "ArrowDown" });
    fireEvent.keyDown(window, { key: "Enter" });
    expect(deleteFn).toHaveBeenCalledTimes(1);
    expect(editFn).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("视口边界收敛: 越界坐标翻转到视口内", () => {
    render(<ContextMenu state={makeState(BASIC_ITEMS, 5000, 5000)} onClose={noop} />);
    const menu = document.body.querySelector('[data-testid="context-menu"]') as HTMLElement;
    const left = parseFloat(menu.style.left);
    const top = parseFloat(menu.style.top);
    expect(left).toBeLessThanOrEqual(window.innerWidth - 4);
    expect(top).toBeLessThanOrEqual(window.innerHeight - 4);
  });
});
