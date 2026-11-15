/*
@cypher schema=1 source_sha256=f6402a835658dbd7c0689b4c980809e00d0b9147b992f93a666219184d0d85ae
MERGE (self:File {path:"frontend/src/components/agent/ContextPanel.tsx"})
MERGE (panel:Symbol {id:"frontend/src/components/agent/ContextPanel.tsx::ContextPanel",kind:"component"})
MERGE (helper:Symbol {id:"frontend/src/components/agent/ContextPanel.tsx::estimateTokens",kind:"function"})
MERGE (self)-[:DEFINES]->(panel)
MERGE (self)-[:DEFINES]->(helper)
@endcypher
*/
"use client";

// =====================================================================
// ContextPanel — 聊天窗口/任务卡命令处上下文查阅面板 (per 2026-11-15 需求)
//   1. 头部: token 估算 (chars/4) + 消息条数 + 关闭按钮
//   2. 消息列表: 按时间倒序, 每条 role / 前 60 字摘要 / 时间
//   3. 手动压缩: 调用 onCompact; 未接入时禁用 + 诚实 title
//   4. 压缩状态条: 压缩中 / 完成 / 失败 三态 (内部 state)
//   5. 底部诚实徽标: 本地上下文估算, 真实 token 由 agent 运行时提供
// 诚实原则: 后端 agent 网关未接入 → 本组件不主动读 chatStore, 上下文数据
//   由调用方从 chatStore 映射传入 (本地模拟), 并通过底部徽标明确标注。
// 弹层样式: 绝对定位卡片, 由调用方放在相对定位容器内。
// =====================================================================

import { useMemo, useState } from "react";

/** 上下文消息 (调用方从 chatStore 的 ChatMessage 映射: id/text/at → id/content/timestamp) */
export interface ContextPanelMessage {
  id: string;
  role: string; // "human" | "agent" | "system" | ...
  content: string;
  timestamp: string; // ISO 时间
}

export interface ContextPanelProps {
  messages: ContextPanelMessage[];
  /** 手动压缩回调 (宿主 agent 网关接入后提供); 未传 → 压缩按钮禁用 */
  onCompact?: () => void | Promise<void>;
  /** 关闭按钮回调; 未传 → 关闭按钮禁用 */
  onClose?: () => void;
}

type CompactState = "idle" | "compacting" | "done" | "failed";

/** token 估算 helper: chars/4 (本地估算口径, 真实 token 由 agent 运行时提供) */
export function estimateTokens(text: string): number {
  return Math.ceil(text.length / 4);
}

function summarize(content: string, max = 60): string {
  const c = content.trim();
  return c.length > max ? `${c.slice(0, max)}…` : c;
}

function formatTime(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "—";
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function ContextPanel({ messages, onCompact, onClose }: ContextPanelProps) {
  const [compactState, setCompactState] = useState<CompactState>("idle");

  // 按时间倒序 (最新在上)
  const sorted = useMemo(
    () => [...messages].sort((a, b) => b.timestamp.localeCompare(a.timestamp)),
    [messages],
  );

  const totalTokens = useMemo(
    () => sorted.reduce((sum, m) => sum + estimateTokens(m.content), 0),
    [sorted],
  );

  const handleCompact = async () => {
    if (!onCompact || compactState === "compacting") return;
    setCompactState("compacting");
    try {
      const r = onCompact();
      if (r && typeof (r as Promise<void>).then === "function") await r;
      setCompactState("done");
    } catch {
      setCompactState("failed");
    }
  };

  return (
    <aside
      data-testid="ctx-panel"
      aria-label="上下文面板"
      className="absolute right-0 top-full z-50 mt-2 w-80 max-w-[calc(100vw-2rem)] overflow-hidden rounded-xl border border-line bg-bg-card shadow-soft"
    >
      {/* ── 头部: token 估算 + 消息条数 + 关闭 ── */}
      <header className="flex items-start gap-2 border-b border-line px-3 py-2">
        <div className="min-w-0 flex-1">
          <div className="text-label font-medium text-ink">上下文</div>
          <div className="text-micro text-ink-mute">
            ≈ <span data-testid="ctx-token-count">{totalTokens}</span> tokens ·{" "}
            <span data-testid="ctx-msg-count">{messages.length}</span> 条消息
          </div>
        </div>
        <button
          type="button"
          data-testid="ctx-close"
          aria-label="关闭上下文面板"
          disabled={!onClose}
          title={onClose ? "关闭" : "未提供 onClose 回调"}
          onClick={onClose}
          className="rounded-md px-1.5 py-0.5 text-ink-mute hover:bg-bg-soft hover:text-ink disabled:cursor-not-allowed disabled:opacity-40"
        >
          ✕
        </button>
      </header>

      {/* ── 消息列表 (时间倒序) ── */}
      <section aria-label="上下文消息列表" className="max-h-72 overflow-y-auto p-2">
        {sorted.length === 0 ? (
          <p className="px-2 py-3 text-body text-ink-mute">暂无消息</p>
        ) : (
          <ul className="space-y-1.5">
            {sorted.map((m) => (
              <li
                key={m.id}
                data-testid={`ctx-msg-${m.id}`}
                className="rounded-lg bg-bg-soft px-2 py-1.5"
              >
                <div className="flex items-center justify-between gap-2">
                  <span className="text-label font-medium text-ink">{m.role}</span>
                  <time className="shrink-0 text-micro text-ink-mute">
                    {formatTime(m.timestamp)}
                  </time>
                </div>
                <p className="mt-0.5 break-words text-body text-ink-dim">
                  {summarize(m.content)}
                </p>
              </li>
            ))}
          </ul>
        )}
      </section>

      {/* ── 手动压缩 + 状态条 ── */}
      <section className="border-t border-line px-3 py-2">
        <button
          type="button"
          data-testid="ctx-compact"
          disabled={!onCompact}
          title={
            onCompact
              ? "手动压缩上下文，减少 token 占用"
              : "压缩服务未接入 — 宿主 agent 网关接入后启用"
          }
          onClick={handleCompact}
          className="w-full rounded-lg border border-line bg-bg-lighter px-2 py-1.5 text-label text-ink hover:border-accent hover:text-accent disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:border-line disabled:hover:text-ink"
        >
          {compactState === "compacting" ? "压缩中…" : "手动压缩上下文"}
        </button>
        {compactState !== "idle" && (
          <p
            data-testid="ctx-compact-state"
            role="status"
            className={
              compactState === "failed"
                ? "mt-1.5 text-micro text-err"
                : compactState === "done"
                  ? "mt-1.5 text-micro text-ok"
                  : "mt-1.5 text-micro text-ink-mute"
            }
          >
            {compactState === "compacting" && "压缩中 — 请稍候"}
            {compactState === "done" && "压缩完成"}
            {compactState === "failed" && "压缩失败 — 请重试"}
          </p>
        )}
      </section>

      {/* ── 底部诚实徽标 ── */}
      <footer className="border-t border-line px-3 py-2">
        <p data-testid="ctx-honest-badge" className="text-micro text-ink-mute">
          本地上下文估算 — 真实 token 由 agent 运行时提供
        </p>
      </footer>
    </aside>
  );
}
