// =====================================================================
// AppShell — 整体布局 (per docs/frontend/design/ui-redesign-multica-style.md §3 + §8.1)
// =====================================================================
// - 顶栏 64px sticky + 主区 calc(100vh - 64px)
// - 暗色背景 bg-bg (#0b0d10 per tailwind.config.ts)
// - 不接受 className — 强制样式 (per multica 严格规范)
// - SubNav 由 U2 接管 (per spec 任务分工), 留 placeholder comment
// - fullBleed: 无限画布类页面 (/agent-view) 跳过 main 的 padding/max-width,
//   自己管理 h-[calc(100vh-4rem)] 布局 (per 2026-09-06 导航栏遮挡修复)
// =====================================================================
import { ReactNode } from "react";
import { AppHeader } from "./AppHeader";

export interface AppShellProps {
  children: ReactNode;
  fullBleed?: boolean;
}

export function AppShell({ children, fullBleed = false }: AppShellProps) {
  return (
    <div
      data-testid="app-shell"
      className="min-h-screen bg-bg bg-[var(--cel-bg)] text-ink text-[var(--cel-text-primary)] transition-colors duration-200 relative"
    >
      {/* 3渲2 Ambient Screentone & Glow Backdrops */}
      <div className="fixed inset-0 pointer-events-none z-0 overflow-hidden">
        <div className="absolute inset-0 bg-screentone opacity-15" />
        <div className="absolute -top-40 -left-40 w-96 h-96 bg-[var(--cel-crimson,#ff184c)] rounded-full blur-[160px] opacity-10" />
        <div className="absolute -bottom-40 -right-40 w-96 h-96 bg-[var(--cel-cyan,#00f0ff)] rounded-full blur-[160px] opacity-10" />
      </div>
      <AppHeader />
      {/*
        SubNav slot — U2 接管, 会在路由为 /projects /agents /analytics 时
        自动渲染 180px sticky 左侧导航 (per §4). 当前 U1 不实现, 留 comment 占位.
      */}
      <main
        data-testid="app-main"
        style={fullBleed ? { height: "calc(100vh - 64px)" } : { minHeight: "calc(100vh - 64px)" }}
        className={
          fullBleed
            ? "relative z-10 overflow-hidden w-full"
            : "relative z-10 px-4 sm:px-6 lg:px-8 py-6 sm:py-8 overflow-x-auto max-w-[1440px] mx-auto w-full"
        }
      >
        {children}
      </main>
    </div>
  );
}
