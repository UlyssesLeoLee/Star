/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/run/useFallbackRunDirectory.ts"})
MERGE (hook:Symbol {id:"frontend/src/lib/run/useFallbackRunDirectory.ts::useFallbackRunDirectory",kind:"function"})
MERGE (store:File {path:"frontend/src/lib/store.ts"})
MERGE (demo:File {path:"frontend/src/lib/run/demoDirectory.ts"})
MERGE (self)-[:DEFINES]->(hook)
MERGE (hook)-[:CALLS]->(store)
MERGE (hook)-[:CALLS]->(demo)
@endcypher
*/
// useFallbackRunDirectory — 网页版降级目录引导
// (per 2026-11-19 用户反馈: Project 导航应是树形结构, 现在只有提示文字)
//
// P0-lane (2026-11-20) 改动: 演示数据源从 /api/run-directory (MSW 通道, 读 lib/seed.ts)
// 移到独立模块 demoDirectory.ts —「演示目录独立存放并明确标注」。
// 本 hook 不再发任何网络请求, 只做三集合全空的注入判定:
//   - 仅在 projects/lanes/worktrees 三个集合全空时注入 (不覆盖用户/宿主已有数据)
//   - real-mode 开启时绝不注入演示数据 (真数据走宿主会话 / 服务端 Lane API)
//   - 等 persist hydration 完成再判断, 避免与 localStorage 恢复竞态
// UI 层以「演示目录」badge (data-testid="demo-directory-badge") 明确标注注入来源。

import { useEffect } from "react";
import { useStore } from "@/lib/store";
import { isRealMode } from "@/mocks/real-mode";
import {
  demoRunDirectoryPayload,
  demoRunDirectoryPayloadIsValid,
} from "./demoDirectory";

/** 三集合全空且非 real-mode 时注入演示目录 (幂等; 非法 payload 不注入) */
function injectDemoDirectory(): void {
  const state = useStore.getState();
  if (state.projects.length > 0 || state.lanes.length > 0 || state.worktrees.length > 0) {
    return;
  }
  if (isRealMode()) return; // 真实模式下绝不混入 seed/演示数据
  const payload = demoRunDirectoryPayload();
  if (!demoRunDirectoryPayloadIsValid(payload)) return;
  // 二次检查 (hydration/其他通道可能已注入)
  const current = useStore.getState();
  if (current.projects.length > 0 || current.lanes.length > 0 || current.worktrees.length > 0) {
    return;
  }
  useStore.setState({
    projects: payload.projects,
    lanes: payload.lanes,
    worktrees: payload.worktrees,
  });
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
      injectDemoDirectory();
    };

    // persist hydration 未完成时等它结束, 避免空判断竞态。
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
