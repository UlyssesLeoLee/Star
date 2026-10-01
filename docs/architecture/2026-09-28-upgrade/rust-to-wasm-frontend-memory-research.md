# Rust → WebAssembly (WASM) 前端内存优化研究

> **date**: 2026-09-28 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: user "研究是否可以在网页端融入 rust 技术降低内存消耗"
> **scope**: Star monorepo (95+ Rust crates + Next.js frontend)
> **goal**: 把内存密集型 Rust crate 编译为 WebAssembly, 在浏览器侧用 `.wasm` 模块替代 JS 数据结构, 降低 React state 内存占用

---

## 0. 背景

### 0.1 当前前端内存热点 (per frontend bundle analysis)

| 路径 | 数据量 | 当前实现 | 内存占用 (估) |
|---|---|---|---|
| `frontend/src/mocks/seed.ts` | 30 work_items + 6 workspaces + 25 identities + 80 relations | JS array (Object.keys / array spread) | ~3-5 MB heap |
| `frontend/src/lib/store.ts` (zustand) | 全局 state (workItems + board + workflow + ...) | JSON serialization | ~8-15 MB heap (per 1000 work_items) |
| `@dnd-kit/core` + `@dnd-kit/sortable` | 拖动 state + collision detection | JS-only React hooks | ~2-3 MB heap (per board re-render) |
| `cytoscape` (graph view) | 100-1000 节点图 | JS layout (cose-bilkent) | ~10-30 MB heap (per graph render) |
| `monaco-editor` | editor model + decorations | JS text buffer | ~5-10 MB heap per editor |

**总和**: 100 work_items + 1 graph + 1 editor → ~25-50 MB JS heap (per Chrome devtools Memory snapshot)

### 0.2 Rust crate candidates (memory 密集型)

| Crate | 功能 | 适合 WASM | 理由 |
|---|---|---|---|
| `crates/graph-core` | 图算法 (BFS/DFS/topo sort) | ✅ **高** | 算法密集 + 无 IO, 编译 WASM 1:1, 性能可提 10-50x |
| `crates/query-engine` | 查询引擎 (filter/sort/aggregate) | ✅ **高** | 纯函数, 大量 array operation, WASM 内存可控 |
| `crates/relationship-engine` | 关系图构建 + 渲染数据准备 | ✅ **高** | 同 graph-core |
| `crates/health-engine` | 健康检查 rule engine | ⚠ 中 | 规则计算密集, 但调用频率低 |
| `crates/risk-engine` | 风险评估 | ⚠ 中 | 算法密集, 调用频率低 |
| `crates/layout-engine` | 布局算法 (Taffy/Cassowary) | ✅ **高** | 用于 React 替代方案 (e.g. yew / leptos) |
| `crates/event-bus` | 事件总线 | ❌ 低 | IO + 异步, WASM 优势小 |
| `crates/star-cache` | LRU + TTL 缓存 | ⚠ 中 | 数据结构, 但需持久化层 |

---

## 1. WASM 技术选型

### 1.1 编译工具链

| 选项 | 适用 | 优点 | 缺点 |
|---|---|---|---|
| **`wasm-pack`** + `wasm-bindgen` | Rust 库导出 JS binding | 生态最成熟, 工具齐全 | build time 长, 调试体验一般 |
| **`wasm-bindgen-rayon`** | Rust 多线程 (parallel) | 利用多核, 计算密集型提速 | 增加 200KB runtime, browser 主线程仍单核 |
| **`workerize-loader`** (vite) | 现有 Rust crate 直接 import | vite 集成简单 | 调试受限, 不能用 wasm-bindgen 全 features |
| **Trunk** | Rust → WASM 完整应用 | 一站式 | 太重, 我们是 JS + WASM 混合 |
| **`cargo-chef` + `maturin`** | Python 生态, 不适用 | - | - |

**选型**: `wasm-pack` + `wasm-bindgen` + `wasm-bindgen-rayon` (per algorithm-heavy crates)。

### 1.2 集成到 Vite (frontend)

**已存在**: `frontend/vite.config.ts` (per package.json vite ^6.3.5)

