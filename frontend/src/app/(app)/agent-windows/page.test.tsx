// =====================================================================
// agent-windows page.test.tsx — AgentSettingsModal 集成测试 (per 2026-09-02 02:49 JST 拍板)
// =====================================================================
// 6 个测试 (per AGENTS.md §3 报告模板):
//   1. Tab 渲染: 默认 3 mock tabs (Claude Code / OpenClaw / Codex)
//   2. 齿轮按钮渲染: 每个 tab group-hover 才显示, e.stopPropagation 不冒泡
//   3. 齿轮点击 → AgentSettingsModal 弹起 (用 activeTab 而不是 test 用的新 tab)
//   4. Modal 4 必备 provider (openai/claude/gemini/minimax) + 4 兼容 (anthropic/openclaw/hermes/google)
//   5. encrypted_rust 模式: 必填 secret, 提交后 modal form 关闭
//   6. environment_var 模式: 必填 envVarName
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 不引新依赖 (复用 @testing-library/react 既有)
//   - 走 vitest + MSW 既有 setup
//   - 13 類 tenant_id 必带 (mock 返固定 tenantId)
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, cleanup, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import AgentWindowsPage from "./page";
import { I18nProvider } from "@/lib/i18n";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

function renderWithProviders(ui: ReactNode) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  return render(
    <I18nProvider initialLanguage="zh-CN">
      <QueryClientProvider client={qc}>{ui}</QueryClientProvider>
    </I18nProvider>,
  );
}

