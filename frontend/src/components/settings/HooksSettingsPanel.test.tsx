/*
@cypher schema=1 source_sha256=dcf2214a82d64d33dfa36656ae917114646c000de2e1c70c31cfee7f0033ee11
MERGE (self:File {path:"frontend/src/components/settings/HooksSettingsPanel.test.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::HooksSettingsPanel",kind:"component"})
MERGE (readHooks:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::readHooks",kind:"function"})
MERGE (writeHooks:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.tsx::writeHooks",kind:"function"})
MERGE (suite:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::HooksSettingsPanel Hooks 管理面板",kind:"suite"})
MERGE (t1:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::renders empty state, add button and honest badge with no hooks",kind:"test"})
MERGE (t2:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::groups hooks by type with badges, command previews and group sections",kind:"test"})
MERGE (t3:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::toggling a hook flips enabled and persists to localStorage immediately",kind:"test"})
MERGE (t4:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::add flow creates a hook via the form modal and persists it",kind:"test"})
MERGE (t5:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::add form requires name and command before saving",kind:"test"})
MERGE (t6:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::edit flow updates an existing hook and persists it",kind:"test"})
MERGE (t7:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::config modal shows full hook details and closes",kind:"test"})
MERGE (t8:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::config modal edit shortcut opens the edit form prefilled",kind:"test"})
MERGE (t9:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::delete flow confirms first and removes the hook and its group",kind:"test"})
MERGE (t10:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::cancelling delete keeps the hook intact",kind:"test"})
MERGE (t11:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::readHooks returns [] for missing or corrupt data",kind:"test"})
MERGE (t12:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::readHooks skips invalid entries and normalizes defaults",kind:"test"})
MERGE (t13:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::writeHooks and readHooks roundtrip through localStorage",kind:"test"})
MERGE (t14:Symbol {id:"frontend/src/components/settings/HooksSettingsPanel.test.tsx::storage event from another tab refreshes the list",kind:"test"})
MERGE (self)-[:IMPORTS]->(panel)
MERGE (self)-[:IMPORTS]->(readHooks)
MERGE (self)-[:IMPORTS]->(writeHooks)
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
MERGE (suite)-[:DEFINES]->(t11)
MERGE (suite)-[:DEFINES]->(t12)
MERGE (suite)-[:DEFINES]->(t13)
MERGE (suite)-[:DEFINES]->(t14)
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
MERGE (t11)-[:TESTS]->(readHooks)
MERGE (t12)-[:TESTS]->(readHooks)
MERGE (t13)-[:TESTS]->(readHooks)
MERGE (t13)-[:TESTS]->(writeHooks)
MERGE (t14)-[:TESTS]->(writeHooks)
MERGE (t14)-[:TESTS]->(panel)
@endcypher
*/
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";

import { HooksSettingsPanel, HOOKS_STORAGE_KEY, readHooks, writeHooks } from "./HooksSettingsPanel";
import type { HookConfig } from "./HooksSettingsPanel";

const HOOK_FIXTURES: HookConfig[] = [
  {
    id: "hk-pre-1",
    name: "Lint 检查",
    type: "pre_commit",
    command: "npm run lint",
    args: ["--fix"],
    env: { CI: "1" },
    enabled: true,
    description: "提交前 lint",
    matchPattern: "**/*.{ts,tsx}",
  },
  {
    id: "hk-push-1",
    name: "单测门禁",
    type: "pre_push",
    command: "npm test",
    enabled: true,
    description: "",
  },
  {
    id: "hk-merge-1",
    name: "合并后广播",
    type: "post_merge",
    command: "notify --channel release",
    enabled: false,
    description: "通知发布频道",
  },
  {
    id: "hk-custom-1",
    name: "文档同步",
    type: "custom",
    command: "node scripts/sync.js",
    enabled: true,
    description: "",
    matchPattern: "docs/**",
  },
];

