/*
@cypher schema=1 source_sha256=f3032b1712fcc6bb7a0dbbacc0a2e66b3d78074bf9452ca7bd564b9f9d5ee28e
MERGE (self:File {path:"frontend/src/components/settings/MemorySettingsPanel.test.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.tsx::MemorySettingsPanel",kind:"component"})
MERGE (readMemory:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.tsx::readMemory",kind:"function"})
MERGE (writeMemory:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.tsx::writeMemory",kind:"function"})
MERGE (suite:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.test.tsx::MemorySettingsPanel 总记忆面板",kind:"suite"})
MERGE (t1:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.test.tsx::renders intro bar, global item, honest badge and empty editor",kind:"test"})
MERGE (t2:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.test.tsx::lists project memory items from the projects prop",kind:"test"})
MERGE (t3:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.test.tsx::selecting a project loads its persisted content into the editor",kind:"test"})
MERGE (t4:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.test.tsx::editing marks the draft dirty and updates the char count",kind:"test"})
MERGE (t5:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.test.tsx::keeps the save button disabled while the draft is clean",kind:"test"})
MERGE (t6:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.test.tsx::saves the draft to localStorage, clears dirty and shows success",kind:"test"})
MERGE (t7:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.test.tsx::switching items keeps unsaved drafts per item",kind:"test"})
MERGE (t8:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.test.tsx::registers beforeunload only while a draft is dirty",kind:"test"})
MERGE (t9:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.test.tsx::disables the new project memory button with the honest reason",kind:"test"})
MERGE (t10:Symbol {id:"frontend/src/components/settings/MemorySettingsPanel.test.tsx::readMemory and writeMemory helpers roundtrip through localStorage",kind:"test"})
MERGE (self)-[:IMPORTS]->(panel)
MERGE (self)-[:IMPORTS]->(readMemory)
MERGE (self)-[:IMPORTS]->(writeMemory)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(t1)
MERGE (suite)-[:DEFINES]->(t2)
MERGE (suite)-[:DEFINES]->(t3)
MERGE (suite)-[:DEFINES]->(t4)
MERGE (suite)-[:DEFINES]->(t5)
MERGE (suite)-[:DEFINES]->(t6)
MERGE (suite)-[:DEFINES]->(t7)
MERGE (suite)-[:DEFINES]->(t8)
MERGE (suite)-[:DEFINES]->(t9)
MERGE (suite)-[:DEFINES]->(t10)
MERGE (t1)-[:TESTS]->(panel)
MERGE (t2)-[:TESTS]->(panel)
MERGE (t3)-[:TESTS]->(panel)
MERGE (t4)-[:TESTS]->(panel)
MERGE (t5)-[:TESTS]->(panel)
MERGE (t6)-[:TESTS]->(panel)
MERGE (t7)-[:TESTS]->(panel)
MERGE (t8)-[:TESTS]->(panel)
MERGE (t9)-[:TESTS]->(panel)
MERGE (t10)-[:TESTS]->(panel)
MERGE (t6)-[:TESTS]->(readMemory)
MERGE (t6)-[:TESTS]->(writeMemory)
MERGE (t10)-[:TESTS]->(readMemory)
MERGE (t10)-[:TESTS]->(writeMemory)
@endcypher
*/
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";

import { MemorySettingsPanel, readMemory, writeMemory } from "./MemorySettingsPanel";

const PROJECTS = [
  { id: "proj-alpha", name: "Alpha 项目" },
  { id: "proj-beta", name: "Beta 项目" },
];

