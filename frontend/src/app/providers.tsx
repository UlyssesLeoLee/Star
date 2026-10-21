// =====================================================================
// Providers — Toaster + QueryClient 顶层 client wrapper
// =====================================================================
// - 1 个 QueryClient instance 共享 (per §8.3)
// - Toaster 全局 toast 通知 (per §8.3 react-hot-toast)
// - onError 全局 handler 接 dev console 调试用
// =====================================================================
/*
@cypher schema=1 source_sha256=71bd74bab4a6d5534dd39c4ef820adf0e043fbc29d55e268c949f93860b0f461
MERGE (self:File {path:"frontend/src/app/providers.tsx"})
MERGE (providers:Symbol {id:"frontend/src/app/providers.tsx::Providers",kind:"function"})
MERGE (runProvider:Symbol {id:"frontend/src/lib/run/runDirectorySession.tsx::RunDirectoryProvider",kind:"function"})
MERGE (groupProvider:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::WorktreeGroupApiProvider",kind:"function"})
MERGE (navigationGuard:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::UnsavedNavigationGuardProvider",kind:"component"})
MERGE (self)-[:DEFINES]->(providers)
MERGE (providers)-[:CALLS]->(runProvider)
MERGE (providers)-[:CALLS]->(groupProvider)
MERGE (providers)-[:CALLS]->(navigationGuard)
@endcypher
*/
"use client";

import { useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "react-hot-toast";
import { ThemeProvider } from "@/components/theme/ThemeProvider";
import { RunDirectoryProvider, type RunDirectoryHostSession } from "@/lib/run/runDirectorySession";
import { WorktreeGroupApiProvider } from "@/lib/group/groupProjection";
import { UnsavedNavigationGuardProvider } from "@/lib/navigation/UnsavedNavigationGuard";

export function Providers({ children, runDirectorySession = null }: { children: React.ReactNode; runDirectorySession?: RunDirectoryHostSession | null }) {
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
    <UnsavedNavigationGuardProvider>
      <ThemeProvider defaultTheme="light" themes={["light", "dark", "christmas", "charisma", "shanghai", "cel"]}>
        <QueryClientProvider client={client}>
          <RunDirectoryProvider session={runDirectorySession}>
            {runDirectorySession ? (
              <WorktreeGroupApiProvider
                getAccessToken={runDirectorySession.getAccessToken}
                sessionKey={runDirectorySession.generation}
                fetcher={runDirectorySession.fetcher}
              >
                {children}
              </WorktreeGroupApiProvider>
            ) : children}
          </RunDirectoryProvider>
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
    </UnsavedNavigationGuardProvider>
  );
}
