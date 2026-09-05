// =====================================================================
// AppShell — 整体布局 (per docs/frontend/design/ui-redesign-multica-style.md §3 + §8.1)
// =====================================================================
// - 顶栏 64px sticky + 主区 calc(100vh - 64px)
// - 暗色背景 bg-bg (#0b0d10 per tailwind.config.ts)
// - 不接受 className — 强制样式 (per multica 严格规范)
// - SubNav 由 U2 接管 (per spec 任务分工), 留 placeholder comment
// =====================================================================
import { ReactNode } from "react";
import { AppHeader } from "./AppHeader";

export interface AppShellProps {
  children: ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  return (
    <div
      data-testid="app-shell"
      className="min-h-screen bg-bg text-ink"
    >
      <AppHeader />
      {/*
        SubNav slot — U2 接管, 会在路由为 /projects /agents /analytics 时
        自动渲染 180px sticky 左侧导航 (per §4). 当前 U1 不实现, 留 comment 占位.
      */}
      <main
        data-testid="app-main"
        style={{ minHeight: "calc(100vh - 64px)" }}
        className="px-6 sm:px-8 py-8 overflow-x-auto max-w-[1440px] mx-auto w-full"
      >
        {children}
      </main>
    </div>
  );
}
