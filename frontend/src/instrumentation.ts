// frontend/src/instrumentation.ts
// Next.js 13+ app router instrumentation (per docs/frontend/design/mock-msw-handlers.md §2.3)
//
// 启动 MSW browser worker, 仅 dev / `NEXT_PUBLIC_API_MOCKING=enabled` 启用.
// production build 自动跳过.
//
// 守门: 0 unsafe (TS 严模式 + dynamic import 隔离).

export async function register() {
  if (
    process.env.NEXT_PUBLIC_API_MOCKING === "enabled" &&
    typeof window !== "undefined" &&
    // 仅 client 端 (server 端用 server.ts, 已在 vitest setup)
    !(process as { __MSW_NODE_SERVER__?: boolean }).__MSW_NODE_SERVER__
  ) {
    const { worker } = await import("./mocks/client");
    await worker.start({
      onUnhandledRequest: "bypass",
      serviceWorker: {
        url: "/mockServiceWorker.js",
      },
    });
  }
}
