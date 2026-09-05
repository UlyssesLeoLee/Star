"use client";

// =====================================================================
// /inbox — 通知列表 (minimal placeholder)
// =====================================================================
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   1. notifications 全部 mock (10 条) — 真实 notification service 接入 P3
//   2. read/unread 仅本地 state, 不持久化, 不联动 store
//   3. 3 column layout (源 / 列表 / 详情) 简化为 1 column
//   4. 实时 SSE 推送 (Phase I+) P3
//   5. light mode (per §7) P3
//   6. useEffect+fetch 阶段用 MOCK_NOTIFS_FALLBACK SSR 兜底 (per mock-msw-handlers §2.4 + §4 #1 缺标)
// =====================================================================

import { useEffect, useState } from "react";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Inbox, Bell } from "lucide-react";
import { MOCK_NOTIFS_FALLBACK } from "@/mocks/data";
import type { MockNotif } from "@/mocks/schemas/inbox";
import { useTranslation } from "@/lib/i18n";

export default function InboxPage() {
  const { t } = useTranslation();
  const [notifs, setNotifs] = useState<ReadonlyArray<MockNotif>>(MOCK_NOTIFS_FALLBACK);

  // local-only read/unread toggle (per 缺标, 不联动 store)
  const [readSet, setReadSet] = useState<Set<string>>(
    () => new Set(MOCK_NOTIFS_FALLBACK.filter((n) => n.read).map((n) => n.id)),
  );

  useEffect(() => {
    fetch("/api/notifications")
      .then((r) => r.json())
      .then((data: ReadonlyArray<MockNotif>) => {
        setNotifs(data);
        // 同步 readSet (per §2.7, fetch 后用 server read state 重置本地)
        setReadSet(new Set(data.filter((n) => n.read).map((n) => n.id)));
      })
      .catch(() => {
        /* keep FALLBACK (per §4 #1 缺标) */
      });
  }, []);

  const toggle = (id: string) => {
    setReadSet((prev) => {
      const next = new Set(prev);
      const wasRead = next.has(id);
      if (wasRead) next.delete(id);
      else next.add(id);
      // PATCH 真实持久化 (per inbox.ts handler §2.2)
      fetch(`/api/notifications/${id}`, { method: "PATCH" }).catch(() => {
        /* P3 真实持久化待 Phase F+, 此处失败本地回滚 */
        if (wasRead) next.add(id);
        else next.delete(id);
      });
      return next;
    });
  };
  const unread = notifs.filter((n) => !readSet.has(n.id)).length;

  return (
    <div className="max-w-5xl mx-auto pb-4" data-testid="inbox-page">
      <PageHeader
        title={t.pageTitles['/inbox'].title}
        subtitle="notification / comment / audit (10 mock; 真实 notification service P3 缺口)"
        icon={<Inbox className="text-accent" size={20} />}
        count={`${unread} unread`}
      />

      <div className="card">
        <SectionTitle>Notifications (mock, local read state)</SectionTitle>
        <ul className="divide-y divide-line/40" data-testid="inbox-list">
          {notifs.map((n) => {
            const isRead = readSet.has(n.id);
            return (
              <li
                key={n.id}
                data-testid={`inbox-item-${n.id}`}
                className="py-2.5 flex items-start gap-3"
              >
                <button
                  type="button"
                  onClick={() => toggle(n.id)}
                  aria-label={isRead ? "mark unread" : "mark read"}
                  className="mt-1 shrink-0"
                >
                  <Bell
                    size={12}
                    className={isRead ? "text-ink-mute" : "text-accent fill-accent/30"}
                  />
                </button>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <StatusPill value={isRead ? "read" : "pending"} size="xs" />
                    <span className="text-sm text-ink truncate">{n.subject}</span>
                    <span className="font-mono text-[10px] text-ink-mute ml-auto shrink-0">
                      {n.ago}
                    </span>
                  </div>
                  <div className="text-xs text-ink-dim mt-0.5">{n.body}</div>
                </div>
              </li>
            );
          })}
        </ul>
      </div>

      <div className="card mt-3 text-xs text-ink-dim">
        <SectionTitle>Notification Service — P3 缺口</SectionTitle>
        <ul className="space-y-1.5 list-disc pl-4 text-ink-dim">
          <li>当前 read/unread 仅本地 state, 刷新页面丢失, 不联动 <span className="font-mono text-ink-mute">useStore.notifications</span></li>
          <li>真实 <span className="font-mono text-ink-mute">/api/notifications</span> 接入待 Phase I+</li>
          <li>3 column layout (源 / 列表 / 详情) 简化为 1 column, 详情侧栏 P2</li>
          <li>实时 SSE 推送 (Phase I+) P3</li>
        </ul>
      </div>
    </div>
  );
}
