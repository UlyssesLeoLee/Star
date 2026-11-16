/*
@cypher schema=1 source_sha256=e772a4571b129f74ccc53694b2eae8e251287fe2bc1dcfe1e70364d60a5a02a2
MERGE (self:File {path:"frontend/src/hooks/useTerminalStackWs.ts"})
@endcypher
*/
/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/hooks/useTerminalStackWs.ts",type:"file",language:"typescript"}),
  (options:Class {name:"UseTerminalStackWsOptions",type:"class",language:"typescript"}),
  (hook:Function {name:"useTerminalStackWs",type:"function",language:"typescript",signature:"useTerminalStackWs(opts): TerminalWsControls"}),
  (count:Function {name:"countPanes",type:"function",language:"typescript"}),
  (depth:Function {name:"computeDepth",type:"function",language:"typescript"}),
  (file)-[:CONTAINS]->(options),(file)-[:CONTAINS]->(hook),(file)-[:CONTAINS]->(count),(file)-[:CONTAINS]->(depth),
  (hook)-[:CALLS]->(count),(hook)-[:CALLS]->(depth);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (file:File {name:"frontend/src/hooks/useTerminalStackWs.ts"}),
      (hook:Function {name:"useTerminalStackWs"});
CREATE (output:Variable {name:"outputByPane",type:"variable",language:"typescript"}),
       (appendOutput:Function {name:"appendTerminalOutput",type:"function",language:"typescript"}),
       (maxOutput:Variable {name:"MAX_RENDERED_OUTPUT_CHARS",type:"variable",language:"typescript"});
CREATE (file)-[:CONTAINS]->(output),
       (file)-[:CONTAINS]->(appendOutput),
       (file)-[:CONTAINS]->(maxOutput),
       (hook)-[:USES]->(output),
       (hook)-[:CALLS]->(appendOutput),
       (appendOutput)-[:USES]->(maxOutput);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (hook:Function {name:"useTerminalStackWs"}),
      (options:Class {name:"UseTerminalStackWsOptions"});
CREATE (onConnectionChange:Variable {name:"UseTerminalStackWsOptions.onConnectionChange",type:"variable",language:"typescript"});
CREATE (options)-[:HAS_FIELD]->(onConnectionChange),
       (hook)-[:CALLS]->(onConnectionChange);
*/

/* CYPHER STRUCTURE MANIFEST ADDENDUM (star-api-rest terminal pane REST mode)
MATCH (file:File {name:"frontend/src/hooks/useTerminalStackWs.ts"}),
      (hook:Function {name:"useTerminalStackWs"}),
      (output:Variable {name:"outputByPane",type:"variable",language:"typescript"});
CREATE (createPane:Function {name:"createPane",type:"function",language:"typescript"}),
       (writeLine:Function {name:"writeLine",type:"function",language:"typescript"}),
       (fetchScrollback:Function {name:"fetchScrollback",type:"function",language:"typescript"}),
       (restBase:Function {name:"restBase",type:"function",language:"typescript"}),
       (restRequest:Function {name:"restRequest",type:"function",language:"typescript"}),
       (extractPaneId:Function {name:"extractPaneId",type:"function",language:"typescript"}),
       (parseScrollback:Function {name:"parseScrollback",type:"function",language:"typescript"}),
       (mergeDelta:Function {name:"mergeDelta",type:"function",language:"typescript"}),
       (mode:Variable {name:"mode",type:"variable",language:"typescript"}),
       (paneId:Variable {name:"paneId",type:"variable",language:"typescript"}),
       (paneIdRef:Variable {name:"paneIdRef",type:"variable",language:"typescript"});
CREATE (file)-[:CONTAINS]->(createPane),(file)-[:CONTAINS]->(writeLine),(file)-[:CONTAINS]->(fetchScrollback),
       (file)-[:CONTAINS]->(restBase),(file)-[:CONTAINS]->(restRequest),(file)-[:CONTAINS]->(extractPaneId),
       (file)-[:CONTAINS]->(parseScrollback),(file)-[:CONTAINS]->(mergeDelta),(file)-[:CONTAINS]->(mode),
       (file)-[:CONTAINS]->(paneId),(file)-[:CONTAINS]->(paneIdRef),
       (hook)-[:CALLS]->(createPane),(hook)-[:CALLS]->(writeLine),(hook)-[:CALLS]->(fetchScrollback),
       (hook)-[:USES]->(mode),(hook)-[:USES]->(paneId),(hook)-[:USES]->(paneIdRef),
       (createPane)-[:CALLS]->(restRequest),(createPane)-[:CALLS]->(extractPaneId),
       (createPane)-[:USES]->(paneIdRef),
       (writeLine)-[:CALLS]->(createPane),(writeLine)-[:CALLS]->(restRequest),
       (writeLine)-[:CALLS]->(appendOutput),
       (fetchScrollback)-[:CALLS]->(createPane),(fetchScrollback)-[:CALLS]->(restRequest),
       (fetchScrollback)-[:CALLS]->(parseScrollback),(fetchScrollback)-[:CALLS]->(mergeDelta),
       (restRequest)-[:CALLS]->(restBase);
*/

