/*
@cypher schema=1 source_sha256=bb084588e75ac5aa96ad703ea30885da7d05d210602f97c600883cf06c32c1dc
MERGE (self:File {path:"frontend/src/app/worktree/chat/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/worktree/chat/page.tsx::WorktreeChatPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
"use client";

// =====================================================================
// /worktree/chat — 聊天页 (per 2026-11-14 用户发令: 参考 Multica 做法)
//   顶栏 tabs: 任务卡聊天 / 全局聊天
//   任务卡聊天: 点击任务卡打开 (kanban → ?task=T-142); 话题列表 + 逐条回复
//   全局聊天: 会话/子会话树 (Agent 常见结构)
//   底部聊天栏: 回复当前话题; 切换"新话题"模式可在同一聊天页开新话题
// =====================================================================

import { Suspense, useEffect, useRef, useState } from "react";
import { useSearchParams } from "next/navigation";
import {
  MessageSquare,
  Send,
  Plus,
  KanbanSquare,
  ChevronDown,
  ChevronRight,
  User,
  Bot,
} from "lucide-react";
import { clsx } from "clsx";
import { useChatStore } from "@/lib/chat/chatStore";

type ChatTab = "task" | "global";

function ChatInner() {
  const searchParams = useSearchParams();
  const urlTaskId = searchParams?.get("task") ?? "";
  const urlTaskTitle = searchParams?.get("title") ?? "";

  const topics = useChatStore((s) => s.topics);
  const globalSessions = useChatStore((s) => s.globalSessions);
  const activeTopicId = useChatStore((s) => s.activeTopicId);
  const openTopicForTask = useChatStore((s) => s.openTopicForTask);
  const openTopicForSession = useChatStore((s) => s.openTopicForSession);
  const createSession = useChatStore((s) => s.createSession);
  const reply = useChatStore((s) => s.reply);
  const createTopic = useChatStore((s) => s.createTopic);
  const setActiveTopic = useChatStore((s) => s.setActiveTopic);

  const [tab, setTab] = useState<ChatTab>("task");
  const [draft, setDraft] = useState("");
  const [newTopicMode, setNewTopicMode] = useState(false);
  const [newTopicTitle, setNewTopicTitle] = useState("");
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const listRef = useRef<HTMLDivElement>(null);

  // URL ?task=T-142 → 打开任务卡聊天
  useEffect(() => {
    if (urlTaskId) {
      const id = openTopicForTask(urlTaskId, urlTaskTitle);
      setActiveTopic(id);
      setTab("task");
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [urlTaskId, urlTaskTitle]);

  const activeTopic = topics.find((t) => t.id === activeTopicId) ?? null;
  const taskTopics = topics.filter((t) => t.scope === "task");
  const globalTopics = topics.filter((t) => t.scope === "global");
  const topSessions = globalSessions.filter((s) => s.parentId === null);
  const childrenOf = (id: string) => globalSessions.filter((s) => s.parentId === id);

  // 滚动到底部
  useEffect(() => {
    listRef.current?.scrollTo({ top: listRef.current.scrollHeight });
  }, [activeTopic?.messages.length, activeTopicId]);

  const send = () => {
    const text = draft.trim();
    if (!text) return;
    if (newTopicMode) {
      if (activeTopic) {
        const id = createTopic(activeTopic.scope, activeTopic.refId, newTopicTitle.trim() || text.slice(0, 24));
        reply(id, "Ulysses", "human", text);
        setNewTopicMode(false);
        setNewTopicTitle("");
      }
    } else if (activeTopic) {
      reply(activeTopic.id, "Ulysses", "human", text);
    }
    setDraft("");
  };

  const toggleCollapse = (id: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const renderSession = (session: { id: string; title: string; parentId: string | null }) => {
    const kids = childrenOf(session.id);
    const isCollapsed = collapsed.has(session.id);
    const topic = globalTopics.find((t) => t.refId === session.id);
    const isActive = topic?.id === activeTopicId;
    return (
      <li key={session.id}>
        <div
          className={clsx(
            "flex items-center gap-1.5 px-2 py-1.5 rounded-lg text-xs cursor-pointer",
            isActive ? "bg-accent/10 text-accent font-semibold" : "text-ink-dim hover:bg-bg-soft/60",
          )}
          onClick={() => {
            const tid = openTopicForSession(session.id);
            setActiveTopic(tid);
          }}
        >
          {kids.length > 0 ? (
            <button
              type="button"
              onClick={(e) => { e.stopPropagation(); toggleCollapse(session.id); }}
              className="shrink-0 text-ink-mute"
              aria-label="展开/折叠"
            >
              {isCollapsed ? <ChevronRight size={11} /> : <ChevronDown size={11} />}
            </button>
          ) : (
            <span className="w-[11px] shrink-0" />
          )}
          <MessageSquare size={11} className="shrink-0" />
          <span className="flex-1 truncate">{session.title}</span>
          {topic && topic.messages.length > 0 && (
            <span className="text-[9px] font-mono text-ink-mute">{topic.messages.length}</span>
          )}
        </div>
        {kids.length > 0 && !isCollapsed && (
          <ul className="ml-4 border-l border-line/60 pl-2 space-y-0.5">
            {kids.map(renderSession)}
          </ul>
        )}
      </li>
    );
  };

  return (
    <div className="flex h-full flex-col bg-bg" data-testid="chat-page">
      {/* 顶端 tabs: 任务卡聊天 / 全局聊天 */}
      <div data-testid="chat-tabs" className="flex items-center gap-1 border-b border-line bg-bg-card px-3">
        <button
          type="button"
          onClick={() => setTab("task")}
          data-testid="chat-tab-task"
          className={clsx(
            "px-3 py-2 text-sm border-b-2 -mb-px transition-colors flex items-center gap-1.5",
            tab === "task" ? "border-accent text-accent font-semibold" : "border-transparent text-ink-dim hover:text-ink",
          )}
        >
          <KanbanSquare size={13} /> 任务卡聊天
          {taskTopics.length > 0 && <span className="text-[9px] font-mono">{taskTopics.length}</span>}
        </button>
        <button
          type="button"
          onClick={() => setTab("global")}
          data-testid="chat-tab-global"
          className={clsx(
            "px-3 py-2 text-sm border-b-2 -mb-px transition-colors flex items-center gap-1.5",
            tab === "global" ? "border-accent text-accent font-semibold" : "border-transparent text-ink-dim hover:text-ink",
          )}
        >
          <MessageSquare size={13} /> 全局聊天
        </button>
      </div>

      <div className="flex flex-1 min-h-0">
        {/* 左: 话题列表 */}
        <aside data-testid="chat-topic-list" className="w-64 shrink-0 border-r border-line bg-bg-card overflow-y-auto p-2">
          {tab === "task" ? (
            taskTopics.length === 0 ? (
              <p className="text-[11px] text-ink-mute p-2">暂无任务卡话题 — 在任务卡上点「聊天」打开。</p>
            ) : (
              <ul className="space-y-0.5">
                {taskTopics.map((t) => (
                  <li key={t.id}>
                    <div
                      data-testid={`chat-topic-${t.id}`}
                      className={clsx(
                        "flex items-center gap-2 px-2 py-1.5 rounded-lg text-xs cursor-pointer",
                        t.id === activeTopicId ? "bg-accent/10 text-accent font-semibold" : "text-ink-dim hover:bg-bg-soft/60",
                      )}
                      onClick={() => setActiveTopic(t.id)}
                    >
                      <KanbanSquare size={11} className="shrink-0" />
                      <span className="flex-1 truncate">{t.refId} · {t.title}</span>
                      <span className="text-[9px] font-mono text-ink-mute">{t.messages.length}</span>
                    </div>
                  </li>
                ))}
              </ul>
            )
          ) : (
            <>
              <div className="flex items-center justify-between px-2 py-1.5">
                <span className="text-[10px] font-mono uppercase tracking-wider text-ink-mute">会话 / 子会话</span>
                <button
                  type="button"
                  data-testid="chat-new-session"
                  onClick={() => {
                    const sid = createSession("新会话", null);
                    const tid = openTopicForSession(sid);
                    setActiveTopic(tid);
                  }}
                  className="p-1 rounded hover:bg-bg-soft text-ink-mute hover:text-accent"
                  aria-label="新建会话"
                >
                  <Plus size={12} />
                </button>
              </div>
              <ul className="space-y-0.5">
                {topSessions.map(renderSession)}
              </ul>
            </>
          )}
        </aside>

        {/* 中: 消息区 */}
        <section className="flex-1 min-w-0 flex flex-col bg-bg">
          <header className="px-4 py-2.5 border-b border-line bg-bg-card flex items-center gap-2">
            <div className="flex-1 min-w-0">
              <h2 className="text-sm font-bold text-ink truncate" data-testid="chat-topic-title">
                {activeTopic
                  ? activeTopic.scope === "task"
                    ? `${activeTopic.refId} · ${activeTopic.title}`
                    : activeTopic.title
                  : "选择一个话题"}
              </h2>
              {activeTopic && (
                <p className="text-[10px] text-ink-mute">
                  {activeTopic.scope === "task" ? "任务卡聊天 · 每个话题可单独回复" : "全局聊天 · 会话/子会话"}
                </p>
              )}
            </div>
          </header>

          <div ref={listRef} data-testid="chat-messages" className="flex-1 overflow-y-auto p-4 space-y-3">
            {!activeTopic ? (
              <div className="h-full flex items-center justify-center text-ink-mute text-xs">
                点击左侧话题开始聊天，或从任务卡进入
              </div>
            ) : activeTopic.messages.length === 0 ? (
              <div className="h-full flex items-center justify-center text-ink-mute text-xs">
                暂无消息 — 在底部输入第一条
              </div>
            ) : (
              activeTopic.messages.map((m) => (
                <div
                  key={m.id}
                  data-testid={`chat-msg-${m.id}`}
                  className={clsx("flex gap-2.5", m.authorKind === "human" && "flex-row-reverse")}
                >
                  <span
                    className={clsx(
                      "size-7 rounded-full grid place-items-center text-[10px] font-bold shrink-0 mt-0.5",
                      m.authorKind === "agent" ? "bg-accent/15 text-accent" : m.authorKind === "system" ? "bg-bg-soft text-ink-mute" : "bg-ok/15 text-ok",
                    )}
                  >
                    {m.authorKind === "agent" ? <Bot size={12} /> : m.authorKind === "system" ? "S" : <User size={12} />}
                  </span>
                  <div className={clsx("max-w-[70%]", m.authorKind === "human" && "text-right")}>
                    <div className={clsx("text-[10px] font-mono mb-0.5", m.authorKind === "human" ? "text-ok" : m.authorKind === "agent" ? "text-accent" : "text-ink-mute")}>
                      {m.author} · {m.at.slice(11, 16)}
                    </div>
                    <div
                      className={clsx(
                        "inline-block text-sm rounded-xl px-3 py-2 border",
                        m.authorKind === "human"
                          ? "bg-accent text-white border-accent rounded-br-sm"
                          : m.authorKind === "agent"
                            ? "bg-bg-card border-line rounded-bl-sm text-ink"
                            : "bg-bg-soft border-line/60 text-ink-dim text-xs",
                      )}
                    >
                      {m.text}
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>

          {/* 底部聊天栏: 回复当前话题 / 开新话题 */}
          <footer data-testid="chat-input-bar" className="border-t border-line bg-bg-card p-3">
            {newTopicMode && (
              <div className="mb-2 flex items-center gap-2">
                <input
                  value={newTopicTitle}
                  onChange={(e) => setNewTopicTitle(e.target.value)}
                  placeholder="新话题标题 (留空则取首句)"
                  data-testid="chat-new-topic-title"
                  className="flex-1 rounded-lg border border-line bg-bg-soft px-2.5 py-1.5 text-xs"
                />
                <button
                  type="button"
                  onClick={() => setNewTopicMode(false)}
                  className="text-xs px-2 py-1.5 rounded-lg border border-line text-ink-dim"
                >
                  取消
                </button>
              </div>
            )}
            <div className="flex items-end gap-2">
              <button
                type="button"
                onClick={() => setNewTopicMode((v) => !v)}
                data-testid="chat-new-topic-toggle"
                className={clsx(
                  "flex items-center gap-1 text-xs px-2.5 py-1.5 rounded-lg border shrink-0",
                  newTopicMode ? "border-accent bg-accent/10 text-accent" : "border-line text-ink-dim hover:border-accent/50",
                )}
              >
                <Plus size={12} /> 新话题
              </button>
              <textarea
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault();
                    send();
                  }
                }}
                rows={2}
                placeholder={newTopicMode ? "输入新话题的第一条消息…" : "输入回复… (Enter 发送, Shift+Enter 换行)"}
                data-testid="chat-input"
                className="flex-1 rounded-lg border border-line bg-bg-soft px-3 py-2 text-sm resize-none focus:border-accent focus:outline-none"
              />
              <button
                type="button"
                onClick={send}
                disabled={!draft.trim()}
                data-testid="chat-send"
                className="flex items-center gap-1.5 text-xs px-3.5 py-2 rounded-lg bg-accent text-white hover:bg-accent-soft disabled:opacity-50 shrink-0"
              >
                <Send size={12} /> 发送
              </button>
            </div>
          </footer>
        </section>
      </div>
    </div>
  );
}

export default function WorktreeChatPage() {
  return (
    <Suspense fallback={null}>
      <ChatInner />
    </Suspense>
  );
}
