/*
@cypher schema=1 source_sha256=432f53a5bb068c11886871c7cb4fed78f3941677cfcdbc448a84a38a85470e64
MERGE (self:File {path:"frontend/src/app/(app)/sandbox/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/sandbox/page.tsx::SandboxPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// =====================================================================
// /sandbox — 沙盒 (per 2026-11-14 页面结构: 2c · 沙盒)
// 前端控制台: 会话列表 + 创建会话 + 运行命令
// 后端 = crates/sandboxd (Sandbox-as-a-Service, WBS 6.5.2, tonic gRPC 4 RPC:
//   CreateSession / RunCommand / DestroySession / StreamLogs)
// 前端现为 mock 展示 + 接入点标注 (sandboxd daemon 需本地宿主装配)
// =====================================================================
"use client";

import { useState } from "react";
import { Boxes, Plus, Play, Trash2, TerminalSquare } from "lucide-react";
import { useTranslation } from "@/lib/i18n";

interface SandboxSession {
  id: string;
  name: string;
  status: "running" | "stopped" | "failed";
  resource_policy: string;
  created_at: string;
}

const MOCK_SESSIONS: SandboxSession[] = [
  { id: "sbx-001", name: "agent-exec-wt-005", status: "running", resource_policy: "mem 512MB · cpu 1 · time 10m", created_at: "2026-11-14 08:12" },
  { id: "sbx-002", name: "python-repl-demo", status: "stopped", resource_policy: "mem 256MB · cpu 1 · time 5m", created_at: "2026-11-14 07:40" },
  { id: "sbx-003", name: "build-check-sandboxd", status: "failed", resource_policy: "mem 1GB · cpu 2 · time 15m", created_at: "2026-11-13 22:05" },
];

export default function SandboxPage() {
  const { t } = useTranslation();
  const [sessions] = useState<SandboxSession[]>(MOCK_SESSIONS);
  const [command, setCommand] = useState("");

  return (
    <div className="mx-auto max-w-5xl p-6">
      <header className="mb-6">
        <h1 className="text-2xl font-bold tracking-tight text-ink">{t.sandbox.title}</h1>
        <p className="mt-1 text-sm text-ink-dim">{t.sandbox.subtitle}</p>
      </header>

      <div className="grid gap-4 lg:grid-cols-3">
        <section className="lg:col-span-2 space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold text-ink">{t.sandbox.sessions}</h2>
            <button type="button" className="flex items-center gap-1.5 rounded-lg bg-accent px-3 py-1.5 text-xs font-semibold text-white">
              <Plus size={13} /> {t.sandbox.newSession}
            </button>
          </div>
          <ul className="space-y-2">
            {sessions.map((session) => (
              <li key={session.id} data-testid={`sandbox-session-${session.id}`} className="flex items-center gap-3 rounded-xl border border-line bg-bg-card p-4">
                <div className="size-8 rounded-lg grid place-items-center border border-amber-500/40 bg-amber-500/15 text-amber-400 shrink-0">
                  <Boxes size={15} />
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-semibold text-ink">{session.name}</span>
                    <span className={`text-[10px] font-mono px-1.5 py-0.5 rounded border ${session.status === "running" ? "border-ok/50 text-ok" : session.status === "failed" ? "border-err/50 text-err" : "border-line text-ink-mute"}`}>
                      {session.status}
                    </span>
                  </div>
                  <div className="mt-0.5 text-[11px] text-ink-mute">{session.resource_policy} · {session.created_at}</div>
                </div>
                <button type="button" aria-label={t.sandbox.destroy} className="p-1.5 rounded-md text-ink-mute hover:text-err hover:bg-err/10">
                  <Trash2 size={14} />
                </button>
              </li>
            ))}
          </ul>
        </section>

        <section className="rounded-xl border border-line bg-bg-card p-4 h-fit">
          <h2 className="text-sm font-semibold text-ink flex items-center gap-1.5">
            <TerminalSquare size={14} className="text-accent" /> {t.sandbox.runCommand}
          </h2>
          <p className="mt-1 text-[11px] text-ink-mute">{t.sandbox.commandHint}</p>
          <textarea
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            rows={3}
            placeholder="echo hello && cargo test -p sandboxd"
            className="mt-2 w-full rounded-lg border border-line bg-bg-soft px-3 py-2 text-xs font-mono"
          />
          <button type="button" disabled={!command.trim()} className="mt-2 flex items-center gap-1.5 rounded-lg bg-accent px-3 py-1.5 text-xs font-semibold text-white disabled:opacity-50">
            <Play size={13} /> {t.sandbox.run}
          </button>
          <p className="mt-3 text-[10px] text-ink-mute">{t.sandbox.backendHint}</p>
        </section>
      </div>
    </div>
  );
}
