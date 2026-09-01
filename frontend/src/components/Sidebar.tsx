"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  ChevronRight,
  Radio,
  Plus,
  X,
  Layers,
  Sparkles,
} from "lucide-react";
import { clsx } from "clsx";
import { useTheme } from "next-themes";
import { useEffect, useState } from "react";
import { useNavStore } from "@/lib/nav/navStore";
import { MODULE_MAP, type ModuleDefinition } from "@/lib/nav/registry";
import { useTranslation, useModuleTranslation } from "@/lib/i18n";

export function Sidebar() {
  const pathname = usePathname() ?? "/";
  const { theme } = useTheme();
  const [mounted, setMounted] = useState(false);
  const { t, tx } = useTranslation();

  const sidebarItemIds = useNavStore((s) => s.sidebarItemIds);
  const removeSidebarItem = useNavStore((s) => s.removeSidebarItem);
  const pinnedViewIds = useNavStore((s) => s.pinnedViewIds);
  const removePinnedView = useNavStore((s) => s.removePinnedView);
  const openMatrix = useNavStore((s) => s.openMatrix);

  useEffect(() => setMounted(true), []);

  const isDark = !mounted || theme === "dark" || (typeof document !== "undefined" && document.documentElement.classList.contains("dark"));

  // 解析当前用户钉选到左侧的模块对象
  const activeSidebarModules = sidebarItemIds
    .map((id) => MODULE_MAP.get(id))
    .filter((m): m is ModuleDefinition => Boolean(m));

  const activePinnedModules = pinnedViewIds
    .map((id) => MODULE_MAP.get(id))
    .filter((m): m is ModuleDefinition => Boolean(m));

  return (
    <aside
      data-testid="app-sidebar"
      // 桌面: w-64 sticky; <768px: 隐藏 (由 MobileBottomNav + AppMatrixDrawer 抽屉替代, per 2026-09-01 PHASE-MOBILE-PWA)
      className="hidden md:flex w-64 shrink-0 border-r border-line bg-bg-soft/75 backdrop-blur-xl flex-col h-screen sticky top-0 select-none z-20 transition-all"
    >
      {/* === Star Anime Cyber Crest (Brand Block) === */}
      <div className="px-4 py-4 border-b border-line shrink-0 flex items-center justify-between">
        <Link
          href="/inbox"
          className="flex items-center gap-3 group"
          data-testid="sidebar-brand"
          aria-label="Star home"
        >
          <div
            aria-hidden="true"
            className="size-9 rounded-xl overflow-hidden border border-accent/40 shadow-[0_0_16px_rgba(0,240,255,0.3)] shrink-0 transition-transform duration-300 group-hover:scale-105 group-hover:rotate-6 bg-black"
          >
            {/* Star brand icon — 96x96 黑底 PNG, 由根目录 icon.png 缩放 (per 2026-08-31 11:30 JST 拍板) */}
            <img
              src="/sidebar-icon.png"
              width={36}
              height={36}
              alt=""
              className="block w-full h-full object-cover"
            />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-1.5">
              <span className={clsx(
                "text-sm font-black tracking-tight transition-colors",
                isDark ? "text-slate-100 group-hover:text-cyan-300" : "text-slate-900 group-hover:text-sky-600"
              )}>
                STAR
              </span>
              <span className="font-mono text-[9px] font-bold px-1.5 py-0.2 rounded border border-accent/40 bg-accent/10 text-accent">
                v0.2
              </span>
            </div>
            <div className="text-[9px] font-mono tracking-widest text-ink-mute uppercase font-medium">
              {t.sidebar.brandTagline}
            </div>
          </div>
        </Link>

        <div className="flex items-center gap-1.5" title="Tactical Link Active">
          <span className="size-2 rounded-full bg-ok animate-pulse shadow-[0_0_10px_rgba(16,185,129,0.9)]" />
        </div>
      </div>

      {/* === Navigation Groups === */}
      <nav className="flex-1 overflow-y-auto px-3 py-4 space-y-5 text-sm scrollbar-none" aria-label="Main Navigation">
        {/* Core Workspaces (用户自定义定制列表) */}
        <div>
          <div className="px-2.5 py-1 flex items-center justify-between text-[10px] font-mono uppercase tracking-wider text-ink-mute">
            <span>{t.sidebar.groupWorkspaces}</span>
            <span className="text-[9px] font-mono opacity-70 border border-line px-1.5 py-0.2 rounded">
              {activeSidebarModules.length} {t.sidebar.pinned}
            </span>
          </div>
          <ul className="mt-1.5 space-y-1">
            {activeSidebarModules.map((item) => (
              <SidebarRow
                key={item.id}
                module={item}
                active={pathname === item.href || (item.href !== "/" && pathname?.startsWith(item.href))}
                onRemove={removeSidebarItem}
                removeTitle={tx(t.sidebar.removeFromSidebar, { label: "" })}
                dataTestIdBase="sidebar-item"
                removeDataTestIdBase="remove-sidebar"
              />
            ))}
          </ul>
        </div>

        {/* Tactical Views (如看板、甘特排期) */}
        {activePinnedModules.length > 0 && (
          <div>
            <div className="px-2.5 py-1 flex items-center justify-between text-[10px] font-mono uppercase tracking-wider text-ink-mute">
              <span>{t.sidebar.groupTactical}</span>
              <span className="text-[9px] font-mono opacity-70 border border-line px-1.5 py-0.2 rounded">{t.sidebar.pinned}</span>
            </div>
            <ul className="mt-1.5 space-y-1">
              {activePinnedModules.map((item) => (
                <SidebarRow
                  key={item.id}
                  module={item}
                  active={pathname === item.href || (item.href !== "/" && pathname?.startsWith(item.href))}
                  onRemove={removePinnedView}
                  removeTitle={tx(t.sidebar.removeFromPinned, { label: "" })}
                  dataTestIdBase="sidebar-item"
                  removeDataTestIdBase="remove-pinned"
                />
              ))}
            </ul>
          </div>
        )}

        {/* === + 自定义添加导航项按钮 === */}
        <div className="pt-2">
          <button
            type="button"
            onClick={openMatrix}
            data-testid="sidebar-add-custom"
            className="w-full flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl border border-dashed border-line hover:border-accent/60 bg-bg-soft/40 hover:bg-accent/10 text-xs font-mono text-ink-mute hover:text-accent transition-all duration-200 group shadow-sm hover:shadow-[0_0_12px_rgba(0,240,255,0.15)]"
          >
            <Plus size={13} className="text-accent group-hover:rotate-90 transition-transform duration-300" />
            <span className="font-semibold">{t.sidebar.customAdd}</span>
          </button>
        </div>
      </nav>

      {/* === Bottom Tactical HUD Footer === */}
      <div className="border-t border-line px-4 py-3 shrink-0 bg-bg-soft/50 space-y-1.5">
        <div className="flex items-center justify-between text-[10px] font-mono text-ink-mute">
          <span className="flex items-center gap-1.5">
            <Radio size={12} className="text-accent animate-spin" />
            <span>{t.sidebar.footerStatus}</span>
          </span>
          <span className="text-ok font-bold drop-shadow-[0_0_6px_rgba(16,185,129,0.7)]">
            {t.sidebar.footerStatusAllGreen}
          </span>
        </div>
        <div className="text-[9px] font-mono text-ink-mute/70 truncate">
          {t.sidebar.footerNode}
        </div>
      </div>
    </aside>
  );
}

