// =====================================================================
// useModuleTranslation.test.tsx — 模块字段翻译响应式 (per 2026-08-31 补缺口)
// =====================================================================
// 覆盖:
//   1. zh-CN 时拿到中文 label
//   2. 切到 en 拿到英文 label
//   3. 切到 ja 拿到日文 label
//   4. 字典找不到的 id 兜底到 registry 原值 (不崩)
//   5. null / undefined 输入安全
// =====================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { act, render, screen, fireEvent, cleanup } from "@testing-library/react";
import { I18nProvider, useModuleTranslation, useTranslation } from "@/lib/i18n";
import type { ModuleDefinition } from "@/lib/nav/registry";

const MOCK_INBOX: ModuleDefinition = {
  id: "inbox",
  label: "Inbox",
  code: "01",
  href: "/inbox",
  category: "core",
  categoryLabel: "Core Workspace",
  description: "notifications",
  icon: () => null,
  isCore: true,
};

const UNKNOWN_MODULE = {
  id: "not-in-dict",
  label: "Unknown",
  code: "ZZ",
  href: "/zz",
  category: "core" as const,
  categoryLabel: "Core",
  description: "fallback",
  icon: () => null,
};

function Probe({ module: m }: { module: ModuleDefinition | null | undefined }) {
  const t = useModuleTranslation(m ?? null);
  const { setLanguage } = useTranslation();
  return (
    <div>
      <span data-testid="label">{t.label}</span>
      <span data-testid="description">{t.description}</span>
      <span data-testid="categoryLabel">{t.categoryLabel}</span>
      <button data-testid="set-en" onClick={() => setLanguage("en")}>
        EN
      </button>
      <button data-testid="set-ja" onClick={() => setLanguage("ja")}>
        JA
      </button>
      <button data-testid="set-zh" onClick={() => setLanguage("zh-CN")}>
        ZH
      </button>
    </div>
  );
}

describe("useModuleTranslation", () => {
  beforeEach(() => {
    cleanup();
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
  });

  it("returns zh-CN localized fields by default", () => {
    render(
      <I18nProvider>
        <Probe module={MOCK_INBOX} />
      </I18nProvider>
    );
    // v0.6 (per 2026-09-05 拍板 C 全 i18n 接管): zh-CN navModules 接管
    expect(screen.getByTestId("label").textContent).toBe("收件箱");
    expect(screen.getByTestId("description").textContent).toBe(
      "通知中心、@提及与审计流聚合工作台"
    );
    expect(screen.getByTestId("categoryLabel").textContent).toBe("核心工作区");
  });

  it("reacts to setLanguage('en') and returns English fields", () => {
    render(
      <I18nProvider>
        <Probe module={MOCK_INBOX} />
      </I18nProvider>
    );
    act(() => {
      fireEvent.click(screen.getByTestId("set-en"));
    });
    expect(screen.getByTestId("label").textContent).toBe("Inbox");
    expect(screen.getByTestId("description").textContent).toBe(
      "Notifications, @mentions and audit stream aggregation hub"
    );
    expect(screen.getByTestId("categoryLabel").textContent).toBe("Core Workspace");
  });

  it("reacts to setLanguage('ja') and returns Japanese fields", () => {
    render(
      <I18nProvider>
        <Probe module={MOCK_INBOX} />
      </I18nProvider>
    );
    act(() => {
      fireEvent.click(screen.getByTestId("set-ja"));
    });
    expect(screen.getByTestId("label").textContent).toBe("受信箱");
    expect(screen.getByTestId("description").textContent).toBe(
      "通知センター・@メンション・監査ストリーム集約"
    );
    expect(screen.getByTestId("categoryLabel").textContent).toBe("コア作業区");
  });

  it("falls back to registry values when module id is not in dictionary", () => {
    // 不传 I18nProvider 的子组件被 Probe 包, Probe 拿 UNKNOWN_MODULE
    render(
      <I18nProvider>
        <Probe module={UNKNOWN_MODULE} />
      </I18nProvider>
    );
    // 字典无 not-in-dict, 兜底回 registry 字段
    expect(screen.getByTestId("label").textContent).toBe("Unknown");
    expect(screen.getByTestId("description").textContent).toBe("fallback");
    expect(screen.getByTestId("categoryLabel").textContent).toBe("Core");
  });

  it("returns empty defaults when module is null or undefined", () => {
    function NullProbe() {
      const t = useModuleTranslation(null);
      return (
        <div>
          <span data-testid="label">{t.label || "<empty>"}</span>
          <span data-testid="description">{t.description || "<empty>"}</span>
          <span data-testid="categoryLabel">{t.categoryLabel || "<empty>"}</span>
        </div>
      );
    }
    render(
      <I18nProvider>
        <NullProbe />
      </I18nProvider>
    );
    expect(screen.getByTestId("label").textContent).toBe("<empty>");
    expect(screen.getByTestId("description").textContent).toBe("<empty>");
    expect(screen.getByTestId("categoryLabel").textContent).toBe("<empty>");
  });
});
