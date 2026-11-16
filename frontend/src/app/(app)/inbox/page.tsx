/*
@cypher schema=1 source_sha256=5c7324e3a683433437dc7529ee202bf0f1e1d9eb600c94229bbc1875585e5ea0
MERGE (self:File {path:"frontend/src/app/(app)/inbox/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/inbox/page.tsx::InboxPage",kind:"component"})
MERGE (notifApi:File {path:"frontend/src/lib/notifications/notificationsApi.ts"})
MERGE (notifTypes:File {path:"frontend/src/types/notifications.ts"})
MERGE (realMode:File {path:"frontend/src/mocks/real-mode.ts"})
MERGE (statusNotice:File {path:"frontend/src/components/status/StatusNotice.tsx"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:IMPORTS]->(notifApi)
MERGE (self)-[:IMPORTS]->(notifTypes)
MERGE (self)-[:IMPORTS]->(realMode)
MERGE (self)-[:IMPORTS]->(statusNotice)
@endcypher
*/
"use client";

// =====================================================================
// /inbox — 收件箱 (per 2026-11-17 接真实后端通知 API)
//   数据源: 真实后端 GET /api/v1/notifications (realFetch, real-mode)
//   三态 (per 2026-11-16 清 seed/mock 显示入口纪律):
//     - real-mode 未开启 / 后端不可达 → StatusNotice "未接入" + 原因
//     - 401/403 → StatusNotice "未授权"
//     - 成功且空 → 诚实空态
//   绝不回落 MOCK_NOTIFS 冒充真实数据。
//   已读/未读: PATCH /api/v1/notifications/{id} (乐观更新 + 失败回滚)
// =====================================================================

import { useCallback, useEffect, useState } from "react";
import { Bell, CheckCircle2, GitBranch, RefreshCw } from "lucide-react";
import { clsx } from "clsx";
import { getRealModeState } from "@/mocks/real-mode";
import {
  listNotifications,
  markRead,
  markUnread,
  NotificationsApiError,
} from "@/lib/notifications/notificationsApi";
import type { Notification } from "@/types/notifications";
import {
  StatusNotice,
  inferNoticeState,
  type NoticeState,
} from "@/components/status/StatusNotice";
import { useTranslation } from "@/lib/i18n";

type Category = "all" | "review" | "comment" | "notice";
const CATEGORY_IDS = ["all", "review", "comment", "notice"] as const;

// 分类映射: 优先按 kind 特征词, 兜底按 subject/body 文本 (沿用旧 mockup 归类规则)
function categoryOf(n: Notification): Exclude<Category, "all"> {
  const text = `${n.kind} ${n.subject} ${n.body}`;
  if (/review|评审|PR\b/i.test(text)) return "review";
  if (/comment|评论/i.test(text)) return "comment";
  return "notice";
}

function errText(err: unknown): string {
  if (err instanceof Error) return err.message;
  return String(err);
}

/** created_at → 紧凑相对时间 (按当前语言, 无依赖) */
function relativeTime(iso: string, language: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  const diffMs = date.getTime() - Date.now();
  const abs = Math.abs(diffMs);
  const locale = language === "ja" ? "ja" : language === "en" ? "en" : "zh-CN";
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  if (abs < 60_000) return rtf.format(Math.round(diffMs / 1_000), "second");
  if (abs < 3_600_000) return rtf.format(Math.round(diffMs / 60_000), "minute");
  if (abs < 86_400_000) return rtf.format(Math.round(diffMs / 3_600_000), "hour");
  if (abs < 7 * 86_400_000) return rtf.format(Math.round(diffMs / 86_400_000), "day");
  return new Intl.DateTimeFormat(locale, { year: "numeric", month: "2-digit", day: "2-digit" }).format(date);
}

type LoadState = "loading" | "ready" | "error";

