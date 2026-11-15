/*
@cypher schema=1 source_sha256=5e43c781adbf9ce45ae402ca18cae1de453a8b40c8933579e35bcfb7d680324f
MERGE (self:File {path:"frontend/src/components/chat/RightChatRail.tsx"})
MERGE (rail:Symbol {id:"frontend/src/components/chat/RightChatRail.tsx::RightChatRail",kind:"component"})
MERGE (panel:Symbol {id:"frontend/src/components/chat/SideChatPanel.tsx::SideChatPanel",kind:"component"})
MERGE (self)-[:DEFINES]->(rail)
MERGE (self)-[:IMPORTS]->(panel)
@endcypher
*/
"use client";

// =====================================================================
// RightChatRail — 全局右侧可折叠聊天栏 (per 2026-11-14 用户发令:
//   "右侧的可折叠聊天栏位……所有界面都要")
//   折叠态: 56px 边条 (图标 + 未读徽标 + 展开按钮)
//   展开态: w-80 聊天面板 (任务卡聊天/全局聊天, 复用 chatStore)
//   状态持久化 localStorage
// =====================================================================

import { useEffect, useState } from "react";
import { MessageSquare, PanelRightClose, PanelRightOpen } from "lucide-react";
import { clsx } from "clsx";
import { SideChatPanel } from "./SideChatPanel";
import { useChatStore } from "@/lib/chat/chatStore";
import { useTranslation } from "@/lib/i18n";

export function RightChatRail() {
  const { t } = useTranslation();
  const topics = useChatStore((s) => s.topics);
  const chatScope = useChatStore((s) => s.chatScope);
  const setChatScope = useChatStore((s) => s.setChatScope);
  const [open, setOpen] = useState(false);

  // 持久化展开状态
  useEffect(() => {
    try {
      const saved = localStorage.getItem("star-chat-rail-open");
      if (saved !== null) setOpen(saved === "1");
    } catch {
      /* ignore */
    }
  }, []);
  const toggle = () => {
    setOpen((v) => {
      const next = !v;
      try {
        localStorage.setItem("star-chat-rail-open", next ? "1" : "0");
      } catch {
        /* ignore */
      }
      // 顶端内容 (AppHeader) 随展开/折叠自适应排版
      document.documentElement.style.setProperty("--chat-rail-w", next ? "320px" : "56px");
      return next;
    });
  };

  // 初始化 CSS 变量 (恢复持久化状态)
  useEffect(() => {
    document.documentElement.style.setProperty("--chat-rail-w", open ? "320px" : "56px");
    return () => {
      document.documentElement.style.removeProperty("--chat-rail-w");
    };
  }, [open]);

  const totalMessages = topics.reduce((acc, t) => acc + t.messages.length, 0);

  return (
    <div
      data-testid="right-chat-rail"
      className={clsx(
        // per 2026-11-15 用户发令: 右侧聊天栏顶格 (fixed top-0 顶到视口最顶, 不被 header 挤压)
        "fixed top-0 right-0 z-40 h-screen border-l-2 border-line bg-bg-card/95 backdrop-blur-xl flex flex-col transition-all duration-200",
        open ? "w-80" : "w-14",
      )}
    >
      {/* 顶部 header 行: 占满上方空间 (per 2026-11-14 用户发令) */}
      <div className="shrink-0 border-b border-line flex items-center justify-between px-3 py-2.5 bg-bg-card">
        {open ? (
          <>
            <span className="text-xs font-bold text-ink flex items-center gap-1.5">
              <MessageSquare size={13} className="text-accent" /> 聊天
            </span>
            <button
              type="button"
              onClick={toggle}
              data-testid="chat-rail-collapse"
              aria-label="折叠聊天栏"
              title="折叠聊天栏"
              className="p-1 rounded hover:bg-bg-soft text-ink-mute hover:text-accent"
            >
              <PanelRightClose size={14} />
            </button>
          </>
        ) : (
          <button
            type="button"
            onClick={toggle}
            data-testid="chat-rail-expand"
            aria-label="展开聊天栏"
            title="展开聊天栏"
            className="flex flex-col items-center gap-2 w-full py-2.5 rounded-lg text-ink-mute hover:text-accent hover:bg-accent/10 transition-colors"
          >
            {/* 折叠/展开按钮在上 (用户一眼识别) */}
            <PanelRightOpen size={14} />
            {/* 聊天图标在下 (带未读徽标) */}
            <span className="relative">
              <MessageSquare size={16} />
              {totalMessages > 0 && (
                <span
                  data-testid="chat-rail-badge"
                  className="absolute -top-1.5 -right-1.5 min-w-[14px] h-3.5 rounded-full bg-err text-white text-[8px] font-mono grid place-items-center px-0.5"
                >
                  {totalMessages > 99 ? "99+" : totalMessages}
                </span>
              )}
            </span>
          </button>
        )}
      </div>

      {/* 范围切换器 (per 2026-11-15 任务: 展开面板顶部, 清楚展示当前范围) */}
      {open && (
        <div
          className="shrink-0 border-b border-line bg-bg-card px-2 py-1.5"
          data-testid="chat-rail-scope-bar"
        >
          <div className="flex items-center gap-0.5 rounded-lg bg-bg-soft p-0.5" role="group" aria-label="聊天范围">
            <button
              type="button"
              onClick={() => setChatScope("WORKTREE")}
              data-testid="chat-rail-scope-worktree"
              aria-pressed={chatScope === "WORKTREE"}
              className={clsx(
                "flex-1 rounded-md px-2 py-1 text-[10px] font-semibold transition-colors",
                chatScope === "WORKTREE"
                  ? "bg-accent text-white shadow-sm"
                  : "text-ink-dim hover:text-ink",
              )}
            >
              {t.chatScope.worktree}
            </button>
            <button
              type="button"
              onClick={() => setChatScope("GLOBAL")}
              data-testid="chat-rail-scope-global"
              aria-pressed={chatScope === "GLOBAL"}
              className={clsx(
                "flex-1 rounded-md px-2 py-1 text-[10px] font-semibold transition-colors",
                chatScope === "GLOBAL"
                  ? "bg-accent text-white shadow-sm"
                  : "text-ink-dim hover:text-ink",
              )}
            >
              {t.chatScope.global}
            </button>
          </div>
          <p className="mt-1 px-0.5 text-[9px] leading-relaxed text-ink-mute" data-testid="chat-rail-scope-label">
            {chatScope === "WORKTREE"
              ? t.chatScope["label.worktree"]
              : t.chatScope["label.global"]}
          </p>
        </div>
      )}

      {/* 聊天面板 */}
      {open && (
        <div className="flex-1 min-h-0" data-testid="chat-rail-panel">
          <SideChatPanel compact />
        </div>
      )}
    </div>
  );
}
