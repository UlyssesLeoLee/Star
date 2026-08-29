"use client";

// =====================================================================
// UserMenu — TopBar 右上角用户菜单 (日漫科技 HUD 风格)
// =====================================================================
// 功能整合:
//   1. 身份展示 (Ulysses · SYS // ADMIN)
//   2. 快速双主题切换 (🌙 Neo-Tokyo Dark / ☀️ Mecha Light)
//   3. 核心工具入口 (任务窗口, CLI Profiles, API Keys)
//   4. 个人偏好与全局设置
//   5. 安全退出
// =====================================================================

import { useState, useRef, useEffect } from "react";
import { User, Terminal, Key, LogOut, Settings, ChevronDown, Sun, Moon, Sparkles, Shield, Cpu } from "lucide-react";
import Link from "next/link";

export function UserMenu() {
  const [open, setOpen] = useState(false);
  const [isDark, setIsDark] = useState(true);
  const [mounted, setMounted] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

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

  // 点击外部关闭
  useEffect(() => {
    function onClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    if (open) document.addEventListener("mousedown", onClick);
    return () => document.removeEventListener("mousedown", onClick);
  }, [open]);

  return (
    <div className="relative" ref={ref}>
      <button
        type="button"
        data-testid="user-avatar"
        onClick={() => setOpen((o) => !o)}
        className="flex items-center gap-2 pl-2 pr-1 h-8 rounded-md border-l border-line hover:bg-bg-soft/50 transition-all duration-150 active:scale-[0.98]"
        aria-label="User menu: Ulysses (tenant_admin)"
        aria-expanded={open}
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
            <div className="text-xs text-ink-dim font-mono mt-0.5">ulysses@mavis.local</div>
          </div>

          {/* 主题切换 (日漫风格双核切换) */}
          {mounted && (
            <div className="px-3.5 py-2.5 border-b border-line bg-bg-soft/30">
              <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-2 flex items-center justify-between font-mono">
                <span className="flex items-center gap-1">
                  <Cpu size={10} className="text-accent" />
                  <span>THEME ENGINE</span>
                </span>
                <span className="text-accent">{isDark ? "OBSIDIAN" : "CERAMIC"}</span>
              </div>
              <div className="grid grid-cols-2 gap-1.5">
                <button
                  type="button"
                  onClick={() => selectTheme(true)}
                  className={`text-xs px-2.5 py-1.5 rounded-md border flex items-center justify-center gap-1.5 transition-all font-mono ${
                    isDark
                      ? "border-accent bg-accent/15 text-accent shadow-[0_0_10px_rgba(0,240,255,0.2)] font-semibold"
                      : "border-line text-ink-dim hover:bg-bg-soft"
                  }`}
                >
                  <Moon size={12} /> Neo-Tokyo
                </button>
                <button
                  type="button"
                  onClick={() => selectTheme(false)}
                  className={`text-xs px-2.5 py-1.5 rounded-md border flex items-center justify-center gap-1.5 transition-all font-mono ${
                    !isDark
                      ? "border-accent bg-accent/15 text-accent shadow-[0_0_10px_rgba(59,130,246,0.2)] font-semibold"
                      : "border-line text-ink-dim hover:bg-bg-soft"
                  }`}
                >
                  <Sun size={12} /> Mecha Lab
                </button>
              </div>
            </div>
          )}

          {/* 核心工作区 & CLI 工具 */}
          <div className="py-1.5 border-b border-line">
            <div className="px-3.5 py-1 text-[9px] uppercase tracking-wider text-ink-mute font-mono">
              Tools & Workspaces
            </div>
            <Link
              href="/agent-windows"
              role="menuitem"
              onClick={() => setOpen(false)}
              className="flex items-center gap-2.5 px-3.5 py-1.5 text-xs text-ink hover:bg-bg-soft hover:text-accent transition-colors"
            >
              <Sparkles size={13} className="text-accent" />
              <span className="flex-1">Agent Windows 任务窗口</span>
              <span className="text-[9px] font-mono text-ok">LIVE</span>
            </Link>
            <Link
              href="/settings/cli-profiles"
              role="menuitem"
              onClick={() => setOpen(false)}
              className="flex items-center gap-2.5 px-3.5 py-1.5 text-xs text-ink hover:bg-bg-soft hover:text-accent transition-colors"
            >
              <Terminal size={13} className="text-info" />
              <span className="flex-1">CLI Profiles</span>
              <span className="text-[9px] font-mono text-ink-mute">6 内置</span>
            </Link>
            <Link
              href="/settings/api-keys"
              role="menuitem"
              onClick={() => setOpen(false)}
              className="flex items-center gap-2.5 px-3.5 py-1.5 text-xs text-ink hover:bg-bg-soft hover:text-accent transition-colors"
            >
              <Key size={13} className="text-warn" />
              <span className="flex-1">API Key 凭据管理</span>
              <span className="text-[9px] font-mono text-ink-mute">双模式</span>
            </Link>
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
              <span>个人中心 (Profile)</span>
            </Link>
            <Link
              href="/settings"
              role="menuitem"
              onClick={() => setOpen(false)}
              className="flex items-center gap-2.5 px-3.5 py-1.5 text-xs text-ink hover:bg-bg-soft hover:text-accent transition-colors"
            >
              <Settings size={13} />
              <span>全局设置 (Preferences)</span>
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
              <span>退出登录 (Sign Out)</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
