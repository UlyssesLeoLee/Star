// =====================================================================
// /agent-view — shared types
// =====================================================================
// Per 2026-09-05 11:25 JST 拍板: Agent view 是"以当前工作 agent 为筛选模式
//   的无限画布, 显示该 agent 关联的 worktree / work-items / 状态, 数据来源跟
//   kanban 等界面共享 store".
//
// 跟 docs/frontend/design/frontend-canvas-design.md v0.1 的 canvas 不同:
//   - 通用 canvas = 用户自由编辑的画布 (sticky_note / shape / image)
//   - Agent canvas = 派生视图, 由 agent → worktree → work-items 自动布局,
//     只读为主 (用户可 zoom/pan/双击跳详情, 不能拖动节点)
//
// 布局策略 (per 9/5 11:25 拍板 #1 "自由散开"):
//   - agent 节点: 画布中心, 大圆形
//   - worktree 节点: 紧贴 agent 下方 / 侧边, 中型矩形
//   - work-item 节点: 围绕 worktree 散开, 按 status 着色
//   - connector: agent → worktree, worktree → 每个 work-item
// =====================================================================

import type {
  AgentSession, Worktree, WorkItem,
} from "@/types/ids";

/** 画布节点类型 (per 拍板 #1: 自由散开) */
export type AgentCanvasNodeKind = "agent" | "worktree" | "work_item";

/** 单个画布节点 (派生数据, 不进 store) */
export interface AgentCanvasNode {
  id: string;
  kind: AgentCanvasNodeKind;
  /** 世界坐标 (画布坐标系, 跟 viewport 转换) */
  x: number;
  y: number;
  width: number;
  height: number;
  /** 渲染用 payload (渲染时从 store 查最新值, 这里只存 id 引用) */
  ref:
    | { kind: "agent"; agentId: string }
    | { kind: "worktree"; worktreeId: string }
    | { kind: "work_item"; workItemId: string };
}

/** 画布连接 (agent → worktree, worktree → work-item) */
export interface AgentCanvasConnector {
  id: string;
  fromNodeId: string;
  toNodeId: string;
  /** 连线颜色 (从 status 派生) */
  color: string;
  label?: string;
}

/** 完整画布 (派生, 仅供 AgentCanvasView 消费) */
export interface AgentCanvas {
  agentId: string;
  nodes: AgentCanvasNode[];
  connectors: AgentCanvasConnector[];
  /** 初始 viewport (fit-to-content) */
  viewport: { x: number; y: number; zoom: number };
  /** 派生时间戳 (用于 E2E / cache invalidation) */
  derivedAt: string;
}

/** "当前工作 agent" 标识 (per 拍板 #2: 自动选 + 手动覆盖 + URL 参数) */
export interface CurrentAgentResolution {
  agentId: string;
  agent: AgentSession;
  /** 自动选 = true, URL 参数 / 手动选 = false */
  auto: boolean;
}

/** 布局输入 (per layout.ts 签名) */
export interface LayoutInput {
  agent: AgentSession;
  worktree: Worktree | null;
  workItems: WorkItem[];
}

/** 布局输出 (per layout.ts 签名) */
export interface LayoutOutput {
  nodes: AgentCanvasNode[];
  connectors: AgentCanvasConnector[];
  /** 包围盒 (用于 fit-to-content) */
  bbox: { minX: number; minY: number; maxX: number; maxY: number };
}
