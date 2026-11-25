// =====================================================================
// WorkItemDetailDrawer Test — kind 锁定 (per G-13 / DD §3.2.3)
//
// 覆盖:
//   - lockKind=true (prop): Kind 渲染为只读文本, 无下拉可切换
//   - lockKind 缺省: Kind 为可编辑下拉, prop defaultKind 生效
//   - mode 级 defaultKind / lockKind 兜底 (open.defaultKind / open.lockKind)
//   - prop 级 defaultKind 优先于 mode 级
// =====================================================================

import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { WorkItemDetailDrawer, type WorkItemDetailDrawerProps } from "./WorkItemDetailDrawer";
import { I18nProvider } from "@/lib/i18n";
import { useStore } from "@/lib/store";

function renderDrawer(overrides: Partial<WorkItemDetailDrawerProps> = {}) {
  return render(
    <I18nProvider initialLanguage="zh-CN">
      <WorkItemDetailDrawer
        open={{ kind: "new", defaultStatus: "todo" }}
        onClose={() => undefined}
        projectId="prj-1"
        tenantId="tn-1"
        reporterId="usr-1"
        {...overrides}
      />
    </I18nProvider>,
  );
}

function renderRaw(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

beforeEach(() => {
  useStore.setState({
    workItems: [],
    identities: [],
    agentSessions: [],
    worktrees: [],
    comments: [],
  });
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("WorkItemDetailDrawer kind 锁定 (G-13)", () => {
  it("lockKind=true: Kind 渲染为只读文本 (bug), 无下拉可切换", () => {
    renderDrawer({ defaultKind: "bug", lockKind: true });

    const locked = screen.getByTestId("work-item-drawer-kind-locked");
    expect(locked).toHaveTextContent("bug");
    expect(locked).toHaveAttribute("data-kind-locked", "true");
    // 不存在可编辑的 Kind 下拉
    expect(screen.queryByTestId("work-item-drawer-kind")).not.toBeInTheDocument();
    expect(document.querySelector('[data-testid="work-item-drawer-kind"]')).toBeNull();
  });

  it("lockKind 缺省: Kind 是可编辑下拉, prop defaultKind 生效", () => {
    renderDrawer({ defaultKind: "bug" });

    const select = screen.getByTestId("work-item-drawer-kind") as HTMLSelectElement;
    expect(select.tagName).toBe("SELECT");
    expect(select.value).toBe("bug");
    expect(screen.queryByTestId("work-item-drawer-kind-locked")).not.toBeInTheDocument();
  });

  it("mode 级 defaultKind 仍生效 (open.defaultKind 兜底)", () => {
    renderRaw(
      <WorkItemDetailDrawer
        open={{ kind: "new", defaultStatus: "todo", defaultKind: "spike" }}
        onClose={() => undefined}
        projectId="prj-1"
        tenantId="tn-1"
        reporterId="usr-1"
      />,
    );

    const select = screen.getByTestId("work-item-drawer-kind") as HTMLSelectElement;
    expect(select.value).toBe("spike");
  });

  it("prop defaultKind 优先于 mode 级 defaultKind", () => {
    renderRaw(
      <WorkItemDetailDrawer
        open={{ kind: "new", defaultStatus: "todo", defaultKind: "task" }}
        onClose={() => undefined}
        projectId="prj-1"
        tenantId="tn-1"
        reporterId="usr-1"
        defaultKind="bug"
      />,
    );

    const select = screen.getByTestId("work-item-drawer-kind") as HTMLSelectElement;
    expect(select.value).toBe("bug");
  });

  it("mode 级 lockKind=true 同样锁定 (无需 prop)", () => {
    renderRaw(
      <WorkItemDetailDrawer
        open={{ kind: "new", defaultStatus: "todo", defaultKind: "bug", lockKind: true }}
        onClose={() => undefined}
        projectId="prj-1"
        tenantId="tn-1"
        reporterId="usr-1"
      />,
    );

    expect(screen.getByTestId("work-item-drawer-kind-locked")).toHaveTextContent("bug");
    expect(screen.queryByTestId("work-item-drawer-kind")).not.toBeInTheDocument();
  });
});
