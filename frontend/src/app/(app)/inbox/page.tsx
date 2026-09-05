"use client";

// =====================================================================
// /inbox — Tactical Speech Comms & Transmission Feed (战术通信与指令流)
// =====================================================================
// 全面应用 3渲2 (3D-to-2D NPR Cel-Shaded) 艺术级日漫科技美学:
//   1. Tactical Speech Bubble 漫画对白气泡架构
//   2. 3D Cel-Beacon 状态指示器与阶梯墨边
//   3. 独立双主题自适应与低认知负荷排版
// =====================================================================

import { useEffect, useState } from "react";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Inbox, Bell, Radio, CheckCircle, ShieldAlert } from "lucide-react";
import { MOCK_NOTIFS_FALLBACK } from "@/mocks/data";
import type { MockNotif } from "@/mocks/schemas/inbox";
import { useTranslation } from "@/lib/i18n";
import { CelBeacon3D } from "@/components/effects/Cel3DUI";

export default function InboxPage() {
  const { t } = useTranslation();
  const [notifs, setNotifs] = useState<ReadonlyArray<MockNotif>>(MOCK_NOTIFS_FALLBACK);

  // local-only read/unread toggle
  const [readSet, setReadSet] = useState<Set<string>>(
    () => new Set(MOCK_NOTIFS_FALLBACK.filter((n) => n.read).map((n) => n.id)),
  );

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

  const toggle = (id: string) => {
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
  const unread = notifs.filter((n) => !readSet.has(n.id)).length;

  return (
    <div className="max-w-5xl mx-auto space-y-6 pb-6" data-testid="inbox-page">
      <PageHeader
        title={t.pageTitles['/inbox'].title}
        subtitle="Tactical Comms Stream & Neural Audit Feed (战术通信与指令审计流)"
        icon={<Inbox className="text-[var(--cel-cyan,#00f0ff)]" size={22} />}
        count={`${unread} unread`}
      />

      {/* Tactical Speech Bubble Briefing Card (from chrono-vibe) */}
      <div className="card relative overflow-hidden clip-hud-corner">
        <div className="flex items-center justify-between border-b-2 border-black pb-2.5 mb-3.5">
          <h3 className="text-sm font-black uppercase tracking-wider text-[var(--cel-text-primary,#ffffff)] flex items-center gap-2">
            <Radio size={15} className="text-[var(--cel-crimson,#ff184c)] animate-pulse" />
            <span>TACTICAL TRANSMISSION // CHANNEL 01</span>
          </h3>
          <span className="text-xs font-mono font-bold bg-black text-[var(--cel-gold,#ffc400)] px-2.5 py-0.5 border border-black">
            HIGH PRIORITY
          </span>
        </div>

        <div className="border-2 border-black p-4 bg-[var(--cel-surface-stage,#090d16)] cel-shadow relative">
          <p className="text-sm font-medium leading-relaxed text-[var(--cel-text-primary,#ffffff)]">
            <strong className="text-[var(--cel-crimson,#ff184c)] font-black mr-2">[MAVIS // 統制官]:</strong>
            「全站 3渲2 视觉构架已接入中央通信信道。所有领域状态流转均以高对比度墨线阶梯卡片呈现，零脏状态逃逸，随时执行战术派发！」
          </p>
        </div>

        <div className="flex items-center justify-between text-xs font-mono text-[var(--cel-text-secondary,#94a3b8)] pt-3">
          <span className="flex items-center gap-1.5">
            <span className="size-2 bg-emerald-400 border border-black rotate-45" />
            COGNITIVE LOAD: 0.12 (OPTIMAL)
          </span>
          <span className="text-[var(--cel-cyan,#00f0ff)] font-bold">CHARISMA: MAXIMAL</span>
        </div>
      </div>

      {/* Notifications Inked Feed */}
      <div className="card">
        <SectionTitle
          action={
            <span className="text-xs font-mono font-bold text-[var(--cel-gold,#ffc400)]">
              {unread} UNREAD / {notifs.length} TOTAL
            </span>
          }
        >
          Transmissions 〔受信通知一覧〕
        </SectionTitle>

        <ul className="space-y-3" data-testid="inbox-list">
          {notifs.map((n, idx) => {
            const isRead = readSet.has(n.id);
            return (
              <li
                key={n.id}
                data-testid={`inbox-item-${n.id}`}
                className={`p-4 border-2 border-black transition-all flex items-start gap-3.5 ${
                  isRead
                    ? "bg-[var(--cel-surface-stage,#090d16)] opacity-75"
                    : "bg-[var(--cel-surface-sub,#151c2c)] cel-shadow hover:-translate-x-0.5 hover:-translate-y-0.5"
                }`}
              >
                <div className="shrink-0 mt-1">
                  <CelBeacon3D
                    status={isRead ? "idle" : idx % 2 === 0 ? "active" : "success"}
                    size={28}
                    title={isRead ? "Read" : "Unread Transmission"}
                  />
                </div>

                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap mb-1">
                    <span className="font-mono text-xs font-bold text-[var(--cel-crimson,#ff184c)]">
                      [{String(idx + 1).padStart(2, "0")}]
                    </span>
                    <StatusPill value={isRead ? "read" : "pending"} size="xs" />
                    <span className="text-sm font-bold text-[var(--cel-text-primary,#ffffff)] truncate">
                      {n.subject}
                    </span>
                    <span className="font-mono text-xs text-[var(--cel-text-secondary,#94a3b8)] ml-auto shrink-0 bg-black/40 px-2 py-0.5 border border-black">
                      {n.ago}
                    </span>
                  </div>

                  <div className="text-sm text-[var(--cel-text-secondary,#94a3b8)] leading-relaxed mt-1.5">
                    {n.body}
                  </div>
                </div>

                <button
                  type="button"
                  onClick={() => toggle(n.id)}
                  aria-label={isRead ? "mark unread" : "mark read"}
                  className="shrink-0 text-xs font-mono font-bold px-3 py-1.5 min-h-[32px] border-2 border-black cel-shadow bg-[var(--cel-surface-stage,#090d16)] hover:bg-[var(--cel-surface-card,#0f1422)] text-[var(--cel-text-primary,#ffffff)] transition-colors"
                >
                  {isRead ? "UNREAD" : "READ"}
                </button>
              </li>
            );
          })}
        </ul>
      </div>

      {/* P3 Notice */}
      <div className="card text-sm text-[var(--cel-text-secondary,#94a3b8)]">
        <SectionTitle>Notification Service — P3 规范</SectionTitle>
        <ul className="space-y-1.5 list-disc pl-4">
          <li>当前 read/unread 本地持久化，与 <span className="font-mono text-[var(--cel-cyan,#00f0ff)]">/api/notifications</span> PATCH 校验联动</li>
          <li>实时 SSE 广播流对接 Phase I+ 战术信道</li>
          <li>全要素遵循 3渲2 日漫科技美学与米勒定律三区信息聚合</li>
        </ul>
      </div>
    </div>
  );
}
