"use client";

import { useStore } from "@/lib/store";
import { useSearchParams, useRouter } from "next/navigation";
import { useEffect, Suspense } from "react";
import { CanvasView } from "@/components/CanvasView";
import { PageHeader } from "@/components/PageHeader";
import { ArrowLeft, Share2, Download, Users } from "lucide-react";
import Link from "next/link";

interface PageProps {
  params: { id: string };
}

function CanvasPageInner({ canvasId }: { canvasId: string }) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const highlightId = searchParams.get("highlight") || undefined;

  const canvas = useStore((s) => s.canvases.find((c) => c.id === canvasId));
  const elements = useStore((s) => s.canvasElements.filter((e) => e.canvas_id === canvasId));
  const connectors = useStore((s) => s.canvasConnectors.filter((c) => c.canvas_id === canvasId));
  const worktrees = useStore((s) => s.worktrees);
  const agentSessions = useStore((s) => s.agentSessions);
  const feedbacks = useStore((s) => s.feedbacks);
  const automationRules = useStore((s) => s.automationRules);
  const comments = useStore((s) => s.comments);

  if (!canvas) {
    return (
      <div className="max-w-3xl">
        <div className="card text-center py-12">
          <h2 className="text-lg font-semibold mb-2">Canvas not found</h2>
          <p className="text-sm text-ink-dim mb-4">id: {canvasId}</p>
          <Link href="/collaboration" className="btn">
            <ArrowLeft size={12} /> Back to Collaboration
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="-mx-6 -mt-5 h-[calc(100vh-3.5rem)] flex flex-col">
      {/* Header */}
      <div className="border-b border-line bg-bg-soft/40 px-6 py-3 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Link href="/collaboration" className="btn p-1.5" title="Back">
            <ArrowLeft size={14} />
          </Link>
          <div>
            <div className="text-sm font-semibold">{canvas.title}</div>
            <div className="text-[10px] text-ink-mute font-mono">
              {canvas.id} · {elements.length} elements · {connectors.length} connectors · {canvas.collaborator_ids.length} collaborators
              {canvas.ref_kind && <> · ref: <span className="text-info">{canvas.ref_kind}/{canvas.ref_id}</span></>}
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2 text-[10px] text-ink-dim font-mono">
          <span className="flex items-center gap-1"><Users size={10} /> 3 online</span>
          <button className="btn text-[10px] py-0.5"><Share2 size={10} /> Share</button>
          <button className="btn text-[10px] py-0.5"><Download size={10} /> Export PNG</button>
        </div>
      </div>

      {/* Canvas */}
      <div className="flex-1 relative">
        <CanvasView canvas={canvas} elements={elements} connectors={connectors} highlightElementId={highlightId} />
      </div>
    </div>
  );
}

export default function CanvasPage({ params }: PageProps) {
  // Next.js 14.2.5: params 是 plain object (不是 Promise)
  // 原 use() unwrap 是 Next 15 API, Next 14 不支持
  // (per 2026-09-04 canvas e2e 守门 prerequisite baseline fix)
  return (
    <Suspense fallback={<div className="card">Loading canvas...</div>}>
      <CanvasPageInner canvasId={params.id} />
    </Suspense>
  );
}
