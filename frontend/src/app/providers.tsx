// =====================================================================
// Providers — Toaster + QueryClient 顶层 client wrapper
// =====================================================================
// - 1 个 QueryClient instance 共享 (per §8.3)
// - Toaster 全局 toast 通知 (per §8.3 react-hot-toast)
// - onError 全局 handler 接 dev console 调试用
// =====================================================================
"use client";

import { useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "react-hot-toast";
import { ThemeProvider } from "@/components/theme/ThemeProvider";

export function Providers({ children }: { children: React.ReactNode }) {
  // useState 保证 QueryClient 在 React 生命周期内只创建 1 次
  const [client] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            // 多人协同 2s 轮询 — staleTime 不宜过长
            staleTime: 1_000,
            retry: 1,
            refetchOnWindowFocus: false,
          },
        },
      })
  );

  return (
    <ThemeProvider defaultTheme="light" themes={["light", "dark"]}>
      <QueryClientProvider client={client}>
        {children}
        <Toaster
          position="top-right"
          toastOptions={{
            duration: 3_000,
            style: {
              background: "#161b22",
              color: "#e6edf3",
              border: "1px solid #30363d",
              fontSize: "12px",
              fontFamily: "monospace",
            },
            success: { iconTheme: { primary: "#3fb950", secondary: "#0d1117" } },
            error:   { iconTheme: { primary: "#f85149", secondary: "#0d1117" } },
          }}
        />
      </QueryClientProvider>
    </ThemeProvider>
  );
}