"use client";

// =====================================================================
// useTerminalStackWs.ts — React hook (per ULYS-222 P1-C + ULYS-223 P1-D 集成)
// (per PR #98.5 修复 PR #98 ↔ PR #106 schema 不对齐)
// =====================================================================
// 流程:
//  1. useEffect: 创建 TerminalWsClient + bind handlers
//  2. onHello → setWsConnected(true)
//  3. onOutput → 写入有界 per-pane output projection，由授权 xterm pane 渲染
//  4. onSplitUpdate → setTree (per PR #98 store.setTree)
//  5. onSnapshot → 替换该 pane 的恢复输出，再应用后续增量
//  6. cleanup: close WS + reset store
// =====================================================================
// star-api-rest REST 模式 (sessionId=null 时从 preview 占位升级为真实接线):
//  - createPane()      POST /api/v1/terminal/pane                  → paneId 存 state
//  - writeLine(text)   POST /api/v1/terminal/pane/:id/line         → 成功后本地滚动缓冲追加
//  - fetchScrollback() GET  /api/v1/terminal/pane/:id/scrollback?since_seq=N → 追平本地缓冲
//  - 端点 base: process.env.NEXT_PUBLIC_API_BASE ?? http://localhost:8081
//  - 失败 (fetch 抛错 / 404 / 网络) → 自动回退 preview 行为, mode='error' 诚实标注
//  - mode 状态机: 'preview' (未尝试) → 'real' (pane 创建成功) → 'error' (尝试失败降级);
//    error 下再次调用可重试 (createPane 幂等复用已有 pane)
// =====================================================================
// 假定契约 (star-api-rest, 同批其他子代理实现; 响应解析做防御性容忍):
//  - POST /api/v1/terminal/pane            → 2xx {"id":"<paneId>"} (容忍 pane_id/paneId/裸字符串)
//  - POST /api/v1/terminal/pane/:id/line   → 请求体 {"text":"..."}, 2xx 即成功
//  - GET  /api/v1/terminal/pane/:id/scrollback?since_seq=N
//        → {"lines":[{"text":"..."}|"...", ...], "next_since_seq":N}
//          (seq 字段容忍 next_since_seq/last_seq/seq; since_seq 缺省 = 全量快照)
// =====================================================================

import { useCallback, useEffect, useRef, useState } from "react";
import { TerminalWsClient } from "@/lib/terminal/wsClient";
import type {
  ScrollbackLine,
  ServerMessage,
} from "@/lib/terminal/wsProtocol";
import {
  newSingleTree,
  type SplitTreeView,
} from "@/components/terminal/types";
import { useTerminalStackStore } from "@/components/terminal/terminalStackStore";

export interface UseTerminalStackWsOptions {
  /** Session id; null means preview placeholder mode without a WebSocket. */
  sessionId: string | null;
  /** Obtain a fresh, one-use attachment ticket before every socket connection. */
  getAttachmentTicket?: () => Promise<string>;
  /** Auto-reconnect after disconnect (per NFR-AC keepalive) */
  autoReconnect?: boolean;
  /** Reports authorized attachment success and terminal connection loss to the owner. */
  onConnectionChange?: (connected: boolean) => void;
}

/** REST pane 状态机 (star-api-rest /api/v1/terminal/pane) */
export type TerminalRestMode = "preview" | "real" | "error";

const MAX_RENDERED_OUTPUT_CHARS = 1_000_000;

function appendTerminalOutput(current: string, next: string): string {
  const combined = current + next;
  return combined.length > MAX_RENDERED_OUTPUT_CHARS
    ? combined.slice(-MAX_RENDERED_OUTPUT_CHARS)
    : combined;
}

/** CRLF 常量 (避免源码内嵌控制字符转义在补丁层被改写) */
const CRLF = String.fromCharCode(13, 10);

/** 增量追平: base 已以换行结尾时不额外插空行 */
function mergeDelta(base: string, delta: string): string {
  if (!delta) return base;
  if (!base) return delta;
  const needsSep =
    !base.endsWith(CRLF) && !base.endsWith(String.fromCharCode(10));
  return appendTerminalOutput(base, needsSep ? CRLF + delta : delta);
}
// =====================================================================
// star-api-rest 端点解析 (per 2026-11-16 用户发令: NEXT_PUBLIC_API_BASE 可配)
// =====================================================================

const DEFAULT_REST_BASE = "http://localhost:8081";
const TERMINAL_PANE_PATH = "/api/v1/terminal/pane";

