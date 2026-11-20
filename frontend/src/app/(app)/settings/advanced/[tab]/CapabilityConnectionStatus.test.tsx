// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/(app)/settings/advanced/[tab]/CapabilityConnectionStatus.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/CapabilityConnectionStatus.test.tsx::capability connection status tests",kind:"test"})
MERGE (notice:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/CapabilityConnectionStatus.tsx::LocalOnlyConnectionNotice",kind:"component"})
MERGE (mcp:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/McpSettingsPanel.tsx::McpSettingsPanel",kind:"component"})
MERGE (registry:Symbol {id:"frontend/src/lib/settings/capabilityRegistry.ts::capabilityRegistryApi",kind:"class"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(notice)
MERGE (suite)-[:TESTS]->(mcp)
MERGE (suite)-[:TESTS]->(registry)
@endcypher
*/
// per P1 设置收敛任务: 未接通状态条 + 本地配置「保存→重新挂载→值仍在」
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { I18nProvider } from "@/lib/i18n";
import { LocalOnlyConnectionNotice } from "./CapabilityConnectionStatus";
import { McpSettingsPanel } from "./McpSettingsPanel";
import { SkillsSettingsPanel, PluginsSettingsPanel } from "./SkillsPluginsPanels";
import {
  resetCapabilityRegistryForTest,
  getCapabilityRegistry,
} from "@/lib/settings/capabilityRegistry";

function renderWithI18n(ui: React.ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

describe("能力接驳状态条 (未接通能力明确显示状态)", () => {
  beforeEach(() => {
    resetCapabilityRegistryForTest();
    window.localStorage.clear();
  });
  afterEach(() => cleanup());

  it("三个能力面板都渲染未接通状态条, 明确标注仅保存在本地", () => {
    const { unmount } = renderWithI18n(<LocalOnlyConnectionNotice capability="mcp" />);
    expect(screen.getByTestId("mcp-local-only-notice")).toBeTruthy();
    expect(screen.getByText(/未接通/)).toBeTruthy();
    expect(screen.getByText(/localStorage/)).toBeTruthy();
    unmount();

    renderWithI18n(<SkillsSettingsPanel />);
    expect(screen.getByTestId("skills-local-only-notice")).toBeTruthy();
    cleanup();

    renderWithI18n(<PluginsSettingsPanel />);
    expect(screen.getByTestId("plugins-local-only-notice")).toBeTruthy();
  });

  it("MCP 面板顶部显示未接通状态条且不影响既有增删改交互", () => {
    renderWithI18n(<McpSettingsPanel />);
    expect(screen.getByTestId("mcp-local-only-notice")).toBeTruthy();
    expect(screen.getByTestId("mcp-server-mcp-seed-filesystem")).toBeTruthy();
    fireEvent.click(screen.getByTestId("mcp-add-server"));
    fireEvent.change(screen.getByTestId("mcp-form-name"), { target: { value: "postgres" } });
    fireEvent.change(screen.getByTestId("mcp-form-command"), { target: { value: "pg-mcp-server" } });
    fireEvent.click(screen.getByTestId("mcp-form-submit"));
    expect(getCapabilityRegistry().mcpServers.some((s) => s.name === "postgres")).toBe(true);
  });

  it("本地配置保存后重新挂载面板, 值仍在 (刷新后仍有效)", () => {
    const first = renderWithI18n(<McpSettingsPanel />);
    fireEvent.click(screen.getByTestId("mcp-add-server"));
    fireEvent.change(screen.getByTestId("mcp-form-name"), { target: { value: "persist-mcp" } });
    fireEvent.change(screen.getByTestId("mcp-form-command"), { target: { value: "persist-cmd" } });
    fireEvent.click(screen.getByTestId("mcp-form-submit"));
    first.unmount();

    renderWithI18n(<McpSettingsPanel />);
    expect(getCapabilityRegistry().mcpServers.some((s) => s.name === "persist-mcp")).toBe(true);
    expect(screen.getByTestId("mcp-server-list").textContent).toContain("persist-mcp");
    // 重新挂载后仍未接通状态条保持可见 — 不假装服务端持久化
    expect(screen.getByTestId("mcp-local-only-notice")).toBeTruthy();
  });
});