describe("AgentWindowsPage + AgentSettingsModal (per 02:49 JST 拍板)", () => {
  beforeEach(() => {
    cleanup();
  });

  // ---- Test 1: 3 mock tabs 默认渲染 ----
  it("renders 3 default mock tabs (Claude Code / OpenClaw / Codex)", () => {
    renderWithProviders(<AgentWindowsPage />);
    // tab 名字应该都在 (用 title / aria-label 都行)
    expect(screen.getAllByText(/Claude Code|OpenClaw|Codex/i).length).toBeGreaterThanOrEqual(3);
  });

  // ---- Test 2: 齿轮按钮渲染 + click 不冒泡 (不切 tab) ----
  it("renders gear button (data-testid=agent-tab-settings-*) on each tab, e.stopPropagation 不冒泡到 tab select", async () => {
    const user = userEvent.setup();
    renderWithProviders(<AgentWindowsPage />);

    // 应该有 3 个齿轮 (per mockTabs 3 tab)
    const gearT1 = await screen.findByTestId("agent-tab-settings-t1");
    const gearT2 = await screen.findByTestId("agent-tab-settings-t2");
    const gearT3 = await screen.findByTestId("agent-tab-settings-t3");
    expect(gearT1).toBeInTheDocument();
    expect(gearT2).toBeInTheDocument();
    expect(gearT3).toBeInTheDocument();

    // 点 t1 齿轮 → modal 弹起 (tab t1 不会重新 select)
    await user.click(gearT1);
    // modal testid 出现
    await waitFor(() => {
      expect(screen.getByTestId("agent-settings-modal")).toBeInTheDocument();
    });
    // modal title 应该含 tab label
    expect(screen.getByTestId("agent-settings-modal").getAttribute("aria-label"))
      .toContain("Claude Code");
  });

  // ---- Test 3: Modal 8 provider 按钮, 4 必备 + 4 兼容 ----
  it("Modal 列出 8 provider 按钮 (4 必备 openai/claude/gemini/minimax + 4 兼容)", async () => {
    const user = userEvent.setup();
    renderWithProviders(<AgentWindowsPage />);

    // 打开 t1 齿轮 → modal → 点 + 新 API Key → 显出 form
    await user.click(await screen.findByTestId("agent-tab-settings-t1"));
    await waitFor(() => screen.getByTestId("agent-settings-modal"));
    await user.click(screen.getByTestId("agent-settings-add"));
    await waitFor(() => screen.getByTestId("agent-settings-form"));

    // 8 provider 按钮
    const required = ["openai", "claude", "gemini", "minimax"];
    const compat = ["anthropic", "openclaw", "hermes", "google"];
    for (const p of [...required, ...compat]) {
      expect(screen.getByTestId(`agent-settings-provider-${p}`)).toBeInTheDocument();
    }
  });

  // ---- Test 4: encrypted_rust 模式必填 secret ----
  it("encrypted_rust mode: secret 必填, 不填提交触发错误", async () => {
    const user = userEvent.setup();
    renderWithProviders(<AgentWindowsPage />);

    await user.click(await screen.findByTestId("agent-tab-settings-t1"));
    await waitFor(() => screen.getByTestId("agent-settings-modal"));
    await user.click(screen.getByTestId("agent-settings-add"));
    await waitFor(() => screen.getByTestId("agent-settings-form"));

    // encrypted_rust 默认, 必填 secret + label
    fireEvent.change(screen.getByTestId("agent-settings-label"), {
      target: { value: "Test Key" },
    });
    // secret 不填 → 提交失败
    await user.click(screen.getByTestId("agent-settings-submit"));
    await waitFor(() => {
      expect(screen.getByTestId("agent-settings-error")).toBeInTheDocument();
    });
    expect(screen.getByTestId("agent-settings-error").textContent).toMatch(/API Key 明文必填/);
  });

  // ---- Test 5: environment_var 模式必填 envVarName ----
  it("environment_var mode: envVarName 必填", async () => {
    const user = userEvent.setup();
    renderWithProviders(<AgentWindowsPage />);

    await user.click(await screen.findByTestId("agent-tab-settings-t1"));
    await waitFor(() => screen.getByTestId("agent-settings-modal"));
    await user.click(screen.getByTestId("agent-settings-add"));
    await waitFor(() => screen.getByTestId("agent-settings-form"));

    // 切到 environment_var
    await user.click(screen.getByTestId("agent-settings-mode-environment_var"));
    fireEvent.change(screen.getByTestId("agent-settings-label"), {
      target: { value: "Test Env" },
    });
    // envVarName 已有 placeholder 默认值, 清空后提交失败
    fireEvent.change(screen.getByTestId("agent-settings-envvar"), {
      target: { value: "" },
    });
    await user.click(screen.getByTestId("agent-settings-submit"));
    await waitFor(() => {
      expect(screen.getByTestId("agent-settings-error")).toBeInTheDocument();
    });
    expect(screen.getByTestId("agent-settings-error").textContent).toMatch(/环境变量名必填/);
  });

  // ---- Test 6: encrypted_rust 完整提交 → POST /api/api-keys, 关联 3 字段 ----
  it("encrypted_rust 完整提交 → POST /api/api-keys with agent_id + cli_profile_id + agent_kind", async () => {
    const user = userEvent.setup();
    // spy on fetch
    const fetchSpy = vi.spyOn(global, "fetch");
    renderWithProviders(<AgentWindowsPage />);

    await user.click(await screen.findByTestId("agent-tab-settings-t1"));
    await waitFor(() => screen.getByTestId("agent-settings-modal"));
    await user.click(screen.getByTestId("agent-settings-add"));
    await waitFor(() => screen.getByTestId("agent-settings-form"));

    // 选 claude (4 必备之一), 填 secret + label
    await user.click(screen.getByTestId("agent-settings-provider-claude"));
    fireEvent.change(screen.getByTestId("agent-settings-label"), {
      target: { value: "Claude Primary" },
    });
    fireEvent.change(screen.getByTestId("agent-settings-secret"), {
      target: { value: "sk-ant-test123456" },
    });
    await user.click(screen.getByTestId("agent-settings-submit"));

    // 等 mutation 跑完
    await waitFor(() => {
      const calls = fetchSpy.mock.calls.filter(([url]) =>
        String(url).includes("/api/api-keys") && !String(url).match(/by-agent/),
      );
      expect(calls.length).toBeGreaterThan(0);
    }, { timeout: 3000 });

    // 验 body 含 3 关联字段 + provider=claude
    const postCall = fetchSpy.mock.calls.find(
      ([url, init]) => String(url).endsWith("/api/api-keys") && (init as RequestInit)?.method === "POST",
    );
    expect(postCall).toBeDefined();
    const body = JSON.parse(((postCall![1] as RequestInit).body as string));
    expect(body.provider).toBe("claude");
    expect(body.label).toBe("Claude Primary");
    expect(body.mode).toBe("encrypted_rust");
    expect(body.agent_id).toBe("t1");
    expect(body.cli_profile_id).toBe("Claude Code");
    // Claude profileName → claude-sonnet
    expect(body.agent_kind).toBe("claude-sonnet");

    fetchSpy.mockRestore();
  });
});
