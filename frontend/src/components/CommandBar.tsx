"use client";

// =====================================================================
// CommandBar — ⌘K 全局命令面板 (per 2026-10-01 OOB 强化)
//   - 微信式: 顶部 search input + 下方 type filter chips + 分组列表
//   - 搜索源: module / task / worktree / canvas / agent / feedback / project /
//     sprint / milestone / changeSet / pullRequest / notification / comment / doc
//   - 见 lib/search/globalSearch.ts 的索引 + scoring
// =====================================================================

import { useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { Search, CornerDownLeft, ArrowUp, ArrowDown, X, Layers, FileText, Hash } from "lucide-react";
import { useCommandBarStore, type RecentItem } from "@/lib/commandBarStore";
import {
  searchAll,
  TYPE_FILTERS,
  TYPE_LABEL,
  type SearchHit,
  type SearchType,
  type SearchResult,
} from "@/lib/search/globalSearch";
import { useTranslation } from "@/lib/i18n";

export function CommandBar() {
  const router = useRouter();
  const isOpen = useCommandBarStore((s) => s.isOpen);
  const query = useCommandBarStore((s) => s.query);
  const setQuery = useCommandBarStore((s) => s.setQuery);
  const filter = useCommandBarStore((s) => s.filter);
  const setFilter = useCommandBarStore((s) => s.setFilter);
  const close = useCommandBarStore((s) => s.close);
  const open = useCommandBarStore((s) => s.open);
  const pushRecent = useCommandBarStore((s) => s.pushRecent);
  const recent = useCommandBarStore((s) => s.recent);
  const { t } = useTranslation();

  // 扁平命中列表 (含 hidden group header), 配合键盘导航
  const [activeIdx, setActiveIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const listRef = useRef<HTMLDivElement | null>(null);

  // ⌘K / Ctrl+K 切换, Esc 关闭
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const isCmdK = (e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k";
      if (isCmdK) {
        e.preventDefault();
        if (isOpen) close();
        else open();
      } else if (e.key === "Escape" && isOpen) {
        e.preventDefault();
        close();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [isOpen, open, close]);

  // 打开时 focus + 重置
  useEffect(() => {
    if (isOpen) {
      setActiveIdx(0);
      const t = window.setTimeout(() => inputRef.current?.focus(), 0);
      return () => window.clearTimeout(t);
    }
    return undefined;
  }, [isOpen]);

  // 搜索: 每次 query/filter 变重新算 (微信搜索依赖)
  const result = useMemo<SearchResult>(() => {
    return searchAll(query, { filter, group: true, limit: 60 });
  }, [query, filter]);

  // 扁平列表 (供键盘导航)
  const flat = useMemo(() => result.groups.flatMap((g) => g.hits), [result]);

  // query/filter 变时 clamp activeIdx
  useEffect(() => {
    if (activeIdx >= flat.length) {
      setActiveIdx(Math.max(0, flat.length - 1));
    }
  }, [flat.length, activeIdx]);

  // 选中跳转
  const commit = (hit: SearchHit) => {
    pushRecent({
      id: hit.id,
      label: hit.label,
      href: hit.href,
      type: "page",
      at: Date.now(),
    });
    close();
    router.push(hit.href);
  };

  // 键盘导航 (input 上)
  const onListKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActiveIdx((i) => Math.min(flat.length - 1, i + 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActiveIdx((i) => Math.max(0, i - 1));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const target = flat[activeIdx];
      if (target) commit(target);
    } else if (e.key === "Tab" && !e.shiftKey) {
      // Tab 切 filter chip 焦点 (forward 视觉焦点切换)
      e.preventDefault();
      const ids = TYPE_FILTERS.map((f) => f.id);
      const cur = ids.indexOf(filter);
      const next = ids[(cur + 1) % ids.length];
      setFilter(next);
    } else if (e.key === "Tab" && e.shiftKey) {
      e.preventDefault();
      const ids = TYPE_FILTERS.map((f) => f.id);
      const cur = ids.indexOf(filter);
      const prev = ids[(cur - 1 + ids.length) % ids.length];
      setFilter(prev);
    }
  };

  if (!isOpen) return null;

  // 计算每个 hit 的扁平 idx (group header 不算 hit, 但参与滚动位置)
  return (
    <div
      data-testid="command-bar-overlay"
      role="dialog"
      aria-modal="true"
      aria-label={t.commandBar.ariaLabel}
      className="fixed inset-0 z-50 flex items-start justify-center pt-[10vh] bg-black/40 backdrop-blur-sm"
      onClick={(e) => {
        if (e.target === e.currentTarget) close();
      }}
    >
      <div
        data-testid="command-bar-panel"
        className="w-full max-w-2xl mx-4 border-2 border-[var(--cel-ink)] bg-[color:var(--color-surface)] cel-shadow-lg overflow-hidden"
        onKeyDown={onListKeyDown}
      >
        {/* 顶部: 搜索框 */}
        <div className="flex items-center gap-2 px-4 h-12 border-b border-line">
          <Search size={16} className="text-accent shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t.commandBar.placeholder}
            data-testid="command-bar-input"
            className="flex-1 bg-transparent outline-none text-sm text-ink placeholder:text-ink-mute"
            autoComplete="off"
            spellCheck={false}
          />
          {query && (
            <span className="text-[10px] font-mono text-ink-mute">
              {result.total} {t.commandBar.hitSuffix}
            </span>
          )}
          <button
            type="button"
            onClick={close}
            data-testid="command-bar-close"
            aria-label={t.commandBar.closeAria}
            className="p-1 text-ink-dim hover:text-ink rounded"
          >
            <X size={14} />
          </button>
        </div>

        {/* 微信式 filter chips: 全部分类横排 */}
        <div
          className="flex items-center gap-1 px-3 py-2 border-b border-line overflow-x-auto bg-bg-soft/30"
          data-testid="command-bar-filter-chips"
        >
          {TYPE_FILTERS.map((f) => {
            const active = filter === f.id;
            return (
              <button
                key={f.id}
                type="button"
                onClick={() => setFilter(f.id)}
                data-testid={`command-bar-filter-${f.id}`}
                className={
                  "shrink-0 px-2.5 py-1 rounded-full text-[11px] font-medium transition-colors border " +
                  (active
                    ? "border-accent bg-accent/15 text-accent"
                    : "border-line bg-bg-card text-ink-dim hover:border-accent/40 hover:text-ink")
                }
              >
                {f.label}
              </button>
            );
          })}
        </div>

        {/* 分组列表 */}
        <div
          ref={listRef}
          data-testid="command-bar-list"
          className="max-h-[60vh] overflow-y-auto py-1"
        >
          {query.trim() === "" ? (
            <EmptyQueryState recent={recent} t={t} onPick={(href) => router.push(href)} />
          ) : result.groups.length === 0 ? (
            <div className="px-4 py-10 text-center text-ink-mute text-xs font-mono">
              {t.commandBar.emptyHint}
            </div>
          ) : (
            <SearchGroupsContainer
              groups={result.groups}
              activeGlobalIdx={activeIdx}
              onCommit={commit}
              onHover={setActiveIdx}
            />
          )}
        </div>

        {/* 底部状态栏 */}
        <div className="flex items-center gap-4 px-4 h-9 border-t border-line text-[10px] font-mono text-ink-mute">
          <span className="flex items-center gap-1">
            <ArrowUp size={10} />
            <ArrowDown size={10} />
            {t.commandBar.hintNavigate}
          </span>
          <span className="flex items-center gap-1">
            <CornerDownLeft size={10} />
            {t.commandBar.hintOpen}
          </span>
          <span>{t.commandBar.hintClose}</span>
          <span className="ml-auto">
            {query.trim() && (
              <span>
                {result.total} {t.commandBar.hitSuffix}
                {filter !== "all" && ` · ${TYPE_LABEL[filter]}`}
              </span>
            )}
            {recent.length > 0 && (
              <span data-testid="command-bar-recent-count" className="ml-2">
                recent: {recent.length}
              </span>
            )}
          </span>
        </div>
      </div>
    </div>
  );
}

// =====================================================================
// SearchGroup — 分组渲染 (header + items)
// =====================================================================
interface SearchGroupProps {
  type: SearchType;
  hits: SearchHit[];
  flatStartIdx: number;
  activeGlobalIdx: number;
  onCommit: (hit: SearchHit) => void;
  onHover: (flatIdx: number) => void;
}

// wrapper that pre-computes flatStartIdx per group (avoids JSX side effects)
function SearchGroupsContainer({ groups, activeGlobalIdx, onCommit, onHover }: {
  groups: Array<{ type: SearchType; hits: SearchHit[] }>;
  activeGlobalIdx: number;
  onCommit: (hit: SearchHit) => void;
  onHover: (flatIdx: number) => void;
}) {
  let cursor = 0;
  return (
    <>
      {groups.map((group) => {
        const flatStartIdx = cursor;
        cursor += group.hits.length;
        return (
          <SearchGroup
            key={group.type}
            type={group.type}
            hits={group.hits}
            flatStartIdx={flatStartIdx}
            activeGlobalIdx={activeGlobalIdx}
            onCommit={onCommit}
            onHover={onHover}
          />
        );
      })}
    </>
  );
}

function SearchGroup({ type, hits, flatStartIdx, activeGlobalIdx, onCommit, onHover }: SearchGroupProps) {
  return (
    <div data-testid={`command-bar-group-${type}`} className="mb-1 last:mb-0">
      <div className="flex items-center gap-1.5 px-4 py-1 text-[10px] uppercase tracking-wider font-mono text-ink-mute">
        <Layers size={10} />
        <span>{TYPE_LABEL[type]}</span>
        <span className="text-ink-mute/60">·</span>
        <span>{hits.length}</span>
      </div>
      {hits.map((hit, localIdx) => {
        const flatIdx = flatStartIdx + localIdx;
        const isActive = flatIdx === activeGlobalIdx;
        return (
          <SearchHitRow
            key={hit.id}
            hit={hit}
            isActive={isActive}
            onClick={() => onCommit(hit)}
            onHover={() => onHover(flatIdx)}
          />
        );
      })}
    </div>
  );
}

// =====================================================================
// SearchHitRow — 单条 hit (跟 type 无关, 通用渲染)
// =====================================================================
interface SearchHitRowProps {
  hit: SearchHit;
  isActive: boolean;
  onClick: () => void;
  onHover: () => void;
}

function SearchHitRow({ hit, isActive, onClick, onHover }: SearchHitRowProps) {
  const Icon = hit.type === "doc" ? FileText : Hash;
  return (
    <button
      type="button"
      onClick={onClick}
      onMouseEnter={onHover}
      data-testid={`command-bar-hit-${hit.id.replace(/:/g, "-")}`}
      data-active={isActive ? "true" : "false"}
      className={
        "w-full flex items-center gap-3 px-4 h-10 text-left transition-colors " +
        (isActive ? "bg-accent/10 text-ink" : "text-ink-dim hover:bg-bg-soft/60")
      }
    >
      <Icon size={13} className={isActive ? "text-accent" : "text-ink-mute"} />
      <span className="text-sm font-medium truncate max-w-[55%]">
        <HighlightMatch text={hit.label} />
      </span>
      {hit.subLabel && (
        <span className="ml-1 text-[10px] text-ink-mute truncate max-w-[35%]">
          {hit.subLabel}
        </span>
      )}
      {hit.tone && (
        <span
          className={
            "ml-auto text-[9px] font-mono px-1.5 py-0.5 rounded border border-line " +
            (hit.tone === "ok"
              ? "bg-ok/15 text-ok border-ok/40"
              : hit.tone === "warn"
              ? "bg-warning/15 text-warning border-warning/40"
              : hit.tone === "err"
              ? "bg-err/15 text-err border-err/40"
              : hit.tone === "info"
              ? "bg-info/15 text-info border-info/40"
              : "text-ink-mute")
          }
        >
          {hit.tone}
        </span>
      )}
    </button>
  );
}

// =====================================================================
// HighlightMatch — 高亮 query 命中片段 (类似微信搜索高亮)
// =====================================================================
function HighlightMatch({ text }: { text: string }) {
  if (text === undefined || text === null) return <>{String(text ?? "")}</>;
  // 由父组件保持 query 全局; 这里简单 split by 不区分大小写
  const { t } = useTranslation();
  const query = useCommandBarStore((s) => s.query.trim());
  if (!query) return <>{text}</>;
  const idx = text.toLowerCase().indexOf(query.toLowerCase());
  if (idx < 0) return <>{text}</>;
  const before = text.slice(0, idx);
  const match = text.slice(idx, idx + query.length);
  const after = text.slice(idx + query.length);
  return (
    <>
      {before}
      <mark className="bg-accent/25 text-accent rounded px-0.5">{match}</mark>
      {after}
    </>
  );
}

// =====================================================================
// EmptyQueryState — 未输入时显示 recent
// =====================================================================
function EmptyQueryState({ recent, t, onPick }: { recent: RecentItem[]; t: any; onPick: (href: string) => void }) {
  return (
    <div className="px-2 py-4">
      <div className="px-3 py-2 text-[10px] uppercase tracking-wider font-mono text-ink-mute">
        最近访问
      </div>
      {recent.length === 0 ? (
        <div className="px-4 py-6 text-center text-ink-mute text-xs font-mono">
          {t.commandBar.placeholder}
        </div>
      ) : (
        recent.map((r) => (
          <button
            key={r.id}
            type="button"
            onClick={() => r.href && onPick(r.href)}
            data-testid={`command-bar-recent-${r.id}`}
            className="w-full flex items-center gap-3 px-4 h-9 text-left text-ink-dim hover:bg-bg-soft/60 hover:text-ink transition-colors"
          >
            <Hash size={12} className="text-ink-mute" />
            <span className="text-sm">{r.label}</span>
            <span className="ml-auto text-[10px] text-ink-mute">{r.type}</span>
          </button>
        ))
      )}
    </div>
  );
}

export { TYPE_LABEL };