describe("MemorySettingsPanel 总记忆面板", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  afterEach(cleanup);

  it("renders intro bar, global item, honest badge and empty editor", () => {
    render(<MemorySettingsPanel />);
    expect(screen.getByTestId("memory-settings-panel")).toBeInTheDocument();
    expect(screen.getByTestId("memory-intro")).toHaveTextContent("项目记忆 = AGENTS.md 式项目级指令文档");
    expect(screen.getByTestId("memory-item-global")).toHaveTextContent("全局记忆");
    expect(screen.getByTestId("memory-item-global")).toHaveTextContent("GLOBAL.md");
    expect(screen.getByTestId("memory-honest-badge")).toHaveTextContent("本地持久化");
    expect(screen.getByTestId("memory-honest-badge")).toHaveTextContent("记忆 API 接入后同步服务端");
    expect(screen.getByTestId("memory-editor")).toHaveValue("");
    expect(screen.getByTestId("memory-editor-count")).toHaveTextContent("0 字");
    expect(screen.queryByTestId("memory-editor-dirty")).not.toBeInTheDocument();
  });

  it("lists project memory items from the projects prop", () => {
    render(<MemorySettingsPanel projects={PROJECTS} />);
    const list = screen.getByTestId("memory-settings-list");
    expect(list).toContainElement(screen.getByTestId("memory-item-global"));
    expect(list).toContainElement(screen.getByTestId("memory-item-proj-alpha"));
    expect(list).toContainElement(screen.getByTestId("memory-item-proj-beta"));
    expect(screen.getByTestId("memory-item-proj-alpha")).toHaveTextContent("Alpha 项目");
    expect(screen.getByTestId("memory-item-proj-beta")).toHaveTextContent("Beta 项目");
  });

  it("selecting a project loads its persisted content into the editor", () => {
    writeMemory("proj-alpha", "# Alpha 记忆\n\n- 约定 A");
    render(<MemorySettingsPanel projects={PROJECTS} />);
    fireEvent.click(screen.getByTestId("memory-item-proj-alpha"));
    expect(screen.getByTestId("memory-editor")).toHaveValue("# Alpha 记忆\n\n- 约定 A");
    expect(screen.queryByTestId("memory-editor-dirty")).not.toBeInTheDocument();
  });

  it("editing marks the draft dirty and updates the char count", () => {
    render(<MemorySettingsPanel />);
    fireEvent.change(screen.getByTestId("memory-editor"), { target: { value: "你好世界" } });
    expect(screen.getByTestId("memory-editor-dirty")).toBeInTheDocument();
    expect(screen.getByTestId("memory-editor-count")).toHaveTextContent("4 字");
  });

  it("keeps the save button disabled while the draft is clean", () => {
    render(<MemorySettingsPanel />);
    expect(screen.getByTestId("memory-editor-save")).toBeDisabled();
    fireEvent.change(screen.getByTestId("memory-editor"), { target: { value: "草稿" } });
    expect(screen.getByTestId("memory-editor-save")).toBeEnabled();
  });

  it("saves the draft to localStorage, clears dirty and shows success", () => {
    render(<MemorySettingsPanel />);
    fireEvent.change(screen.getByTestId("memory-editor"), { target: { value: "# 全局约定" } });
    fireEvent.click(screen.getByTestId("memory-editor-save"));
    expect(window.localStorage.getItem("star-memory-global")).toBe("# 全局约定");
    expect(readMemory("global")).toBe("# 全局约定");
    expect(screen.queryByTestId("memory-editor-dirty")).not.toBeInTheDocument();
    expect(screen.getByTestId("memory-editor-save")).toBeDisabled();
    expect(screen.getByTestId("memory-editor-saved")).toHaveTextContent("已保存到本地");
  });

  it("switching items keeps unsaved drafts per item", () => {
    render(<MemorySettingsPanel projects={PROJECTS} />);
    fireEvent.change(screen.getByTestId("memory-editor"), { target: { value: "全局草稿" } });
    fireEvent.click(screen.getByTestId("memory-item-proj-alpha"));
    expect(screen.getByTestId("memory-editor")).toHaveValue("");
    expect(screen.queryByTestId("memory-editor-dirty")).not.toBeInTheDocument();
    fireEvent.click(screen.getByTestId("memory-item-global"));
    expect(screen.getByTestId("memory-editor")).toHaveValue("全局草稿");
    expect(screen.getByTestId("memory-editor-dirty")).toBeInTheDocument();
  });

  it("registers beforeunload only while a draft is dirty", () => {
    render(<MemorySettingsPanel />);
    let evt = new Event("beforeunload", { cancelable: true });
    window.dispatchEvent(evt);
    expect(evt.defaultPrevented).toBe(false);

    fireEvent.change(screen.getByTestId("memory-editor"), { target: { value: "未保存草稿" } });
    evt = new Event("beforeunload", { cancelable: true });
    window.dispatchEvent(evt);
    expect(evt.defaultPrevented).toBe(true);

    fireEvent.click(screen.getByTestId("memory-editor-save"));
    evt = new Event("beforeunload", { cancelable: true });
    window.dispatchEvent(evt);
    expect(evt.defaultPrevented).toBe(false);
  });

  it("disables the new project memory button with the honest reason", () => {
    render(<MemorySettingsPanel projects={PROJECTS} />);
    expect(screen.getByTestId("memory-item-new")).toBeDisabled();
    expect(screen.getByTestId("memory-item-new-reason")).toHaveTextContent("未接入");
    expect(screen.getByTestId("memory-item-new-reason")).toHaveTextContent("记忆 API 接入后");
  });

  it("readMemory and writeMemory helpers roundtrip through localStorage", () => {
    expect(readMemory("never-written")).toBeNull();
    expect(writeMemory("proj-x", "# X 项目约定")).toBe(true);
    expect(window.localStorage.getItem("star-memory-proj-x")).toBe("# X 项目约定");
    expect(readMemory("proj-x")).toBe("# X 项目约定");
    expect(writeMemory("proj-x", "覆盖后的内容")).toBe(true);
    expect(readMemory("proj-x")).toBe("覆盖后的内容");
  });
});
