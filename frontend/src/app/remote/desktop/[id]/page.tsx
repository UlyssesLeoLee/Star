// Star Mobile Remote Desktop (per 2026-09-01 PHASE-MOBILE-PWA v0.2)
"use client";

import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { useStore } from "@/lib/store";
import { NoVncViewer } from "@/components/remote/NoVncViewer";

export default function RemoteDesktopPage({
  params,
}: {
  params: { id: string };
}) {
  const { id } = params;
  const runtime = useStore((s) => s.localRuntimes.find((r) => r.id === id));

  if (!runtime) {
    return (
      <div className="p-6 text-center text-ink-dim">
        <p className="text-sm">Runtime {id} not found</p>
        <Link href="/remote" className="text-accent text-xs mt-2 inline-block">
          ← 返回远程控制
        </Link>
      </div>
    );
  }

  return (
    <div className="h-[calc(100vh-3.5rem)] md:h-[calc(100vh-4rem)] flex flex-col">
      <div className="flex items-center gap-2 px-3 py-2 border-b border-line bg-bg-soft/60">
        <Link
          href="/remote"
          data-testid="remote-desktop-back"
          className="p-1 text-ink-dim hover:text-ink"
          aria-label="Back"
        >
          <ArrowLeft size={16} />
        </Link>
        <span className="text-sm font-semibold text-ink">远程桌面 · {runtime.hostname}</span>
      </div>
      <div className="flex-1 min-h-0">
        <NoVncViewer runtimeId={id} hostname={runtime.hostname} />
      </div>
    </div>
  );
}
