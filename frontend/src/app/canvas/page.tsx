/*
@cypher schema=1 source_sha256=39f45fda4ce492e833de78227514415aa8bd9feca12b2a421a73590edbbafb58
MERGE (self:File {path:"frontend/src/app/canvas/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/canvas/page.tsx::CanvasPage",kind:"component"})
MERGE (resolveCanvasId:Symbol {id:"frontend/src/lib/canvas-entry.ts::resolveCanvasId",kind:"function"})
MERGE (createCanvasLocal:Symbol {id:"frontend/src/lib/canvas-entry.ts::createCanvasLocal",kind:"function"})
MERGE (navStore:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (self)-[:DEFINES]->(page)
MERGE (self)-[:CALLS]->(resolveCanvasId)
MERGE (self)-[:CALLS]->(createCanvasLocal)
MERGE (self)-[:IMPORTS]->(navStore)
@endcypher
*/
"use client";

// =====================================================================
// /canvas — per 2026-11-16 用户发令 (10 条第 4 条): 取消固定跳 canvas-001,
// 按当前 Worktree / Lane 上下文进入正确画布:
//   selectedWorktreeId → 该 worktree 的关联画布 (ref_kind=worktree)
//   selectedLaneId     → Lane 默认画布 (所属 project 的画布, ref_kind=project)
//   无上下文 / 无匹配  → 未接入状态 + 新建画布按钮
// 注: 服务端 redirect 读不到 zustand 持久化上下文, 改为客户端解析后
//   router.replace; SSR 首帧渲染 loading 占位, 水合后执行解析跳转。
// =====================================================================

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Loader2, Plus } from "lucide-react";
import { useStore } from "@/lib/store";
import { useNavStore } from "@/lib/nav/navStore";
import { createCanvasLocal, resolveCanvasId } from "@/lib/canvas-entry";

export default function CanvasPage() {
  const router = useRouter();
  const selectedWorktreeId = useNavStore((s) => s.selectedWorktreeId);
  const selectedLaneId = useNavStore((s) => s.selectedLaneId);
  const worktrees = useStore((s) => s.worktrees);
  const lanes = useStore((s) => s.lanes);
  const canvases = useStore((s) => s.canvases);

  // mounted 门闩: persist 上下文只在客户端水合后有值, SSR 首帧不渲染业务 UI
  const [mounted, setMounted] = useState(false);
  useEffect(() => {
    setMounted(true);
  }, []);

  const target = mounted
    ? resolveCanvasId(
        { worktreeId: selectedWorktreeId, laneId: selectedLaneId },
        worktrees,
        lanes,
        canvases,
      )
    : null;

  useEffect(() => {
    if (target) router.replace(`/canvas/${target.id}`);
  }, [target, router]);

  const handleCreate = () => {
    const s = useStore.getState();
    const wt = selectedWorktreeId
      ? s.worktrees.find((w) => w.id === selectedWorktreeId)
      : undefined;
    const lane = selectedLaneId
      ? s.lanes.find((l) => l.id === selectedLaneId)
      : undefined;
    let input: Parameters<typeof createCanvasLocal>[0];
    if (wt) {
      input = { ref_kind: "worktree", ref_id: wt.id, title: `${wt.name} · 画布` };
    } else if (lane) {
      input = { ref_kind: "project", ref_id: lane.project_id, title: `${lane.name} · 默认画布` };
    } else {
      input = { ref_kind: "free" };
    }
    const canvas = createCanvasLocal(input);
    router.replace(`/canvas/${canvas.id}`);
  };

  if (!mounted) {
    return (
      <div
        className="flex h-full items-center justify-center gap-2 text-ink-mute"
        data-testid="canvas-entry-loading"
      >
        <Loader2 size={16} className="animate-spin" />
        <span className="text-sm">正在按当前 Worktree / Lane 上下文解析画布…</span>
      </div>
    );
  }

  if (target) {
    // 解析成功 → 跳转中占位 (router.replace 在 effect 中执行)
    return (
      <div
        className="flex h-full items-center justify-center gap-2 text-ink-mute"
        data-testid="canvas-entry-redirecting"
      >
        <Loader2 size={16} className="animate-spin" />
        <span className="text-sm">进入画布 {target.id}…</span>
      </div>
    );
  }

  // 未接入状态 + 新建画布
  const ctxLabel = selectedWorktreeId
    ? `Worktree ${selectedWorktreeId}`
    : selectedLaneId
      ? `Lane ${selectedLaneId}`
      : "当前上下文";
  return (
    <div
      className="flex h-full flex-col items-center justify-center gap-4 bg-bg px-6"
      data-testid="canvas-entry-unbound"
    >
      <div className="text-center">
        <p className="text-sm font-semibold text-ink">画布未接入</p>
        <p className="mt-1 max-w-md text-xs text-ink-mute" data-testid="canvas-entry-unbound-hint">
          {ctxLabel} 尚未关联画布 — 新建一个画布后进入, 或在侧栏选择已绑定画布的 Worktree / Lane
        </p>
      </div>
      <button
        type="button"
        onClick={handleCreate}
        data-testid="canvas-entry-create"
        className="flex items-center gap-1.5 rounded-lg border border-accent/40 bg-accent/10 px-3 py-2 text-xs font-semibold text-accent transition-colors hover:bg-accent/20"
      >
        <Plus size={14} /> 新建画布
      </button>
    </div>
  );
}
