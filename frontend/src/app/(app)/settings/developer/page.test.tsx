/*
@cypher schema=1 source_sha256=e7695a8e3244fe64215ac59d11b70bb01b8a8b0c31d25fbc633ed31cc46154db
MERGE (self:File {path:"frontend/src/app/(app)/settings/developer/page.test.tsx"})
MERGE (apiTab:Symbol {id:"frontend/src/app/(app)/settings/developer/page.test.tsx::renders_api_reference_tab_by_default",kind:"test"})
MERGE (webhookCrud:Symbol {id:"frontend/src/app/(app)/settings/developer/page.test.tsx::webhook_crud_roundtrip_in_localstorage",kind:"test"})
MERGE (webhookTab:Symbol {id:"frontend/src/app/(app)/settings/developer/page.test.tsx::url_tab_webhooks_opens_webhooks_tab",kind:"test"})
MERGE (self)-[:DEFINES]->(apiTab)
MERGE (self)-[:DEFINES]->(webhookCrud)
MERGE (self)-[:DEFINES]->(webhookTab)
@endcypher
*/
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import type { ReactNode } from "react";

const mockSearchParams = vi.fn(() => null as URLSearchParams | null);
vi.mock("next/navigation", () => ({
  useSearchParams: () => mockSearchParams(),
}));

import DeveloperSettingsPage from "./page";
import { I18nProvider } from "@/lib/i18n";

function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

describe("Developer Console 页 (per 2026-11-12 导航完善: 补建 /settings/developer)", () => {
  beforeEach(() => {
    cleanup();
    window.localStorage.clear();
    mockSearchParams.mockReturnValue(null);
  });

  it("默认渲染 Developer API 参考 tab", () => {
    renderWithI18n(<DeveloperSettingsPage />);
    expect(screen.getByTestId("developer-api-tab")).toBeTruthy();
    expect(screen.getByTestId("developer-api-table")).toBeTruthy();
    expect(screen.getByTestId("developer-tab-api")).toHaveAttribute("data-active", "true");
  });

  it("URL ?tab=webhooks 打开 webhooks tab (registry href 直达)", () => {
    mockSearchParams.mockReturnValue(new URLSearchParams("tab=webhooks"));
    renderWithI18n(<DeveloperSettingsPage />);
    expect(screen.getByTestId("developer-webhooks-tab")).toBeTruthy();
  });

  it("webhook CRUD roundtrip 写入 localStorage", () => {
    mockSearchParams.mockReturnValue(new URLSearchParams("tab=webhooks"));
    renderWithI18n(<DeveloperSettingsPage />);

    fireEvent.change(screen.getByTestId("webhook-name"), { target: { value: "Feishu 通知" } });
    fireEvent.change(screen.getByTestId("webhook-url"), { target: { value: "https://open.feishu.cn/hook/x" } });
    fireEvent.click(screen.getByTestId("webhook-add"));

    // 列表出现新行
    const rows = screen.getAllByTestId(/^webhook-row-/);
    expect(rows.length).toBe(1);
    expect(screen.getByText("Feishu 通知")).toBeTruthy();

    // localStorage 持久化
    const stored = JSON.parse(window.localStorage.getItem("star.outbound-webhooks.v1") ?? "[]");
    expect(stored.length).toBe(1);
    expect(stored[0].name).toBe("Feishu 通知");
    expect(stored[0].active).toBe(true);

    // toggle 停用
    const rowId = stored[0].id as string;
    fireEvent.click(screen.getByTestId(`webhook-toggle-${rowId}`));
    const after = JSON.parse(window.localStorage.getItem("star.outbound-webhooks.v1") ?? "[]");
    expect(after[0].active).toBe(false);

    // 删除
    fireEvent.click(screen.getByTestId(`webhook-delete-${rowId}`));
    expect(screen.queryByTestId(`webhook-row-${rowId}`)).toBeNull();
  });

  it("test button echoes demo payload without network", () => {
    mockSearchParams.mockReturnValue(new URLSearchParams("tab=webhooks"));
    renderWithI18n(<DeveloperSettingsPage />);
    fireEvent.change(screen.getByTestId("webhook-name"), { target: { value: "T" } });
    fireEvent.change(screen.getByTestId("webhook-url"), { target: { value: "https://x.example" } });
    fireEvent.click(screen.getByTestId("webhook-add"));
    const stored = JSON.parse(window.localStorage.getItem("star.outbound-webhooks.v1") ?? "[]");
    const rowId = stored[0].id as string;
    fireEvent.click(screen.getByTestId(`webhook-test-${rowId}`));
    expect(screen.getByTestId(`webhook-test-result-${rowId}`).textContent).toContain("demo payload");
  });
});
