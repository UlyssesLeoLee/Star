// =====================================================================
// /agent-view — free-form layout algorithm
// =====================================================================
// Per 2026-09-05 11:25 JST 拍板 #1: 自由散开 (Miro 风格), 不强制 swimlane.
// 节点按关联性散开, agent 居中, worktree 紧贴 agent 右侧, work-items
//   围绕 worktree 在圆周上分布, 排序 = [kanban status 优先级, due_date, id].
//
// 设计约束:
//   - 节点坐标是世界坐标 (画布坐标系), 跟 viewport 独立
//   - fit-to-content 算出 bbox, 第一次加载时 fit 进去
//   - 不重叠: 圆周上每个节点占 36° 槽位 (10 个), > 10 走第二圈
//   - 稳定: 同样输入永远出同样输出 (sort + deterministic), 避免 SSR/CSR hydration 漂移
// =====================================================================

import type { WorkItem, WorkItemStatus, Worktree, AgentSession } from "@/types/ids";
import type {
  AgentCanvasConnector, AgentCanvasNode, LayoutInput, LayoutOutput,
} from "./types";

/** 节点尺寸常量 (世界坐标, 用户 zoom in/out 时视觉缩放) */
const AGENT_W = 220;
const AGENT_H = 110;
const WORKTREE_W = 240;
const WORKTREE_H = 80;
const WORK_ITEM_W = 180;
const WORK_ITEM_H = 64;

/** Kanban status 排序权重 (越小越靠前/越活跃, 决定圆周位置) */
const STATUS_ORDER: Record<WorkItemStatus, number> = {
  in_progress: 0,
  review: 1,
  blocked: 2,
  todo: 3,
  done: 4,
  wontfix: 5,
};

/** Connector 颜色 (跟 StatusPill 配色一致, dark mode 友好) */
function connectorColorForWorkItemStatus(s: WorkItemStatus): string {
  switch (s) {
    case "in_progress": return "#2f81f7";   // info / blue
    case "review":      return "#d29922";   // warn / amber
    case "blocked":     return "#f85149";   // err / red
    case "todo":        return "#8b949e";   // ink-dim
    case "done":        return "#3fb950";   // ok / green
    case "wontfix":     return "#6e7681";   // ink-mute
    default:            return "#8b949e";
  }
}

/**
 * 主布局入口 — 给定 agent + worktree + work-items, 返回画布节点和连接.
 *   - 无 worktree: 仅返回 agent 节点, 不画连接
 *   - 无 work-items: 仅返回 agent + worktree 节点
 *   - 全部空: 返回空数组
 */
export function layoutAgentCanvas(input: LayoutInput): LayoutOutput {
  const { agent, worktree, workItems } = input;

  const nodes: AgentCanvasNode[] = [];
  const connectors: AgentCanvasConnector[] = [];

  // 1) agent 节点 (中心, (0, 0))
  const agentNode: AgentCanvasNode = {
    id: `n-agent-${agent.id}`,
    kind: "agent",
    x: 0,
    y: 0,
    width: AGENT_W,
    height: AGENT_H,
    ref: { kind: "agent", agentId: agent.id },
  };
  nodes.push(agentNode);

  if (!worktree) {
    return finalize(nodes, connectors);
  }

  // 2) worktree 节点 (agent 右侧, 居中对齐)
  const wtX = AGENT_W + 80; // 80px gap
  const wtY = (AGENT_H - WORKTREE_H) / 2;
  const worktreeNode: AgentCanvasNode = {
    id: `n-wt-${worktree.id}`,
    kind: "worktree",
    x: wtX,
    y: wtY,
    width: WORKTREE_W,
    height: WORKTREE_H,
    ref: { kind: "worktree", worktreeId: worktree.id },
  };
  nodes.push(worktreeNode);

  // agent → worktree connector
  connectors.push({
    id: `c-agent-wt-${agent.id}-${worktree.id}`,
    fromNodeId: agentNode.id,
    toNodeId: worktreeNode.id,
    color: "#2f81f7",
    label: "executes on",
  });

  if (workItems.length === 0) {
    return finalize(nodes, connectors);
  }

  // 3) work-items 节点 — 围绕 worktree 中心, 圆周分布
  //    排序: [kanban status order, due_date ASC, id ASC] (稳定)
  const sorted = [...workItems].sort(compareWorkItems);
  const wtCenterX = wtX + WORKTREE_W / 2;
  const wtCenterY = wtY + WORKTREE_H / 2;

  // 第一圈容量: 内圈 8 个, 超出走外圈 (外圈 12)
  const RING1_CAPACITY = 8;
  const RING1_RADIUS = 280;
  const RING2_RADIUS = 460;

  sorted.forEach((wi, idx) => {
    let cx: number;
    let cy: number;
    if (idx < RING1_CAPACITY) {
      // 内圈: 起始角度 -90° (12 点钟方向), 顺时针均分
      const angle = -Math.PI / 2 + (2 * Math.PI * idx) / RING1_CAPACITY;
      cx = wtCenterX + Math.cos(angle) * RING1_RADIUS - WORK_ITEM_W / 2;
      cy = wtCenterY + Math.sin(angle) * RING1_RADIUS - WORK_ITEM_H / 2;
    } else {
      // 外圈
      const outerIdx = idx - RING1_CAPACITY;
      // 外圈比内圈多 12 槽
      const outerCap = 12;
      const angle = -Math.PI / 2 + (2 * Math.PI * outerIdx) / outerCap;
      cx = wtCenterX + Math.cos(angle) * RING2_RADIUS - WORK_ITEM_W / 2;
      cy = wtCenterY + Math.sin(angle) * RING2_RADIUS - WORK_ITEM_H / 2;
    }

    const wiNode: AgentCanvasNode = {
      id: `n-wi-${wi.id}`,
      kind: "work_item",
      x: cx,
      y: cy,
      width: WORK_ITEM_W,
      height: WORK_ITEM_H,
      ref: { kind: "work_item", workItemId: wi.id },
    };
    nodes.push(wiNode);

    // worktree → work-item connector
    connectors.push({
      id: `c-wt-wi-${worktree.id}-${wi.id}`,
      fromNodeId: worktreeNode.id,
      toNodeId: wiNode.id,
      color: connectorColorForWorkItemStatus(wi.status),
      label: wi.status,
    });
  });

  return finalize(nodes, connectors);
}

