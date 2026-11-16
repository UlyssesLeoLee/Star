/*
@cypher schema=1 source_sha256=256ee3326e129b9d6995857dea2fc20bbc52a867842b923a634d8f5f992f7592
MERGE (self:File {path:"frontend/src/components/chat/DmCenterPanel.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/chat/DmCenterPanel.tsx::DmCenterPanel",kind:"component"})
MERGE (dmStore:Symbol {id:"frontend/src/lib/chat/dmStore.ts::useDmStore",kind:"hook"})
MERGE (self)-[:DEFINES]->(panel)
MERGE (self)-[:USES]->(dmStore)
@endcypher
*/
"use client";

// =====================================================================
// DmCenterPanel — 用户间即时通信中心 (DM + 群聊) 面板
//   1. 会话列表: 成员头像首字母 / 未读数 / 最后消息摘要 / 时间
//   2. 会话详情: 消息流 + 输入框 + 已读状态标
//   3. 新建会话弹窗: 选 1 人 → DM; 选多人 → 群聊
//   4. 顶部状态条: 实时已连接 / 实时未接入（本地模式）— 诚实显示
//   数据源: dmStore (useDmStore); 实时网关由 sibling imGateway.ts 提供
// =====================================================================

import { useEffect, useMemo, useRef, useState } from "react";
import { MessageSquare, Plus, Send, X } from "lucide-react";
import { clsx } from "clsx";
import {
  useDmStore,
  dmUserName,
  DM_KNOWN_USERS,
  type DmConversation,
} from "@/lib/chat/dmStore";

export interface DmCenterPanelProps {
  className?: string;
}

const pad = (n: number) => String(n).padStart(2, "0");