function restBase(): string {
  if (typeof process !== "undefined" && process.env.NEXT_PUBLIC_API_BASE) {
    return process.env.NEXT_PUBLIC_API_BASE.replace(/\/+$/, "");
  }
  return DEFAULT_REST_BASE;
}

/** fetch 包装: 非 2xx (含 404) 一律抛错 → 由调用方降级 */
async function restRequest(path: string, init?: RequestInit): Promise<Response> {
  const res = await fetch(`${restBase()}${path}`, init);
  if (!res.ok) {
    throw new Error(
      `terminal-rest ${res.status} ${res.statusText} for ${path}`,
    );
  }
  return res;
}

/** 容忍 id / pane_id / paneId / 裸字符串 四种响应形状 */
function extractPaneId(raw: unknown): string | null {
  if (typeof raw === "string" && raw.trim()) return raw.trim();
  if (raw && typeof raw === "object") {
    const record = raw as Record<string, unknown>;
    for (const key of ["id", "pane_id", "paneId"]) {
      const value = record[key];
      if (typeof value === "string" && value.trim()) return value.trim();
    }
  }
  return null;
}

interface ParsedScrollback {
  lines: string[];
  nextSeq: number;
}

/** 容忍 lines 元素为 string 或 {text|line|data}; seq 字段多形状 */
function parseScrollback(
  raw: unknown,
  sinceSeq: number | undefined,
): ParsedScrollback {
  const body =
    raw && typeof raw === "object" ? (raw as Record<string, unknown>) : null;
  const rawLines: unknown = Array.isArray(raw) ? raw : body?.lines;
  const lines: string[] = Array.isArray(rawLines)
    ? rawLines.map((line) => {
        if (typeof line === "string") return line;
        if (line && typeof line === "object") {
          const record = line as Record<string, unknown>;
          return String(record.text ?? record.line ?? record.data ?? "");
        }
        return "";
      })
    : [];
  const seqField = body
    ? (body.next_since_seq ?? body.last_seq ?? body.seq)
    : undefined;
  const nextSeq =
    typeof seqField === "number" ? seqField : (sinceSeq ?? 0) + lines.length;
  return { lines, nextSeq };
}

/**
 * React hook: wire TerminalWsClient to zustand store.
 *
 * Returns a stable callback for sending client messages (stdin/resize/ping).
 */
