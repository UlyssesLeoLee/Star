/*
@cypher schema=1 source_sha256=3afc5841a83a906aed152450e6133d84e1a0b8549b9c136ce5dfd23bcf1ba00c
MERGE (self:File {path:"frontend/src/components/settings/SkillsSettingsPanel.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.test.tsx::SkillsSettingsPanel",kind:"test"})
MERGE (panel:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::SkillsSettingsPanel",kind:"component"})
MERGE (readSkills:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::readSkills",kind:"function"})
MERGE (writeSkills:Symbol {id:"frontend/src/components/settings/SkillsSettingsPanel.tsx::writeSkills",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(panel)
MERGE (suite)-[:TESTS]->(readSkills)
MERGE (suite)-[:TESTS]->(writeSkills)
@endcypher
*/
// SkillsSettingsPanel 组件测试 — 列表/空态/新增弹窗/配置弹窗/卸载确认
// + toggle 即时生效 + localStorage "star-skills" 持久化 + 诚实徽标
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { render, screen, fireEvent, cleanup, waitFor } from "@testing-library/react";
import {
  SkillsSettingsPanel,
  readSkills,
  writeSkills,
  SKILLS_STORAGE_KEY,
  type StarSkill,
} from "./SkillsSettingsPanel";

function seedSkill(overrides: Partial<StarSkill> = {}): StarSkill {
  return {
    id: "skill-test-seed",
    name: "code-review",
    description: "双轴审查: Standards + Spec",
    version: "1.2.0",
    author: "star-team",
    capabilities: ["review", "standards"],
    config: { retries: 3 },
    enabled: true,
    installedAt: "2026-11-16T00:00:00.000Z",
    ...overrides,
  };
}

