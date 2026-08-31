"use client";

// Star Frontend — CLI 终端输出组件
// Per 2026-08-29 04:09 JST: 实时输出 + 14 行 buffer

import { useEffect, useRef, useState } from "react";
import { Play, Square, Upload, Trash2, Download, Copy } from "lucide-react";
import type { CliTab } from "./WindowsTabBar";

interface CliTerminalProps {
  tab: CliTab | null;
  onRun: (prompt: string) => void;
  onCancel: () => void;
  onUpload: () => void;
  onClear: () => void;
}

export function CliTerminal({ tab, onRun, onCancel, onUpload, onClear }: CliTerminalProps) {
  const [prompt, setPrompt] = useState("");
  const outputRef = useRef<HTMLPreElement>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (outputRef.current) {
      outputRef.current.scrollTop = outputRef.current.scrollHeight;
    }
  }, [tab?.lastOutput]);

  if (!tab) {
    return (
      <div className="flex-1 flex items-center justify-center text-sm text-[color:var(--color-text-dim)] bg-[color:var(--color-surface)]">
        <div className="text-center">
          <div className="text-2xl mb-2 opacity-40">▷</div>
          选择或新建一个 Tab 启动 CLI 任务
        </div>
      </div>
    );
  }

  const isRunning = tab.state === "running";

  return (
    <div className="flex-1 flex flex-col bg-[color:var(--color-surface)] overflow-hidden">
      {/* Tab 头部 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-[color:var(--color-border)] bg-[color:var(--color-surface-2)]">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">{tab.label}</span>
          <span className="text-[10px] text-[color:var(--color-text-dim)]">{tab.profileName}</span>
        </div>
        <div className="flex items-center gap-1">
          {isRunning ? (
            <button
              onClick={onCancel}
              className="flex items-center gap-1 px-2 py-1 text-xs rounded border border-[color:var(--color-danger)]/30 bg-[color:var(--color-danger)]/10 text-[color:var(--color-danger)] hover:bg-[color:var(--color-danger)]/20"
            >
              <Square size={10} />
              取消
            </button>
          ) : (
            <button
              onClick={() => onRun(prompt)}
              disabled={!prompt.trim()}
              className="btn-primary px-2 py-1 text-xs"
            >
              <Play size={10} />
              运行
            </button>
          )}
          <button
            onClick={onUpload}
            disabled={!tab.filesChanged}
            title={tab.filesChanged ? `${tab.filesChanged} 个文件变更` : "无文件变更"}
            className="flex items-center gap-1 px-2 py-1 text-xs rounded border border-[color:var(--color-border)] hover:bg-[color:var(--color-surface)] disabled:opacity-40 disabled:cursor-not-allowed"
          >
            <Upload size={10} />
            上传
            {tab.filesChanged !== undefined && tab.filesChanged > 0 && (
              <span className="ml-0.5 px-1 rounded bg-[color:var(--color-primary)]/20 text-[color:var(--color-primary)] text-[9px]">
                {tab.filesChanged}
              </span>
            )}
          </button>
          <button
            onClick={onClear}
            className="flex items-center gap-1 px-2 py-1 text-xs rounded border border-[color:var(--color-border)] hover:bg-[color:var(--color-surface)]"
            aria-label="clear"
          >
            <Trash2 size={10} />
          </button>
        </div>
      </div>

      {/* 终端输出 */}
      <pre
        ref={outputRef}
        className="flex-1 overflow-auto p-3 text-xs font-mono whitespace-pre-wrap break-all bg-[#0F172A] text-[#E2E8F0]"
        aria-live="polite"
      >
        {tab.lastOutput || <span className="text-[#94A3B8]">[等待运行...]</span>}
      </pre>

      {/* Prompt 输入 */}
      <div className="border-t border-[color:var(--color-border)] p-3 bg-[color:var(--color-surface-2)]">
        <div className="flex items-end gap-2">
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && (e.metaKey || e.ctrlKey) && !isRunning) {
                onRun(prompt);
              }
            }}
            placeholder="输入 prompt (Cmd/Ctrl + Enter 运行)..."
            rows={2}
            className="flex-1 text-sm rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-3 py-2 resize-none focus:outline-none focus:border-[color:var(--color-primary)]"
          />
          <div className="flex flex-col gap-1 text-[10px] text-[color:var(--color-text-dim)]">
            <span>{prompt.length} 字符</span>
            {tab.lastOutput && (
              <button
                onClick={() => {
                  navigator.clipboard.writeText(tab.lastOutput || "");
                  setCopied(true);
                  setTimeout(() => setCopied(false), 1500);
                }}
                className="flex items-center gap-1 hover:text-[color:var(--color-primary)]"
              >
                <Copy size={9} />
                {copied ? "已复制" : "复制输出"}
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
