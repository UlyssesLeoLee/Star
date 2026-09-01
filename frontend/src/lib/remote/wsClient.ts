// Star Remote Control WebSocket Client (per 2026-09-01 PHASE-MOBILE-PWA v0.2)
//
// 走 Star BFF relay 模式: 手机 → /v1/remote/{kind}/{id} WebSocket →
//   BFF (认证 / 审计 / 限流) → local-runtime agent 进程 →
//   VNC server / shell exec / SFTP subsystem
//
// MVP 阶段: WebSocket 连接失败时降级为 mock demo (让 UI 可点可看),
// 真实后端落地后 v1.0 切真。

export type RemoteKind = "desktop" | "terminal" | "files";

export interface RemoteEndpoint {
  /** local runtime id (lr-001 等) */
  runtimeId: string;
  kind: RemoteKind;
  /** WebSocket URL; mock 模式下用 null 走本地 demo */
  url: string | null;
}

export interface RemoteSession {
  id: string;
  kind: RemoteKind;
  runtimeId: string;
  status: "connecting" | "connected" | "disconnected" | "error";
  startedAt: number;
  lastError?: string;
}

/**
 * 构造 WebSocket URL。
 *
 * 后端契约 (per /v1/remote WebSocket relay 提案):
 *   desktop:  /v1/remote/desktop/{runtimeId}    (binary frames = RFB protocol)
 *   terminal: /v1/remote/terminal/{runtimeId}   (text frames, JSON)
 *   files:    /v1/remote/files/{runtimeId}      (text frames, JSON)
 */
export function buildRemoteUrl(
  kind: RemoteKind,
  runtimeId: string,
  basePath = "/v1/remote",
): string {
  if (typeof window === "undefined") {
    // SSR 时不构造(客户端再调)
    return `${basePath}/${kind}/${runtimeId}`;
  }
  const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
  return `${proto}//${window.location.host}${basePath}/${kind}/${runtimeId}`;
}

/**
 * 判断当前是否 mock 模式 (NEXT_PUBLIC_API_MOCKING 启用 或 WebSocket 不可用)
 *
 * - SSR 永远返回 true
 * - 客户端: 检查环境变量 + 浏览器能力
 */
export function isRemoteMockMode(): boolean {
  if (typeof window === "undefined") return true;
  // MSW 模式已接管 fetch,WS 不接管 → 默认 mock
  if (process.env.NEXT_PUBLIC_API_MOCKING === "enabled") return true;
  if (!("WebSocket" in window)) return true;
  return false;
}

/**
 * 通用 WebSocket 连接建立 (Promise 化)
 *
 * 10s 超时, 错误抛给上层。
 */
export function connectRemote(
  url: string,
  options: { protocols?: string | string[]; timeoutMs?: number } = {},
): Promise<WebSocket> {
  const { protocols, timeoutMs = 10_000 } = options;
  return new Promise((resolve, reject) => {
    const ws = protocols ? new WebSocket(url, protocols) : new WebSocket(url);
    const timer = setTimeout(() => {
      try { ws.close(); } catch { /* ignore */ }
      reject(new Error(`Remote WS timeout after ${timeoutMs}ms`));
    }, timeoutMs);

    ws.onopen = () => {
      clearTimeout(timer);
      resolve(ws);
    };
    ws.onerror = (e) => {
      clearTimeout(timer);
      reject(new Error(`Remote WS error: ${(e as Event).type}`));
    };
  });
}

// =====================================================================
// Files protocol (SFTP-style JSON 契约)
// =====================================================================
export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified_at: string;
}

export interface FileListMessage {
  type: "list";
  request_id: string;
  path: string;
  entries: FileEntry[];
}

export interface FileReadMessage {
  type: "read";
  request_id: string;
  path: string;
  content: string; // base64
}

export interface FileWriteMessage {
  type: "write";
  request_id: string;
  path: string;
  content: string; // base64
  ok: boolean;
}

export interface FileErrorMessage {
  type: "error";
  request_id: string;
  message: string;
}

export type FilesMessage =
  | { type: "list_req"; request_id: string; path: string }
  | { type: "read_req"; request_id: string; path: string }
  | { type: "write_req"; request_id: string; path: string; content: string }
  | FileListMessage
  | FileReadMessage
  | FileWriteMessage
  | FileErrorMessage;