/** 时间显示: 当天 → HH:mm; 更早 → MM-DD */
function fmtTime(iso: string): string {
  const d = new Date(iso);
  const now = new Date();
  const sameDay = d.toDateString() === now.toDateString();
  return sameDay
    ? `${pad(d.getHours())}:${pad(d.getMinutes())}`
    : `${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

const initialOf = (name: string) => name.trim().slice(0, 1).toUpperCase() || "?";

const STATE_LABEL = { sent: "发送", delivered: "送达", read: "已读" } as const;

export function DmCenterPanel({ className }: DmCenterPanelProps) {
  const conversations = useDmStore((s) => s.conversations);
  const activeConversationId = useDmStore((s) => s.activeConversationId);
  const currentUserId = useDmStore((s) => s.currentUserId);
  const realtimeConnected = useDmStore((s) => s.realtimeConnected);
  const openOrCreateConversation = useDmStore((s) => s.openOrCreateConversation);
  const sendDm = useDmStore((s) => s.sendDm);
  const markConversationRead = useDmStore((s) => s.markConversationRead);
  const setActiveConversation = useDmStore((s) => s.setActiveConversation);
  const bindImGateway = useDmStore((s) => s.bindImGateway);
  const tryLoadImGateway = useDmStore((s) => s.tryLoadImGateway);

  const [draft, setDraft] = useState("");
  const [modalOpen, setModalOpen] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const [groupTitle, setGroupTitle] = useState("");
  const listRef = useRef<HTMLDivElement>(null);

  // 挂载时尝试加载 sibling imGateway.ts (模块未落地/不可达 → null → 本地模式)
  useEffect(() => {
    let cancelled = false;
    void tryLoadImGateway().then((gw) => {
      if (!cancelled && gw) bindImGateway(gw);
    });
    return () => {
      cancelled = true;
      bindImGateway(null);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const sorted = useMemo(
    () =>
      [...conversations].sort(
        (a, b) => new Date(b.lastActivityAt).getTime() - new Date(a.lastActivityAt).getTime(),
      ),
    [conversations],
  );

  const active = conversations.find((c) => c.id === activeConversationId) ?? null;

  // 新消息 → 滚动到底部
  useEffect(() => {
    // jsdom 无 Element.scrollTo → 可选调用守卫
    listRef.current?.scrollTo?.({ top: listRef.current.scrollHeight });
  }, [active?.messages.length, activeConversationId]);

  const displayTitle = (c: DmConversation): string => {
    if (c.title) return c.title;
    const peers = c.memberIds.filter((id) => id !== currentUserId);
    return peers.map(dmUserName).join("、") || "会话";
  };

  const peerOf = (c: DmConversation): string => {
    const peers = c.memberIds.filter((id) => id !== currentUserId);
    return peers[0] ?? c.memberIds[0] ?? "";
  };

  const lastMessage = (c: DmConversation) =>
    c.messages.length > 0 ? c.messages[c.messages.length - 1] : null;

  const selectConversation = (id: string) => {
    setActiveConversation(id);
    markConversationRead(id);
  };

  const send = () => {
    const text = draft.trim();
    if (!text || !active) return;
    sendDm(active.id, text, { id: currentUserId, name: dmUserName(currentUserId) });
    setDraft("");
  };

  const confirmNew = () => {
    if (selected.length === 0) return;
    const id = openOrCreateConversation(
      selected,
      selected.length >= 2 ? groupTitle : undefined,
    );
    if (id) {
      setModalOpen(false);
      setSelected([]);
      setGroupTitle("");
    }
  };

  const selectableUsers = DM_KNOWN_USERS.filter((u) => u.id !== currentUserId);

  const toggleUser = (id: string) =>
    setSelected((prev) => (prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id]));

  // 会话列表项头像 (DM: 对方首字母; 群聊: 前 3 名成员首字母叠加)
  const renderAvatars = (c: DmConversation) => {
    const ids =
      c.kind === "dm" ? [peerOf(c)] : c.memberIds.filter((id) => id !== currentUserId).slice(0, 3);
    return (
      <span className="flex shrink-0 -space-x-1.5">
        {ids.map((id, i) => (
          <span
            key={id}
            className={clsx(
              "size-6 rounded-full grid place-items-center text-[9px] font-bold border border-bg-card",
              i % 2 === 0 ? "bg-accent/15 text-accent" : "bg-ok/15 text-ok",
            )}
          >
            {initialOf(dmUserName(id))}
          </span>
        ))}
      </span>
    );
  };

  return (
    <div className={clsx("flex h-full flex-col bg-bg", className)}>
      {/* 顶部状态条 */}
      <header className="flex items-center gap-2 border-b border-line bg-bg-card px-3 py-2">
        <h2 className="flex items-center gap-1.5 text-xs font-bold text-ink">
          <MessageSquare size={13} />
          即时通信
        </h2>
        <span
          data-testid="dm-realtime-status"
          title={
            realtimeConnected
              ? "IM 网关已连接，消息实时送达"
              : "IM 网关未接入 — 消息仅本地暂存，对方不会实时收到"
          }
          className={clsx(
            "flex items-center gap-1.5 rounded-full border px-2 py-0.5 text-[10px] font-medium",
            realtimeConnected
              ? "border-ok/40 bg-ok/10 text-ok"
              : "border-warn/40 bg-warn/10 text-warn",
          )}
        >
          <span className={clsx("size-1.5 rounded-full", realtimeConnected ? "bg-ok" : "bg-warn")} />
          {realtimeConnected ? "实时已连接" : "实时未接入（本地模式）"}
        </span>
        <div className="flex-1" />
        <button
          type="button"
          data-testid="dm-new-btn"
          onClick={() => setModalOpen(true)}
          className="flex items-center gap-1 rounded-lg border border-line px-2 py-1 text-[11px] text-ink-dim hover:border-accent/50 hover:text-accent"
        >
          <Plus size={11} /> 新建会话
        </button>
      </header>

      <div className="flex min-h-0 flex-1">
        {/* 左: 会话列表 */}
        <aside
          data-testid="dm-conversations"
          className="w-64 shrink-0 overflow-y-auto border-r border-line bg-bg-card p-2"
        >
          {sorted.length === 0 ? (
            <p className="p-2 text-[11px] leading-relaxed text-ink-mute">
              暂无会话 — 点右上角「新建会话」与同事私聊或建群。
            </p>
          ) : (
            <ul className="space-y-0.5">
              {sorted.map((c) => {
                const last = lastMessage(c);
                const isActive = c.id === activeConversationId;
                return (
                  <li key={c.id}>
                    <button
                      type="button"
                      data-testid={`dm-conversation-${c.id}`}
                      onClick={() => selectConversation(c.id)}
                      className={clsx(
                        "flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left",
                        isActive ? "bg-accent/10" : "hover:bg-bg-soft/60",
                      )}
                    >
                      {renderAvatars(c)}
                      <span className="min-w-0 flex-1">
                        <span className="flex items-center gap-1">
                          <span
                            className={clsx(
                              "truncate text-xs",
                              isActive ? "font-semibold text-accent" : "font-semibold text-ink",
                            )}
                          >
                            {displayTitle(c)}
                          </span>
                          <span
                            className={clsx(
                              "shrink-0 rounded border px-1 text-[8px] leading-3",
                              c.kind === "group"
                                ? "border-accent/40 text-accent"
                                : "border-line text-ink-mute",
                            )}
                          >
                            {c.kind === "group" ? "群聊" : "私聊"}
                          </span>
                        </span>
                        <span className="block truncate text-[10px] text-ink-mute">
                          {last ? `${last.authorName}: ${last.text}` : "暂无消息"}
                        </span>
                      </span>
                      <span className="flex shrink-0 flex-col items-end gap-0.5">
                        <span className="font-mono text-[9px] text-ink-mute">
                          {fmtTime(c.lastActivityAt)}
                        </span>
                        {c.unread > 0 && (
                          <span
                            data-testid={`dm-unread-${c.id}`}
                            className="grid h-4 min-w-4 place-items-center rounded-full bg-err px-1 text-[9px] font-bold text-white"
                          >
                            {c.unread}
                          </span>
                        )}
                      </span>
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
        </aside>

        {/* 右: 会话详情 */}
        <section className="flex min-w-0 flex-1 flex-col bg-bg">
          <header className="border-b border-line bg-bg-card px-3 py-2">
            <div className="flex items-center justify-between gap-2">
              <h3 className="truncate text-xs font-bold text-ink" data-testid="dm-active-title">
                {active ? displayTitle(active) : "选择一个会话"}
              </h3>
              {active && (
                <span className="shrink-0 text-[9px] text-ink-mute">
                  {active.memberIds.length} 人
                </span>
              )}
            </div>
            {active && (
              <p className="mt-0.5 text-[10px] text-ink-mute">
                {active.memberIds
                  .map((id) => (id === currentUserId ? "我" : dmUserName(id)))
                  .join("、")}{" "}
                · {active.kind === "group" ? "群聊" : "私聊"} ·{" "}
                {active.realtimeConnected ? "实时已连接" : "本地模式"}
              </p>
            )}
          </header>

          {/* 消息流 */}
          <div ref={listRef} data-testid="dm-messages" className="flex-1 overflow-y-auto p-3">
            {!active ? (
              <div className="flex h-full items-center justify-center text-[11px] text-ink-mute">
                从左侧选择一个会话，或新建一个
              </div>
            ) : active.messages.length === 0 ? (
              <div className="flex h-full items-center justify-center text-[11px] text-ink-mute">
                暂无消息 — 在底部输入第一条
              </div>
            ) : (
              <div className="space-y-2.5">
                {active.messages.map((m) => {
                  const isOwn = m.authorId === currentUserId;
                  return (
                    <div
                      key={m.id}
                      data-testid={`dm-message-${m.id}`}
                      className={clsx("flex gap-2", isOwn && "flex-row-reverse")}
                    >
                      <span
                        className={clsx(
                          "mt-0.5 grid size-6 shrink-0 place-items-center rounded-full text-[9px] font-bold",
                          isOwn ? "bg-accent text-white" : "bg-ok/15 text-ok",
                        )}
                      >
                        {initialOf(m.authorName)}
                      </span>
                      <div className={clsx("max-w-[80%]", isOwn && "text-right")}>
                        <div className="mb-0.5 font-mono text-[9px] text-ink-mute">
                          {isOwn ? "我" : m.authorName} · {m.at.slice(11, 16)}
                        </div>
                        <div
                          className={clsx(
                            "inline-block rounded-xl border px-2.5 py-1.5 text-xs",
                            isOwn
                              ? "rounded-br-sm border-accent bg-accent text-white"
                              : "rounded-bl-sm border-line bg-bg-card text-ink",
                          )}
                        >
                          {m.text}
                        </div>
                        {/* 已读状态标 (IM1.0) */}
                        <div className="mt-0.5 space-x-1.5 text-[9px] text-ink-mute">
                          {isOwn && (
                            <span data-testid={`dm-msg-state-${m.id}`}>
                              {STATE_LABEL[m.state]}
                            </span>
                          )}
                          {active.kind === "group" && m.readBy.length > 1 && (
                            <span data-testid={`dm-msg-readby-${m.id}`} className="text-accent/80">
                              {m.readBy.length - 1} 人已读
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>

          {/* 输入区 */}
          <footer className="border-t border-line bg-bg-card p-2.5">
            <div className="flex items-end gap-1.5">
              <input
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    send();
                  }
                }}
                placeholder={
                  active
                    ? realtimeConnected
                      ? "输入消息…"
                      : "输入消息…（本地模式，对方不会实时收到）"
                    : "先选择一个会话"
                }
                disabled={!active}
                data-testid="dm-send-input"
                className="flex-1 rounded-lg border border-line bg-bg-soft px-2.5 py-1.5 text-xs focus:border-accent focus:outline-none disabled:opacity-50"
              />
              <button
                type="button"
                onClick={send}
                disabled={!active || !draft.trim()}
                data-testid="dm-send-btn"
                className="flex shrink-0 items-center gap-1 rounded-lg bg-accent px-2.5 py-2 text-[11px] text-white hover:bg-accent-soft disabled:opacity-50"
              >
                <Send size={11} /> 发送
              </button>
            </div>
            {!realtimeConnected && (
              <p className="mt-1.5 text-[9px] leading-relaxed text-warn">
                IM 网关未接入 — 消息仅本地暂存，对方不会实时收到。
              </p>
            )}
          </footer>
        </section>
      </div>

      {/* 新建会话弹窗 */}
      {modalOpen && (
        <div
          className="fixed inset-0 z-50 grid place-items-center bg-black/40"
          data-testid="dm-new-modal"
          onClick={() => setModalOpen(false)}
        >
          <div
            className="w-80 rounded-xl border border-line bg-bg-card p-3 shadow-xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between">
              <h3 className="text-xs font-bold text-ink">新建会话</h3>
              <button
                type="button"
                onClick={() => setModalOpen(false)}
                className="rounded p-0.5 text-ink-mute hover:text-err"
                aria-label="关闭"
              >
                <X size={12} />
              </button>
            </div>
            <p className="mt-1 text-[10px] text-ink-mute">选 1 人 = 私聊 (DM) · 选多人 = 群聊</p>
            <ul className="mt-2 max-h-56 space-y-0.5 overflow-y-auto">
              {selectableUsers.map((u) => (
                <li key={u.id}>
                  <label className="flex cursor-pointer items-center gap-2 rounded-lg px-2 py-1.5 text-xs hover:bg-bg-soft/60">
                    <input
                      type="checkbox"
                      data-testid={`dm-new-user-${u.id}`}
                      checked={selected.includes(u.id)}
                      onChange={() => toggleUser(u.id)}
                    />
                    <span className="grid size-5 place-items-center rounded-full bg-accent/15 text-[8px] font-bold text-accent">
                      {initialOf(u.name)}
                    </span>
                    <span className="flex-1 truncate text-ink">{u.name}</span>
                    <span className="font-mono text-[9px] text-ink-mute">{u.id}</span>
                  </label>
                </li>
              ))}
            </ul>
            {selected.length >= 2 && (
              <input
                value={groupTitle}
                onChange={(e) => setGroupTitle(e.target.value)}
                placeholder="群聊名称（可选）"
                data-testid="dm-new-group-title"
                className="mt-2 w-full rounded-lg border border-line bg-bg-soft px-2 py-1.5 text-xs focus:border-accent focus:outline-none"
              />
            )}
            <div className="mt-3 flex justify-end gap-1.5">
              <button
                type="button"
                data-testid="dm-new-cancel"
                onClick={() => setModalOpen(false)}
                className="rounded-lg border border-line px-2.5 py-1 text-[11px] text-ink-dim hover:bg-bg-soft"
              >
                取消
              </button>
              <button
                type="button"
                data-testid="dm-new-confirm"
                onClick={confirmNew}
                disabled={selected.length === 0}
                className="rounded-lg bg-accent px-2.5 py-1 text-[11px] text-white hover:bg-accent-soft disabled:opacity-50"
              >
                {selected.length >= 2 ? "创建群聊" : "开始私聊"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default DmCenterPanel;