describe("SkillsSettingsPanel", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });
  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("renders empty state and the honest badge", () => {
    render(<SkillsSettingsPanel />);
    expect(screen.getByTestId("skills-panel")).toBeTruthy();
    expect(screen.getByTestId("skills-empty")).toHaveTextContent("暂无已安装技能");
    expect(screen.getByTestId("skills-honest-badge")).toHaveTextContent(
      "本地持久化 — Skills 注册表 API 接入后同步",
    );
  });

  it("lists persisted skills with name, version, author and capability tags", () => {
    writeSkills([seedSkill()]);
    render(<SkillsSettingsPanel />);
    expect(screen.getByTestId("skill-item-skill-test-seed")).toBeTruthy();
    expect(screen.getByText("code-review")).toBeTruthy();
    expect(screen.getByText("v1.2.0")).toBeTruthy();
    expect(screen.getByText("by star-team")).toBeTruthy();
    expect(screen.getByText("review")).toBeTruthy();
    expect(screen.getByText("standards")).toBeTruthy();
  });

  it("installs a skill from manual JSON and persists to localStorage", () => {
    render(<SkillsSettingsPanel />);
    fireEvent.click(screen.getByTestId("skills-add"));
    expect(screen.getByTestId("skills-add-modal")).toBeTruthy();
    fireEvent.click(screen.getByTestId("skills-add-tab-json"));
    fireEvent.change(screen.getByTestId("skills-add-json"), {
      target: {
        value: JSON.stringify({
          name: "tdd",
          version: "0.9.0",
          author: "star",
          capabilities: ["test", "red-green"],
          configSchema: { type: "object", properties: { strict: { type: "boolean" } } },
        }),
      },
    });
    fireEvent.click(screen.getByTestId("skills-add-submit"));

    const installed = readSkills();
    expect(installed).toHaveLength(1);
    expect(installed[0].name).toBe("tdd");
    expect(installed[0].version).toBe("0.9.0");
    expect(installed[0].author).toBe("star");
    expect(installed[0].capabilities).toEqual(["test", "red-green"]);
    expect(installed[0].enabled).toBe(true);
    expect(screen.getByTestId(`skill-item-${installed[0].id}`)).toBeTruthy();
    expect(screen.queryByTestId("skills-add-modal")).toBeNull();
    expect(window.localStorage.getItem(SKILLS_STORAGE_KEY)).toContain("tdd");
  });

  it("installs a skill from URL via fetch (supports {skill:{...}} wrapper)", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        json: async () => ({
          skill: { name: "web-skill", version: "2.1.0", capabilities: ["web"] },
        }),
      }),
    );
    render(<SkillsSettingsPanel />);
    fireEvent.click(screen.getByTestId("skills-add"));
    fireEvent.change(screen.getByTestId("skills-add-url"), {
      target: { value: "https://skills.example.com/web-skill.json" },
    });
    fireEvent.click(screen.getByTestId("skills-add-submit"));

    await waitFor(() => expect(readSkills()).toHaveLength(1));
    const [installed] = readSkills();
    expect(installed.name).toBe("web-skill");
    expect(installed.capabilities).toEqual(["web"]);
    expect(installed.author).toBe("unknown");
    await waitFor(() => expect(screen.queryByTestId("skills-add-modal")).toBeNull());
  });

  it("shows the official catalog placeholder tab", () => {
    render(<SkillsSettingsPanel />);
    fireEvent.click(screen.getByTestId("skills-add"));
    fireEvent.click(screen.getByTestId("skills-add-tab-catalog"));
    expect(screen.getByTestId("skills-add-catalog-placeholder")).toHaveTextContent(
      "官方技能目录尚未接入",
    );
    expect(screen.getByTestId("skills-add-submit")).toBeDisabled();
  });

  it("rejects invalid JSON and missing name with an inline error", () => {
    render(<SkillsSettingsPanel />);
    fireEvent.click(screen.getByTestId("skills-add"));
    fireEvent.click(screen.getByTestId("skills-add-tab-json"));

    fireEvent.change(screen.getByTestId("skills-add-json"), { target: { value: "{not json" } });
    fireEvent.click(screen.getByTestId("skills-add-submit"));
    expect(screen.getByTestId("skills-add-error")).toHaveTextContent("JSON 解析失败");
    expect(readSkills()).toHaveLength(0);

    fireEvent.change(screen.getByTestId("skills-add-json"), {
      target: { value: JSON.stringify({ version: "1.0.0" }) },
    });
    fireEvent.click(screen.getByTestId("skills-add-submit"));
    expect(screen.getByTestId("skills-add-error")).toHaveTextContent("name");
    expect(readSkills()).toHaveLength(0);
  });

  it("toggle takes effect immediately and persists to localStorage", () => {
    writeSkills([seedSkill()]);
    render(<SkillsSettingsPanel />);
    const toggle = screen.getByTestId("skill-action-toggle-skill-test-seed");
    expect(toggle.getAttribute("aria-checked")).toBe("true");

    fireEvent.click(toggle);
    expect(toggle.getAttribute("aria-checked")).toBe("false");
    expect(readSkills()[0].enabled).toBe(false);

    fireEvent.click(toggle);
    expect(toggle.getAttribute("aria-checked")).toBe("true");
    expect(readSkills()[0].enabled).toBe(true);
  });

  it("renders a form from configSchema and saves values", () => {
    writeSkills([
      seedSkill({
        configSchema: {
          type: "object",
          properties: {
            model: { type: "string", title: "模型", default: "gpt-4" },
            temperature: { type: "number", title: "温度", default: 0.7 },
            verbose: { type: "boolean", title: "详细输出", default: false },
            mode: { type: "string", title: "模式", enum: ["fast", "deep"] },
          },
        },
      }),
    ]);
    render(<SkillsSettingsPanel />);
    fireEvent.click(screen.getByTestId("skill-action-config-skill-test-seed"));
    expect(screen.getByTestId("skills-config-modal")).toBeTruthy();

    expect(screen.getByTestId("skills-config-field-model").getAttribute("value")).toBe("gpt-4");
    fireEvent.change(screen.getByTestId("skills-config-field-temperature"), {
      target: { value: "0.2" },
    });
    fireEvent.click(screen.getByTestId("skills-config-field-verbose"));
    fireEvent.change(screen.getByTestId("skills-config-field-mode"), { target: { value: "deep" } });
    fireEvent.click(screen.getByTestId("skills-config-save"));

    expect(screen.queryByTestId("skills-config-modal")).toBeNull();
    expect(readSkills()[0].config).toEqual({
      model: "gpt-4",
      temperature: 0.2,
      verbose: true,
      mode: "deep",
    });
  });

  it("falls back to a raw JSON editor when configSchema is missing", () => {
    writeSkills([seedSkill({ config: { retries: 3 }, configSchema: undefined })]);
    render(<SkillsSettingsPanel />);
    fireEvent.click(screen.getByTestId("skill-action-config-skill-test-seed"));

    const rawEditor = screen.getByTestId("skills-config-json");
    expect(rawEditor.textContent).toContain("retries");
    fireEvent.change(rawEditor, { target: { value: '{"retries":5,"verbose":true}' } });
    fireEvent.click(screen.getByTestId("skills-config-save"));

    expect(readSkills()[0].config).toEqual({ retries: 5, verbose: true });
  });

  it("rejects malformed raw JSON config without saving", () => {
    writeSkills([seedSkill({ configSchema: undefined, config: {} })]);
    render(<SkillsSettingsPanel />);
    fireEvent.click(screen.getByTestId("skill-action-config-skill-test-seed"));
    fireEvent.change(screen.getByTestId("skills-config-json"), { target: { value: "{oops" } });
    fireEvent.click(screen.getByTestId("skills-config-save"));
    expect(screen.getByTestId("skills-config-error")).toHaveTextContent("JSON 解析失败");
    expect(readSkills()[0].config).toEqual({});
  });

  it("uninstall requires confirmation: cancel keeps, confirm removes", () => {
    writeSkills([seedSkill(), seedSkill({ id: "skill-test-2", name: "tdd" })]);
    render(<SkillsSettingsPanel />);

    fireEvent.click(screen.getByTestId("skill-action-uninstall-skill-test-seed"));
    expect(screen.getByTestId("skills-uninstall-modal")).toHaveTextContent("code-review");
    expect(readSkills()).toHaveLength(2);

    fireEvent.click(screen.getByTestId("skills-uninstall-cancel"));
    expect(screen.queryByTestId("skills-uninstall-modal")).toBeNull();
    expect(readSkills()).toHaveLength(2);

    fireEvent.click(screen.getByTestId("skill-action-uninstall-skill-test-seed"));
    fireEvent.click(screen.getByTestId("skills-uninstall-confirm"));
    expect(readSkills()).toHaveLength(1);
    expect(readSkills()[0].id).toBe("skill-test-2");
    expect(screen.queryByTestId("skill-item-skill-test-seed")).toBeNull();
  });

  it("readSkills tolerates corrupt localStorage payloads", () => {
    window.localStorage.setItem(SKILLS_STORAGE_KEY, "not-json{{{");
    expect(readSkills()).toEqual([]);
    window.localStorage.setItem(SKILLS_STORAGE_KEY, '{"not":"an array"}');
    expect(readSkills()).toEqual([]);
    window.localStorage.setItem(SKILLS_STORAGE_KEY, '[{"id":1,"name":42}, {"id":"ok","name":"good"}]');
    expect(readSkills()).toEqual([expect.objectContaining({ id: "ok", name: "good" })]);
  });
});
