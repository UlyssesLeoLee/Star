import type { Metadata } from "next";
import "./globals.css";
import { Sidebar } from "@/components/Sidebar";
import { Providers } from "./providers";

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
  themeColor: "#0b0d10",
  manifest: "/manifest.json",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    // suppressHydrationWarning: next-themes 在 client 注入 .dark class, SSR 不一致是预期.
    <html lang="en" suppressHydrationWarning>
      <body className="min-h-screen bg-[color:var(--color-surface)] text-[color:var(--color-text)]">
        <Providers>
          <div className="flex min-h-screen">
            <Sidebar />
            <div className="flex-1 flex flex-col min-w-0">
              {/* Topbar 已在 (app) 路由下由 AppShell/AppHeader 渲染, 移除外层重复 (per 2026-08-29 17:18 JST) */}
              <main className="flex-1 px-6 py-5 overflow-x-auto">{children}</main>
            </div>
          </div>
        </Providers>
      </body>
    </html>
  );
}
