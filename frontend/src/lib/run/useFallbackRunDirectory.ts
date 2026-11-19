/*
@cypher schema=1 source_sha256=18c2f5f484f68d7cad3d784971b03e5062ac21d6ac54f32fd7eb002e02523126
MERGE (self:File {path:"frontend/src/lib/run/useFallbackRunDirectory.ts"})
MERGE (hook:Symbol {id:"frontend/src/lib/run/useFallbackRunDirectory.ts::useFallbackRunDirectory",kind:"function"})
MERGE (store:File {path:"frontend/src/lib/store.ts"})
MERGE (handler:File {path:"frontend/src/mocks/handlers/run-directory.ts"})
MERGE (self)-[:DEFINES]->(hook)
MERGE (hook)-[:CALLS]->(store)
MERGE (handler)-[:SERVES]->(hook)
@endcypher
*/
// useFallbackRunDirectory — 网页版降级目录引导
// (per 2026-11-19 用户反馈: Project 导航应是树形结构, 现在只有提示文字)
//
// 根因: 5864582b 清 seed 后 store.projects/lanes/worktrees 诚实空初始态,
// 网页版无宿主会话 → FallbackProjectRunTree map 空数组 → 只剩提示文字。
// 本 hook 在 store 为空时从 /api/run-directory (MSW 演示通道, 数据源 seed.ts)
// 拉取层级注入 store, 让降级树真实渲染 Project → Lane → Worktree → Branch。
//
// 约束:
// - 仅在三个集合全空时注入 (不覆盖用户/宿主已有数据)
// - real-mode 开启时不引导 (真数据走宿主会话/真 API, 不用演示数据顶替)
// - 等 persist hydration 完成再判断, 避免与 localStorage 恢复竞态

import { useEffect } from "react";
import { useStore } from "@/lib/store";
import { isRealMode } from "@/mocks/real-mode";
import type { Lane, Project, Worktree } from "@/types/ids";

interface RunDirectoryPayload {
  projects: Project[];
  lanes: Lane[];
  worktrees: Worktree[];
}

let inflight: Promise<void> | null = null;

async function loadOnce(): Promise<void> {
  const res = await fetch("/api/run-directory");
  if (!res.ok) return;
  const data = (await res.json()) as Partial<RunDirectoryPayload>;
  if (!Array.isArray(data.projects) || !Array.isArray(data.lanes) || !Array.isArray(data.worktrees)) return;
  const s = useStore.getState();
  // 二次检查: hydration/其他通道可能已注入数据
  if (s.projects.length > 0 || s.lanes.length > 0 || s.worktrees.length > 0) return;
  useStore.setState({ projects: data.projects, lanes: data.lanes, worktrees: data.worktrees });
}

export function useFallbackRunDirectory(): void {
  const projectCount = useStore((s) => s.projects.length);
  const laneCount = useStore((s) => s.lanes.length);
  const worktreeCount = useStore((s) => s.worktrees.length);

  useEffect(() => {
    if (projectCount > 0 || laneCount > 0 || worktreeCount > 0) return;
    if (isRealMode()) return;

    let cancelled = false;
    const run = () => {
      if (cancelled) return;
      if (!inflight) {
        inflight = loadOnce()
          .catch(() => undefined)
          .finally(() => {
            inflight = null;
          });
      }
    };

    // persist hydration 未完成时等它结束, 避免空判断竞态。
    // 注: hydration 链可能因 storage/migrate 异步而迟于 effect 完成,
    // 订阅后主动 rehydrate() 兜底 (幂等, 已完成时是 no-op 级重读)。
    const persist = useStore.persist;
    if (persist && typeof persist.hasHydrated === "function" && !persist.hasHydrated()) {
      const unsub = persist.onFinishHydration(run);
      void persist.rehydrate().catch(() => undefined);
      return () => {
        cancelled = true;
        unsub();
      };
    }
    run();
    return () => {
      cancelled = true;
    };
  }, [projectCount, laneCount, worktreeCount]);
}