// =====================================================================
// SidebarRow — 单行 nav 渲染 (per 2026-08-31 i18n 补缺口)
// =====================================================================
// 抽出来是为了在子组件内合法调用 useModuleTranslation (hook 不能在
// 父组件 .map() 回调里直接调, 否则会破坏 hooks 规则).
//
// data-testid 仍用 registry 原 label (英文) 生成, 保证既有测试稳定
// (sidebar-item-inbox / sidebar-item-issues / ...) 不受语言切换影响.
// =====================================================================
interface SidebarRowProps {
  module: ModuleDefinition;
  active: boolean;
  onRemove: (id: string) => void;
  removeTitle: string;
  dataTestIdBase: string;
  removeDataTestIdBase: string;
}

function SidebarRow({
  module: item,
  active,
  onRemove,
  removeTitle,
  dataTestIdBase,
  removeDataTestIdBase,
}: SidebarRowProps) {
  const mod = useModuleTranslation(item);
  const { tx } = useTranslation();
  const Icon = item.icon;
  // 用 registry 静态 label 生成 testid, 避免翻译切换导致 testid 漂移
  const testIdSlug = item.label.toLowerCase().replace(/\s+/g, "-");
  return (
    <li className="relative group">
      <Link
        href={item.href}
        data-testid={`${dataTestIdBase}-${testIdSlug}`}
        className={clsx(
          "relative flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-medium transition-all duration-200",
          active
            ? "bg-accent/15 text-accent border border-accent/40 shadow-[0_0_16px_rgba(0,240,255,0.18)] font-semibold translate-x-0.5"
            : "text-ink-dim hover:bg-bg-soft/80 hover:text-ink border border-transparent hover:translate-x-0.5"
        )}
      >
        <Icon
          size={16}
          className={clsx(
            "shrink-0 transition-transform duration-200 group-hover:scale-110",
            active ? "text-accent drop-shadow-[0_0_6px_rgba(0,240,255,0.6)]" : "text-ink-dim group-hover:text-ink"
          )}
        />
        <span className="flex-1 truncate tracking-tight">{mod.label}</span>

        <span className="text-[9px] font-mono text-ink-mute px-1.5 py-0.2 rounded border border-line/50 bg-bg/40 opacity-70 group-hover:opacity-100 transition-opacity">
          {item.code}
        </span>

        {item.count !== undefined && item.count > 0 && (
          <span className="min-w-[16px] h-4 rounded-full bg-err text-white text-[9px] font-mono grid place-items-center px-1 shadow-[0_0_8px_rgba(255,51,102,0.7)] font-bold">
            {item.count}
          </span>
        )}

        {active && (
          <ChevronRight size={12} className="text-accent animate-pulse" />
        )}
      </Link>

      <button
        type="button"
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
          onRemove(item.id);
        }}
        title={tx(removeTitle, { label: mod.label })}
        data-testid={`${removeDataTestIdBase}-${item.id}`}
        className="absolute right-1.5 top-1/2 -translate-y-1/2 p-1 rounded-md hover:bg-err/20 hover:text-err text-ink-mute opacity-0 group-hover:opacity-100 transition-all duration-150"
      >
        <X size={11} />
      </button>
    </li>
  );
}