describe("HooksSettingsPanel Hooks 管理面板", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  afterEach(cleanup);

  it("renders empty state, add button and honest badge with no hooks", () => {
    render(<HooksSettingsPanel />);
    expect(screen.getByTestId("hooks-panel")).toBeInTheDocument();
    expect(screen.getByTestId("hooks-empty")).toHaveTextContent("暂无 Hooks");
    expect(screen.getByTestId("hook-action-add")).toHaveTextContent("新增 Hook");
    expect(screen.getByTestId("hooks-honest-badge")).toHaveTextContent("本地持久化");
    expect(screen.getByTestId("hooks-honest-badge")).toHaveTextContent("Hooks 执行引擎接入后同步");
    expect(screen.queryByTestId("hook-group-pre_commit")).not.toBeInTheDocument();
  });

  it("groups hooks by type with badges, command previews and group sections", () => {
    writeHooks(HOOK_FIXTURES);
    render(<HooksSettingsPanel />);
    for (const type of ["pre_commit", "pre_push", "post_merge", "custom"]) {
      expect(screen.getByTestId(`hook-group-${type}`)).toBeInTheDocument();
    }
    const pre = within(screen.getByTestId("hook-item-hk-pre-1"));
    expect(pre.getByTestId("hook-type-badge-pre_commit")).toHaveTextContent("pre_commit");
    expect(screen.getByTestId("hook-item-hk-pre-1")).toHaveTextContent("Lint 检查");
    expect(screen.getByTestId("hook-item-hk-pre-1")).toHaveTextContent("npm run lint");
    expect(screen.getByTestId("hook-item-hk-pre-1")).toHaveTextContent("提交前 lint");
    expect(screen.getByTestId("hook-item-hk-pre-1")).toHaveTextContent("已启用");
    expect(screen.getByTestId("hook-item-hk-merge-1")).toHaveTextContent("已停用");
    expect(screen.queryByTestId("hooks-empty")).not.toBeInTheDocument();
  });

  it("toggling a hook flips enabled and persists to localStorage immediately", () => {
    writeHooks(HOOK_FIXTURES);
    render(<HooksSettingsPanel />);
    const toggle = screen.getByTestId("hook-action-toggle-hk-pre-1");
    expect(toggle).toHaveAttribute("aria-checked", "true");
    fireEvent.click(toggle);
    expect(screen.getByTestId("hook-action-toggle-hk-pre-1")).toHaveAttribute("aria-checked", "false");
    const stored = JSON.parse(window.localStorage.getItem(HOOKS_STORAGE_KEY) ?? "[]") as HookConfig[];
    expect(stored.find((h) => h.id === "hk-pre-1")?.enabled).toBe(false);
    fireEvent.click(screen.getByTestId("hook-action-toggle-hk-pre-1"));
    expect(readHooks().find((h) => h.id === "hk-pre-1")?.enabled).toBe(true);
  });

  it("add flow creates a hook via the form modal and persists it", () => {
    render(<HooksSettingsPanel />);
    expect(screen.getByTestId("hooks-empty")).toBeInTheDocument();
    fireEvent.click(screen.getByTestId("hook-action-add"));
    expect(screen.getByTestId("hook-form-title")).toHaveTextContent("新增 Hook");

    fireEvent.change(screen.getByTestId("hook-form-name"), { target: { value: "发布检查" } });
    fireEvent.change(screen.getByTestId("hook-form-type"), { target: { value: "custom" } });
    fireEvent.change(screen.getByTestId("hook-form-command"), { target: { value: "node check.js" } });
    fireEvent.change(screen.getByTestId("hook-form-args"), { target: { value: "--strict\n--watch" } });
    fireEvent.change(screen.getByTestId("hook-form-env"), { target: { value: "CI=1\nNODE_ENV=test" } });
    fireEvent.change(screen.getByTestId("hook-form-pattern"), { target: { value: "**/*.rs" } });
    fireEvent.change(screen.getByTestId("hook-form-description"), { target: { value: "发布前静态检查" } });
    fireEvent.click(screen.getByTestId("hook-form-enabled")); // 取消勾选 → 创建后停用
    fireEvent.click(screen.getByTestId("hook-form-save"));

    const stored = readHooks();
    expect(stored).toHaveLength(1);
    expect(stored[0]).toMatchObject({
      name: "发布检查",
      type: "custom",
      command: "node check.js",
      args: ["--strict", "--watch"],
      env: { CI: "1", NODE_ENV: "test" },
      matchPattern: "**/*.rs",
      description: "发布前静态检查",
      enabled: false,
    });
    expect(screen.queryByTestId("hook-form-modal")).not.toBeInTheDocument();
    expect(screen.queryByTestId("hooks-empty")).not.toBeInTheDocument();
    const row = screen.getByTestId(`hook-item-${stored[0].id}`);
    expect(row).toHaveTextContent("发布检查");
    expect(row).toHaveTextContent("node check.js");
    expect(row).toHaveTextContent("已停用");
  });

  it("add form requires name and command before saving", () => {
    render(<HooksSettingsPanel />);
    fireEvent.click(screen.getByTestId("hook-action-add"));
    expect(screen.getByTestId("hook-form-save")).toBeDisabled();
    fireEvent.change(screen.getByTestId("hook-form-name"), { target: { value: "只有名字" } });
    expect(screen.getByTestId("hook-form-save")).toBeDisabled();
    fireEvent.change(screen.getByTestId("hook-form-command"), { target: { value: "echo hi" } });
    expect(screen.getByTestId("hook-form-save")).toBeEnabled();
  });

  it("edit flow updates an existing hook and persists it", () => {
    writeHooks(HOOK_FIXTURES);
    render(<HooksSettingsPanel />);
    fireEvent.click(screen.getByTestId("hook-action-edit-hk-pre-1"));
    expect(screen.getByTestId("hook-form-title")).toHaveTextContent("编辑 Hook");
    const cmd = screen.getByTestId("hook-form-command");
    expect(cmd).toHaveValue("npm run lint");
    fireEvent.change(cmd, { target: { value: "npm run lint --quiet" } });
    fireEvent.change(screen.getByTestId("hook-form-pattern"), { target: { value: "src/**/*" } });
    fireEvent.click(screen.getByTestId("hook-form-save"));

    const stored = readHooks().find((h) => h.id === "hk-pre-1");
    expect(stored?.command).toBe("npm run lint --quiet");
    expect(stored?.matchPattern).toBe("src/**/*");
    expect(stored?.name).toBe("Lint 检查"); // 其余字段不变
    expect(screen.getByTestId("hook-item-hk-pre-1")).toHaveTextContent("npm run lint --quiet");
  });

  it("config modal shows full hook details and closes", () => {
    writeHooks(HOOK_FIXTURES);
    render(<HooksSettingsPanel />);
    fireEvent.click(screen.getByTestId("hook-action-config-hk-pre-1"));
    const modal = screen.getByTestId("hook-config-modal");
    expect(modal).toHaveTextContent("npm run lint");
    expect(modal).toHaveTextContent("--fix");
    expect(modal).toHaveTextContent("CI=1");
    expect(modal).toHaveTextContent("**/*.{ts,tsx}");
    expect(modal).toHaveTextContent("提交前 lint");
    fireEvent.click(screen.getByTestId("hook-config-close"));
    expect(screen.queryByTestId("hook-config-modal")).not.toBeInTheDocument();
  });

  it("config modal edit shortcut opens the edit form prefilled", () => {
    writeHooks(HOOK_FIXTURES);
    render(<HooksSettingsPanel />);
    fireEvent.click(screen.getByTestId("hook-action-config-hk-pre-1"));
    fireEvent.click(screen.getByTestId("hook-config-edit"));
    expect(screen.queryByTestId("hook-config-modal")).not.toBeInTheDocument();
    expect(screen.getByTestId("hook-form-modal")).toBeInTheDocument();
    expect(screen.getByTestId("hook-form-name")).toHaveValue("Lint 检查");
    expect(screen.getByTestId("hook-form-command")).toHaveValue("npm run lint");
    expect(screen.getByTestId("hook-form-args")).toHaveValue("--fix");
    expect(screen.getByTestId("hook-form-env")).toHaveValue("CI=1");
  });

  it("delete flow confirms first and removes the hook and its group", () => {
    writeHooks(HOOK_FIXTURES);
    render(<HooksSettingsPanel />);
    fireEvent.click(screen.getByTestId("hook-action-delete-hk-merge-1"));
    expect(screen.getByTestId("hook-delete-confirm")).toHaveTextContent("合并后广播");
    fireEvent.click(screen.getByTestId("hook-delete-confirm-yes"));
    expect(screen.queryByTestId("hook-item-hk-merge-1")).not.toBeInTheDocument();
    expect(screen.queryByTestId("hook-group-post_merge")).not.toBeInTheDocument();
    expect(readHooks().find((h) => h.id === "hk-merge-1")).toBeUndefined();
    expect(readHooks()).toHaveLength(3);
  });

  it("cancelling delete keeps the hook intact", () => {
    writeHooks(HOOK_FIXTURES);
    render(<HooksSettingsPanel />);
    fireEvent.click(screen.getByTestId("hook-action-delete-hk-pre-1"));
    expect(screen.getByTestId("hook-delete-confirm")).toBeInTheDocument();
    fireEvent.click(screen.getByTestId("hook-delete-confirm-cancel"));
    expect(screen.getByTestId("hook-item-hk-pre-1")).toBeInTheDocument();
    expect(readHooks()).toHaveLength(4);
  });

  it("readHooks returns [] for missing or corrupt data", () => {
    expect(readHooks()).toEqual([]);
    window.localStorage.setItem(HOOKS_STORAGE_KEY, "{not valid json");
    expect(readHooks()).toEqual([]);
    window.localStorage.setItem(HOOKS_STORAGE_KEY, '{"not":"an array"}');
    expect(readHooks()).toEqual([]);
    window.localStorage.setItem(HOOKS_STORAGE_KEY, '"a string"');
    expect(readHooks()).toEqual([]);
  });

  it("readHooks skips invalid entries and normalizes defaults", () => {
    const mixed = [
      { id: "good", name: "合法", type: "pre_commit", command: "echo ok", enabled: true },
      { name: "缺 id 和 command", type: "pre_commit", enabled: true },
      { id: "bad-type", name: "坏类型", type: "not-a-type", command: "echo x", enabled: true },
      { id: "no-enabled", name: "无 enabled", type: "custom", command: "echo y" },
    ];
    window.localStorage.setItem(HOOKS_STORAGE_KEY, JSON.stringify(mixed));
    expect(readHooks()).toEqual([
      {
        id: "good",
        name: "合法",
        type: "pre_commit",
        command: "echo ok",
        enabled: true,
        args: [],
        env: {},
        description: "",
      },
      {
        id: "no-enabled",
        name: "无 enabled",
        type: "custom",
        command: "echo y",
        enabled: true,
        args: [],
        env: {},
        description: "",
      },
    ]);
  });

  it("writeHooks and readHooks roundtrip through localStorage", () => {
    expect(writeHooks(HOOK_FIXTURES)).toBe(true);
    expect(window.localStorage.getItem(HOOKS_STORAGE_KEY)).toBe(JSON.stringify(HOOK_FIXTURES));
    // readHooks 会补默认值 (args/env/description)
    expect(readHooks()).toEqual(
      HOOK_FIXTURES.map((h) => ({
        ...h,
        args: h.args ?? [],
        env: h.env ?? {},
        description: h.description ?? "",
      })),
    );
  });

  it("storage event from another tab refreshes the list", () => {
    render(<HooksSettingsPanel />);
    expect(screen.getByTestId("hooks-empty")).toBeInTheDocument();
    writeHooks(HOOK_FIXTURES);
    act(() => {
      window.dispatchEvent(new Event("storage"));
    });
    expect(screen.getByTestId("hook-item-hk-pre-1")).toBeInTheDocument();
    expect(screen.queryByTestId("hooks-empty")).not.toBeInTheDocument();
  });
});
