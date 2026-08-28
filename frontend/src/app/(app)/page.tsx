// =====================================================================
// (app) page — 根路由 redirect → /inbox (per U5 spec + design §2)
// =====================================================================
// - 临时 placeholder — U5 会在 next.config.js / middleware.ts 配真正的 redirect
// - 此处用 client-side router.replace 模拟
// - 与 22 旧路由 redirect 整合由 U5 处理 (per 任务已知缺口)
// =====================================================================
"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";

export default function AppRootPage() {
  const router = useRouter();

  useEffect(() => {
    router.replace("/inbox");
  }, [router]);

  return (
    <div
      data-testid="app-root-redirect"
      className="flex items-center justify-center h-full min-h-[60vh] text-ink-dim"
    >
      <span className="text-sm font-mono">Redirecting to /inbox…</span>
    </div>
  );
}
