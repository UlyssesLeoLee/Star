// =====================================================================
// next.config.js — U5 (multica-style route consolidation)
//
// What this file does:
//   1. Wires the 26 legacy → 6-panel redirect map from src/lib/redirects.ts
//      into Next.js's redirects() async function. This is the canonical
//      Next.js 14 way to add 307/308 redirects at the routing layer.
//   2. Keeps the W2 (Gantt) typedRoutes=false / ignoreBuildErrors safety
//      net from main HEAD — these are pre-existing tech debt unrelated
//      to U5 and should be removed by a follow-up worker (per next.config.js
//      git-blame 3b834b4).
//
// Order of imports: we require() the .ts compile-on-the-fly using
// @swc-node/register at runtime, BUT next.config.js is loaded by Next's
// bundler BEFORE TypeScript compilation kicks in. So we instead duplicate
// the redirect list in this file using a require() against a tiny .js
// shim. The shim re-exports the typed list. This keeps next.config.js
// strictly JavaScript (a Next.js requirement) while still sourcing the
// truth from src/lib/redirects.ts via the e2e test.
//
// Why not just inline the 26 redirects here? Because the vitest spec at
// e2e/redirects.spec.ts imports the .ts module directly and we want a
// single source of truth for both runtime (next.config.js) and tests
// (vitest). The shim re-exports the same list.
// =====================================================================

/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  experimental: { typedRoutes: false },
  // W2 (Gantt) 临时跳过 TS 类型检查:
  // 已知 main HEAD 3e182d9 pre-existing 类型问题:
  //   - Sidebar.tsx NavItem.icon 类型 与 lucide-react icon 类型不匹配 (ForwardRefExoticComponent)
  //   - work-item/page.tsx / worktree/page.tsx 已有 missing import 修复 (本次提交)
  // 不属于 W2 任务范围, 期待 W5 (d3d40fb) merge 后移除此 ignoreBuildErrors
  typescript: { ignoreBuildErrors: true },
  async redirects() {
    // Load the redirect list from the JS shim that re-exports the
    // canonical .ts list. This avoids duplicating 26 entries in two
    // places and lets the e2e spec validate the same array.
    const { LEGACY_REDIRECTS } = require("./src/lib/redirects.shim.cjs");
    return LEGACY_REDIRECTS;
  },
};

module.exports = nextConfig;
