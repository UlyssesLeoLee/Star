"use client";

import { useState, useMemo } from "react";
import { ALL_MODULES, CATEGORY_STYLES, getCategoryStyles, type ModuleCategory, type ModuleDefinition } from "@/lib/nav/registry";
import { useNavStore } from "@/lib/nav/navStore";
import {
  Search,
  X,
  Plus,
  Check,
  ExternalLink,
  Sparkles,
  Layers,
  ArrowRight,
  RotateCcw,
  Pin,
} from "lucide-react";
import Link from "next/link";
import { clsx } from "clsx";
import { useTranslation, useModuleTranslation, type Dictionary } from "@/lib/i18n";

type CategoryId = "all" | ModuleCategory;

export function AppMatrixDrawer() {
  const { t } = useTranslation();
  const isOpen = useNavStore((s) => s.isMatrixOpen);
  const close = useNavStore((s) => s.closeMatrix);
  const sidebarItemIds = useNavStore((s) => s.sidebarItemIds);
  const toggleSidebarItem = useNavStore((s) => s.toggleSidebarItem);
  const headerTabIds = useNavStore((s) => s.headerTabIds);
  const toggleHeaderTab = useNavStore((s) => s.toggleHeaderTab);
  const resetToDefault = useNavStore((s) => s.resetToDefault);

  const [query, setQuery] = useState("");
  const [selectedCat, setSelectedCat] = useState<CategoryId>("all");

  // 6 个 category tab, label/tag 走 i18n
  // 5 域各带独立色 (per 2026-09-02 15:42 JST 拍板), 'all' 用中性色
  const CATEGORIES: Array<{
    id: CategoryId;
    label: string;
    tag: string;
    /** 域色 (only for 5 categories) */
    cat?: ModuleCategory;
  }> = [
    { id: "all", label: t.appMatrix.categories.all.label, tag: t.appMatrix.categories.all.tag },
    { id: "core", label: t.appMatrix.categories.core.label, tag: t.appMatrix.categories.core.tag, cat: "core" },
    { id: "work", label: t.appMatrix.categories.work.label, tag: t.appMatrix.categories.work.tag, cat: "work" },
    { id: "agent", label: t.appMatrix.categories.agent.label, tag: t.appMatrix.categories.agent.tag, cat: "agent" },
    { id: "integration", label: t.appMatrix.categories.integration.label, tag: t.appMatrix.categories.integration.tag, cat: "integration" },
    { id: "system", label: t.appMatrix.categories.system.label, tag: t.appMatrix.categories.system.tag, cat: "system" },
  ];

  // 过滤: 用翻译后的 label / description / categoryLabel 做子串匹配
  //  (使用 Dictionary['modules'] 类型 + 字典查表)
  const filteredModules = useMemo(() => {
    return ALL_MODULES.filter((m) => {
      const mod = (t.modules as Dictionary["modules"])[m.id];
      const matchCat = selectedCat === "all" || m.category === selectedCat;
      const q = query.toLowerCase();
      const haystack = [
        mod?.label ?? m.label,
        m.code,
        mod?.description ?? m.description,
        mod?.categoryLabel ?? m.categoryLabel,
      ]
        .join(" ")
        .toLowerCase();
      const matchQuery = !q || haystack.includes(q);
      return matchCat && matchQuery;
    });
  }, [query, selectedCat, t.modules]);

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 md:p-6 bg-black/65 backdrop-blur-md animate-in fade-in duration-200"
      onClick={(e) => {
        if (e.target === e.currentTarget) close();
      }}
    >
      <div
        data-testid="app-matrix-modal"
        className="w-full max-w-5xl max-h-[86vh] rounded-3xl border border-line bg-bg-card/95 shadow-2xl flex flex-col overflow-hidden backdrop-blur-2xl animate-in zoom-in-95 duration-200"
      >
        {/* === Header (黄金分割比例顶部区域) === */}
        <div className="px-6 py-4.5 border-b border-line flex items-center justify-between bg-bg-soft/70">
          <div className="flex items-center gap-3.5">
            <div className="size-10 rounded-2xl bg-gradient-to-br from-accent/30 via-accent-violet/20 to-secondary/20 border border-accent/40 grid place-items-center text-accent shadow-[0_0_16px_rgba(0,240,255,0.35)] shrink-0">
              <Layers size={20} />
            </div>
            <div>
              <div className="flex items-center gap-2.5">
                <h2 className="text-base font-black text-ink tracking-tight">
                  {t.appMatrix.title}
                </h2>
                <span className="font-mono text-[9px] font-bold px-1.5 py-0.2 rounded border border-accent/40 bg-accent/10 text-accent">
                  {t.appMatrix.capabilities}
                </span>
              </div>
              <p className="text-xs text-ink-dim font-medium">
                {t.appMatrix.subtitle}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2.5">
            <button
              type="button"
              onClick={resetToDefault}
              title={t.appMatrix.resetDefaultTitle}
              className="px-3 py-1.5 text-xs font-mono text-ink-mute hover:text-ink rounded-lg border border-line hover:bg-bg-soft flex items-center gap-1.5 transition-colors"
            >
              <RotateCcw size={12} />
              <span>{t.appMatrix.resetDefault}</span>
            </button>
            <button
              type="button"
              onClick={close}
              className="p-2 text-ink-dim hover:text-ink rounded-lg hover:bg-bg-soft transition-colors"
            >
              <X size={18} />
            </button>
          </div>
        </div>

        {/* === Filter & Search Bar === */}
        <div className="px-6 py-3.5 border-b border-line bg-bg-soft/30 space-y-3">
          <div className="relative">
            <Search size={15} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-accent" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t.appMatrix.searchPlaceholder}
              className="w-full pl-10 pr-4 py-2 text-xs rounded-xl border border-line bg-bg text-ink placeholder:text-ink-mute focus:outline-none focus:border-accent focus:shadow-[0_0_12px_rgba(0,240,255,0.2)] transition-all font-mono"
              autoFocus
            />
          </div>

          <div className="flex items-center gap-2 overflow-x-auto pb-0.5 scrollbar-none">
            {CATEGORIES.map((cat) => {
              // 5 域 tab 用对应色, 'all' 用 accent
              const cs = cat.cat ? CATEGORY_STYLES[cat.cat] : null;
              const isSelected = selectedCat === cat.id;
              return (
                <button
                  key={cat.id}
                  type="button"
                  onClick={() => setSelectedCat(cat.id)}
                  data-testid={`matrix-cat-${cat.id}`}
                  className={clsx(
                    "px-3 py-1.5 rounded-xl text-xs font-medium whitespace-nowrap transition-all duration-200 border flex items-center gap-1.5",
                    isSelected
                      ? cs
                        ? clsx(cs.bgActive, cs.borderActive, cs.text, "font-bold", cs.glow)
                        : "bg-accent/15 border-accent text-accent shadow-[0_0_12px_rgba(0,240,255,0.22)] font-bold"
                      : cs
                        ? clsx("border-line bg-bg-soft/40", cs.text, "opacity-70 hover:opacity-100", "hover:bg-bg-soft")
                        : "border-line bg-bg-soft/40 text-ink-dim hover:text-ink hover:bg-bg-soft"
                  )}
                >
                  {cs && (
                    <span
                      data-testid={`matrix-cat-dot-${cat.id}`}
                      className={clsx("size-1.5 rounded-full transition-opacity", cs.dot, isSelected ? "opacity-100" : "opacity-50")}
                      aria-hidden="true"
                    />
                  )}
                  <span>{cat.label}</span>
                  <span className="font-mono text-[9px] opacity-60">({cat.tag})</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* === Modules Grid (3 列黄金卡片布局) === */}
        <div className="flex-1 overflow-y-auto p-6 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredModules.map((m) => (
            <MatrixCard
              key={m.id}
              module={m}
              inSidebar={sidebarItemIds.includes(m.id)}
              inHeader={headerTabIds.includes(m.id)}
              onToggleSidebar={() => toggleSidebarItem(m.id)}
              onToggleHeader={() => toggleHeaderTab(m.id)}
              onOpen={close}
            />
          ))}
        </div>

        {/* === Footer === */}
        <div className="px-6 py-3.5 border-t border-line bg-bg-soft/50 flex items-center justify-between text-xs text-ink-mute font-mono">
          <span className="flex items-center gap-3">
            <span>
              {t.appMatrix.footerPinnedSidebar}{" "}
              <strong className="text-accent font-bold">{sidebarItemIds.length}</strong>
            </span>
            <span className="text-line">|</span>
            <span>
              {t.appMatrix.footerPinnedHeader}{" "}
              <strong className="text-purple-400 font-bold">{headerTabIds.length}</strong>
            </span>
          </span>
          <button
            type="button"
            onClick={close}
            className="px-5 py-1.5 rounded-xl border border-accent bg-accent/20 text-accent hover:bg-accent/30 shadow-[0_0_12px_rgba(0,240,255,0.25)] transition-all text-xs font-bold"
          >
            {t.appMatrix.done}
          </button>
        </div>
      </div>
    </div>
  );
}

