import type { Metadata, Viewport } from "next";
import "./globals.css";
import { Sidebar } from "@/components/Sidebar";
import { Providers } from "./providers";
import { I18nProvider } from "@/lib/i18n";
import { Toaster } from "react-hot-toast";
import { CommandBar } from "@/components/CommandBar"; // per DRIFT-α-020 (2026-08-31 12:07 JST 试水)
import { MobileHeader } from "@/components/MobileHeader";
import { MobileBottomNav } from "@/components/MobileBottomNav";
import { PwaBoot } from "@/components/PwaBoot";
import { PwaUpdateToast } from "@/components/PwaUpdateToast";
import { PwaInstallPrompt } from "@/components/PwaInstallPrompt";
import { OnboardingGuard } from "@/components/OnboardingGuard";

export const metadata: Metadata = {
  title: "Star — Vibe Coding Work Management",
  description:
    "Star Platform — Control Plane for 25-module Vibe Coding Work Management SaaS",
  // Icon 全部来自根目录 icon.png (per 2026-08-29 19:01 JST 拍板), Next.js 14 自动从 src/app/icon.png 生成 favicon
  icons: {
    icon: [
      { url: "/favicon.ico", sizes: "32x32 16x16", type: "image/x-icon" },
      { url: "/icon-512.png", sizes: "512x512", type: "image/png" },
    ],
    apple: "/apple-touch-icon.png",
  },
  manifest: "/manifest.json",
  appleWebApp: {
    capable: true,
    statusBarStyle: "black-translucent",
    title: "Star",
  },
  formatDetection: { telephone: false },
};

// themeColor 必须在 viewport export (per Next.js 14+ 要求, 否则触发 /offline build 告警)
export const viewport: Viewport = {
  themeColor: "#0b0d10",
  width: "device-width",
  initialScale: 1,
  maximumScale: 5,
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    // suppressHydrationWarning: next-themes 在 client 注入 .dark class, SSR 不一致是预期.
    // 另: I18nProvider 会在 mount 后同步 <html lang="...">, 故此处先用默认 zh-CN 避免 hydration mismatch
    <html lang="zh-CN" suppressHydrationWarning>
      <body className="min-h-screen bg-[color:var(--color-surface)] text-[color:var(--color-text)] antialiased transition-colors">
        <I18nProvider>
          <Providers>
            <div className="flex min-h-screen">
              <Sidebar />
              <div
                className="flex-1 flex flex-col min-w-0 md:pb-0 relative"
                style={{
                  // <768px: viewport 高度 - MobileHeader - Bottom Nav, 自带滚动
                  height: "100dvh",
                  overflowY: "auto",
                  paddingBottom: "calc(4rem + env(safe-area-inset-bottom))",
                }}
              >
                {/* <768px: MobileHeader 顶栏 (per 2026-09-01 PHASE-MOBILE-PWA)
                    ≥768px: AppHeader 顶栏由各 page 内部 AppShell 提供,
                    此处 RootLayout 不重复 mount,避免双顶栏 */}
                <MobileHeader />
                {children}
              </div>
            </div>
            {/* <768px 底部 5 域快捷导航 (per 2026-09-01 PHASE-MOBILE-PWA)
                fixed bottom-0 + z-[9999] 保证不被 inbox 列表等内 stacking 拦截,
                main 区域 height: 100dvh + overflowY auto 让 list 自身滚动 */}
            <MobileBottomNav />
            {/* PWA Service Worker 注册 (production only) */}
            <PwaBoot />
            {/* PWA 更新提示 toast (监听 star:pwa-updated) */}
            <PwaUpdateToast />
            {/* PWA install prompt (beforeinstallprompt + iOS 引导) */}
            <PwaInstallPrompt />
            {/* \u9996\u6b21\u542f\u52a8 onboarding \u5f15\u5bfc (per ADR-0042, 2026-09-02 08:01 JST 4 \u62cd\u677f) */}
            <OnboardingGuard />
            {/* Global toast: GanttBar 拖拽冲突 / Board 列删除 / 主题切换等全局反馈
                - 暗色模式背景, 适配 next-themes
                - duration 4000 (默认), Gantt 冲突可单独传 duration:1500
                - position top-right 不挡 Sidebar + MainWorkArea
                per 2026-08-29 19:24 JST 实装 */}
            <Toaster
              position="top-right"
              toastOptions={{
                duration: 4000,
                style: {
                  background: "var(--color-surface-2)",
                  color: "var(--color-text)",
                  border: "1px solid var(--color-border)",
                  fontSize: "13px",
                },
                success: { iconTheme: { primary: "#10b981", secondary: "#fff" } },
                error: { iconTheme: { primary: "#ff3366", secondary: "#fff" } },
              }}
            />
            {/* CommandBar: ⌘K 全局命令面板消费组件 (per DRIFT-α-020 修复) */}
            <CommandBar />
          </Providers>
        </I18nProvider>
      </body>
    </html>
  );
}

