/*
@cypher schema=1 source_sha256=e0720cb6a11648cb3036f011115d8aada099271e09f19311b7909b85a494f78f
MERGE (self:File {path:"frontend/src/components/chat/SideChatPanel.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/chat/SideChatPanel.tsx::SideChatPanel",kind:"component"})
MERGE (statusNotice:Symbol {id:"frontend/src/components/status/StatusNotice.tsx::StatusNotice",kind:"component"})
MERGE (chatStore:Symbol {id:"frontend/src/lib/chat/chatStore.ts::useChatStore",kind:"hook"})
MERGE (appStore:Symbol {id:"frontend/src/lib/store.ts::useStore",kind:"hook"})
MERGE (self)-[:DEFINES]->(panel)
MERGE (self)-[:IMPORTS]->(statusNotice)
MERGE (self)-[:USES]->(chatStore)
MERGE (self)-[:IMPORTS]->(appStore)
@endcypher
*/
"use client";

// =====================================================================
// SideChatPanel — 右侧栏聊天面板 (per 2026-11-14 用户发令: 右侧可折叠聊天栏, 所有界面通用)
// 从 /worktree/chat 提取, 供 RightChatRail(全局右侧栏) + chat 页共用
//   - 任务卡聊天: 话题逐条回复; 底部聊天栏可回复当前话题或开新话题
//   - 全局聊天: 会话/子会话树
// =====================================================================

import { useEffect, useRef, useState } from "react";
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
import { useWorktreeGroupSessionState } from "@/lib/group/groupProjection";
import { useStore } from "@/lib/store";
import { useTranslation } from "@/lib/i18n";
import { StatusNotice } from "@/components/status/StatusNotice";

type ChatTab = "task" | "global";

export interface SideChatPanelProps {
  /** 紧凑模式 (右侧栏 w-80): 隐藏左侧话题列表, 只显示消息流 */
  compact?: boolean;
  /** 任务卡 id (?task= 深链) */
  taskId?: string;
  taskTitle?: string;
}

