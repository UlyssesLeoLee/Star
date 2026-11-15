/*
@cypher schema=1 source_sha256=197771698870fe40a297a3ef3e423651e107378c8555fa0a8eb97134db54e43e
MERGE (self:File {path:"frontend/src/components/settings/IntegrationPanels.test.tsx"})
MERGE (vitest:ExternalService {name:"vitest"})
MERGE (rtl:ExternalService {name:"@testing-library/react"})
MERGE (integrationPanels:File {path:"frontend/src/components/settings/IntegrationPanels.tsx"})
MERGE (self)-[:IMPORTS]->(vitest)
MERGE (self)-[:IMPORTS]->(rtl)
MERGE (self)-[:IMPORTS]->(integrationPanels)
MERGE (self)-[:TESTS]->(integrationPanels)
@endcypher
*/

// =====================================================================
// IntegrationPanels.test.tsx — 7 面板 + 容器切换测试
// =====================================================================
// 覆盖:
//   1. 容器默认渲染第一面板 (scm)
//   2. active prop 切换面板
//   3. tab 点击切换 + onActiveChange 回调
//   4. ScmPanel: 仓库列表 (git-001..003) + 绑定按钮禁用/未接入/原因
//   5. AutomationPanel: 触发器列表 + 新建触发器禁用/未接入/原因
//   6. WebhooksPanel: 端点表 + 生成密钥禁用/未接入/原因
//   7. SkillsPanel: 技能列表
//   8. PluginsPanel: 插件列表 + 本地 toggle 启停
//   9. HooksPanel: hook 列表
//  10. FlagsPanel: flag 列表 + 本地 toggle 启停
// =====================================================================

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import IntegrationPanels, {
  ScmPanel,
  AutomationPanel,
  WebhooksPanel,
  SkillsPanel,
  PluginsPanel,
  HooksPanel,
  FlagsPanel,
} from "./IntegrationPanels";

describe("IntegrationPanels 容器", () => {
  it("默认渲染第一面板 ScmPanel (settings-scm-panel)", () => {
    render(<IntegrationPanels />);
    expect(screen.getByTestId("settings-integration-panels")).toBeInTheDocument();
    expect(screen.getByTestId("settings-scm-panel")).toBeInTheDocument();
    expect(screen.queryByTestId("settings-flags-panel")).not.toBeInTheDocument();
  });

  it("active prop 切换面板 (active=\"flags\")", () => {
    render(<IntegrationPanels active="flags" />);
    expect(screen.getByTestId("settings-flags-panel")).toBeInTheDocument();
    expect(screen.queryByTestId("settings-scm-panel")).not.toBeInTheDocument();
  });

  it("tab 点击切换面板并回调 onActiveChange", () => {
    const onActiveChange = vi.fn();
    render(<IntegrationPanels onActiveChange={onActiveChange} />);
    fireEvent.click(screen.getByTestId("settings-integration-tab-hooks"));
    expect(screen.getByTestId("settings-hooks-panel")).toBeInTheDocument();
    expect(screen.queryByTestId("settings-scm-panel")).not.toBeInTheDocument();
    expect(onActiveChange).toHaveBeenCalledWith("hooks");
  });
});

describe("ScmPanel 代码托管", () => {
  it("列出 3 个已有仓库 (git-001..003) + 预览数据徽标", () => {
    render(<ScmPanel />);
    expect(screen.getByTestId("settings-scm-panel")).toBeInTheDocument();
    expect(screen.getByTestId("settings-scm-repo-git-001")).toBeInTheDocument();
    expect(screen.getByTestId("settings-scm-repo-git-002")).toBeInTheDocument();
    expect(screen.getByTestId("settings-scm-repo-git-003")).toBeInTheDocument();
    expect(screen.getByTestId("badge-preview")).toHaveTextContent("预览数据");
  });

  it("绑定 Git Provider 按钮禁用 + 未接入徽标 + 原因", () => {
    render(<ScmPanel />);
    expect(screen.getByTestId("settings-scm-bind-provider")).toBeDisabled();
    expect(screen.getByTestId("badge-not-connected")).toHaveTextContent("未接入");
    expect(screen.getByTestId("settings-scm-disabled-reason")).toHaveTextContent("后端未接入");
  });
});

