import type { Metadata } from "next";
import "./globals.css";
import { Sidebar } from "@/components/Sidebar";
import { Topbar } from "@/components/Topbar";
import { Providers } from "./providers";

export const metadata: Metadata = {
  title: "Star — Vibe Coding Work Management",
  description:
    "Star Platform — Control Plane for 25-module Vibe Coding Work Management SaaS",
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
              <Topbar />
              <main className="flex-1 px-6 py-5 overflow-x-auto">{children}</main>
            </div>
          </div>
        </Providers>
      </body>
    </html>
  );
}
