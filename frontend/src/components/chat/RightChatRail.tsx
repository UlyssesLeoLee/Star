/*
@cypher schema=1 source_sha256=6995789fcd228e2199fe12379132138a3f231103c25378da13e00b67edb15540
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

export function RightChatRail() {
  const topics = useChatStore((s) => s.topics);
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
        "shrink-0 border-l-2 border-line bg-bg-card/95 backdrop-blur-xl flex flex-col h-full transition-all duration-200",
        open ? "w-80" : "w-14",
      )}
    >
      {/* 边条 / 头部 */}
      <div className="shrink-0 border-b border-line flex items-center justify-between px-2 py-2">
        {open ? (
          <>
            <span className="text-xs font-semibold text-ink flex items-center gap-1.5">
              <MessageSquare size={13} className="text-accent" /> 聊天
            </span>
            <button
              type="button"
              onClick={toggle}
              data-testid="chat-rail-collapse"
              aria-label="折叠聊天栏"
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
            className="flex flex-col items-center gap-1 w-full py-2 rounded-lg text-ink-mute hover:text-accent hover:bg-accent/10 transition-colors"
          >
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
            <PanelRightOpen size={11} />
          </button>
        )}
      </div>

      {/* 聊天面板 */}
      {open && (
        <div className="flex-1 min-h-0" data-testid="chat-rail-panel">
          <SideChatPanel compact />
        </div>
      )}
    </div>
  );
}
