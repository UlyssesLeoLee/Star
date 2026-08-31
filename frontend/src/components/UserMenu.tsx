"use client";

// =====================================================================
// UserMenu — TopBar 右上角用户菜单 (日漫科技 HUD 风格)
// =====================================================================
// 功能整合:
//   1. 身份展示 (Ulysses · SYS // ADMIN)
//   2. 快速双主题切换 (🌙 Neo-Tokyo Dark / ☀️ Mecha Light)
//   3. 核心工具入口 (任务窗口, CLI Profiles, API Keys)
//   4. 个人偏好与全局设置
//   5. 界面语言切换 (zh-CN / en / ja) — 2026-08-31 新增
//   6. 安全退出
// =====================================================================

import { useState, useRef, useEffect } from "react";
import {
  User,
  Terminal,
  Key,
  LogOut,
  Settings,
  ChevronDown,
  Sun,
  Moon,
  Sparkles,
  Shield,
  Cpu,
  Globe,
  Check,
} from "lucide-react";
import Link from "next/link";
import {
  useTranslation,
  LANGUAGE_META,
  SUPPORTED_LANGUAGES,
  type Language,
} from "@/lib/i18n";

export function UserMenu() {
  const { t, tx, language, setLanguage, mounted: i18nMounted } = useTranslation();
  const [open, setOpen] = useState(false);
  const [isDark, setIsDark] = useState(true);
  const [mounted, setMounted] = useState(false);
  const [langMenuOpen, setLangMenuOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const langMenuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setMounted(true);
    if (typeof window !== "undefined") {
      const saved = localStorage.getItem("star-theme");
      if (saved === "light") {
        setIsDark(false);
        document.documentElement.classList.remove("dark");
        document.documentElement.classList.add("light");
      } else {
        setIsDark(true);
        document.documentElement.classList.remove("light");
        document.documentElement.classList.add("dark");
      }
    }
  }, []);
  // i18nMounted is read once to ensure I18nProvider effect has run; we don't gate UI on it
  // because translation dictionary always returns a value (zh-CN is the SSR fallback).
  void i18nMounted;

  const selectTheme = (dark: boolean) => {
    setIsDark(dark);
    if (dark) {
      document.documentElement.classList.remove("light");
      document.documentElement.classList.add("dark");
      localStorage.setItem("star-theme", "dark");
    } else {
      document.documentElement.classList.remove("dark");
      document.documentElement.classList.add("light");
      localStorage.setItem("star-theme", "light");
    }
  };

  const selectLanguage = (next: Language) => {
    setLanguage(next);
    setLangMenuOpen(false);
  };

  // 点击外部关闭（整个菜单 / 语言子菜单）
  useEffect(() => {
    function onClick(e: MouseEvent) {
      const target = e.target as Node;
      if (ref.current && !ref.current.contains(target)) {
        setOpen(false);
        setLangMenuOpen(false);
      } else if (
        langMenuOpen &&
        langMenuRef.current &&
        !langMenuRef.current.contains(target)
      ) {
        // 关闭仅语言子菜单但保留主菜单
        setLangMenuOpen(false);
      }
    }
    if (open) document.addEventListener("mousedown", onClick);
    return () => document.removeEventListener("mousedown", onClick);
  }, [open, langMenuOpen]);

  return (
    <div className="relative" ref={ref}>
      <button
        type="button"
        data-testid="user-avatar"
        onClick={() => setOpen((o) => !o)}
        className="flex items-center gap-2 pl-2 pr-1 h-8 rounded-md border-l border-line hover:bg-bg-soft/50 transition-all duration-150 active:scale-[0.98]"
        aria-label={tx(t.userMenu.menuLabel, {
          name: "Ulysses",
          role: "tenant_admin",
        })}
        aria-expanded={open}
        aria-haspopup="menu"
      >
        <div className="size-6 rounded-full bg-accent/15 border border-accent/40 grid place-items-center text-accent text-xs font-bold shadow-[0_0_8px_rgba(0,240,255,0.25)]">
          U
        </div>
        <div className="text-left hidden md:block leading-tight">
          <div className="text-xs font-medium text-ink flex items-center gap-1">
            <span>Ulysses</span>
            <ChevronDown size={11} className="text-ink-mute" />
          </div>
          <div className="text-[9px] text-ink-mute font-mono">SYS // ADMIN</div>
        </div>
      </button>

      {open && (
        <div
          role="menu"
          className="absolute right-0 mt-2 w-64 rounded-lg border border-line bg-bg-card shadow-2xl z-50 overflow-hidden backdrop-blur-md animate-in fade-in zoom-in-95 duration-100"
        >
          {/* 用户信息区 */}
          <div className="px-3.5 py-3 border-b border-line bg-bg-soft/70">
            <div className="flex items-center justify-between">
              <div className="text-sm font-semibold text-ink">Ulysses</div>
              <span className="text-[9px] font-mono px-1.5 py-0.2 rounded border border-accent/40 bg-accent/10 text-accent">
                ADMIN
              </span>
            </div>
            <div className="text-xs text-ink-dim font-mono mt-0.5">
              ulysses@mavis.local
            </div>
          </div>

          {/* 主题切换 (日漫风格双核切换) */}
          {mounted && (
            <div className="px-3.5 py-2.5 border-b border-line bg-bg-soft/30">
              <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-2 flex items-center justify-between font-mono">
                <span className="flex items-center gap-1">
                  <Cpu size={10} className="text-accent" />
                  <span>{t.userMenu.themeEngine}</span>
                </span>
                <span className="text-accent">
                  {isDark ? "OBSIDIAN" : "CERAMIC"}
                </span>
              </div>
              <div className="grid grid-cols-2 gap-1.5">
                <button
                  type="button"
                  onClick={() => selectTheme(true)}
                  title={t.userMenu.themeDark}
                  className={`text-xs px-2.5 py-1.5 rounded-md border flex items-center justify-center gap-1.5 transition-all font-mono ${
                    isDark
                      ? "border-accent bg-accent/15 text-accent shadow-[0_0_10px_rgba(0,240,255,0.2)] font-semibold"
                      : "border-line text-ink-dim hover:bg-bg-soft"
                  }`}
                >
                  <Moon size={12} /> {t.userMenu.themeDarkShort}
                </button>
                <button
                  type="button"
                  onClick={() => selectTheme(false)}
                  title={t.userMenu.themeLight}
                  className={`text-xs px-2.5 py-1.5 rounded-md border flex items-center justify-center gap-1.5 transition-all font-mono ${
                    !isDark
                      ? "border-accent bg-accent/15 text-accent shadow-[0_0_10px_rgba(59,130,246,0.2)] font-semibold"
                      : "border-line text-ink-dim hover:bg-bg-soft"
                  }`}
                >
                  <Sun size={12} /> {t.userMenu.themeLightShort}
                </button>
              </div>
            </div>
          )}

          {/* 核心工作区 & CLI 工具 */}
          <div className="py-1.5 border-b border-line">
            <div className="px-3.5 py-1 text-[9px] uppercase tracking-wider text-ink-mute font-mono">
              {t.userMenu.toolsAndWorkspaces}
            </div>
            <Link
              href="/agent-windows"
              role="menuitem"
              onClick={() => setOpen(false)}
              className="flex items-center gap-2.5 px-3.5 py-1.5 text-xs text-ink hover:bg-bg-soft hover:text-accent transition-colors"
            >
              <Sparkles size={13} className="text-accent" />
              <span className="flex-1">{t.userMenu.agentWindows}</span>
              <span className="text-[9px] font-mono text-ok">
                {t.userMenu.agentWindowsStatus}
              </span>
            </Link>
            <Link
              href="/settings/cli-profiles"
              role="menuitem"
              onClick={() => setOpen(false)}
              className="flex items-center gap-2.5 px-3.5 py-1.5 text-xs text-ink hover:bg-bg-soft hover:text-accent transition-colors"
            >
              <Terminal size={13} className="text-info" />
              <span className="flex-1">{t.userMenu.cliProfiles}</span>
              <span className="text-[9px] font-mono text-ink-mute">
                {tx(t.userMenu.cliProfilesCount, { count: 6 })}
              </span>
            </Link>
            <Link
              href="/settings/api-keys"
              role="menuitem"
              onClick={() => setOpen(false)}
              className="flex items-center gap-2.5 px-3.5 py-1.5 text-xs text-ink hover:bg-bg-soft hover:text-accent transition-colors"
            >
              <Key size={13} className="text-warn" />
              <span className="flex-1">{t.userMenu.apiKeys}</span>
              <span className="text-[9px] font-mono text-ink-mute">
                {t.userMenu.apiKeysMode}
              </span>
            </Link>
          </div>

          {/* === 语言切换 (2026-08-31 新增) === */}
          <div className="py-1.5 border-b border-line">
            <button
              type="button"
              role="menuitem"
              aria-haspopup="menu"
              aria-expanded={langMenuOpen}
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                setLangMenuOpen((o) => !o);
              }}
              data-testid="user-menu-language-trigger"
              className="w-full flex items-center gap-2.5 px-3.5 py-1.5 text-xs text-ink hover:bg-bg-soft hover:text-accent transition-colors"
            >
              <Globe size={13} className="text-accent" />
              <span className="flex-1 text-left">{t.userMenu.language}</span>
              <span className="flex items-center gap-1.5">
                <span className="text-[10px] font-mono text-ink-dim">
                  {LANGUAGE_META[language].flag}
                </span>
                <span className="text-[10px] font-mono text-accent font-semibold">
                  {LANGUAGE_META[language].nativeLabel}
                </span>
                <ChevronDown
                  size={10}
                  className={`text-ink-mute transition-transform duration-200 ${
                    langMenuOpen ? "rotate-180" : ""
                  }`}
                />
              </span>
            </button>

            {langMenuOpen && (
              <div
                ref={langMenuRef}
                role="menu"
                data-testid="user-menu-language-menu"
                className="mx-2 mb-1.5 mt-0.5 rounded-md border border-line bg-bg/95 overflow-hidden shadow-inner"
              >
                {SUPPORTED_LANGUAGES.map((lang) => {
                  const meta = LANGUAGE_META[lang];
                  const isActive = language === lang;
                  return (
                    <button
                      key={lang}
                      type="button"
                      role="menuitemradio"
                      aria-checked={isActive}
                      data-testid={`user-menu-language-${lang}`}
                      onClick={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        selectLanguage(lang);
                      }}
                      className={`w-full flex items-center gap-2.5 px-3 py-1.5 text-xs transition-colors ${
                        isActive
                          ? "bg-accent/10 text-accent"
                          : "text-ink hover:bg-bg-soft"
                      }`}
                    >
                      <span className="text-[14px] leading-none" aria-hidden>
                        {meta.flag}
                      </span>
                      <span className="flex-1 text-left font-medium">
                        {meta.nativeLabel}
                      </span>
                      <span className="text-[9px] font-mono text-ink-mute">
                        {meta.label}
                      </span>
                      {isActive && (
                        <Check
                          size={11}
                          className="text-accent shrink-0"
                          aria-hidden
                        />
                      )}
                    </button>
                  );
                })}
              </div>
            )}
          </div>

          {/* 设置入口 */}
          <div className="py-1.5 border-b border-line">
            <Link
              href="/settings/profile"
              role="menuitem"
              onClick={() => setOpen(false)}
              className="flex items-center gap-2.5 px-3.5 py-1.5 text-xs text-ink hover:bg-bg-soft hover:text-accent transition-colors"
            >
              <User size={13} />
              <span>{t.userMenu.profile}</span>
            </Link>
            <Link
              href="/settings"
              role="menuitem"
              onClick={() => setOpen(false)}
              className="flex items-center gap-2.5 px-3.5 py-1.5 text-xs text-ink hover:bg-bg-soft hover:text-accent transition-colors"
            >
              <Settings size={13} />
              <span>{t.userMenu.settings}</span>
            </Link>
          </div>

          {/* 退出登录 */}
          <div className="py-1">
            <button
              type="button"
              role="menuitem"
              onClick={() => {
                setOpen(false);
              }}
              className="w-full flex items-center gap-2.5 px-3.5 py-1.5 text-xs text-err hover:bg-err/10 transition-colors"
            >
              <LogOut size={13} />
              <span>{t.userMenu.signOut}</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
