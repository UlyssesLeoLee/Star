/*
@cypher schema=1 source_sha256=b21546252c480161944ab8f37f21c689c745b65eef79a603da7812cd6cf10171
MERGE (self:File {path:"frontend/src/app/(app)/settings/advanced/[tab]/CapabilityPanels.test.tsx"})
MERGE (mcp:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/CapabilityPanels.test.tsx::McpSettingsPanel",kind:"test"})
MERGE (skills:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/CapabilityPanels.test.tsx::SkillsSettingsPanel",kind:"test"})
MERGE (plugins:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/CapabilityPanels.test.tsx::PluginsSettingsPanel",kind:"test"})
MERGE (registry:Symbol {id:"frontend/src/lib/settings/capabilityRegistry.ts::capabilityRegistryApi",kind:"class"})
MERGE (self)-[:DEFINES]->(mcp)
MERGE (self)-[:DEFINES]->(skills)
MERGE (self)-[:DEFINES]->(plugins)
MERGE (mcp)-[:TESTS]->(registry)
MERGE (skills)-[:TESTS]->(registry)
MERGE (plugins)-[:TESTS]->(registry)
@endcypher
*/
// per WBS §23.33-B: Settings 能力管理面板 (MCP/Skills/Plugins) 组件测试
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { I18nProvider } from "@/lib/i18n";
import { McpSettingsPanel } from "./McpSettingsPanel";
import { SkillsSettingsPanel, PluginsSettingsPanel } from "./SkillsPluginsPanels";
import { resetCapabilityRegistryForTest, getCapabilityRegistry } from "@/lib/settings/capabilityRegistry";

function renderWithI18n(ui: React.ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

describe("Settings capability panels (WBS §23.33-B)", () => {
  beforeEach(() => {
    resetCapabilityRegistryForTest();
    window.localStorage.clear();
  });
  afterEach(() => cleanup());

  it("McpSettingsPanel lists seeded servers with transport and scope badges", () => {
    renderWithI18n(<McpSettingsPanel />);
    expect(screen.getByTestId("advanced-settings-mcp-panel")).toBeTruthy();
    expect(screen.getByTestId("mcp-server-mcp-seed-filesystem")).toBeTruthy();
    expect(screen.getByTestId("mcp-server-mcp-seed-github")).toBeTruthy();
    // 连接类型徽章 stdio / http
    expect(screen.getByText("stdio")).toBeTruthy();
    expect(screen.getByText("http")).toBeTruthy();
  });

  it("McpSettingsPanel adds a new server through the form", () => {
    renderWithI18n(<McpSettingsPanel />);
    fireEvent.click(screen.getByTestId("mcp-add-server"));
    fireEvent.change(screen.getByTestId("mcp-form-name"), { target: { value: "postgres" } });
    fireEvent.change(screen.getByTestId("mcp-form-command"), { target: { value: "pg-mcp-server" } });
    fireEvent.change(screen.getByTestId("mcp-form-transport"), { target: { value: "sse" } });
    fireEvent.change(screen.getByTestId("mcp-form-scope"), { target: { value: "project" } });
    fireEvent.click(screen.getByTestId("mcp-form-submit"));

    const added = getCapabilityRegistry().mcpServers.find((s) => s.name === "postgres");
    expect(added).toBeTruthy();
    expect(added?.transport).toBe("sse");
    expect(added?.scope).toBe("project");
    expect(added?.enabled).toBe(true);
  });

  it("McpSettingsPanel refuses an empty name and surfaces the error", () => {
    renderWithI18n(<McpSettingsPanel />);
    fireEvent.click(screen.getByTestId("mcp-add-server"));
    fireEvent.click(screen.getByTestId("mcp-form-submit"));
    expect(screen.getByTestId("mcp-form-error")).toHaveTextContent(/名称/);
    expect(getCapabilityRegistry().mcpServers.length).toBe(2);
  });

  it("McpSettingsPanel toggles and removes a server", () => {
    renderWithI18n(<McpSettingsPanel />);
    fireEvent.click(screen.getByTestId("mcp-toggle-mcp-seed-filesystem"));
    expect(getCapabilityRegistry().mcpServers.find((s) => s.id === "mcp-seed-filesystem")?.enabled).toBe(false);
    fireEvent.click(screen.getByTestId("mcp-remove-mcp-seed-github"));
    expect(getCapabilityRegistry().mcpServers.find((s) => s.id === "mcp-seed-github")).toBeUndefined();
  });

  it("SkillsSettingsPanel lists, adds, and removes skills", () => {
    renderWithI18n(<SkillsSettingsPanel />);
    expect(screen.getByTestId("skill-skill-seed-code-review")).toBeTruthy();
    fireEvent.click(screen.getByTestId("skills-add"));
    fireEvent.change(screen.getByTestId("skills-form-name"), { target: { value: "my-new-skill" } });
    fireEvent.change(screen.getByTestId("skills-form-version"), { target: { value: "2.0.0" } });
    fireEvent.click(screen.getByTestId("skills-form-submit"));
    const added = getCapabilityRegistry().skills.find((s) => s.name === "my-new-skill");
    expect(added?.version).toBe("2.0.0");
    fireEvent.click(screen.getByTestId("skill-remove-skill-seed-tdd"));
    expect(getCapabilityRegistry().skills.find((s) => s.id === "skill-seed-tdd")).toBeUndefined();
  });

  it("PluginsSettingsPanel lists permissions and adds a plugin with parsed permissions", () => {
    renderWithI18n(<PluginsSettingsPanel />);
    expect(screen.getByTestId("plugin-plugin-seed-kanban-orchestrator")).toBeTruthy();
    // seed 权限 chip 渲染
    expect(screen.getByText("work-item:read")).toBeTruthy();
    fireEvent.click(screen.getByTestId("plugins-add"));
    fireEvent.change(screen.getByTestId("plugins-form-name"), { target: { value: "my-plugin" } });
    fireEvent.change(screen.getByTestId("plugins-form-permissions"), { target: { value: "tenant:read, audit:write" } });
    fireEvent.click(screen.getByTestId("plugins-form-submit"));
    const added = getCapabilityRegistry().plugins.find((p) => p.name === "my-plugin");
    expect(added?.permissions).toEqual(["tenant:read", "audit:write"]);
  });

  it("registry persists additions to localStorage", () => {
    renderWithI18n(<SkillsSettingsPanel />);
    fireEvent.click(screen.getByTestId("skills-add"));
    fireEvent.change(screen.getByTestId("skills-form-name"), { target: { value: "persist-me" } });
    fireEvent.click(screen.getByTestId("skills-form-submit"));
    const raw = window.localStorage.getItem("star-capability-registry:v1");
    expect(raw).toContain("persist-me");
  });
});
