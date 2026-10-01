// crates/star-desktop/frontend/src/pages/WorktreeVizPage.tsx
// =====================================================================
// Star Desktop — WorktreeVizPage (PR-265 follow-up of PR #252 BoardView)
// Per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2 P5
// WASM hook 实战: useLayoutEngine 集成 100+ worktree nodes 可视化
//
// 复用 (0 改):
//   - crates/star-desktop/frontend/src/hooks/useLayoutEngine.ts (per PR #251)
//   - src/types/ids.ts (per PR #247)
//
// 设计:
//   - makeMockNodes(count): 确定性生成 mock worktree 数据 (id, label, depth, commitDistance)
//   - useLayoutEngine() → pickAlgorithm 决定布局 (tree/dependency/risk/agent/history × dagre/d3_force/elk)
//   - visualDistance(commitDistance) 计算节点半径
//   - 手卷 SVG radial/tree 布局 (0 d3 依赖)
//   - Toolbar: slider + select + meta
// =====================================================================

import { useState, useMemo } from 'react';
import { useLayoutEngine, type ViewMode } from '../hooks/useLayoutEngine';

interface WorktreeNode {
  id: string;
  label: string;
  depth: number; // 0-4
  commitDistance: number; // 0-50
}

// 确定性 mock 生成 (无 random, 便于 vitest 验证)
function makeMockNodes(count: number): WorktreeNode[] {
  const nodes: WorktreeNode[] = [];
  for (let i = 0; i < count; i++) {
    const depth = i % 5; // 0-4 轮转
    const commitDistance = (i * 7) % 51; // 0-50 伪随机但确定性
    nodes.push({
      id: `wt-${String(i).padStart(4, '0')}`,
      label: `wt-${i.toString().padStart(3, '0')}`,
      depth,
      commitDistance,
    });
  }
  return nodes;
}

// Hand-rolled radial layout: x/y 由 depth + index 决定 (0 d3)
function layoutNodes(nodes: WorktreeNode[], visualDistance: (n: number) => number) {
  const byDepth = new Map<number, WorktreeNode[]>();
  for (const node of nodes) {
    if (!byDepth.has(node.depth)) byDepth.set(node.depth, []);
    byDepth.get(node.depth)!.push(node);
  }
  const maxDepth = Math.max(...byDepth.keys(), 0);
  const cx = 600;
  const cy = 400;
  const ringStep = 130;

  const positions = new Map<string, { x: number; y: number }>();
  for (let depth = 0; depth <= maxDepth; depth++) {
    const ring = byDepth.get(depth) || [];
    const radius = (depth + 1) * ringStep;
    const n = ring.length;
    for (let i = 0; i < n; i++) {
      const angle = (2 * Math.PI * i) / Math.max(n, 1) - Math.PI / 2;
      positions.set(ring[i].id, {
        x: cx + radius * Math.cos(angle),
        y: cy + radius * Math.sin(angle),
      });
    }
  }
  return positions;
}

export default function WorktreeVizPage() {
  const [nodeCount, setNodeCount] = useState(120);
  const [viewMode, setViewMode] = useState<ViewMode>('tree');
  const layout = useLayoutEngine();

  const nodes = useMemo(() => makeMockNodes(nodeCount), [nodeCount]);
  const algorithm = useMemo(
    () => layout.pickAlgorithm(viewMode, nodes.length),
    [layout, viewMode, nodes.length],
  );
  const positions = useMemo(
    () => layoutNodes(nodes, layout.visualDistance),
    [nodes, layout],
  );

  return (
    <div data-testid="worktree-viz-page" className="p-4">
      {/* Toolbar */}
      <div
        className="flex items-center gap-3 mb-3 p-2 bg-gray-50 rounded border"
        data-testid="viz-toolbar"
      >
        <label className="flex items-center gap-2 text-sm">
          Nodes:
          <input
            type="range"
            min={10}
            max={500}
            step={10}
            value={nodeCount}
            onChange={(e) => setNodeCount(parseInt(e.target.value, 10))}
            data-testid="viz-node-count"
            className="w-32"
            aria-label="Worktree node count"
          />
          <span className="font-mono">{nodeCount}</span>
        </label>

        <label className="flex items-center gap-2 text-sm">
          View:
          <select
            value={viewMode}
            onChange={(e) => setViewMode(e.target.value as ViewMode)}
            data-testid="viz-view-mode"
            className="border rounded px-2 py-1"
            aria-label="View mode"
          >
            <option value="tree">tree</option>
            <option value="dependency">dependency</option>
            <option value="risk">risk</option>
            <option value="agent">agent</option>
            <option value="history">history</option>
          </select>
        </label>

        <span className="text-xs text-gray-500 ml-auto" data-testid="viz-algorithm">
          algo: <code>{algorithm}</code> | WASM: layout={layout.ready ? 'Y' : 'N'} ({layout.version ?? 'loading…'})
        </span>
      </div>

      {/* SVG visualization */}
      <svg
        data-testid="viz-svg"
        viewBox="0 0 1200 800"
        width="100%"
        height="800"
        style={{ border: '1px solid #e0e0e0', background: 'white' }}
      >
        {nodes.map((node) => {
          const pos = positions.get(node.id);
          if (!pos) return null;
          const radius = Math.max(4, layout.visualDistance(node.commitDistance) / 6);
          return (
            <g
              key={node.id}
              data-testid={`viz-node-${node.id}`}
              data-depth={node.depth}
              data-commit-distance={node.commitDistance}
            >
              <circle
                cx={pos.x}
                cy={pos.y}
                r={radius}
                fill="#1976d2"
                opacity={0.6 + (node.depth % 5) * 0.08}
                stroke="#0d47a1"
                strokeWidth={1}
              />
              <text
                x={pos.x}
                y={pos.y + radius + 12}
                fontSize={10}
                textAnchor="middle"
                fill="#333"
              >
                {node.label}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}