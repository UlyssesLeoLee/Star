// Star Mobile Remote Control Home (per 2026-09-01 PHASE-MOBILE-PWA v0.2)
// 三件套入口: Desktop / Terminal / Files
// 每个 runtime 一个卡片,3 个快捷按钮
"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useStore } from "@/lib/store";
import { PageHeader } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Monitor, Terminal as TerminalIcon, FolderOpen, Server } from "lucide-react";

export default function RemoteHomePage() {
  const runtimes = useStore((s) => s.localRuntimes);
  const router = useRouter();

  const online = runtimes.filter((r) => r.status === "online");

  return (
    <div className="max-w-5xl">
      <PageHeader
        title="远程控制"
        subtitle="手机端直连 local-runtime, 三件套: 远程桌面 (noVNC) / 远程终端 (xterm.js) / 远程文件 (SFTP)"
        icon={<Server className="text-accent" size={20} />}
        track="M"
        count={online.length}
      />

      <div className="mb-3 text-[11px] font-mono text-ink-mute px-1">
        走 Star BFF WebSocket relay: 手机 → /v1/remote/{`{kind}`}/{`{id}`} → BFF (auth/audit/rate-limit) → local-runtime agent
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {runtimes.map((r) => (
          <div
            key={r.id}
            data-testid={`remote-runtime-${r.id}`}
            className="card flex flex-col gap-3"
          >
            <div className="flex items-center gap-3">
              <div className="size-10 rounded-xl border border-accent/40 grid place-items-center bg-accent/10 shrink-0">
                <Server size={18} className="text-accent" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="font-semibold text-ink truncate">{r.hostname}</span>
                  <StatusPill value={r.status} />
                </div>
                <div className="text-[10px] font-mono text-ink-mute truncate">
                  {r.id} · {r.mount_root}
                </div>
              </div>
            </div>

            <div className="grid grid-cols-3 gap-2">
              <Link
                href={`/remote/desktop/${r.id}`}
                data-testid={`remote-desktop-${r.id}`}
                className="flex flex-col items-center gap-1 px-3 py-2.5 rounded-lg border border-line bg-bg-soft/40 hover:border-accent/40 hover:text-accent text-ink-dim transition-colors"
                onClick={(e) => {
                  if (r.status === "offline") {
                    e.preventDefault();
                    alert(`${r.hostname} 当前 offline，无法连接远程桌面`);
                  }
                }}
              >
                <Monitor size={16} />
                <span className="text-[10px] font-mono">Desktop</span>
              </Link>
              <Link
                href={`/remote/terminal/${r.id}`}
                data-testid={`remote-terminal-${r.id}`}
                className="flex flex-col items-center gap-1 px-3 py-2.5 rounded-lg border border-line bg-bg-soft/40 hover:border-accent/40 hover:text-accent text-ink-dim transition-colors"
              >
                <TerminalIcon size={16} />
                <span className="text-[10px] font-mono">Terminal</span>
              </Link>
              <Link
                href={`/remote/files/${r.id}`}
                data-testid={`remote-files-${r.id}`}
                className="flex flex-col items-center gap-1 px-3 py-2.5 rounded-lg border border-line bg-bg-soft/40 hover:border-accent/40 hover:text-accent text-ink-dim transition-colors"
              >
                <FolderOpen size={16} />
                <span className="text-[10px] font-mono">Files</span>
              </Link>
            </div>
          </div>
        ))}
      </div>

      {runtimes.length === 0 && (
        <div className="card text-center text-ink-mute text-sm py-12">
          暂无可连接的 local-runtime
        </div>
      )}
    </div>
  );
}
