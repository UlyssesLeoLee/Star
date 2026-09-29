# Rust → WASM 浏览器 build 集成指南 (P2 — 1 page integration prep)

> **date**: 2026-09-30 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: PR-233 follow-up of PR #229/#231 (frontend hooks ready, need build pipeline)
> **scope**: wasm-pack build + Next.js static serving + 1 page integration
> **status**: 📋 指南 v0.1 (步骤 ready, 待 wasm-pack install + 1 page 实装)

---

## 0. 前置状态

✅ **已完成**:
- PR #219: `crates/layout-engine-wasm/` PoC (10/10 tests)
- PR #229: `frontend/src/lib/wasm/layout-engine.ts` (useLayoutEngine hook)
- PR #230: `crates/query-engine-wasm/` PoC (7/7 tests)
- PR #231: `frontend/src/lib/wasm/query-engine.ts` (useQueryEngine hook)

⏳ **待完成 (本 PR 范围)**:
1. wasm-pack install + Rust → WASM build pipeline
2. 1 page 集成 (推荐: `/worktree-canvas` 因为有 cytoscape + graph)
3. Next.js public/ 静态 serving WASM binary
4. 性能 A/B test (JS-only vs WASM)

---

## 1. wasm-pack 安装

### 1.1 系统要求
- Rust 1.75+ (per workspace rust-version)
- Node.js 18+ (per frontend/package.json "node": 26.6.1)
- wasm32 target: `rustup target add wasm32-unknown-unknown`

### 1.2 安装命令 (per https://rustwasm.github.io/wasm-pack/installer/)

```bash
# macOS / Linux (one-line installer)
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# 或 cargo install
cargo install wasm-pack

# 验证
wasm-pack --version
# 期望输出: wasm-pack 0.12.x 或更新
```

### 1.3 添加 wasm32 target

```bash
# 全局 (一次性)
rustup target add wasm32-unknown-unknown

# 验证
rustup target list --installed | grep wasm32
# 期望输出: wasm32-unknown-unknown (unknown)
```

---

## 2. WASM build 流水线

### 2.1 单个 crate build (e.g. layout-engine-wasm)

```bash
cd crates/layout-engine-wasm
wasm-pack build --target web --release

# 输出位置: pkg/
#   pkg/
#   ├── layout_engine_wasm_bg.wasm    (~50KB binary)
#   ├── layout_engine_wasm.js         (~3KB glue)
#   ├── layout_engine_wasm.d.ts       (TypeScript types)
#   └── package.json
```

### 2.2 复制到 Next.js public/

```bash
# 单 crate
cp crates/layout-engine-wasm/pkg/* frontend/public/wasm/

# 或全部 WASM crates (脚本化)
for crate in layout-engine-wasm query-engine-wasm; do
  cargo build --manifest-path crates/$crate/Cargo.toml --target wasm32-unknown-unknown --release
  mkdir -p frontend/public/wasm
  cp crates/$crate/pkg/* frontend/public/wasm/
done
```

### 2.3 Next.js 自动 serving

Next.js 默认 serve `public/` 下静态文件,无需额外配置:
- `/wasm/layout_engine_wasm_bg.wasm` → 自动 serving
- `/wasm/layout_engine_wasm.js` → 自动 serving
- WASM 文件 mime type 自动 `application/wasm`

验证:
```bash
cd frontend
pnpm dev
curl -I http://localhost:3000/wasm/layout_engine_wasm_bg.wasm
# 期望: HTTP/1.1 200 OK, Content-Type: application/wasm
```

---

## 3. 1 Page 集成 — `/worktree-canvas` 推荐

### 3.1 选型理由

per docs/architecture/2026-09-29-upgrade/rust-app-end-research.md:
- ✅ 重计算图布局 (cytoscape + cose-bilkent)
- ✅ 已有 React Three Fiber + Cytoscape 集成
- ✅ Worktree 树可视化需求
- ✅ 100+ 节点场景性能瓶颈明显

### 3.2 集成步骤 (预估 1-2 周)

#### Step 1: 修改 worktree-canvas/page.tsx

```tsx
"use client";

import { useLayoutEngine } from "@/lib/wasm/layout-engine";
import { useEffect, useState } from "react";

export default function WorktreeCanvasPage() {
  const { ready, pickAlgorithm, visualDistance } = useLayoutEngine();
  const [nodes, setNodes] = useState<Node[]>([]);
  
  // WASM 算法选择 (代替 cytoscape 自动选)
  const algorithm = ready 
    ? pickAlgorithm("tree", nodes.length) 
    : "dagre"; // JS fallback
  
  // visualDistance 算节点距离
  const distance = (commitD: number) => 
    ready ? (visualDistance(commitD) ?? Math.log(commitD + 1) * 30) 
         : Math.log(commitD + 1) * 30;
  
  return (
    <div>
      <h1>Worktree Canvas (WASM: {ready ? "✅" : "❌ JS fallback"})</h1>
      <p>Algorithm: {algorithm}</p>
      {/* ... */}
    </div>
  );
}
```

#### Step 2: 用 WASM 算法替换 cytoscape layout

