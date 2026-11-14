/*
@cypher schema=1 source_sha256=f4ec1897d57dae7866e87634cc85801db354c90545c4fa00f5863a7f80816619
MERGE (self:File {path:"frontend/src/lib/nav/__tests__/navStore.test.ts"})
MERGE (store:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (suite:Symbol {id:"frontend/src/lib/nav/__tests__/navStore.test.ts::navStore customization",kind:"suite"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(store)
@endcypher
*/
import { describe, it, expect, beforeEach } from "vitest";
import { useNavStore, DEFAULT_SIDEBAR_ITEMS, DEFAULT_HEADER_TABS } from "../navStore";
import { ALL_MODULES, MODULE_MAP } from "../registry";

describe("navStore customization", () => {
  beforeEach(() => {
    useNavStore.getState().resetToDefault();
    useNavStore.getState().closeMatrix();
  });

  // per 2026-11-14 设计书对齐: 5 组全量承载, sidebar/pinned 默认空
  it("initializes with empty sidebar defaults (5 组全量承载) and header tabs", () => {
    const { sidebarItemIds, headerTabIds } = useNavStore.getState();
    expect(sidebarItemIds).toEqual(DEFAULT_SIDEBAR_ITEMS);
    expect(sidebarItemIds).toEqual([]);
    expect(headerTabIds).toEqual(DEFAULT_HEADER_TABS);
  });

  it("adds and removes items from sidebar dynamically", () => {
    const { addSidebarItem, removeSidebarItem } = useNavStore.getState();
    addSidebarItem("scm");
    expect(useNavStore.getState().sidebarItemIds).toContain("scm");

    removeSidebarItem("scm");
    expect(useNavStore.getState().sidebarItemIds).not.toContain("scm");
  });

  it("adds and removes tabs from header dynamically", () => {
    const { addHeaderTab, removeHeaderTab } = useNavStore.getState();
    addHeaderTab("agent-windows");
    expect(useNavStore.getState().headerTabIds).toContain("agent-windows");

    removeHeaderTab("agent-windows");
    expect(useNavStore.getState().headerTabIds).not.toContain("agent-windows");
  });

  it("toggles items in sidebar and header", () => {
    const { toggleSidebarItem, toggleHeaderTab } = useNavStore.getState();
    toggleSidebarItem("validation");
    expect(useNavStore.getState().sidebarItemIds).toContain("validation");
    toggleSidebarItem("validation");
    expect(useNavStore.getState().sidebarItemIds).not.toContain("validation");

    toggleHeaderTab("settings");
    expect(useNavStore.getState().headerTabIds).toContain("settings");
    toggleHeaderTab("settings");
    expect(useNavStore.getState().headerTabIds).not.toContain("settings");
  });

  it("opens and closes the App Matrix drawer", () => {
    expect(useNavStore.getState().isMatrixOpen).toBe(false);
    useNavStore.getState().openMatrix();
    expect(useNavStore.getState().isMatrixOpen).toBe(true);
    useNavStore.getState().closeMatrix();
    expect(useNavStore.getState().isMatrixOpen).toBe(false);
  });

  it("registry contains all 25+ modules", () => {
    expect(ALL_MODULES.length).toBeGreaterThanOrEqual(25);
    expect(MODULE_MAP.has("inbox")).toBe(true);
    expect(MODULE_MAP.has("issues")).toBe(true);
    expect(MODULE_MAP.has("scm")).toBe(true);
    expect(MODULE_MAP.has("permission")).toBe(true);
    expect(MODULE_MAP.has("tenant")).toBe(true);
  });
});

