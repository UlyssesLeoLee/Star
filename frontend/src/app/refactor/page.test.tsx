// =====================================================================
// /refactor/page.test.tsx — Refactor Sweep 冒烟测试 (per 2026-09-02 拍板)
// =====================================================================
// 范围: 页面级冒烟, 不测深度逻辑 (深度逻辑在 store.test.ts / 单组件 test)
//   1. 页面渲染无 crash
//   2. PageHeader title 显示
//   3. Project switcher 渲染 3 个 chip
//   4. 没 active round 且 project 0 done 任务 -> 空状态
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/react";

// store mock — 默认空
const mockStoreState: Record<string, unknown> = {
  projects: [
    { id: "p-1", key: "PHYSIS", name: "Physis", tenant_id: "t-1", visibility: "private", owner_id: "u-1", member_count: 5, created_at: "2026-01-01T00:00:00Z" },
    { id: "p-2", key: "SG", name: "StarGate", tenant_id: "t-1", visibility: "internal", owner_id: "u-2", member_count: 3, created_at: "2026-02-01T00:00:00Z" },
  ],
  workItems: [],
  refactorRounds: [],
  refactorBoardConfigs: {},
  ensureRefactorBoardConfig: vi.fn(),
  addRefactorColumn: vi.fn(),
  removeRefactorColumn: vi.fn(),
  renameRefactorColumn: vi.fn(),
  reorderRefactorColumns: vi.fn(),
  resetRefactorColumns: vi.fn(),
  setRefactorBatchSize: vi.fn(),
  openRefactorRound: vi.fn(() => "rr-1"),
  closeRefactorRound: vi.fn(),
  startNextRefactorRound: vi.fn(() => "rr-2"),
  moveRefactorCard: vi.fn(),
  addRefactorCard: vi.fn(),
  mergeRefactorCard: vi.fn(() => "ok"),
  worktrees: [],
  transitionWorktree: vi.fn(),
  transitionPR: vi.fn(),
  transitionWorkItem: vi.fn(),
  pullRequests: [],
};

vi.mock("@/lib/store", () => ({
  useStore: (selector: (s: typeof mockStoreState) => unknown) =>
    selector(mockStoreState),
}));

vi.mock("next/navigation", () => ({
  usePathname: () => "/refactor",
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), back: vi.fn() }),
}));

vi.mock("@/lib/i18n", () => ({
  useTranslation: () => ({
    t: {
      pageHeader: {
        trackPill: "Track {track}",
        telemetryTag: "// TELEMETRY",
      },
      ariaLabels: {
        projectSwitcher: "项目切换",
        // v0.6 (per 2026-09-05 拍板 C): 测试 mock 需含所有引用 key
        sidebarScope: "侧栏范围",
        openAppMatrix: "打开 APP 矩阵",
        openCommandBar: "打开命令栏",
        collapse: "折叠",
        expand: "展开",
      },
      refactor: {
        title: "Refactor Sweep 重构专项",
        subtitle: "分批次对已完成任务做重构",
        noDoneWorkItems: "当前项目暂无 status=done 的任务可重构",
        columnsCustomizeHint: "拖动重排",
        historyTitle: "历史重构轮次",
        finishedCards: "已完成",
        addCards: "添加任务",
        batchSizeLabel: "每批卡数",
        batchSizeHint: "默认 5",
        resetColumns: "重置为默认 5 列",
        resetColumnsTitle: "重置后丢失自定义列与命名, 不可恢复",
        resetColumnsConfirm: "确认重置?",
        kpiTodo: "待办",
        kpiDoing: "处理中",
        kpiTesting: "测试中",
        kpiReview: "评审中",
        kpiDone: "已完成",
        refactorRoundBadge: "第 N 次重构",
        refactorMovedAt: "更新于 {time}",
        roundLabel: "Round #",
        totalCards: "总卡数",
        openNextRound: "开启 Round #",
        batchLabel: "批",
        dragToReorder: "拖动 ⋮⋮ 重排",
        addColumn: "+ Add Column",
        addColumnTitle: "添加新重构列 (状态名, 如 spike / blocked)",
        dropCardHere: "拖卡到此",
        wipExceeded: "WIP 超过限制",
        renameColumn: "点击改列名",
        fallbackProtected: "兜底列 {name} 不可删除",
        fallbackNotRemovable: "{name} 不可删除",
      },
    },
    tx: (s: string, _p: unknown) => s,
  }),
  useStatusLabel: (_kind: string, value: string) => value,
  interpolate: (s: string, p: Record<string, unknown>) =>
    s.replace(/\{(\w+)\}/g, (_, k) => String(p[k] ?? "")),
}));

import RefactorPage from "./page";

describe("RefactorPage smoke", () => {
  beforeEach(() => {
    cleanup();
    // 重置 mock store
    Object.keys(mockStoreState).forEach((k) => {
      if (typeof mockStoreState[k] === "function" && k !== "useStore") {
        (mockStoreState[k] as ReturnType<typeof vi.fn>).mockClear?.();
      }
    });
  });

  it("renders without crash", async () => {
    render(<RefactorPage />);
    await waitFor(() => {
      expect(screen.getByTestId("refactor-page")).toBeInTheDocument();
    });
  });

  it("shows PageHeader title", async () => {
    render(<RefactorPage />);
    await waitFor(() => {
      expect(screen.getByText(/Refactor Sweep/)).toBeInTheDocument();
    });
  });

  it("renders project switcher with multiple chips", async () => {
    render(<RefactorPage />);
    await waitFor(() => {
      const switcher = screen.getByTestId("refactor-project-switcher");
      expect(switcher).toBeInTheDocument();
      expect(switcher.querySelectorAll("button").length).toBe(2);
    });
  });

  it("shows empty state when no done work items", async () => {
    render(<RefactorPage />);
    await waitFor(() => {
      expect(screen.getByTestId("refactor-empty")).toBeInTheDocument();
    });
  });
});
