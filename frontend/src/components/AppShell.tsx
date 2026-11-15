/*
@cypher schema=1 source_sha256=ca77642bc0956b24ca34f1e852b4c22e04e0cda80b63370b10a8ad7cc00fb817
MERGE (self:File {path:"frontend/src/components/AppShell.tsx"})
@endcypher
*/
// =====================================================================
// AppShell — 整体布局 (per 前端布局设计说明 §3 + §8.1)
// =====================================================================
// - 顶栏 64px sticky + 主区 calc(100vh - 64px)
// - 暗色背景 bg-bg (#0b0d10 per tailwind.config.ts)
// - 不接受 className — 统一强制样式
// - SubNav 由 U2 接管 (per spec 任务分工), 留 placeholder comment
// - fullBleed: 无限画布类页面 (/agent-view) 跳过 main 的 padding/max-width,
//   自己管理 h-[calc(100vh-4rem)] 布局 (per 2026-09-06 导航栏遮挡修复)
// - wide: 保留 padding + 正常滚动, 但去掉 max-w-[1440px] mx-auto 居中 —
//   宽屏下 mx-auto 会在 sidebar 右侧留出很大的左侧空白 (per 2026-09-07 反馈:
//   项目页面左边距过大, 应接近 agent-view 的贴边观感, 但仍需正常文档滚动/不能 overflow-hidden)
// =====================================================================
import { ReactNode, Suspense } from "react";
import { AppHeader } from "./AppHeader";
import { RightChatRail } from "./chat/RightChatRail";
import { PhysisBackdrop } from "./effects/PhysisBackdrop";
import { PhysisHud } from "./effects/PhysisHud";

export interface AppShellProps {
  children: ReactNode;
  fullBleed?: boolean;
  wide?: boolean;
}

export function AppShell({ children, fullBleed = false, wide = false }: AppShellProps) {
  return (
    <div
      data-testid="app-shell"
      className="min-h-screen bg-bg bg-[var(--cel-bg)] text-ink text-[var(--cel-text-primary)] transition-colors duration-200 relative"
    >
      {/* Physis 3D 物理背景层 + 游戏化 HUD (仅 physis 主题挂载) */}
      <PhysisBackdrop />
      <PhysisHud />
      {/* 3渲2 Ambient Screentone & Glow Backdrops */}
      <div className="fixed inset-0 pointer-events-none z-0 overflow-hidden">
        <div className="absolute inset-0 bg-screentone opacity-15" />
        <div className="absolute -top-40 -left-40 w-96 h-96 bg-[var(--cel-crimson, var(--err-DEFAULT))] rounded-full blur-[160px] opacity-10" />
        <div className="absolute -bottom-40 -right-40 w-96 h-96 bg-[var(--cel-cyan, var(--accent-DEFAULT))] rounded-full blur-[160px] opacity-10" />
      </div>
      {/* per 2026-11-12: AppHeader 用 useSearchParams 判定 query-href 高亮,
          必须包 Suspense (Next 16 静态 prerender bailout 规则, 与 Sidebar 同) */}
      <Suspense fallback={null}>
        <AppHeader />
      </Suspense>
      {/*
        SubNav slot — U2 接管, 会在路由为 /projects /agents /analytics 时
        自动渲染 180px sticky 左侧导航 (per §4). 当前 U1 不实现, 留 comment 占位.
      */}
      {/* 主区 + 全局右侧可折叠聊天栏 (per 2026-11-14: 所有界面通用;
          header 76px → 高度按 76px 对齐, 聊天栏顶端不被截断) */}
      <div className="relative z-10 flex" style={{ height: "calc(100vh - 76px)" }}>
        <main
          data-testid="app-main"
          style={fullBleed ? { minHeight: "calc(100vh - 76px)" } : { minHeight: "calc(100vh - 76px)" }}
          className={
            fullBleed
              ? "overflow-hidden w-full"
              : wide
                ? "px-4 sm:px-6 lg:px-8 py-6 sm:py-8 overflow-y-auto w-full"
                : "px-4 sm:px-6 lg:px-8 py-6 sm:py-8 overflow-y-auto max-w-[1440px] mx-auto w-full"
          }
        >
          {children}
        </main>
        <RightChatRail />
      </div>
    </div>
  );
}
