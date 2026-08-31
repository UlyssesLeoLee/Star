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