export default function InboxPage() {
  const { t, tx, language } = useTranslation();
  const [notifs, setNotifs] = useState<ReadonlyArray<Notification>>([]);
  const [loadState, setLoadState] = useState<LoadState>("loading");
  const [loadError, setLoadError] = useState<unknown>(null);
  const [activeCategory, setActiveCategory] = useState<Category>("all");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [toggleErrorId, setToggleErrorId] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoadState("loading");
    setLoadError(null);
    setToggleErrorId(null);
    // 未配置 real-mode → 诚实未接入 (不发请求, 原因直接给用户)
    if (!getRealModeState().enabled) {
      setLoadError(new NotificationsApiError("real-mode 未开启", "real_mode_disabled"));
      setLoadState("error");
      return;
    }
    try {
      // 一次拉满当前单页上限 (后端 limit clamp 1..=200), 不做分页 UI
      const data = await listNotifications({ limit: 200 });
      setNotifs(data);
      setLoadState("ready");
      setSelectedId((prev) =>
        prev && data.some((n) => n.id === prev) ? prev : (data[0]?.id ?? null),
      );
    } catch (err) {
      setNotifs([]);
      setLoadError(err);
      setLoadState("error");
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  // 已读/未读切换: 乐观更新 + 失败回滚 (真实 API)
  const toggleRead = async (n: Notification) => {
    const nextRead = !n.read;
    setToggleErrorId(null);
    setNotifs((prev) => prev.map((x) => (x.id === n.id ? { ...x, read: nextRead } : x)));
    try {
      const updated = nextRead ? await markRead(n.id) : await markUnread(n.id);
      // 以真实后端响应为准对齐
      setNotifs((prev) => prev.map((x) => (x.id === n.id ? { ...x, read: updated.read } : x)));
    } catch {
      // 失败回滚
      setNotifs((prev) => prev.map((x) => (x.id === n.id ? { ...x, read: n.read } : x)));
      setToggleErrorId(n.id);
    }
  };

  const categoryLabel = (cat: Category): string => {
    switch (cat) {
      case "review": return t.inbox.categoryReview;
      case "comment": return t.inbox.categoryComment;
      case "notice": return t.inbox.categoryNotice;
      default: return t.inbox.categoryAll;
    }
  };

  const visible = activeCategory === "all"
    ? notifs
    : notifs.filter((n) => categoryOf(n) === activeCategory);
  const unread = notifs.filter((n) => !n.read).length;
  const selected = notifs.find((n) => n.id === selectedId) ?? visible[0] ?? null;

  // 三态映射 (未接入 / 未授权 / 空 + 加载/失败)
  let notice: { state: NoticeState; title: string; hint: string } | null = null;
  if (loadState === "loading") {
    notice = { state: "loading", title: t.inbox.loadingTitle, hint: t.inbox.loadingHint };
  } else if (loadState === "error") {
    if (loadError instanceof NotificationsApiError && loadError.code === "real_mode_disabled") {
      notice = { state: "disconnected", title: t.inbox.disconnectedTitle, hint: t.inbox.notConfiguredHint };
    } else {
      const inferred = inferNoticeState(loadError);
      if (inferred === "unauthorized") {
        notice = { state: "unauthorized", title: t.inbox.unauthorizedTitle, hint: t.inbox.unauthorizedHint };
      } else if (inferred === "disconnected") {
        notice = { state: "disconnected", title: t.inbox.disconnectedTitle, hint: t.inbox.disconnectedHint };
      } else {
        notice = { state: "failed", title: t.inbox.loadFailedTitle, hint: t.inbox.loadFailedHint };
      }
    }
  } else if (notifs.length === 0) {
    notice = { state: "empty", title: t.inbox.emptyTitle, hint: t.inbox.emptyHint };
  }

  return (
    <div className="flex h-full flex-col bg-bg" data-testid="inbox-page">
      {/* 头部: 全局 / 收件箱 */}
      <header className="border-b border-line bg-bg-card px-4 py-3 flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm">
          <span className="text-ink-dim">{t.inbox.global}</span>
          <span className="text-ink-dim">/</span>
          <h1 className="font-bold text-ink">{t.pageTitles["/inbox"].title}</h1>
          {loadState === "ready" && (
            <span
              className="text-[10px] px-1.5 py-0.5 rounded-full bg-err/10 text-err font-semibold"
              data-testid="inbox-unread-count"
            >
              {unread}
            </span>
          )}
        </div>
      </header>

      {/* 分类 tabs */}
      <div data-testid="inbox-categories" className="border-b border-line bg-bg-card px-3 flex items-center gap-1">
        {CATEGORY_IDS.map((cat) => (
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
            {categoryLabel(cat)}
          </button>
        ))}
      </div>

      {notice ? (
        // 未接入 / 未授权 / 加载 / 失败 / 空 — 诚实展示, 绝不回落 mock
        <div className="flex flex-1 min-h-0 items-center justify-center p-4">
          <div className="w-full max-w-md space-y-2">
            <StatusNotice state={notice.state} title={notice.title} hint={notice.hint} />
            {loadState === "error" && loadError != null && (
              <p className="text-[10px] font-mono text-ink-mute break-all" data-testid="inbox-error-detail">
                {t.inbox.errorDetailLabel}: {errText(loadError)}
              </p>
            )}
            {loadState === "error" && (
              <button
                type="button"
                onClick={() => void load()}
                data-testid="inbox-retry"
                className="inline-flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-lg border border-line text-ink-dim hover:bg-bg-soft"
              >
                <RefreshCw size={12} /> {t.inbox.retry}
              </button>
            )}
          </div>
        </div>
      ) : (
        /* 列表 + 详情 */
        <div className="flex flex-1 min-h-0">
          {/* 消息列表 */}
          <ul className="w-96 shrink-0 border-r border-line overflow-y-auto" data-testid="inbox-list">
            {visible.map((n) => {
              const cat = categoryOf(n);
              return (
                <li
                  key={n.id}
                  data-testid={`inbox-item-${n.id}`}
                  onClick={() => setSelectedId(n.id)}
                  className={clsx(
                    "px-4 py-3 border-b border-line/60 cursor-pointer transition-colors",
                    selectedId === n.id ? "bg-accent/5 border-l-2 border-l-accent" : "hover:bg-bg-soft/60",
                    n.read && "opacity-60",
                  )}
                >
                  <div className="flex items-center gap-2">
                    <span className={clsx(
                      "text-[10px] px-1.5 py-0.5 rounded border shrink-0",
                      cat === "review" ? "bg-accent/10 text-accent border-accent/40" :
                      cat === "comment" ? "bg-info/10 text-info border-info/40" : "bg-bg-soft text-ink-dim border-line",
                    )}>
                      {categoryLabel(cat)}
                    </span>
                    <span className="font-mono text-xs font-semibold text-ink truncate flex-1">{n.subject}</span>
                    {!n.read && <span className="size-1.5 rounded-full bg-accent shrink-0" />}
                  </div>
                  <div className="mt-1 flex items-center justify-between gap-2">
                    <span className="text-xs text-ink-dim truncate">{n.body}</span>
                    <span className="text-[10px] font-mono text-ink-mute shrink-0">{relativeTime(n.created_at, language)}</span>
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
                    categoryOf(selected) === "review" ? "bg-accent/10 text-accent border-accent/40" :
                    categoryOf(selected) === "comment" ? "bg-info/10 text-info border-info/40" : "bg-bg-soft text-ink-dim border-line",
                  )}>
                    {categoryLabel(categoryOf(selected))} · {relativeTime(selected.created_at, language)}
                  </span>
                  <span className="font-mono text-xs text-ink-mute">{selected.actor_name ? tx(t.inbox.fromActor, { name: selected.actor_name }) : selected.actor_id}</span>
                </div>
                <h2 className="mt-2 text-base font-bold text-ink">{selected.subject}</h2>
                <p className="mt-2 text-sm text-ink-dim leading-relaxed">{selected.body}</p>
                <div className="mt-4 flex items-center gap-2">
                  {selected.subject_url && (
                    <a
                      href={selected.subject_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      data-testid="inbox-open-branch"
                      className="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-lg bg-accent text-white hover:bg-accent/90"
                    >
                      <GitBranch size={12} /> {t.inbox.openInBranch}
                    </a>
                  )}
                  <button
                    type="button"
                    onClick={() => void toggleRead(selected)}
                    data-testid="inbox-mark-read"
                    className="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-lg border border-line text-ink-dim hover:bg-bg-soft"
                  >
                    <CheckCircle2 size={12} /> {selected.read ? t.inbox.markUnread : t.inbox.markRead}
                  </button>
                </div>
                {toggleErrorId === selected.id && (
                  <div className="mt-3">
                    <StatusNotice state="failed" title={t.inbox.loadFailedTitle} hint={t.inbox.toggleFailedHint} />
                  </div>
                )}
              </div>
            ) : (
              <div className="h-full flex items-center justify-center text-ink-mute text-sm">
                <Bell size={32} className="mb-2" /> {t.inbox.emptyCategory}
              </div>
            )}
          </aside>
        </div>
      )}
    </div>
  );
}
