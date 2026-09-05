// =====================================================================
// AppShell.test.tsx — AppShell layout test (per design §3 + §8.1)
// =====================================================================
// 3 个测试 (per 任务要求 ≥2):
//   1. 渲染 [data-testid=app-shell] + [data-testid=app-header] + [data-testid=app-main]
//   2. children 透传到 main 区
//   3. 容器用 dark theme (bg-bg text-ink) + 高度 ≥ 64px 顶栏
// =====================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, cleanup } from "@testing-library/react";
import type { ReactNode } from "react";

// Mock next/navigation — AppHeader 通过 usePathname 读 active tab
vi.mock("next/navigation", () => ({
  usePathname: () => "/sprint",
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}));

import { AppShell } from "../AppShell";
import { I18nProvider } from "@/lib/i18n";

// per 2026-08-31 i18n 实装: AppShell -> AppHeader 内 useTranslation() 必须包 I18nProvider
function renderWithI18n(ui: ReactNode) {
  return render(<I18nProvider initialLanguage="zh-CN">{ui}</I18nProvider>);
}

describe("AppShell", () => {
  beforeEach(() => {
    cleanup();
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
  });

  it("renders app-shell + app-header + app-main (per §3 三层架构)", () => {
    renderWithI18n(
      <AppShell>
        <div>panel content</div>
      </AppShell>
    );
    expect(screen.getByTestId("app-shell")).toBeInTheDocument();
    expect(screen.getByTestId("app-header")).toBeInTheDocument();
    expect(screen.getByTestId("app-main")).toBeInTheDocument();
  });

  it("passes children through to app-main (panel page renders inside shell)", () => {
    renderWithI18n(
      <AppShell>
        <p data-testid="child-marker">hello panel</p>
      </AppShell>
    );
    const child = screen.getByTestId("child-marker");
    expect(child).toBeInTheDocument();
    expect(child.textContent).toBe("hello panel");
    // main 区域包含 children
    const main = screen.getByTestId("app-main");
    expect(main.contains(child)).toBe(true);
  });

  it("uses dark theme classes bg-bg text-ink (per §7 token)", () => {
    renderWithI18n(
      <AppShell>
        <span>x</span>
      </AppShell>
    );
    const shell = screen.getByTestId("app-shell");
    expect(shell.className).toMatch(/bg-bg/);
    expect(shell.className).toMatch(/text-ink/);
    // min-h-screen 保证 full viewport height
    expect(shell.className).toMatch(/min-h-screen/);
  });

  it("app-main has min-height calc(100vh - 64px) inline style (per §3 64px 顶栏)", () => {
    renderWithI18n(
      <AppShell>
        <span>x</span>
      </AppShell>
    );
    const main = screen.getByTestId("app-main");
    expect(main.style.minHeight).toBe("calc(100vh - 64px)");
  });
});