// ---- helpers ----

function compareWorkItems(a: WorkItem, b: WorkItem): number {
  // 1) status 优先级
  const sa = STATUS_ORDER[a.status] ?? 99;
  const sb = STATUS_ORDER[b.status] ?? 99;
  if (sa !== sb) return sa - sb;
  // 2) due_date 升序 (近的先)
  const da = a.due_date ?? "9999";
  const db = b.due_date ?? "9999";
  if (da !== db) return da < db ? -1 : 1;
  // 3) id 升序 (稳定)
  if (a.id !== b.id) return a.id < b.id ? -1 : 1;
  return 0;
}

function finalize(
  nodes: AgentCanvasNode[],
  connectors: AgentCanvasConnector[],
): LayoutOutput {
  // 包围盒
  if (nodes.length === 0) {
    return {
      nodes,
      connectors,
      bbox: { minX: 0, minY: 0, maxX: 1200, maxY: 800 },
    };
  }
  const minX = Math.min(...nodes.map((n) => n.x)) - 80;
  const minY = Math.min(...nodes.map((n) => n.y)) - 80;
  const maxX = Math.max(...nodes.map((n) => n.x + n.width)) + 80;
  const maxY = Math.max(...nodes.map((n) => n.y + n.height)) + 80;
  return { nodes, connectors, bbox: { minX, minY, maxX, maxY } };
}

/** 由 bbox 算出 fit-to-content viewport (zoom + x/y 偏移)
 *  容器 1200x800, fit 进去, 留 40px padding
 */
export function fitToContentViewport(
  bbox: LayoutOutput["bbox"],
  containerW = 1200,
  containerH = 800,
  padding = 40,
): { x: number; y: number; zoom: number } {
  const bw = bbox.maxX - bbox.minX;
  const bh = bbox.maxY - bbox.minY;
  const usableW = containerW - padding * 2;
  const usableH = containerH - padding * 2;
  const zoom = Math.min(usableW / bw, usableH / bh, 1.5);
  const cx = (bbox.minX + bbox.maxX) / 2;
  const cy = (bbox.minY + bbox.maxY) / 2;
  return {
    x: cx - containerW / 2 / zoom,
    y: cy - containerH / 2 / zoom,
    zoom: Math.max(0.2, zoom),
  };
}

// Re-export 给组件用
export const NODE_DIMENSIONS = {
  agent: { width: AGENT_W, height: AGENT_H },
  worktree: { width: WORKTREE_W, height: WORKTREE_H },
  work_item: { width: WORK_ITEM_W, height: WORK_ITEM_H },
} as const;

// 防止 unused-warning (占位: 类型再导出, 后续组件会用到)
export type { WorkItem, Worktree, AgentSession };