export function SideChatPanel({ compact = false, taskId, taskTitle }: SideChatPanelProps) {
  const { t } = useTranslation();
  const topics = useChatStore((s) => s.topics);
  const globalSessions = useChatStore((s) => s.globalSessions);
  const activeTopicId = useChatStore((s) => s.activeTopicId);
  const chatScope = useChatStore((s) => s.chatScope);
  const globalAuthTargetId = useChatStore((s) => s.globalAuthTargetId);
  const setGlobalAuthTarget = useChatStore((s) => s.setGlobalAuthTarget);
  const openTopicForTask = useChatStore((s) => s.openTopicForTask);
  const openTopicForSession = useChatStore((s) => s.openTopicForSession);
  const createSession = useChatStore((s) => s.createSession);
  const reply = useChatStore((s) => s.reply);
  // IM1.0 对齐: 引用回复 (reply_to) — 点击消息"引用"后回复它
  const [replyingTo, setReplyingTo] = useState<{ id: string; preview: string } | null>(null);
  const createTopic = useChatStore((s) => s.createTopic);
  const setActiveTopic = useChatStore((s) => s.setActiveTopic);
  // GLOBAL 授权目标候选 = 当前账号可见的 worktree 列表 (本地 seed; 真实授权由服务端校验)
  const worktrees = useStore((s) => s.worktrees);

  const [tab, setTab] = useState<ChatTab>("task");
  const [draft, setDraft] = useState("");
  const [newTopicMode, setNewTopicMode] = useState(false);
  const [newTopicTitle, setNewTopicTitle] = useState("");
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [scopeError, setScopeError] = useState<string | null>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // ?task= 深链 → 打开任务卡聊天
  useEffect(() => {
    if (taskId) {
      const id = openTopicForTask(taskId, taskTitle ?? "");
      setActiveTopic(id);
      setTab("task");
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [taskId, taskTitle]);

  // 切换范围时清除上一次的授权校验错误提示
  useEffect(() => {
    setScopeError(null);
  }, [chatScope]);

  const activeTopic = topics.find((t) => t.id === activeTopicId) ?? null;
  const taskTopics = topics.filter((t) => t.scope === "task");
  const globalTopics = topics.filter((t) => t.scope === "global");
  const topSessions = globalSessions.filter((s) => s.parentId === null);
  const childrenOf = (id: string) => globalSessions.filter((s) => s.parentId === id);

  // GLOBAL 授权目标校验 (提交时还会重新校验一次, 见 send)
  const authTarget = worktrees.find((w) => w.id === globalAuthTargetId) ?? null;
  const authTargetValid = chatScope !== "GLOBAL" || authTarget != null;

  useEffect(() => {
    listRef.current?.scrollTo({ top: listRef.current.scrollHeight });
  }, [activeTopic?.messages.length, activeTopicId]);

  // 聊天后端接入状态 (宿主 Group API 会话; 未接入 = 不写入本地消息冒充真实记录)
  const groupSession = useWorktreeGroupSessionState();
  const backendConnected = groupSession.status === "ready";

  const send = () => {
    const text = draft.trim();
    if (!text) return;
    if (!backendConnected) {
      setScopeError("聊天后端尚未接入 — 只读模式, 消息不会写入 (接入后启用发送)。");
      return;
    }
    // GLOBAL 范围: 提交前重新校验授权目标 (无真实后端, 校验=目标已选且非空且仍存在)。
    // 校验失败 → 提示并保留草稿, 不写入任何消息。
    if (chatScope === "GLOBAL") {
      if (globalAuthTargetId == null || globalAuthTargetId.trim() === "") {
        setScopeError(t.chatScope["error.noTarget"]);
        return;
      }
      if (!worktrees.some((w) => w.id === globalAuthTargetId)) {
        setScopeError(t.chatScope["error.staleTarget"]);
        return;
      }
    }
    setScopeError(null);
    // Physis 主题联动: 发消息触发粒子爆发 (per 2026-11-14 游戏化设计)
    if (typeof window !== "undefined") {
      window.dispatchEvent(new CustomEvent("physis-burst"));
    }
    if (newTopicMode) {
      if (activeTopic) {
        const id = createTopic(activeTopic.scope, activeTopic.refId, newTopicTitle.trim() || text.slice(0, 24));
        reply(id, "Ulysses", "human", text);
        setNewTopicMode(false);
        setNewTopicTitle("");
      }
    } else if (activeTopic) {
      reply(activeTopic.id, "Ulysses", "human", text, replyingTo?.id);
      setReplyingTo(null);
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
    <div className="flex h-full flex-col bg-bg" data-testid="chat-panel">
      {/* 顶端 tabs */}
      <div data-testid="chat-tabs" className="flex items-center gap-1 border-b border-line bg-bg-card px-2">
        <button
          type="button"
          onClick={() => setTab("task")}
          data-testid="chat-tab-task"
          className={clsx(
            "px-2.5 py-2 text-xs border-b-2 -mb-px transition-colors flex items-center gap-1",
            tab === "task" ? "border-accent text-accent font-semibold" : "border-transparent text-ink-dim hover:text-ink",
          )}
        >
          <KanbanSquare size={12} /> 任务
        </button>
        <button
          type="button"
          onClick={() => setTab("global")}
          data-testid="chat-tab-global"
          className={clsx(
            "px-2.5 py-2 text-xs border-b-2 -mb-px transition-colors flex items-center gap-1",
            tab === "global" ? "border-accent text-accent font-semibold" : "border-transparent text-ink-dim hover:text-ink",
          )}
        >
          <MessageSquare size={12} /> 全局
        </button>
      </div>

      <div className="flex flex-1 min-h-0">
        {/* 左: 话题列表 (compact 模式隐藏) */}
        {!compact && (
          <aside data-testid="chat-topic-list" className="w-56 shrink-0 border-r border-line bg-bg-card overflow-y-auto p-2">
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
                  <span className="text-[10px] font-mono uppercase tracking-wider text-ink-mute">会话</span>
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
        )}

        {/* 消息区 */}
        <section className="flex-1 min-w-0 flex flex-col bg-bg">
          <header className="px-3 py-2 border-b border-line bg-bg-card">
            <div className="flex items-center justify-between gap-2">
              <h2 className="text-xs font-bold text-ink truncate" data-testid="chat-topic-title">
                {activeTopic
                  ? activeTopic.scope === "task"
                    ? `${activeTopic.refId} · ${activeTopic.title}`
                    : activeTopic.title
                  : "选择一个话题"}
              </h2>
              {/* 未接入状态 (per 2026-11-15 任务: 聊天后端未接入 → 显示明确状态并禁用相关能力;
                  内部渲染统一收敛到 StatusNotice disconnected 态, 外层保留 chat-backend-status testid) */}
              <span
                data-testid="chat-backend-status"
                title="聊天后端尚未接入，消息仅本地暂存"
                className="shrink-0"
              >
                <StatusNotice state="disconnected" compact title={t.chatScope.backendNotConnected} />
              </span>
            </div>
            {!compact && activeTopic && (
              <p className="text-[10px] text-ink-mute">
                {activeTopic.scope === "task" ? "任务卡聊天 · 每个话题可单独回复" : "全局聊天 · 会话/子会话"}
              </p>
            )}
          </header>

          <div ref={listRef} data-testid="chat-messages" className="flex-1 overflow-y-auto p-3 space-y-2.5">
            {!activeTopic ? (
              <div className="h-full flex items-center justify-center text-ink-mute text-[11px]">
                从任务卡「聊天」进入，或切「全局」选会话
              </div>
            ) : activeTopic.messages.length === 0 ? (
              <div className="h-full flex items-center justify-center text-ink-mute text-[11px]">
                暂无消息 — 在底部输入第一条
              </div>
            ) : (
              activeTopic.messages.map((m) => (
                <div
                  key={m.id}
                  data-testid={`chat-msg-${m.id}`}
                  className={clsx("flex gap-2", m.authorKind === "human" && "flex-row-reverse")}
                >
                  <span
                    className={clsx(
                      "size-6 rounded-full grid place-items-center text-[9px] font-bold shrink-0 mt-0.5",
                      m.authorKind === "agent" ? "bg-accent/15 text-accent" : m.authorKind === "system" ? "bg-bg-soft text-ink-mute" : "bg-ok/15 text-ok",
                    )}
                  >
                    {m.authorKind === "agent" ? <Bot size={11} /> : m.authorKind === "system" ? "S" : <User size={11} />}
                  </span>
                  <div className={clsx("max-w-[80%]", m.authorKind === "human" && "text-right")}>
                    <div className={clsx("text-[9px] font-mono mb-0.5", m.authorKind === "human" ? "text-ok" : m.authorKind === "agent" ? "text-accent" : "text-ink-mute")}>
                      {m.author} · {m.at.slice(11, 16)}
                    </div>
                    <div
                      className={clsx(
                        "inline-block text-xs rounded-xl px-2.5 py-1.5 border",
                        m.authorKind === "human"
                          ? "bg-accent text-white border-accent rounded-br-sm"
                          : m.authorKind === "agent"
                            ? "bg-bg-card border-line rounded-bl-sm text-ink"
                            : "bg-bg-soft border-line/60 text-ink-dim text-[10px]",
                      )}
                    >
                      {m.replyTo && (() => {
                        const parent = activeTopic.messages.find((x) => x.id === m.replyTo);
                        return parent ? (
                          <div className="mb-1 rounded border-l-2 border-accent/60 bg-accent/10 px-1.5 py-0.5 text-[9px] text-ink-mute" data-testid={`chat-msg-quote-${m.id}`}>
                            <span className="font-mono text-accent">{parent.author}: </span>
                            {parent.text.slice(0, 40)}
                            {parent.text.length > 40 ? "…" : ""}
                          </div>
                        ) : null;
                      })()}
                      {m.text}
                      {m.authorKind === "human" && (
                        <span className="ml-1.5 text-[8px] opacity-70" data-testid={`chat-msg-state-${m.id}`}>
                          {m.state === "read" ? "已读" : m.state === "delivered" ? "送达" : "发送"}
                        </span>
                      )}
                    </div>
                    {m.authorKind !== "system" && (
                      <button
                        type="button"
                        onClick={() => setReplyingTo({ id: m.id, preview: m.text.slice(0, 40) })}
                        data-testid={`chat-msg-reply-btn-${m.id}`}
                        title="引用回复"
                        className="mt-0.5 text-[9px] text-ink-mute hover:text-accent"
                      >
                        引用
                      </button>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>

          {/* 底部聊天栏 */}
          <footer data-testid="chat-input-bar" className="border-t border-line bg-bg-card p-2.5">
            {replyingTo && (
              <div className="mb-1.5 flex items-center gap-2 rounded border border-accent/40 bg-accent/10 px-2 py-1 text-[10px]" data-testid="chat-replying-banner">
                <span className="text-accent">引用:</span>
                <span className="flex-1 truncate text-ink-dim">{replyingTo.preview}</span>
                <button type="button" onClick={() => setReplyingTo(null)} className="text-ink-mute hover:text-err">✕</button>
              </div>
            )}
            {/* GLOBAL 范围: 授权目标选择 (per 2026-11-15 任务: 未选目标禁用发送, 提交时重新校验) */}
            {chatScope === "GLOBAL" && (
              <div className="mb-2" data-testid="chat-scope-auth">
                <div className="mb-1 flex items-center justify-between">
                  <label
                    htmlFor="chat-auth-target"
                    className="text-[10px] font-semibold uppercase tracking-wider text-ink-mute"
                  >
                    {t.chatScope.authTarget}
                  </label>
                  <span className="text-[9px] text-ink-mute">
                    {authTarget ? `${authTarget.name} · ${authTarget.id}` : t.chatScope["authTarget.none"]}
                  </span>
                </div>
                <select
                  id="chat-auth-target"
                  value={globalAuthTargetId ?? ""}
                  onChange={(e) => {
                    setGlobalAuthTarget(e.target.value || null);
                    setScopeError(null);
                  }}
                  data-testid="chat-scope-auth-target"
                  className="w-full rounded-lg border border-line bg-bg-soft px-2 py-1.5 text-xs focus:border-accent focus:outline-none"
                >
                  <option value="">{t.chatScope["authTarget.placeholder"]}</option>
                  {worktrees.map((w) => (
                    <option key={w.id} value={w.id}>
                      {w.name} · {w.id}
                    </option>
                  ))}
                </select>
                <p className="mt-1 text-[9px] leading-relaxed text-ink-mute">
                  {t.chatScope["authTarget.hint"]}
                </p>
              </div>
            )}
            {scopeError && (
              <p
                role="alert"
                data-testid="chat-scope-auth-error"
                className="mb-1.5 rounded border border-err/30 bg-err/5 px-2 py-1 text-[10px] text-err"
              >
                {scopeError}
              </p>
            )}
            {newTopicMode && (
              <div className="mb-2 flex items-center gap-1.5">
                <input
                  value={newTopicTitle}
                  onChange={(e) => setNewTopicTitle(e.target.value)}
                  placeholder="新话题标题 (留空取首句)"
                  data-testid="chat-new-topic-title"
                  className="flex-1 rounded-lg border border-line bg-bg-soft px-2 py-1 text-[11px]"
                />
                <button
                  type="button"
                  onClick={() => setNewTopicMode(false)}
                  className="text-[11px] px-2 py-1 rounded-lg border border-line text-ink-dim"
                >
                  取消
                </button>
              </div>
            )}
            <div className="flex items-end gap-1.5">
              <button
                type="button"
                onClick={() => setNewTopicMode((v) => !v)}
                data-testid="chat-new-topic-toggle"
                className={clsx(
                  "flex items-center gap-1 text-[11px] px-2 py-1.5 rounded-lg border shrink-0",
                  newTopicMode ? "border-accent bg-accent/10 text-accent" : "border-line text-ink-dim hover:border-accent/50",
                )}
              >
                <Plus size={11} /> 话题
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
                placeholder={newTopicMode ? "新话题第一条消息…" : "输入回复…"}
                data-testid="chat-input"
                className="flex-1 rounded-lg border border-line bg-bg-soft px-2.5 py-1.5 text-xs resize-none focus:border-accent focus:outline-none"
              />
              <button
                type="button"
                onClick={send}
                disabled={!draft.trim() || !authTargetValid || !backendConnected}
                data-testid="chat-send"
                title={chatScope === "GLOBAL" && !authTargetValid ? t.chatScope["error.noTarget"] : undefined}
                className="flex items-center gap-1 text-[11px] px-2.5 py-2 rounded-lg bg-accent text-white hover:bg-accent-soft disabled:opacity-50 shrink-0"
              >
                <Send size={11} /> 发
              </button>
            </div>
            {/* 未接入状态说明 (per 2026-11-15 任务: 明确状态; 不生成示例消息冒充真实记录) */}
            <p
              data-testid="chat-scope-status"
              className="mt-1.5 text-[9px] leading-relaxed text-warn"
            >
              {chatScope === "GLOBAL"
                ? "聊天后端尚未接入，消息仅本地暂存；选择授权目标不授予权限，未选目标时发送禁用。"
                : "聊天后端尚未接入，消息仅本地暂存；当前不生成示例消息。"}
            </p>
          </footer>
        </section>
      </div>
    </div>
  );
}
