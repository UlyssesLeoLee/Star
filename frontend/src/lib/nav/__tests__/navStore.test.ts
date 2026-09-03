import { describe, it, expect, beforeEach } from "vitest";
import { useNavStore, DEFAULT_SIDEBAR_ITEMS, DEFAULT_HEADER_TABS } from "../navStore";
import { ALL_MODULES, MODULE_MAP } from "../registry";

describe("navStore customization", () => {
  beforeEach(() => {
    useNavStore.getState().resetToDefault();
    useNavStore.getState().closeMatrix();
  });

  it("initializes with default core items in sidebar and header", () => {
    const { sidebarItemIds, headerTabIds } = useNavStore.getState();
    expect(sidebarItemIds).toEqual(DEFAULT_SIDEBAR_ITEMS);
    expect(headerTabIds).toEqual(DEFAULT_HEADER_TABS);
    expect(sidebarItemIds).toContain("inbox");
    expect(sidebarItemIds).toContain("issues");
    expect(sidebarItemIds).toContain("projects");
    expect(sidebarItemIds).toContain("agents");
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
});