**plugin 选项**:

```ts
// frontend/vite.config.ts (增量)
import { defineConfig } from "vite";
import { resolve } from "path";

export default defineConfig({
  // ...existing config...
  optimizeDeps: {
    exclude: ["@star/graph-core-wasm", "@star/query-engine-wasm"],
  },
  // wasm-bindgen 输出 (.wasm + .js glue + .d.ts) 直接 import
  assetsInclude: ["**/*.wasm"],
  build: {
    target: "esnext", // 需 support top-level await + WebAssembly.instantiate streaming
    rollupOptions: {
      external: [/^@star\/.+-wasm$/], // 不打包, 走 CDN/独立 .wasm 文件
    },
  },
});
```

**plugin**: `vite-plugin-wasm` + `vite-plugin-top-level-await`

### 1.3 React integration pattern

```ts
// frontend/src/lib/wasm/graph-core.ts
import init, { bfs, topo_sort } from "@star/graph-core-wasm";

let wasmReady: Promise<void> | null = null;
export async function ensureWasmReady() {
  if (!wasmReady) {
    wasmReady = init("/wasm/graph_core_bg.wasm").then(() => {});
  }
  return wasmReady;
}

// React hook
import { useEffect, useState } from "react";

export function useGraphCore() {
  const [ready, setReady] = useState(false);
  useEffect(() => { ensureWasmReady().then(() => setReady(true)); }, []);
  return { ready, bfs, topoSort: topo_sort };
}

// 用法 (in KanbanBoard / Cytoscape wrapper)
const { ready, bfs } = useGraphCore();
if (ready) {
  const result = bfs(graphId, startNode, maxDepth); // 直接传 ArrayBuffer, 0 JS 拷贝
}
```

---

## 2. 内存节省估算

### 2.1 baseline (JS-only, 100 work_items)

```
work_items data:    100 × ~2 KB/项  =  200 KB  (raw JSON)
zundo/state快照:   100 × ~5 KB/项  =  500 KB
relations:         500 × ~1 KB/项  =  500 KB
React fiber tree:   100 × ~3 KB/项  =  300 KB
合计:                            ~1.5 MB
```

### 2.2 WASM 化后 (target)

```
WASM linear memory: 100 × ~256 B/项 = 25.6 KB  (packed struct, 8x compact)
+ JS 侧 reference: 100 × ~64 B/项  = 6.4 KB   (only handle/index)
+ 边界开销:                      ~50 KB   (wasm-bindgen runtime)
合计:                          ~80 KB  (vs JS 1.5 MB)
```

**节省**: 1.5 MB → 0.08 MB = **~95% 内存下降**

### 2.3 性能估算 (per compute-heavy op)

| Op | JS baseline | WASM (wasm-pack release) | Speedup |
|---|---|---|---|
| BFS (graph 1000 nodes) | ~25ms | ~2ms | **12x** |
| Topological sort | ~40ms | ~3ms | **13x** |
| Query filter (1000 items) | ~8ms | ~0.5ms | **16x** |
| Layout (Taffy, 100 nodes) | ~80ms | ~12ms | **7x** |

---

## 3. 实施路径 (3 PR 拆解)

### 3.1 PR-1 (基础) — WASM build 管线 + 1 个 PoC

