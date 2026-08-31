// =====================================================================
// CommandBar — ⌘K 全局命令面板消费组件 (per DRIFT-α-020 / DRIFT-α-029 修复)
// =====================================================================
// 修复前乖离:
//   - AppHeader.tsx:152 已有 ⌘K 触发器, 调 openCommandBar() 设 isOpen=true
//   - lib/commandBarStore.ts:71 open() 实现完整
//   - **全项目无消费组件** (per sibling mvs_eab2555e alpha-frontend-drift.md
//     实测 `Get-ChildItem CommandBar*` 0 命中)
//   - 后果: 按 ⌘K 后 store state 变但 UI 无任何反馈 (破坏性)
//
// 修复目标 (per frontend-internal-04 §1.1 MVP 承诺):
//   - 订阅 useCommandBarStore.isOpen, isOpen=true 时渲染浮层
//   - 列表 source: ALL_MODULES (25+ 模块, 来自 nav/registry.ts)
//   - 键盘: ⌘K / Ctrl+K 切换, Esc 关闭, ↑↓ 选择, Enter 跳转
//   - 选中: pushRecent() + close() + router.push(href)
//   - 查询: 子串不区分大小写命中 label/code/description
//
// 守门 #1+#9+#12 实证: 本文件独立, 不动 store/store, 单测 1 个, commit 由 root 直实装.
// =====================================================================
"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { Search, CornerDownLeft, ArrowUp, ArrowDown, X } from "lucide-react";
import { useCommandBarStore, type RecentItem } from "@/lib/commandBarStore";
import { ALL_MODULES, type ModuleDefinition } from "@/lib/nav/registry";

export function CommandBar() {
  const router = useRouter();
  const isOpen = useCommandBarStore((s) => s.isOpen);
  const query = useCommandBarStore((s) => s.query);
  const setQuery = useCommandBarStore((s) => s.setQuery);
  const close = useCommandBarStore((s) => s.close);
  const open = useCommandBarStore((s) => s.open);
  const pushRecent = useCommandBarStore((s) => s.pushRecent);
  const recent = useCommandBarStore((s) => s.recent);

  const [activeIdx, setActiveIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const listRef = useRef<HTMLDivElement | null>(null);

  // 全局 ⌘K / Ctrl+K 切换 (per frontend-internal-04 §1.1 承诺: Topbar 7 行 + MVP 实现)
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

  // 打开时自动 focus 输入框 + 重置 activeIdx
  useEffect(() => {
    if (isOpen) {
      setActiveIdx(0);
      // 下一帧 focus, 避免 SSR 焦点警告
      const t = window.setTimeout(() => inputRef.current?.focus(), 0);
      return () => window.clearTimeout(t);
    }
    return undefined;
  }, [isOpen]);

  // 过滤: 子串不区分大小写命中 label / code / description / categoryLabel
  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return ALL_MODULES;
    return ALL_MODULES.filter(
      (m) =>
        m.label.toLowerCase().includes(q) ||
        m.code.toLowerCase().includes(q) ||
        m.description.toLowerCase().includes(q) ||
        m.categoryLabel.toLowerCase().includes(q),
    );
  }, [query]);

  // filtered 变化时, activeIdx 收回 [0, filtered.length-1]
  useEffect(() => {
    if (activeIdx >= filtered.length) {
      setActiveIdx(Math.max(0, filtered.length - 1));
    }
  }, [filtered.length, activeIdx]);

  const commit = (m: ModuleDefinition) => {
    pushRecent({ id: m.id, label: m.label, href: m.href, type: "page", at: Date.now() });
    close();
    router.push(m.href);
  };

  const onListKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActiveIdx((i) => Math.min(filtered.length - 1, i + 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActiveIdx((i) => Math.max(0, i - 1));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const target = filtered[activeIdx];
      if (target) commit(target);
    }
  };

  if (!isOpen) return null;

  return (
    <div
      data-testid="command-bar-overlay"
      role="dialog"
      aria-modal="true"
      aria-label="Command bar"
      className="fixed inset-0 z-50 flex items-start justify-center pt-[12vh] bg-black/40 backdrop-blur-sm"
      onClick={(e) => {
        // 点击空白处关闭
        if (e.target === e.currentTarget) close();
      }}
    >
      <div
        data-testid="command-bar-panel"
        className="w-full max-w-2xl mx-4 rounded-xl border border-line bg-[color:var(--color-surface)] shadow-2xl overflow-hidden"
        onKeyDown={onListKeyDown}
      >
        {/* 搜索框 */}
        <div className="flex items-center gap-2 px-4 h-12 border-b border-line">
          <Search size={16} className="text-accent shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="搜索 25+ 模块 (按 label / code / description) ..."
            data-testid="command-bar-input"
            className="flex-1 bg-transparent outline-none text-sm text-ink placeholder:text-ink-mute"
            autoComplete="off"
            spellCheck={false}
          />
          <button
            type="button"
            onClick={close}
            data-testid="command-bar-close"
            aria-label="Close command bar (Esc)"
            className="p-1 text-ink-dim hover:text-ink rounded"
          >
            <X size={14} />
          </button>
        </div>

        {/* 命中列表 */}
        <div
          ref={listRef}
          data-testid="command-bar-list"
          className="max-h-[60vh] overflow-y-auto py-1"
        >
          {filtered.length === 0 ? (
            <div className="px-4 py-8 text-center text-ink-mute text-xs font-mono">
              0 命中 — 按 Esc 退出
            </div>
          ) : (
            filtered.map((m, idx) => {
              const isActive = idx === activeIdx;
              const Icon = m.icon;
              return (
                <button
                  type="button"
                  key={m.id}
                  onClick={() => commit(m)}
                  onMouseEnter={() => setActiveIdx(idx)}
                  data-testid={`command-bar-item-${m.id}`}
                  data-active={isActive ? "true" : "false"}
                  className={
                    "w-full flex items-center gap-3 px-4 h-10 text-left transition-colors " +
                    (isActive
                      ? "bg-accent/10 text-ink"
                      : "text-ink-dim hover:bg-bg-soft/60")
                  }
                >
                  <Icon size={14} className={isActive ? "text-accent" : "text-ink-mute"} />
                  <span className="text-sm font-medium">{m.label}</span>
                  <span className="text-[10px] font-mono text-ink-mute ml-1">[{m.code}]</span>
                  <span className="text-[10px] text-ink-mute ml-1 px-1.5 py-0.5 rounded border border-line">
                    {m.categoryLabel}
                  </span>
                  <span className="ml-auto text-[10px] text-ink-mute truncate max-w-[40%]">
                    {m.description}
                  </span>
                </button>
              );
            })
          )}
        </div>

        {/* 底部状态栏 */}
        <div className="flex items-center gap-4 px-4 h-9 border-t border-line text-[10px] font-mono text-ink-mute">
          <span className="flex items-center gap-1">
            <ArrowUp size={10} />
            <ArrowDown size={10} />
            navigate
          </span>
          <span className="flex items-center gap-1">
            <CornerDownLeft size={10} />
            open
          </span>
          <span>Esc close</span>
          <span className="ml-auto">
            {filtered.length} / {ALL_MODULES.length} modules
            {recent.length > 0 && (
              <span data-testid="command-bar-recent-count" className="ml-2">
                · {recent.length} recent
              </span>
            )}
          </span>
        </div>
      </div>
    </div>
  );
}

// 暴露 helper: 渲染最近条目 (Phase 2+ 用; MVP 不在 panel 内显示, 仅持久化)
export function getRecentItems(limit = 5): RecentItem[] {
  return useCommandBarStore.getState().recent.slice(0, limit);
}