export function useTerminalStackWs(opts: UseTerminalStackWsOptions) {
  const clientRef = useRef<TerminalWsClient | null>(null);
  const setWsConnected = useTerminalStackStore((s) => s.setWsConnected);
  const setTree = useTerminalStackStore((s) => s.setTree);
  const reset = useTerminalStackStore((s) => s.reset);
  const [outputByPane, setOutputByPane] = useState<Record<string, string>>({});
  const sendStdin = useCallback((data: string) => clientRef.current?.sendStdin(data), []);
  const sendResize = useCallback(
    (cols: number, rows: number) => clientRef.current?.sendResize(cols, rows),
    [],
  );
  const isConnected = useCallback(() => clientRef.current?.isConnected() ?? false, []);

  // ---- REST pane 状态 (star-api-rest) ----
  const [mode, setMode] = useState<TerminalRestMode>("preview");
  const [paneId, setPaneId] = useState<string | null>(null);
  const paneIdRef = useRef<string | null>(null);

  useEffect(() => {
    if (!opts.sessionId) {
      // A missing session is a preview/placeholder, never a live connection.
      setWsConnected(false);
      setOutputByPane({});
      return;
    }

    const client = new TerminalWsClient({
      sessionId: opts.sessionId,
      getAttachmentTicket: opts.getAttachmentTicket,
      autoReconnect: opts.autoReconnect ?? true,
      handlers: {
        onHello: (msg) => {
          setWsConnected(true);
          if (msg.panes.length === 1) {
            setTree(newSingleTree(msg.panes[0], "Task CLI"));
          }
        },
        onSnapshot: (msg) => {
          const snapshot = msg.lines.map((line: ScrollbackLine) => line.text).join("\r\n");
          setOutputByPane((current) => ({ ...current, [msg.pane_id]: snapshot }));
        },
        onOutput: (msg) => {
          setOutputByPane((current) => ({
            ...current,
            [msg.pane_id]: appendTerminalOutput(current[msg.pane_id] ?? "", msg.data),
          }));
        },
        onSplitUpdate: (msg) => {
          // msg.tree is PaneNodeView (1:1 mirror of SplitTree).
          // Per wsProtocol.ts: split_update has `tree:` (not `root:`) field.
          const treeNode = msg.tree;
          const wrapped: SplitTreeView = {
            root: treeNode,
            paneCount: countPanes(treeNode),
            depth: computeDepth(treeNode),
            kind: treeNode.kind === "split" ? "split" : "single",
          };
          setTree(wrapped);
        },
        onConnectionChange: (connected) => {
          setWsConnected(connected);
          opts.onConnectionChange?.(connected);
        },
        onError: (msg: Extract<ServerMessage, { type: "error" }>) => {
          // eslint-disable-next-line no-console
          console.warn(
            `[terminal-ws] server error ${msg.code}: ${msg.message}`,
          );
        },
      },
      });

    clientRef.current = client;
    client.connect();

    return () => {
      client.close();
      clientRef.current = null;
      reset();
      setWsConnected(false);
      setOutputByPane({});
    };
  }, [opts.sessionId, opts.getAttachmentTicket, opts.autoReconnect, opts.onConnectionChange, setWsConnected, setTree, reset]);

  // ---- REST: createPane (POST /api/v1/terminal/pane) ----
  const createPane = useCallback(async (): Promise<string | null> => {
    const existing = paneIdRef.current;
    if (existing) return existing; // 幂等: 已建 pane 直接复用
    try {
      const res = await restRequest(TERMINAL_PANE_PATH, { method: "POST" });
      const raw: unknown = await res.json().catch(() => null);
      const id = extractPaneId(raw);
      if (!id) throw new Error("terminal-rest createPane response missing id");
      paneIdRef.current = id;
      setPaneId(id);
      setMode("real");
      setOutputByPane((current) =>
        id in current ? current : { ...current, [id]: "" },
      );
      return id;
    } catch (err) {
      // 失败 → 自动回退 preview 行为, mode='error' 诚实标注
      setMode("error");
      // eslint-disable-next-line no-console
      console.warn("[terminal-rest] createPane failed, preview fallback:", err);
      return null;
    }
  }, []);

  // ---- REST: writeLine (POST /api/v1/terminal/pane/:id/line) ----
  const writeLine = useCallback(
    async (text: string): Promise<boolean> => {
      if (!text) return false;
      // 无 pane 时自动创建 (首条命令即建 pane); 失败已由 createPane 降级
      const id = paneIdRef.current ?? (await createPane());
      if (!id) return false;
      try {
        await restRequest(
          `${TERMINAL_PANE_PATH}/${encodeURIComponent(id)}/line`,
          {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ text }),
          },
        );
        // 成功后本地滚动缓冲追加
        setOutputByPane((current) => ({
          ...current,
          [id]: appendTerminalOutput(current[id] ?? "", text + CRLF),
        }));
        return true;
      } catch (err) {
        setMode("error");
        // eslint-disable-next-line no-console
        console.warn("[terminal-rest] writeLine failed, preview fallback:", err);
        return false;
      }
    },
    [createPane],
  );

  // ---- REST: fetchScrollback (GET .../scrollback?since_seq=N) ----
  const fetchScrollback = useCallback(
    async (sinceSeq?: number): Promise<number | null> => {
      const id = paneIdRef.current ?? (await createPane());
      if (!id) return null;
      const query =
        typeof sinceSeq === "number" && sinceSeq > 0
          ? `?since_seq=${Math.floor(sinceSeq)}`
          : "";
      try {
        const res = await restRequest(
          `${TERMINAL_PANE_PATH}/${encodeURIComponent(id)}/scrollback${query}`,
        );
        const raw: unknown = await res.json().catch(() => null);
        const { lines, nextSeq } = parseScrollback(raw, sinceSeq);
        const text = lines.join(CRLF);
        setOutputByPane((current) => ({
          ...current,
          // since_seq 提供 → 增量追平; 缺省 → 全量快照替换
          [id]:
            typeof sinceSeq === "number"
              ? mergeDelta(current[id] ?? "", text)
              : text,
        }));
        return nextSeq;
      } catch (err) {
        setMode("error");
        // eslint-disable-next-line no-console
        console.warn("[terminal-rest] fetchScrollback failed, preview fallback:", err);
        return null;
      }
    },
    [createPane],
  );

  // Return send helpers as a stable API
  return {
    sendStdin,
    sendResize,
    isConnected,
    outputByPane,
    // star-api-rest REST pane 状态机 (preview → real → error 降级)
    mode,
    paneId,
    createPane,
    writeLine,
    fetchScrollback,
  };
}

// =====================================================================
// helpers
// =====================================================================

import type { PaneNodeView } from "@/components/terminal/types";

function countPanes(node: PaneNodeView): number {
  if (node.kind === "pane") return 1;
  return node.children.reduce((acc, c) => acc + countPanes(c), 0);
}

function computeDepth(node: PaneNodeView, depth = 0): number {
  if (node.kind === "pane") return depth;
  return Math.max(...node.children.map((c) => computeDepth(c, depth + 1)));
}

// Suppress unused warnings on initialSingleTree re-export
export { newSingleTree };