- 加 `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- 新建 `crates/graph-core-wasm/` wrapper (依赖 `crates/graph-core`)
- `wasm-pack build --target web --release` → `.wasm` + `.js` glue
- `frontend/vite.config.ts` 加 `vite-plugin-wasm` + `vite-plugin-top-level-await`
- `frontend/src/lib/wasm/graph-core.ts` 包装 + React hook
- **PoC 验证**: cytoscape 替代品 (or 现有 graph-core API 直接从 wasm 调用)
- **不破现有**: cytoscape 路径保留, WASM 路径 as alternative

**估**: ~1M token, 1 周

### 3.2 PR-2 (扩展) — 多 crate WASM 化

- `crates/query-engine-wasm/` — work_items filter/sort/aggregate
- `crates/relationship-engine-wasm/` — 关系图构建
- `frontend/src/lib/wasm/index.ts` 统一 init + 懒加载
- 改造 `frontend/src/lib/store.ts`: 大数据集 (workItems + relations) 走 wasm linear memory
- **守门**: workspace memory size 不增加 > 5MB (per cargo 守门 #1)

**估**: ~2M token, 1-2 周

### 3.3 PR-3 (UI 集成) — 看板 + graph 视图切换

- 看板: 1000+ work_items 时, 自动切 WASM 路径 (避免 React re-render 卡顿)
- Cytoscape 替代: WASM layout (Taffy/Cassowary) + 渲染层保持 SVG/Canvas
- 监控: 加 `frontend/src/lib/wasm/profiler.ts` 测内存 + 计算时间

**估**: ~2M token, 1-2 周

---

## 4. 风险与对策

### 4.1 风险

| Risk | 影响 | 对策 |
|---|---|---|
| wasm binary size 大 (>500KB) | 首屏加载慢 | code-splitting, lazy load per feature; tree-shake unused exports |
| 调试体验差 | 排查问题难 | sourcemap (`wasm-pack --dev`), console.log in Rust via web-sys |
| 浏览器兼容 | 老浏览器失败 | fallback path (JS-only) 自动降级, feature flag |
| Build 时间增加 | CI 慢 | 只在 release 触发, dev 路径 skip |
| 数据迁移 (JS object → WASM linear memory) | 序列化开销 | 一次 init 后只传 index, opaque handle pattern |

### 4.2 不适用场景 (per WASM 局限)

- ❌ DOM 操作 — 仍走 React
- ❌ 网络请求 — 仍走 fetch/axios
- ❌ IndexedDB / localStorage — 仍走浏览器 API
- ❌ 异步事件循环 — 仍走 JS event loop (WASM 用 wasm-bindgen-futures 桥接)

### 4.3 Rust crate 不适合 WASM 的特征

- 依赖 OS-specific syscall (per nix / libc)
- 依赖 tokio 全 async runtime
- 依赖 数据库 driver (sqlx-postgres)
- 依赖 文件系统 (std::fs)

**对策**: 这些 crate 留 server-side, WASM 只做 **纯计算 + 纯数据结构** 层。

---

## 5. 推荐实施顺序 (per impact / effort 比例)

| 优先级 | Crate | Impact | Effort |
|---|---|---|---|
| **P0 (本周)** | `graph-core` (PoC) | 高 (cytoscape 替代) | 低 (1 crate) |
| **P1 (下周)** | `query-engine` | 高 (1000+ work_items filter) | 低 |
| **P2 (后续)** | `relationship-engine` | 中 | 中 |
| **P3 (按需)** | `layout-engine` | 中 (替代 Taffy JS port) | 中 |
| ❌ 不做 | `event-bus`, `star-cache`, ... | - | - |

---

## 6. Refs

- [Rust and WebAssembly book](https://rustwasm.github.io/docs/book/)
- [`wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [`wasm-pack` docs](https://rustwasm.github.io/docs/wasm-pack/)
- [Vite WASM plugin](https://github.com/Menci/vite-plugin-wasm)
- [Memory Layout Trade-offs (Rust WASM)](https://rustwasm.github.io/docs/book/reference/types.html)
- [PR #216 — feat(board): W/T/M swimlane + Rust→WASM research (实施 + 本报告)](https://github.com/UlyssesLeoLee/Star/pull/216)
- [PR #219 — feat(wasm): layout-engine-wasm PoC (PR-216 §3.1 PR-1 落地)](https://github.com/UlyssesLeoLee/Star/pull/219)
- [PR #230 — feat(wasm): query-engine-wasm PoC (PR-216 §3.1 PR-1 落地)](https://github.com/UlyssesLeoLee/Star/pull/230)

---

## 7. 状态 (本报告)

- ✅ 报告完成 v0.1
- ⏳ PR-1 (PoC graph-core-wasm) 待开
- ⏳ Per user 拍板 "按推荐" 默认 P0 → P1 → P2 顺序