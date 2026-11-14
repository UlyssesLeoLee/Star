"use client";

/*
@cypher schema=1 source_sha256=b24b096f1ebb490450cdee91e181cbb509f0b348d30b28308be9081d279f90f8
MERGE (self:File {path:"frontend/src/components/nav/AppMatrixDrawer.tsx"})
MERGE (matrix:Symbol {id:"frontend/src/components/nav/AppMatrixDrawer.tsx::AppMatrixDrawer",kind:"component"})
MERGE (guardedLink:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::GuardedLink",kind:"component"})
MERGE (self)-[:DEFINES]->(matrix)
MERGE (self)-[:IMPORTS]->(guardedLink)
MERGE (matrix)-[:CALLS]->(guardedLink)
@endcypher
*/

import { useState, useMemo } from "react";
import { ALL_MODULES, CATEGORY_STYLES, MODULE_GROUPS, SETTINGS_DOMAIN_IDS, getCategoryStyles, type ModuleCategory, type ModuleDefinition } from "@/lib/nav/registry";
import { useNavStore, WORKTREE_INDEX_OWNED_MODULES } from "@/lib/nav/navStore";
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
  KanbanSquare,
  PenTool,
  Rocket,
  BarChart3,
  Bot,
  Puzzle,
  Settings,
} from "lucide-react";
import { GuardedLink as Link } from "@/components/navigation/GuardedLink";
import { clsx } from "clsx";
import { useTranslation, useModuleTranslation, type Dictionary } from "@/lib/i18n";

type CategoryId = "all" | ModuleCategory;

