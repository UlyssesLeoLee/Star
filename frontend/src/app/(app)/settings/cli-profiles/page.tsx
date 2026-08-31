"use client";

// Star Frontend — CLI Profiles 设置页
// Per 2026-08-29 09:07 JST 用户拍板: 分组 + 表单, 不混乱
// 入口: TopBar 用户菜单 → CLI Profiles

import { useState } from "react";
import { Terminal, Globe, Plus, Edit2, Trash2, Check, X, RefreshCw, AlertCircle } from "lucide-react";
import { PageHeader, SectionTitle } from "@/components/PageHeader";

interface CliProfile {
  id: string;
  name: string;
  kind: "cli" | "api";
  command: string;
  args: string;
  env: string;
  worktreeBinding: "auto" | "prompt";
  enabled: boolean;
  hasApiKey: boolean;
}

const PROFILES: CliProfile[] = [
  { id: "claude", name: "Claude Code", kind: "cli", command: "claude", args: "--model claude-3-5-sonnet", env: "PATH=$PATH", worktreeBinding: "auto", enabled: true, hasApiKey: true },
  { id: "codex", name: "OpenAI Codex", kind: "cli", command: "codex", args: "--model gpt-4", env: "OPENAI_API_KEY", worktreeBinding: "auto", enabled: true, hasApiKey: true },
  { id: "openclaw", name: "OpenClaw", kind: "api", command: "https://api.openclaw.dev/v1", args: "model=gpt-4", env: "OPENCLAW_API_KEY", worktreeBinding: "auto", enabled: false, hasApiKey: false },
  { id: "hermes", name: "Hermes", kind: "api", command: "https://api.hermes.dev/v1", args: "model=hermes-2", env: "HERMES_API_KEY", worktreeBinding: "auto", enabled: false, hasApiKey: false },
  { id: "gemini", name: "Google Gemini", kind: "cli", command: "gemini", args: "", env: "GOOGLE_API_KEY", worktreeBinding: "auto", enabled: true, hasApiKey: true },
  { id: "aider", name: "Aider", kind: "cli", command: "aider", args: "--model gpt-4", env: "OPENAI_API_KEY", worktreeBinding: "auto", enabled: false, hasApiKey: true },
];

export default function CliProfilesPage() {
  const [profiles, setProfiles] = useState<CliProfile[]>(PROFILES);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");

  const cliCount = profiles.filter((p) => p.kind === "cli").length;
  const apiCount = profiles.filter((p) => p.kind === "api").length;

  return (
    <div className="space-y-6">
      <PageHeader
        title="CLI Profiles"
        description="管理 6 个内置 + 自定义 CLI / API Agent · {cliCount} CLI · {apiCount} API"
        actions={
          <button className="btn-primary-ghost flex items-center gap-1">
            <Plus size={12} />
            新 Profile
          </button>
        }
      />

      {/* 6 个内置 */}
      <div>
        <SectionTitle>内置 Profiles (6)</SectionTitle>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {profiles.filter((p) => ["claude","codex","openclaw","hermes","gemini","aider"].includes(p.id)).map((p) => (
            <div
              key={p.id}
              className={`p-3 rounded-md border ${p.enabled ? "border-[color:var(--color-border)]" : "border-[color:var(--color-border)] opacity-60"} bg-[color:var(--color-surface)]`}
            >
              <div className="flex items-center justify-between mb-1.5">
                <div className="flex items-center gap-2">
                  {p.kind === "cli" ? <Terminal size={12} className="text-[color:var(--color-primary)]" /> : <Globe size={12} className="text-[color:var(--color-warning)]" />}
                  {editingId === p.id ? (
                    <div className="flex items-center gap-1">
                      <input
                        value={editName}
                        onChange={(e) => setEditName(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") {
                            setProfiles((prev) => prev.map((x) => x.id === p.id ? { ...x, name: editName } : x));
                            setEditingId(null);
                          }
                        }}
                        autoFocus
                        className="text-sm rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-2 py-0.5"
                      />
                      <button onClick={() => setEditingId(null)}><X size={12} /></button>
                    </div>
                  ) : (
                    <span className="text-sm font-medium">{p.name}</span>
                  )}
                  <span className="text-[10px] px-1 rounded bg-[color:var(--color-surface-2)] text-[color:var(--color-text-dim)]">
                    {p.kind.toUpperCase()}
                  </span>
                </div>
                <div className="flex items-center gap-1">
                  {p.hasApiKey ? <span className="text-[10px] text-[color:var(--color-success)]" title="已配置 API Key">●</span> : <span className="text-[10px] text-[color:var(--color-text-dim)]" title="未配置 API Key">○</span>}
                  <button onClick={() => { setEditingId(p.id); setEditName(p.name); }} className="opacity-60 hover:opacity-100"><Edit2 size={10} /></button>
                </div>
              </div>
              <div className="font-mono text-[10px] text-[color:var(--color-text-dim)] break-all">
                {p.command} {p.args}
              </div>
              <div className="mt-1.5 flex items-center justify-between text-[10px]">
                <span className="text-[color:var(--color-text-dim)]">{p.env}</span>
                <button
                  onClick={() => setProfiles((prev) => prev.map((x) => x.id === p.id ? { ...x, enabled: !x.enabled } : x))}
                  className={`px-2 py-0.5 rounded ${p.enabled ? "bg-[color:var(--color-success)]/20 text-[color:var(--color-success)]" : "bg-[color:var(--color-surface-2)] text-[color:var(--color-text-dim)]"}`}
                >
                  {p.enabled ? "已启用" : "已禁用"}
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* 自定义 Profiles */}
      <div>
        <SectionTitle>自定义 Profiles (0)</SectionTitle>
        <div className="rounded-md border border-dashed border-[color:var(--color-border)] p-6 text-center text-sm text-[color:var(--color-text-dim)]">
          暂无自定义 Profile。点击右上角"新 Profile"创建。
        </div>
      </div>

      {/* 提示 */}
      <div className="rounded-md border border-[color:var(--color-info)]/30 bg-[color:var(--color-info)]/5 p-3 flex items-start gap-2 text-xs">
        <AlertCircle size={14} className="text-[color:var(--color-info)] flex-shrink-0 mt-0.5" />
        <div>
          <div className="font-medium mb-1">关于 API Agent (OpenClaw / Hermes)</div>
          <div className="text-[color:var(--color-text-dim)]">
            OpenClaw 和 Hermes 通过 HTTP API 调用而非本地进程,需要对应的 API Key (在 <a href="/settings/api-keys" className="text-[color:var(--color-primary)] underline">API Keys 设置</a> 配置)。其他 4 个 (Claude/Codex/Gemini/Aider) 走本地 CLI 进程,需要系统已安装对应命令。
          </div>
        </div>
      </div>
    </div>
  );
}