```tsx
const layoutOptions = {
  name: algorithm, // "dagre" | "d3_force" | "elk" — 来自 WASM
  animate: true,
  // visualDistance 影响 edge length
  edgeLength: distance(10),
};
```

#### Step 3: A/B test 性能对比

- 加载 100 / 500 / 1000 个节点
- 测量 layout 计算时间 (JS-only vs WASM)
- 期望加速: **12-16x** per research doc §3.1 P0

---

## 4. 自动化 build 脚本 (CI/CD)

### 4.1 GitHub Actions (per .github/workflows/)

```yaml
# .github/workflows/wasm-build.yml
name: WASM Build
on:
  push:
    paths:
      - 'crates/*-wasm/**'
      - 'frontend/public/wasm/**'

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install wasm32 target
        run: rustup target add wasm32-unknown-unknown
      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
      - name: Build layout-engine-wasm
        run: |
          cd crates/layout-engine-wasm
          wasm-pack build --target web --release
      - name: Build query-engine-wasm
        run: |
          cd crates/query-engine-wasm
          wasm-pack build --target web --release
      - name: Copy to Next.js public
        run: |
          mkdir -p frontend/public/wasm
          cp crates/layout-engine-wasm/pkg/* frontend/public/wasm/
          cp crates/query-engine-wasm/pkg/* frontend/public/wasm/
      - name: Commit
        run: |
          git config user.email "agent@minimax.local"
          git config user.name "minimax-agent"
          git add frontend/public/wasm/
          git commit -m "ci(wasm): auto-build WASM binaries from PR-219/230" || echo "no changes"
          git push
```

### 4.2 本地 dev 脚本

```bash
# scripts/build-wasm.sh (新增)
#!/bin/bash
set -e

# Install if missing
if ! command -v wasm-pack &> /dev/null; then
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Add target
rustup target add wasm32-unknown-unknown

# Build all *-wasm crates
for crate in crates/*-wasm; do
  echo "Building $crate..."
  (cd "$crate" && wasm-pack build --target web --release)
done

# Copy to frontend public
mkdir -p frontend/public/wasm
for crate in crates/*-wasm; do
  cp "$crate"/pkg/* frontend/public/wasm/
done

echo "✅ WASM build complete. Files in frontend/public/wasm/:"
ls -la frontend/public/wasm/
```

---

## 5. 性能预期 (per docs/architecture/2026-09-28-upgrade §3.1)

| Op | JS baseline | WASM expected | Speedup |
|---|---|---|---|
| Layout 100 nodes | ~80ms | ~12ms | **7x** |
| Layout 1000 nodes | ~2500ms | ~180ms | **14x** |
| Search DSL parse | ~3ms (JS string) | ~0.2ms (Rust regex) | **15x** |
| Filter 1000 work items | ~8ms | ~0.5ms | **16x** |
| Memory (1000 items) | ~25 MB | ~3 MB | **~8x** |

---

## 6. 已知坑 (per session memory)

### 6.1 wasm-pack build 失败

**坑 1**: `wasm-pack build` requires `wasm-bindgen` dep in Cargo.toml
```toml
[lib]
crate-type = ["cdylib", "rlib"]  # 必须有 cdylib
```

**坑 2**: wasm-bindgen-test only works on wasm32 target
```bash
wasm-pack test --node    # wasm32 test
cargo test --lib          # native test
```

### 6.2 Next.js WASM serving

**坑 1**: Next.js 默认 Content-Type 需验证
```bash
curl -I http://localhost:3000/wasm/test.wasm
# 期望: Content-Type: application/wasm (per Next.js 14+)
```

**坑 2**: WSL2 + Windows 浏览器 fetch WASM 可能 hit Cross-Origin 头
- dev server 默认无 COOP/COEP
- production 部署需加 COOP/COEP for SharedArrayBuffer + threads

### 6.3 wasm-bindgen 0.2 ABI

**坑 1**: `JsValue::from_str` (我们的方案) vs `JsValue::from` (更通用)
- `from_str` works on native + wasm ✅ (per PR #229)
- `from` only wasm ⚠️ (per PR #230 教训)

**坑 2**: 字符串传递 4-byte length prefix
- 必须用 `DataView.getUint32(ptr - 4, true)` 读 length (little-endian)
- 否则字符串 decode 错位

---

## 7. 后续 PR 计划

| PR | 内容 | 估 |
|---|---|---|
| **PR-233** (本 PR) | wasm-pack build + Next.js serving + build script | 1 周 |
| **PR-234** | 1 page integration (worktree-canvas) | 2 周 |
| **PR-235** | A/B test 性能对比 + profiling | 1 周 |
| **PR-236** | relationship-engine-wasm PoC | 1 周 |
| **PR-237** | query-engine-wasm DslParser 集成 (完整 parser) | 1 周 |

---

## 8. 状态 (本指南 v0.1)

- ✅ Frontend hooks ready (PR #229 + #231)
- ✅ Rust crates WASM-ready (PR #219 + #230, 17/17 tests pass)
- ⏳ wasm-pack install 待执行 (session 内 30min)
- ⏳ 1 page 集成待执行 (PR #234)
- ⏳ A/B test 待执行 (PR #235)

**Next step**: session 内执行 wasm-pack install + 第 1 个 build + 第 1 page 集成。