// 模块汇总卡图标 (per 2026-11-14 6 模块归并)
const GROUP_ICONS: Record<string, typeof KanbanSquare> = {
  tasks: KanbanSquare,
  canvas: PenTool,
  execution: Rocket,
  analytics: BarChart3,
  agents: Bot,
  plugins: Puzzle,
};

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
  // 视图: 默认 = 6 模块汇总 (降低认知负荷); 搜索/分类时切到全部应用网格
  const [view, setView] = useState<"groups" | "all">("groups");

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
        className="w-full max-w-5xl max-h-[86vh] border-2 border-black bg-[var(--cel-surface-card,#0f1422)] text-[var(--cel-text-primary,#ffffff)] cel-shadow-lg flex flex-col overflow-hidden backdrop-blur-2xl animate-in zoom-in-95 duration-200 relative"
      >
        {/* === Header (3渲2 战术指令台顶部) === */}
        <div className="px-6 py-4 border-b-2 border-black flex items-center justify-between bg-[var(--cel-surface-card,#0f1422)]">
          <div className="flex items-center gap-3.5">
            <div className="size-10 border-2 border-black bg-gradient-to-br from-[var(--cel-cyan,#00f0ff)] to-[var(--cel-crimson,#ff184c)] grid place-items-center text-black font-black cel-shadow shrink-0">
              <Layers size={20} className="stroke-[2.5]" />
            </div>
            <div>
              <div className="flex items-center gap-2.5">
                <h2 className="text-base font-black uppercase tracking-tight text-white flex items-center gap-2">
                  {t.appMatrix.title}
                  <span className="text-[10px] font-mono font-black px-1.5 py-0.5 bg-[var(--cel-crimson,#ff184c)] text-black border border-black italic">
                    MODULE_MATRIX
                  </span>
                </h2>
                <span className="font-mono text-[9px] font-bold px-1.5 py-0.5 border border-black bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-cyan,#00f0ff)]">
                  {t.appMatrix.capabilities}
                </span>
              </div>
              <p className="text-xs text-[var(--cel-text-secondary,#94a3b8)] font-mono">
                {t.appMatrix.subtitle}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2.5">
            <button
              type="button"
              onClick={resetToDefault}
              title={t.appMatrix.resetDefaultTitle}
              className="px-3 py-1.5 text-xs font-mono text-[var(--cel-text-secondary,#94a3b8)] hover:text-white border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] cel-shadow flex items-center gap-1.5 transition-all active:translate-y-0.5"
            >
              <RotateCcw size={12} />
              <span>{t.appMatrix.resetDefault}</span>
            </button>
            <button
              type="button"
              onClick={close}
              className="p-1.5 text-[var(--cel-text-secondary,#94a3b8)] hover:text-white border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] cel-shadow transition-all active:translate-y-0.5"
            >
              <X size={18} />
            </button>
          </div>
        </div>

        {/* === Filter & Search Bar === */}
        <div className="px-6 py-3.5 border-b-2 border-black bg-[var(--cel-surface-stage,#090d16)] space-y-3">
          <div className="relative">
            <Search size={15} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-[var(--cel-cyan,#00f0ff)]" />
            <input
              type="text"
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                if (e.target.value) setView("all");
                else setView("groups");
              }}
              placeholder={t.appMatrix.searchPlaceholder}
              className="w-full pl-10 pr-4 py-2 text-xs border-2 border-black bg-[var(--cel-surface-card,#0f1422)] text-white placeholder:text-[var(--cel-text-secondary,#94a3b8)] focus:outline-none focus:border-[var(--cel-cyan,#00f0ff)] cel-shadow font-mono"
              autoFocus
            />
          </div>

          <div className="flex items-center gap-2 overflow-x-auto pb-0.5 scrollbar-none">
            {CATEGORIES.map((cat) => {
              const cs = cat.cat ? CATEGORY_STYLES[cat.cat] : null;
              const isSelected = selectedCat === cat.id;
              return (
                <button
                  key={cat.id}
                  type="button"
                  onClick={() => { setSelectedCat(cat.id); setView("all"); }}
                  data-testid={`matrix-cat-${cat.id}`}
                  className={clsx(
                    "px-3.5 py-1.5 text-xs font-mono font-bold whitespace-nowrap transition-all border-2 border-black flex items-center gap-1.5 cel-shadow",
                    isSelected
                      ? "bg-[var(--cel-cyan,#00f0ff)] text-black"
                      : "bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-secondary,#94a3b8)] hover:text-white"
                  )}
                >
                  {cs && (
                    <span
                      data-testid={`matrix-cat-dot-${cat.id}`}
                      className={clsx("size-1.5 rotate-45 border border-black transition-opacity", isSelected ? "bg-black" : "bg-[var(--cel-cyan,#00f0ff)]")}
                      aria-hidden="true"
                    />
                  )}
                  <span>{cat.label}</span>
                  <span className={clsx("font-mono text-[9px]", isSelected ? "text-black/80 font-bold" : "opacity-60")}>({cat.tag})</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* === 模块汇总视图 (per 2026-11-14 用户发令: 汇总成少数模块 + 标签, 降低认知负荷) === */}
        {view === "groups" ? (
          <div className="flex-1 overflow-y-auto p-6 space-y-3" data-testid="matrix-groups-view">
            <div className="flex items-center justify-between mb-1">
              <p className="text-[11px] font-mono text-[var(--cel-text-secondary,#94a3b8)]">
                6 个模块 · 近似功能已合并为标签 · 点击「+ 添加」将该模块默认标签加入顶栏
              </p>
              <button
                type="button"
                onClick={() => setView("all")}
                data-testid="matrix-view-all"
                className="text-[11px] font-mono px-2.5 py-1 border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-cyan,#00f0ff)] hover:text-white cel-shadow flex items-center gap-1"
              >
                全部应用 ({ALL_MODULES.length}) <ArrowRight size={11} />
              </button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
              {MODULE_GROUPS.map((g) => {
                const Icon = GROUP_ICONS[g.id] ?? Layers;
                const added = g.tabs.some((id) => headerTabIds.includes(id));
                const tabNames = g.tabs.map((id) => {
                  const mod = (t.modules as Dictionary["modules"])[id];
                  return mod?.label ?? id;
                });
                return (
                  <div
                    key={g.id}
                    data-testid={`matrix-group-${g.id}`}
                    className="relative border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] hover:bg-[var(--cel-surface-card,#0f1422)] cel-shadow hover:translate-x-[-2px] hover:translate-y-[-2px] hover:shadow-[6px_6px_0px_#000000] transition-all flex flex-col p-4"
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex items-center gap-2.5">
                        <span className="size-9 grid place-items-center border-2 border-black bg-black text-[var(--cel-cyan,#00f0ff)] cel-shadow shrink-0">
                          <Icon size={16} />
                        </span>
                        <div className="min-w-0">
                          <h3 className="text-sm font-black text-white leading-tight">{g.name}</h3>
                          <p className="text-[10px] font-mono text-[var(--cel-text-secondary,#94a3b8)] truncate">{g.blurb}</p>
                        </div>
                      </div>
                      <button
                        type="button"
                        onClick={() => {
                          const target = g.defaultTab ?? g.tabs[0];
                          if (target && !headerTabIds.includes(target)) toggleHeaderTab(target);
                          close();
                        }}
                        data-testid={`matrix-group-add-${g.id}`}
                        title={added ? "已添加, 点击打开" : "添加该模块到顶栏"}
                        className={clsx(
                          "shrink-0 size-7 grid place-items-center border-2 border-black cel-shadow transition-all active:translate-y-0.5",
                          added ? "bg-[var(--cel-cyan,#00f0ff)] text-black" : "bg-[var(--cel-crimson,#ff184c)] text-black"
                        )}
                      >
                        {added ? <Check size={13} /> : <Plus size={13} />}
                      </button>
                    </div>

                    {/* 标签 chips (近似功能合并为标签) */}
                    <div className="mt-3 flex flex-wrap gap-1">
                      {tabNames.map((name, i) => (
                        <span
                          key={i}
                          data-testid={`matrix-group-tab-${g.id}-${g.tabs[i]}`}
                          className={clsx(
                            "text-[10px] font-mono px-1.5 py-0.5 border border-black",
                            i === 0 ? "bg-[var(--cel-cyan,#00f0ff)] text-black font-bold" : "bg-black/40 text-[var(--cel-text-secondary,#94a3b8)]"
                          )}
                        >
                          {name}
                        </span>
                      ))}
                    </div>
                  </div>
                );
              })}

              {/* 设置卡 (右上角用户菜单, 不算模块) */}
              <Link
                href="/settings"
                onClick={close}
                data-testid="matrix-group-settings"
                className="relative border-2 border-dashed border-[var(--cel-text-secondary,#94a3b8)] bg-transparent hover:bg-[var(--cel-surface-sub,#151c2c)] transition-all flex items-center gap-3 p-4"
              >
                <span className="size-9 grid place-items-center border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-gold,#ffc400)] shrink-0">
                  <Settings size={16} />
                </span>
                <div className="min-w-0">
                  <h3 className="text-sm font-black text-white">设置</h3>
                  <p className="text-[10px] font-mono text-[var(--cel-text-secondary,#94a3b8)] truncate">
                    账户 · 权限 · 租户 · 集成 · API 密钥 ({SETTINGS_DOMAIN_IDS.length} 项 · 右上角用户菜单)
                  </p>
                </div>
                <ArrowRight size={13} className="ml-auto text-[var(--cel-text-secondary,#94a3b8)]" />
              </Link>
            </div>
          </div>
        ) : (
          /* === Modules Grid (3 列 3渲2 战术卡片) === */
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
        )}

        {/* === Footer === */}
        <div className="px-6 py-3.5 border-t-2 border-black bg-[var(--cel-surface-card,#0f1422)] flex items-center justify-between text-xs text-[var(--cel-text-secondary,#94a3b8)] font-mono">
          <span className="flex items-center gap-3">
            <span>
              {t.appMatrix.footerPinnedSidebar}{" "}
              <strong className="text-[var(--cel-cyan,#00f0ff)] font-bold">{sidebarItemIds.length}</strong>
            </span>
            <span className="text-black font-black">|</span>
            <span>
              {t.appMatrix.footerPinnedHeader}{" "}
              <strong className="text-[var(--cel-gold,#ffc400)] font-bold">{headerTabIds.length}</strong>
            </span>
          </span>
          <button
            type="button"
            onClick={close}
            className="px-5 py-1.5 border-2 border-black bg-[var(--cel-crimson,#ff184c)] text-black hover:brightness-110 cel-shadow transition-all text-xs font-black uppercase italic"
          >
            {t.appMatrix.done}
          </button>
        </div>
      </div>
    </div>
  );
}

// =====================================================================
// MatrixCard — 单张模块卡片 (3渲2 战术卡片风格)
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
  const cs = getCategoryStyles(m.category);
  // per WBS §23.32-A: worktree 由 Project Worktree Index 承载, sidebar 钉选按钮禁用
  const indexOwned = (WORKTREE_INDEX_OWNED_MODULES as readonly string[]).includes(m.id);
  return (
    <div
      data-testid={`matrix-card-${m.id}`}
      className="group relative border-2 border-black bg-[var(--cel-surface-sub,#151c2c)] hover:bg-[var(--cel-surface-card,#0f1422)] p-4 cel-shadow hover:translate-x-[-2px] hover:translate-y-[-2px] hover:shadow-[6px_6px_0px_#000000] transition-all flex flex-col justify-between"
    >
      <div>
        <div className="flex items-start justify-between gap-2 mb-2.5">
          <div className="flex items-center gap-3">
            {/* 战术 icon tile — 墨边黑框 + 3渲2 强调色 */}
            <div
              data-testid={`matrix-card-icon-tile-${m.id}`}
              className="size-10 grid place-items-center shrink-0 border-2 border-black bg-black text-[var(--cel-cyan,#00f0ff)] cel-shadow transition-all group-hover:scale-105"
              aria-hidden="true"
            >
              <Icon size={20} strokeWidth={2.25} />
            </div>
            <div>
              <div className="flex items-center gap-1.5">
                <span className="text-xs font-black text-white transition-colors group-hover:text-[var(--cel-cyan,#00f0ff)]">
                  {mod.label}
                </span>
                <span className="font-mono text-[9px] px-1.5 py-0.5 border border-black bg-black text-[var(--cel-gold,#ffc400)] font-bold">
                  {m.code}
                </span>
              </div>
              <span className="text-[10px] font-mono inline-flex items-center gap-1 text-[var(--cel-text-secondary,#94a3b8)]">
                <span className="size-1.5 rotate-45 border border-black bg-[var(--cel-crimson,#ff184c)]" aria-hidden="true" />
                {mod.categoryLabel}
              </span>
            </div>
          </div>
        </div>

        <p className="text-xs text-[var(--cel-text-secondary,#94a3b8)] line-clamp-2 leading-relaxed mb-4 font-normal">
          {mod.description}
        </p>
      </div>

      <div className="border-t-2 border-black/60 pt-3 flex items-center justify-between gap-2">
        <div className="flex items-center gap-1.5">
          {/* 钉选到左侧 */}
          <button
            type="button"
            onClick={onToggleSidebar}
            disabled={indexOwned}
            data-testid={`pin-sidebar-${m.id}`}
            className={clsx(
              "px-2.5 py-1 text-[10px] font-mono font-bold flex items-center gap-1 transition-all border-2 border-black cel-shadow",
              indexOwned
                ? "bg-black/40 text-[var(--cel-text-secondary,#94a3b8)] cursor-not-allowed opacity-60"
                : inSidebar
                  ? "bg-[var(--cel-cyan,#00f0ff)] text-black"
                  : "bg-black/60 text-[var(--cel-text-secondary,#94a3b8)] hover:text-white hover:border-[var(--cel-cyan,#00f0ff)]"
            )}
            title={indexOwned ? t.appMatrix.indexOwnedHint : inSidebar ? t.appMatrix.unpinFromSidebar : t.appMatrix.pinToSidebar}
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
              "px-2.5 py-1 text-[10px] font-mono font-bold flex items-center gap-1 transition-all border-2 border-black cel-shadow",
              inHeader
                ? "bg-[var(--cel-gold,#ffc400)] text-black"
                : "bg-black/60 text-[var(--cel-text-secondary,#94a3b8)] hover:text-white hover:border-[var(--cel-gold,#ffc400)]"
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
          onNavigate={onOpen}
          className="p-1.5 text-[var(--cel-text-secondary,#94a3b8)] hover:text-white border border-black bg-black/60 hover:bg-black transition-colors"
          title={t.appMatrix.openNow}
        >
          <ArrowRight size={14} />
        </Link>
      </div>
    </div>
  );
}