// =====================================================================
// Sidebar 折叠 / scope / selectedProjectId (per 2026-09-03 12:36 JST 拍板)
// =====================================================================
// 持久化 key: star-nav-store:v2 (bump 自 v1, 加 3 个字段)
// 初始默认值: fold=expanded / scope=main / selectedProjectId=""
// =====================================================================
describe("navStore sidebar fold/scope/selectedProjectId (v2)", () => {
  beforeEach(() => {
    // 重置: 手动设回 default, resetToDefault 不动 fold/scope/projectId
    useNavStore.setState({
      sidebarFold: "expanded",
      sidebarScope: "main",
      selectedProjectId: "",
    });
  });

  it("initializes fold=expanded, scope=main, selectedProjectId=empty", () => {
    const s = useNavStore.getState();
    expect(s.sidebarFold).toBe("expanded");
    expect(s.sidebarScope).toBe("main");
    expect(s.selectedProjectId).toBe("");
  });

  it("toggleSidebarFold: expanded ↔ collapsed", () => {
    const { toggleSidebarFold } = useNavStore.getState();
    expect(useNavStore.getState().sidebarFold).toBe("expanded");
    toggleSidebarFold();
    expect(useNavStore.getState().sidebarFold).toBe("collapsed");
    toggleSidebarFold();
    expect(useNavStore.getState().sidebarFold).toBe("expanded");
  });

  it("setSidebarFold: explicit state setting", () => {
    const { setSidebarFold } = useNavStore.getState();
    setSidebarFold("collapsed");
    expect(useNavStore.getState().sidebarFold).toBe("collapsed");
    setSidebarFold("expanded");
    expect(useNavStore.getState().sidebarFold).toBe("expanded");
  });

  it("setSidebarScope: main ↔ project", () => {
    const { setSidebarScope } = useNavStore.getState();
    setSidebarScope("project");
    expect(useNavStore.getState().sidebarScope).toBe("project");
    setSidebarScope("main");
    expect(useNavStore.getState().sidebarScope).toBe("main");
  });

  it("setSelectedProjectId: persists across reads", () => {
    const { setSelectedProjectId } = useNavStore.getState();
    setSelectedProjectId("proj-abc-123");
    expect(useNavStore.getState().selectedProjectId).toBe("proj-abc-123");
  });

  it("resetToDefault does NOT reset fold/scope/selectedProjectId (per 守门 #11 缺标比错标)", () => {
    const { setSidebarFold, setSidebarScope, setSelectedProjectId, resetToDefault } = useNavStore.getState();
    setSidebarFold("collapsed");
    setSidebarScope("project");
    setSelectedProjectId("proj-keep-me");
    // reset sidebar/pinned/header 走旧逻辑
    useNavStore.getState().toggleHeaderTab("scm"); // 改 header 后再 reset
    resetToDefault();
    const s = useNavStore.getState();
    // header tab 已 reset
    expect(s.headerTabIds).toEqual(DEFAULT_HEADER_TABS);
    // 但 fold/scope/selectedProjectId 不动
    expect(s.sidebarFold).toBe("collapsed");
    expect(s.sidebarScope).toBe("project");
    expect(s.selectedProjectId).toBe("proj-keep-me");
  });

  it("persists fold/scope/selectedProjectId via localStorage star-nav-store:v2", () => {
    // jsdom 默认有 localStorage; 写入并模拟刷新 (重新读)
    const { setSidebarFold, setSidebarScope, setSelectedProjectId } = useNavStore.getState();
    setSidebarFold("collapsed");
    setSidebarScope("project");
    setSelectedProjectId("proj-persist-test");

    // 读 localStorage 验证
    const raw = window.localStorage.getItem("star-nav-store:v2");
    expect(raw).not.toBeNull();
    const parsed = JSON.parse(raw ?? "{}");
    expect(parsed.state.sidebarFold).toBe("collapsed");
    expect(parsed.state.sidebarScope).toBe("project");
    expect(parsed.state.selectedProjectId).toBe("proj-persist-test");
  });

  it("adds Advanced Settings once when migrating saved sidebar preferences", async () => {
    window.localStorage.setItem(
      "star-nav-store:v2",
      JSON.stringify({
        state: {
          sidebarItemIds: ["inbox", "projects"],
          pinnedViewIds: ["kanban"],
          headerTabIds: ["projects"],
          sidebarFold: "collapsed",
          sidebarScope: "project",
          selectedProjectId: "proj-migration-test",
        },
        version: 0,
      })
    );

    await useNavStore.persist.rehydrate();

    // per 2026-11-14: 已映射模块在 migrate 时清理 (组内已显示, 不重复钉选)
    expect(useNavStore.getState().sidebarItemIds).toEqual([]);
    expect(useNavStore.getState().sidebarFold).toBe("collapsed");
    expect(useNavStore.getState().sidebarScope).toBe("project");
    expect(useNavStore.getState().selectedProjectId).toBe("proj-migration-test");

  });

  // per WBS §23.32-A: worktree 由 Project Worktree Index 承载 — migrate 时从
  // sidebar/pinned 双区清理; kanban/timeline/backlog 归并进 pinned (去重)。
  it("migrates v3 state: strips worktree from sidebar/pinned, merges kanban/timeline/backlog into pinned", async () => {
    window.localStorage.setItem(
      "star-nav-store:v2",
      JSON.stringify({
        state: {
          sidebarItemIds: ["inbox", "projects", "worktree"],
          pinnedViewIds: ["worktree", "webhooks"],
          headerTabIds: ["projects"],
          sidebarFold: "expanded",
          sidebarScope: "main",
          selectedProjectId: "",
        },
        version: 3,
      })
    );

    await useNavStore.persist.rehydrate();

    const s = useNavStore.getState();
    // per 2026-11-14: 已映射模块 (inbox/worktree/kanban/... 均入 5 组) 在 migrate 时清理,
    // 用户钉选里未映射的自定义项 (webhooks 在 admin-integration 组 → 也清理) → 全空
    expect(s.sidebarItemIds).toEqual([]);
    expect(s.pinnedViewIds).toEqual([]);
  });

  // per WBS §23.32-A: worktree 禁止写回 sidebar/pinned — 走 add 入口也拦
  it("refuses to add worktree module to sidebar or pinned views", () => {
    useNavStore.getState().addSidebarItem("worktree");
    expect(useNavStore.getState().sidebarItemIds).not.toContain("worktree");
    useNavStore.getState().addPinnedView("worktree");
    expect(useNavStore.getState().pinnedViewIds).not.toContain("worktree");
    // 正常模块不受影响
    useNavStore.getState().addSidebarItem("scm");
    expect(useNavStore.getState().sidebarItemIds).toContain("scm");
  });
});