describe("AutomationPanel 自动化触发", () => {
  it("列出 3 个触发器 (trg-001..003)", () => {
    render(<AutomationPanel />);
    expect(screen.getByTestId("settings-automation-panel")).toBeInTheDocument();
    expect(screen.getByTestId("settings-automation-trigger-trg-001")).toBeInTheDocument();
    expect(screen.getByTestId("settings-automation-trigger-trg-002")).toBeInTheDocument();
    expect(screen.getByTestId("settings-automation-trigger-trg-003")).toBeInTheDocument();
  });

  it("新建触发器禁用 + 未接入徽标 + 原因", () => {
    render(<AutomationPanel />);
    expect(screen.getByTestId("settings-automation-new-trigger")).toBeDisabled();
    expect(screen.getByTestId("badge-not-connected")).toHaveTextContent("未接入");
    expect(screen.getByTestId("settings-automation-disabled-reason")).toHaveTextContent("未接入");
  });
});

describe("WebhooksPanel API 与 Webhook", () => {
  it("列出 3 个端点 (wh-001..003, 含路径)", () => {
    render(<WebhooksPanel />);
    expect(screen.getByTestId("settings-webhooks-panel")).toBeInTheDocument();
    expect(screen.getByTestId("settings-webhooks-endpoint-wh-001")).toBeInTheDocument();
    expect(screen.getByTestId("settings-webhooks-endpoint-wh-002")).toBeInTheDocument();
    expect(screen.getByTestId("settings-webhooks-endpoint-wh-003")).toBeInTheDocument();
    expect(screen.getByText("/api/v1/webhooks/workitems/status")).toBeInTheDocument();
  });

  it("生成密钥按钮禁用 + 未接入徽标 + 原因", () => {
    render(<WebhooksPanel />);
    expect(screen.getByTestId("settings-webhooks-generate-secret")).toBeDisabled();
    expect(screen.getByTestId("badge-not-connected")).toHaveTextContent("未接入");
    expect(screen.getByTestId("settings-webhooks-disabled-reason")).toHaveTextContent("未接入");
  });
});

describe("SkillsPanel Skills", () => {
  it("列出 4 个技能 (skl-001..004)", () => {
    render(<SkillsPanel />);
    expect(screen.getByTestId("settings-skills-panel")).toBeInTheDocument();
    expect(screen.getByTestId("settings-skills-item-skl-001")).toBeInTheDocument();
    expect(screen.getByTestId("settings-skills-item-skl-002")).toBeInTheDocument();
    expect(screen.getByTestId("settings-skills-item-skl-003")).toBeInTheDocument();
    expect(screen.getByTestId("settings-skills-item-skl-004")).toBeInTheDocument();
  });
});

describe("PluginsPanel Plugins", () => {
  it("列出 3 个插件 (plg-001..003)", () => {
    render(<PluginsPanel />);
    expect(screen.getByTestId("settings-plugins-panel")).toBeInTheDocument();
    expect(screen.getByTestId("settings-plugins-item-plg-001")).toBeInTheDocument();
    expect(screen.getByTestId("settings-plugins-item-plg-002")).toBeInTheDocument();
    expect(screen.getByTestId("settings-plugins-item-plg-003")).toBeInTheDocument();
  });

  it("toggle 本地启停 (aria-checked 翻转)", () => {
    render(<PluginsPanel />);
    const toggle = screen.getByTestId("settings-plugins-toggle-plg-001");
    expect(toggle).toHaveAttribute("aria-checked", "true");
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-checked", "false");
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-checked", "true");
  });
});

describe("HooksPanel Hooks", () => {
  it("列出 3 个 hook (hk-001..003)", () => {
    render(<HooksPanel />);
    expect(screen.getByTestId("settings-hooks-panel")).toBeInTheDocument();
    expect(screen.getByTestId("settings-hooks-item-hk-001")).toBeInTheDocument();
    expect(screen.getByTestId("settings-hooks-item-hk-002")).toBeInTheDocument();
    expect(screen.getByTestId("settings-hooks-item-hk-003")).toBeInTheDocument();
  });
});

describe("FlagsPanel Feature Flags", () => {
  it("列出 3 个 flag (flag-001..003)", () => {
    render(<FlagsPanel />);
    expect(screen.getByTestId("settings-flags-panel")).toBeInTheDocument();
    expect(screen.getByTestId("settings-flags-item-flag-001")).toBeInTheDocument();
    expect(screen.getByTestId("settings-flags-item-flag-002")).toBeInTheDocument();
    expect(screen.getByTestId("settings-flags-item-flag-003")).toBeInTheDocument();
  });

  it("toggle 本地启停 (aria-checked 翻转)", () => {
    render(<FlagsPanel />);
    const toggle = screen.getByTestId("settings-flags-toggle-flag-001");
    expect(toggle).toHaveAttribute("aria-checked", "true");
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-checked", "false");
  });
});
