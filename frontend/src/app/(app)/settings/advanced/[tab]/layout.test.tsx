/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/(app)/settings/advanced/[tab]/layout.test.tsx",type:"file",language:"tsx"}),
  (test:Function {name:"advancedSettingsTabsStayGrouped",type:"function",signature:"advancedSettingsTabsStayGrouped()",visibility:"private",complexity:"simple"}),
  (render:Function {name:"render",type:"function",signature:"render(element)",visibility:"imported",complexity:"simple"}),
  (screen:Variable {name:"screen",type:"variable",language:"typescript",visibility:"imported"}),
  (layout:Function {name:"AdvancedSettingsLayout",type:"function",signature:"AdvancedSettingsLayout({children})",visibility:"imported",complexity:"simple"}),
  (allModules:Variable {name:"ALL_MODULES",type:"variable",language:"typescript",visibility:"imported"}),
  (sidebarItems:Variable {name:"DEFAULT_SIDEBAR_ITEMS",type:"variable",language:"typescript",visibility:"imported"}),
  (i18nProvider:Function {name:"I18nProvider",type:"function",signature:"I18nProvider({children,initialLanguage})",visibility:"imported",complexity:"simple"}),
  (expect:Function {name:"expect",type:"function",signature:"expect(value)",visibility:"imported",complexity:"simple"}),
  (file)-[:CONTAINS]->(test),(file)-[:CONTAINS]->(render),(file)-[:CONTAINS]->(screen),
  (test)-[:CALLS]->(render),(test)-[:CALLS]->(expect),(test)-[:USES]->(screen),(test)-[:USES]->(layout),
  (test)-[:USES]->(allModules),(test)-[:USES]->(sidebarItems),(test)-[:USES]->(i18nProvider);
*/

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import AdvancedSettingsLayout from "../layout";
import { DEFAULT_SIDEBAR_ITEMS } from "@/lib/nav/navStore";
import { ALL_MODULES } from "@/lib/nav/registry";
import { I18nProvider } from "@/lib/i18n";

describe("Advanced Settings navigation", () => {
  it("keeps Hooks as a sibling tab under the existing Advanced Settings navigation", () => {
    render(<I18nProvider initialLanguage="zh-CN"><AdvancedSettingsLayout><p>Hooks policy</p></AdvancedSettingsLayout></I18nProvider>);

    const advancedSettings = ALL_MODULES.find((module) => module.id === "advanced-settings");
    expect(advancedSettings?.href).toBe("/settings/advanced/hooks");
    expect(DEFAULT_SIDEBAR_ITEMS).toContain("advanced-settings");
    expect(ALL_MODULES.some((module) => module.id === "hooks")).toBe(false);

    const nav = screen.getByRole("navigation", { name: "高级设置导航" });
    const expectedTabs = [
      ["Skills", "/settings/advanced/skills"],
      ["Hooks", "/settings/advanced/hooks"],
      ["MCP", "/settings/advanced/mcp"],
      ["Plugins", "/settings/advanced/plugins"],
    ];

    for (const [label, href] of expectedTabs) {
      const link = screen.getByRole("link", { name: new RegExp(label) });
      expect(nav.contains(link)).toBe(true);
      expect(link.getAttribute("href")).toBe(href);
    }

    expect(screen.getByText("Hooks policy")).toBeTruthy();
  });
});
