/*
@cypher schema=1 source_sha256=89ce0730ffd95f604388ef98d3c047bc18ab93665fd8970de902d4e9faa1a63e
MERGE (self:File {path:"frontend/src/app/(app)/inbox/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/inbox/page.tsx::InboxPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
"use client";

// =====================================================================
// /inbox — 收件箱 (per 2026-11-14 参考 mockup: Inbox.dc.html)
// 分类 tabs (全部/评审请求/评论/通知) + 消息列表 + 右侧详情面板
// 保留真实功能: read/unread toggle + /api/notifications 联动
// =====================================================================

import { useEffect, useState } from "react";
import { Bell, GitBranch, MessageSquare, CheckCircle2, X } from "lucide-react";
import { clsx } from "clsx";
import { MOCK_NOTIFS_FALLBACK } from "@/mocks/data";
import type { MockNotif } from "@/mocks/schemas/inbox";
import { useTranslation } from "@/lib/i18n";

type Category = "全部" | "评审请求" | "评论" | "通知";

// 分类映射: mock 通知按 subject/body 特征归类 (per mockup 的分类结构)
function categoryOf(n: MockNotif): Exclude<Category, "全部"> {
  const text = `${n.subject} ${n.body}`;
  if (/PR|评审|review|Review/i.test(text)) return "评审请求";
  if (/评论|comment|Comment/i.test(text)) return "评论";
  return "通知";
}

export default function InboxPage() {
  const { t } = useTranslation();
  const [notifs, setNotifs] = useState<ReadonlyArray<MockNotif>>(MOCK_NOTIFS_FALLBACK);
  const [readSet, setReadSet] = useState<Set<string>>(
    () => new Set(MOCK_NOTIFS_FALLBACK.filter((n) => n.read).map((n) => n.id)),
  );
  const [activeCategory, setActiveCategory] = useState<Category>("全部");
  const [selectedId, setSelectedId] = useState<string | null>(null);

  useEffect(() => {
    fetch("/api/notifications")
      .then((r) => r.json())
      .then((data: ReadonlyArray<MockNotif>) => {
        setNotifs(data);
        setReadSet(new Set(data.filter((n) => n.read).map((n) => n.id)));
      })
      .catch(() => {
        /* keep FALLBACK (per §4 #1 缺标) */
      });
  }, []);

  const toggleRead = (id: string) => {
    setReadSet((prev) => {
      const next = new Set(prev);
      const wasRead = next.has(id);
      if (wasRead) next.delete(id);
      else next.add(id);
      fetch(`/api/notifications/${id}`, { method: "PATCH" }).catch(() => {
        if (wasRead) next.add(id);
        else next.delete(id);
      });
      return next;
    });
  };

  const visible = activeCategory === "全部"
    ? notifs
    : notifs.filter((n) => categoryOf(n) === activeCategory);
  const unread = notifs.filter((n) => !readSet.has(n.id)).length;
  const selected = notifs.find((n) => n.id === selectedId) ?? visible[0] ?? null;

  return (
    <div className="flex h-full flex-col bg-bg" data-testid="inbox-page">
      {/* 头部: 全局 / 收件箱 */}
      <header className="border-b border-line bg-bg-card px-4 py-3 flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm">
          <span className="text-ink-dim">全局</span>
          <span className="text-ink-dim">/</span>
          <h1 className="font-bold text-ink">收件箱</h1>
          <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-err/10 text-err font-semibold" data-testid="inbox-unread-count">
            {unread}
          </span>
        </div>
      </header>

      {/* 分类 tabs */}
      <div data-testid="inbox-categories" className="border-b border-line bg-bg-card px-3 flex items-center gap-1">
        {(["全部", "评审请求", "评论", "通知"] as const).map((cat) => (
          <button
            key={cat}
            type="button"
            onClick={() => setActiveCategory(cat)}
            data-testid={`inbox-category-${cat}`}
            className={clsx(
              "px-3 py-2 text-sm border-b-2 -mb-px transition-colors",
              activeCategory === cat
                ? "border-accent text-accent font-semibold"
                : "border-transparent text-ink-dim hover:text-ink",
            )}
          >
            {cat}
          </button>
        ))}
      </div>

      {/* 列表 + 详情 */}
      <div className="flex flex-1 min-h-0">
        {/* 消息列表 */}
        <ul className="w-96 shrink-0 border-r border-line overflow-y-auto" data-testid="inbox-list">
          {visible.map((n) => {
            const isRead = readSet.has(n.id);
            const cat = categoryOf(n);
            return (
              <li
                key={n.id}
                data-testid={`inbox-item-${n.id}`}
                onClick={() => setSelectedId(n.id)}
                className={clsx(
                  "px-4 py-3 border-b border-line/60 cursor-pointer transition-colors",
                  selectedId === n.id ? "bg-accent/5 border-l-2 border-l-accent" : "hover:bg-bg-soft/60",
                  isRead && "opacity-60",
                )}
              >
                <div className="flex items-center gap-2">
                  <span className={clsx(
                    "text-[10px] px-1.5 py-0.5 rounded border shrink-0",
                    cat === "评审请求" ? "bg-accent/10 text-accent border-accent/40" :
                    cat === "评论" ? "bg-info/10 text-info border-info/40" : "bg-bg-soft text-ink-dim border-line",
                  )}>
                    {cat}
                  </span>
                  <span className="font-mono text-xs font-semibold text-ink truncate flex-1">{n.subject}</span>
                  {!isRead && <span className="size-1.5 rounded-full bg-accent shrink-0" />}
                </div>
                <div className="mt-1 flex items-center justify-between gap-2">
                  <span className="text-xs text-ink-dim truncate">{n.body}</span>
                  <span className="text-[10px] font-mono text-ink-mute shrink-0">{n.ago}</span>
                </div>
              </li>
            );
          })}
        </ul>

        {/* 详情面板 */}
        <aside className="flex-1 min-w-0 overflow-y-auto p-4">
          {selected ? (
            <div data-testid="inbox-detail" className="bg-bg-card rounded-xl border border-line p-4">
              <div className="flex items-center gap-2 flex-wrap">
                <span className={clsx(
                  "text-[10px] px-1.5 py-0.5 rounded border",
                  categoryOf(selected) === "评审请求" ? "bg-accent/10 text-accent border-accent/40" :
                  categoryOf(selected) === "评论" ? "bg-info/10 text-info border-info/40" : "bg-bg-soft text-ink-dim border-line",
                )}>
                  {categoryOf(selected)} · {selected.ago}
                </span>
                <span className="font-mono text-xs text-ink-mute">{selected.subject}</span>
              </div>
              <h2 className="mt-2 text-base font-bold text-ink">{selected.subject}</h2>
              <p className="mt-2 text-sm text-ink-dim leading-relaxed">{selected.body}</p>
              <div className="mt-4 flex items-center gap-2">
                <button type="button" data-testid="inbox-open-branch" className="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-lg bg-accent text-white hover:bg-accent/90">
                  <GitBranch size={12} /> 在 Branch 中打开
                </button>
                <button
                  type="button"
                  onClick={() => toggleRead(selected.id)}
                  data-testid="inbox-mark-read"
                  className="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-lg border border-line text-ink-dim hover:bg-bg-soft"
                >
                  <CheckCircle2 size={12} /> {readSet.has(selected.id) ? "标为未读" : "标为已读"}
                </button>
              </div>
            </div>
          ) : (
            <div className="h-full flex items-center justify-center text-ink-mute text-sm">
              <Bell size={32} className="mb-2" /> 该分类暂无消息
            </div>
          )}
        </aside>
      </div>
    </div>
  );
}