// =====================================================================
// MatrixCard — 单张模块卡片 (per 2026-08-31 i18n 补缺口)
// =====================================================================
interface MatrixCardProps {
  module: ModuleDefinition;
  inSidebar: boolean;
  inHeader: boolean;
  onToggleSidebar: () => void;
  onToggleHeader: () => void;
  onOpen: () => void;
}

function MatrixCard({
  module: m,
  inSidebar,
  inHeader,
  onToggleSidebar,
  onToggleHeader,
  onOpen,
}: MatrixCardProps) {
  const mod = useModuleTranslation(m);
  const { t } = useTranslation();
  const Icon = m.icon;
  // Jira 风格: 域分色 icon tile (per 2026-09-02 15:42 JST 拍板)
  const cs = getCategoryStyles(m.category);
  return (
    <div
      data-testid={`matrix-card-${m.id}`}
      className="group relative rounded-2xl border border-line bg-bg-soft/40 hover:bg-bg-soft/80 p-4.5 transition-all duration-200 hover:shadow-[0_8px_24px_rgba(0,0,0,0.35)] flex flex-col justify-between"
    >
      <div>
        <div className="flex items-start justify-between gap-2 mb-2.5">
          <div className="flex items-center gap-3">
            {/* Jira 风格 icon tile — 10x10 圆角色块 + 域分色 + line icon */}
            <div
              data-testid={`matrix-card-icon-tile-${m.id}`}
              className={clsx(
                "size-10 rounded-xl grid place-items-center shrink-0 border transition-all duration-200 group-hover:scale-110 group-hover:rotate-3",
                cs.bg,
                cs.border,
                cs.text
              )}
              aria-hidden="true"
            >
              <Icon size={20} strokeWidth={2.25} />
            </div>
            <div>
              <div className="flex items-center gap-1.5">
                <span className={clsx("text-xs font-bold text-ink transition-colors", `group-hover:${cs.text}`)}>
                  {mod.label}
                </span>
                <span className="font-mono text-[9px] px-1.5 py-0.2 rounded border border-line bg-bg/60 text-ink-mute font-semibold">
                  {m.code}
                </span>
              </div>
              <span className={clsx("text-[10px] font-mono inline-flex items-center gap-1", cs.text, "opacity-80")}>
                <span className={clsx("size-1 rounded-full", cs.dot)} aria-hidden="true" />
                {mod.categoryLabel}
              </span>
            </div>
          </div>
        </div>

        <p className="text-xs text-ink-dim line-clamp-2 leading-relaxed mb-4 font-normal">
          {mod.description}
        </p>
      </div>

      <div className="border-t border-line/50 pt-3 flex items-center justify-between gap-2">
        <div className="flex items-center gap-1.5">
          {/* 钉选到左侧 */}
          <button
            type="button"
            onClick={onToggleSidebar}
            data-testid={`pin-sidebar-${m.id}`}
            className={clsx(
              "px-2.5 py-1 rounded-lg text-[10px] font-mono font-bold flex items-center gap-1 transition-all duration-150 border",
              inSidebar
                ? "bg-accent/20 border-accent text-accent shadow-[0_0_8px_rgba(0,240,255,0.3)]"
                : "bg-bg/60 border-line text-ink-dim hover:text-ink hover:border-accent/40"
            )}
            title={inSidebar ? t.appMatrix.unpinFromSidebar : t.appMatrix.pinToSidebar}
          >
            {inSidebar ? <Check size={11} className="stroke-[3]" /> : <Plus size={11} />}
            <span>{t.appMatrix.sidebarLabel}</span>
          </button>

          {/* 钉选到顶部 */}
          <button
            type="button"
            onClick={onToggleHeader}
            data-testid={`pin-header-${m.id}`}
            className={clsx(
              "px-2.5 py-1 rounded-lg text-[10px] font-mono font-bold flex items-center gap-1 transition-all duration-150 border",
              inHeader
                ? "bg-accent-violet/20 border-accent-violet text-purple-400 shadow-[0_0_8px_rgba(168,85,247,0.3)]"
                : "bg-bg/60 border-line text-ink-dim hover:text-ink hover:border-accent/40"
            )}
            title={inHeader ? t.appMatrix.unpinFromHeader : t.appMatrix.pinToHeader}
          >
            {inHeader ? <Check size={11} className="stroke-[3]" /> : <Plus size={11} />}
            <span>{t.appMatrix.headerLabel}</span>
          </button>
        </div>

        {/* 直接跳转 */}
        <Link
          href={m.href}
          onClick={onOpen}
          className="p-1.5 text-ink-dim hover:text-accent rounded-lg hover:bg-bg transition-colors"
          title={t.appMatrix.openNow}
        >
          <ArrowRight size={14} />
        </Link>
      </div>
    </div>
  );
}
