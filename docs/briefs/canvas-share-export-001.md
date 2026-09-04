# Brief: canvas-share-export-001

**Agent**: mavis (root 直实装, 0 子代理 per 守门 #9 #3 实证 5/5 RPC 不可靠)
**Phase**: P3-D.4 frontend Canvas UI 完整闭环
**Created**: 2026-09-04 23:05 JST
**Worktree**: `wt-canvas-share-export-002` @ 5b6af95 (从 canvas e2e 守门 commit 拉, 含 3 处 baseline fix)
**Token 预算**: ~0.4M (守门 #4 / #19 估算)

---

## 0. 触发

2026-09-04 23:00 JST Ulysses 拍板"补缺口 — Share + Export PNG 实装 (推荐)"; 23:04 JST 库选型拍板"html2canvas (推荐, 库依赖)"。
per 2026-09-04 19:14 JST 无限画布 status report §4 缺口 #6 (Share / Export PNG 按钮无 handler) + commit `5b6af95` 报告已知缺口 #4。

## 1. 范围 (in-scope)

### 1.1 装 html2canvas (前端依赖)

| 包 | 版本 | 用途 |
|---|---|---|
| `html2canvas` | 1.4.1 (最新 stable) | DOM → canvas → PNG Blob |

### 1.2 page.tsx 改 2 handler

#### Share (line 61)

```ts
const onShare = async () => {
  const url = window.location.href;
  await navigator.clipboard.writeText(url);
  toast.success("Canvas link copied to clipboard");
};
```

#### Export PNG (line 62)

```ts
const canvasContainerRef = useRef<HTMLDivElement>(null);
const onExportPng = async () => {
  if (!canvasContainerRef.current) return;
  const canvas = await html2canvas(canvasContainerRef.current, {
    backgroundColor: "#0b0d10",
    scale: 2,
    logging: false,
  });
  canvas.toBlob((blob) => { /* download */ }, "image/png");
};
```

#### CanvasView 容器加 ref (line 67 in page.tsx)

```tsx
<div ref={canvasContainerRef} className="flex-1 relative">
  <CanvasView ... />
</div>
```

### 1.3 CanvasView.tsx 加 1 testid

| testid | 位置 | 用途 |
|---|---|---|
| `canvas-container` | CanvasView line 372 `<div className="relative w-full h-full ...">` | e2e 验证 + html2canvas 抓取起点 (但实际上 page.tsx 用 ref, testid 仅 e2e 验证) |

### 1.4 frontend/e2e/canvas-share-export.spec.ts 新建 (3 case)

| # | 守门项 |
|---|---|
| 1 | Share 按钮点击 → clipboard 写入当前 URL (navigator.clipboard.readText 验证) |
| 2 | Export PNG 按钮点击 → 触发 download (Playwright `page.waitForEvent('download')` 验证文件名 + .png 后缀) |
| 3 | Share 按钮点击后 → toast 出现 (react-hot-toast 默认 `[role="status"]` 验证) |

## 2. 范围外 (out-of-scope)

- 协作 presence 真实接入 (硬编码 "3 online") — 被 P3-C Realtime 选型拍板阻塞
- 真实数据接入 (canvas 数据仍 mock) — AGENTS.md §7 #2 阻塞
- 跨 device 同步 + 协作编辑 — 后续 P3 阶段
- Share modal 配权限 (per 协作域) — 简单剪贴板覆盖 MVP

## 3. 已知缺口 (per 守门 #11 缺标比错标安全)

- **html2canvas foreignObject 限制** (1.4.1 已知): CanvasView 用了 `<foreignObject>` 嵌入 sticky_note / text / work_item_card 文字 — html2canvas **不渲染 foreignObject 内容**, sticky_note 文字会丢失, 只剩彩色矩形. 接受此限制, 改用 SVG-native `<text>` 重写 sticky_note 留给 P2. test 2 验证 PNG 已下载即可, 不验内容完整性.
- **toast 不可见** 在 playwright 默认 viewport (1280x720) — react-hot-toast 默认 top-right, 可能被 sidebar 遮挡. test 3 不验证 toast 文本, 只验证 role="status" 节点存在.
- **navigator.clipboard 需 HTTPS / localhost** — dev mode 跑 localhost OK, CI runner 需配 permission. Playwright 配 `context.grantPermissions(['clipboard-read', 'clipboard-write'])`.
- **页面 top header (`page.tsx:46-64`) 不在 export PNG 范围** — 用 ref 指 `<div className="flex-1 relative">` (line 67), 不含 header. 用户要 header 导出需手动 expand ref 范围, 留 P2.

## 4. 守门硬约束 (per 守门 #1)

- `cd frontend && pnpm typecheck` 0 错 (新增文件范围)
- `cd frontend && pnpm test` (vitest) 0 失败 (排除 pre-existing refactor baseline)
- `cd frontend && pnpm test:e2e -- canvas-share-export` 3/3 全过
- 0 子代理调用 (per 守门 #9 #3)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 8/27 19:39 JST 授权)
- 1 commit 落档 (per 守门 #15 docs 同步饱和, 等新事件触发)

## 5. 落地清单

| # | 文件 | 内容 |
|---|---|---|
| 1 | `frontend/package.json` + `pnpm-lock.yaml` | +html2canvas@1.4.1 |
| 2 | `frontend/src/app/canvas/[id]/page.tsx` | +ref + 2 handler + import 调整 (~30 行) |
| 3 | `frontend/src/components/CanvasView.tsx` | +1 testid (`canvas-container`) 1 行 |
| 4 | `frontend/e2e/canvas-share-export.spec.ts` | 新建 3 case, ~80 行 |
| 5 | `frontend/vitest.config.ts` | +1 行 exclude (playwright-only spec) |
| 6 | `docs/automation-design.md` §4 任务卡表追加 1 行 (守门 #21) |
| 7 | `scripts/automation/registry.md` 索引追加 1 行 (守门 #21) |

预估 1 commit, 7 file, ~110 行新增。
