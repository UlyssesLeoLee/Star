"use client";

// Star Frontend — TopBar 右上角用户菜单 (per 2026-08-29 09:07 JST 用户拍板)
// 入口: 用户头像下拉 → 个人设置 / CLI Profiles / API Keys / 主题 / 退出
// 设计: 4 个独立入口, 不混乱 (per 04:09 + 09:07 JST 多次拍板)

import { useState, useRef, useEffect } from "react";
import { User, Terminal, Key, LogOut, Settings, ChevronDown, Sun, Moon, Sparkles } from "lucide-react";
import Link from "next/link";
import { useTheme } from "next-themes";
import { THEMES, type ThemeId } from "@/lib/theme/types";

export function UserMenu() {
  const [open, setOpen] = useState(false);
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => setMounted(true), []);

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
        onClick={() => setOpen((o) => !o)}
        className="flex items-center gap-2 pl-2 pr-1 py-0.5 rounded hover:bg-[color:var(--color-surface-2)] transition-colors"
        aria-label="user menu"
        aria-expanded={open}
      >
        <div className="size-7 rounded-full bg-[color:var(--color-primary)]/15 border border-[color:var(--color-primary)]/40 grid place-items-center text-[color:var(--color-primary)] text-xs font-bold">U</div>
        <div className="text-left">
          <div className="text-xs leading-tight">Ulysses</div>
          <div className="text-[10px] text-[color:var(--color-text-dim)] leading-tight">tenant_admin</div>
        </div>
        <ChevronDown size={12} className="opacity-60" />
      </button>

      {open && (
        <div
          role="menu"
          className="absolute right-0 mt-1 w-56 rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface)] shadow-[var(--shadow-lg)] z-50 overflow-hidden"
        >
          {/* 用户信息区 */}
          <div className="px-3 py-2 border-b border-[color:var(--color-border)] bg-[color:var(--color-surface-2)]">
            <div className="text-sm font-medium">Ulysses</div>
            <div className="text-[10px] text-[color:var(--color-text-dim)]">ulysses@mavis.local</div>
          </div>

          {/* 主题切换 (内嵌, 快速) */}
          {mounted && (
            <div className="px-3 py-2 border-b border-[color:var(--color-border)]">
              <div className="text-[10px] uppercase tracking-wider text-[color:var(--color-text-dim)] mb-1.5 flex items-center gap-1">
                {theme === "dark" ? <Moon size={10} /> : <Sun size={10} />}
                主题
              </div>
              <div className="flex gap-1">
                {THEMES.map((t) => (
                  <button
                    key={t.id}
                    role="menuitemradio"
                    aria-checked={t.id === theme}
                    onClick={() => setTheme(t.id as ThemeId)}
                    className={`flex-1 text-xs px-2 py-1 rounded border ${
                      t.id === theme
                        ? "border-[color:var(--color-primary)] bg-[color:var(--color-primary)]/10 text-[color:var(--color-primary)]"
                        : "border-[color:var(--color-border)] hover:bg-[color:var(--color-surface-2)]"
                    }`}
                  >
                    {t.isDark ? "🌙" : "☀️"} {t.displayName}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* 4 个独立入口 — 不混乱 */}
          <div className="py-1">
            <Link
              href="/settings/profile"
              role="menuitem"
              className="flex items-center gap-2 px-3 py-1.5 text-sm hover:bg-[color:var(--color-surface-2)]"
            >
              <User size={14} />
              个人设置
            </Link>
            <Link
              href="/settings/cli-profiles"
              role="menuitem"
              className="flex items-center gap-2 px-3 py-1.5 text-sm hover:bg-[color:var(--color-surface-2)]"
            >
              <Terminal size={14} />
              <span className="flex-1">CLI Profiles</span>
              <span className="text-[10px] text-[color:var(--color-text-dim)]">6 内置</span>
            </Link>
            <Link
              href="/settings/api-keys"
              role="menuitem"
              className="flex items-center gap-2 px-3 py-1.5 text-sm hover:bg-[color:var(--color-surface-2)]"
            >
              <Key size={14} />
              <span className="flex-1">API Keys</span>
              <span className="text-[10px] text-[color:var(--color-text-dim)]">双模式</span>
            </Link>
            <Link
              href="/settings"
              role="menuitem"
              className="flex items-center gap-2 px-3 py-1.5 text-sm hover:bg-[color:var(--color-surface-2)]"
            >
              <Settings size={14} />
              所有设置
            </Link>
          </div>

          <div className="border-t border-[color:var(--color-border)] py-1">
            <Link
              href="/agent-windows"
              role="menuitem"
              className="flex items-center gap-2 px-3 py-1.5 text-sm hover:bg-[color:var(--color-surface-2)]"
            >
              <Sparkles size={14} />
              任务窗口
            </Link>
          </div>

          <div className="border-t border-[color:var(--color-border)] py-1">
            <button
              role="menuitem"
              onClick={() => { /* TODO: 接入 logout */ }}
              className="w-full flex items-center gap-2 px-3 py-1.5 text-sm text-[color:var(--color-danger)] hover:bg-[color:var(--color-surface-2)]"
            >
              <LogOut size={14} />
              退出登录
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
