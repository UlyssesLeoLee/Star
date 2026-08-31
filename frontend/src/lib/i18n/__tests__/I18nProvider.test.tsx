// =====================================================================
// I18nProvider.test.tsx — i18n 核心单测 (per 2026-08-31 实装)
// =====================================================================
// 覆盖:
//   1. 默认 zh-CN (per 用户拍板"默认应该是中文版本")
//   2. setLanguage 切到 en / ja, 字典切换正确
//   3. localStorage 持久化 (key=star-language)
//   4. <html lang="..."> 同步
// =====================================================================

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { act, render, screen, fireEvent, cleanup } from "@testing-library/react";
import {
  I18nProvider,
  useTranslation,
  STORAGE_KEY,
  LANGUAGE_META,
} from "@/lib/i18n";

function Display() {
  const { t, language, setLanguage } = useTranslation();
  return (
    <div>
      <span data-testid="current-lang">{language}</span>
      <span data-testid="signout">{t.userMenu.signOut}</span>
      <span data-testid="tactical">{t.appHeader.tacticalJump}</span>
      <span data-testid="html-lang-attr">{document.documentElement.lang}</span>
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

describe("I18nProvider", () => {
  beforeEach(() => {
    cleanup();
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
    if (typeof document !== "undefined") {
      document.documentElement.lang = "en";
    }
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("defaults to zh-CN when no localStorage entry exists (per 用户拍板 默认中文)", () => {
    render(
      <I18nProvider>
        <Display />
      </I18nProvider>
    );
    expect(screen.getByTestId("current-lang").textContent).toBe("zh-CN");
    // 字典命中 zh-CN 字面量
    expect(screen.getByTestId("signout").textContent).toBe("退出登录");
    expect(screen.getByTestId("tactical").textContent).toBe("战术跳转...");
  });

  it("switches to en when setLanguage('en') is called", () => {
    render(
      <I18nProvider>
        <Display />
      </I18nProvider>
    );
    act(() => {
      fireEvent.click(screen.getByTestId("set-en"));
    });
    expect(screen.getByTestId("current-lang").textContent).toBe("en");
    expect(screen.getByTestId("signout").textContent).toBe("Sign Out");
    expect(screen.getByTestId("tactical").textContent).toBe("Tactical Jump...");
  });

  it("switches to ja when setLanguage('ja') is called", () => {
    render(
      <I18nProvider>
        <Display />
      </I18nProvider>
    );
    act(() => {
      fireEvent.click(screen.getByTestId("set-ja"));
    });
    expect(screen.getByTestId("current-lang").textContent).toBe("ja");
    expect(screen.getByTestId("signout").textContent).toBe("サインアウト");
    expect(screen.getByTestId("tactical").textContent).toBe("タクティカルジャンプ...");
  });

  it("persists selected language to localStorage with key=star-language", () => {
    render(
      <I18nProvider>
        <Display />
      </I18nProvider>
    );
    act(() => {
      fireEvent.click(screen.getByTestId("set-ja"));
    });
    expect(window.localStorage.getItem(STORAGE_KEY)).toBe("ja");
  });

  it("reads stored language on mount (localStorage 'en' -> start as en)", async () => {
    window.localStorage.setItem(STORAGE_KEY, "en");
    render(
      <I18nProvider>
        <Display />
      </I18nProvider>
    );
    // mount 后 effect 才会读 localStorage; 等一次微任务
    await act(async () => {
      await Promise.resolve();
    });
    expect(screen.getByTestId("current-lang").textContent).toBe("en");
    expect(screen.getByTestId("signout").textContent).toBe("Sign Out");
  });

  it("syncs <html lang='...'> attribute to the active language", () => {
    render(
      <I18nProvider>
        <Display />
      </I18nProvider>
    );
    // 默认 zh-CN -> html lang = "zh-CN"
    expect(document.documentElement.lang).toBe(LANGUAGE_META["zh-CN"].htmlLang);
    act(() => {
      fireEvent.click(screen.getByTestId("set-ja"));
    });
    expect(document.documentElement.lang).toBe(LANGUAGE_META.ja.htmlLang);
  });

  it("accepts an initialLanguage override prop (for tests / Storybook)", () => {
    render(
      <I18nProvider initialLanguage="en">
        <Display />
      </I18nProvider>
    );
    expect(screen.getByTestId("current-lang").textContent).toBe("en");
    expect(screen.getByTestId("signout").textContent).toBe("Sign Out");
  });

  it("rejects unknown language by falling back to default", () => {
    window.localStorage.setItem(STORAGE_KEY, "fr-FR");
    render(
      <I18nProvider>
        <Display />
      </I18nProvider>
    );
    // mount 后 effect 读取, "fr-FR" 不在白名单, 保留默认 zh-CN
    expect(screen.getByTestId("current-lang").textContent).toBe("zh-CN");
  });
});
