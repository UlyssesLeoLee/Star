/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/hooks/useIdeKernel.ts"})
MERGE (hook:Symbol {id:"frontend/src/hooks/useIdeKernel.ts::useIdeKernel",kind:"function"})
MERGE (restData:Symbol {id:"frontend/src/hooks/useIdeKernel.ts::restData",kind:"function"})
MERGE (restBase:Symbol {id:"frontend/src/hooks/useIdeKernel.ts::restBase",kind:"function"})
MERGE (probe:Symbol {id:"frontend/src/hooks/useIdeKernel.ts::probe",kind:"function"})
MERGE (openFile:Symbol {id:"frontend/src/hooks/useIdeKernel.ts::openFile",kind:"function"})
MERGE (syncText:Symbol {id:"frontend/src/hooks/useIdeKernel.ts::syncText",kind:"function"})
MERGE (self)-[:DEFINES]->(hook)
MERGE (self)-[:DEFINES]->(restData)
MERGE (self)-[:DEFINES]->(restBase)
MERGE (hook)-[:CALLS]->(probe)
MERGE (hook)-[:CALLS]->(openFile)
MERGE (hook)-[:CALLS]->(syncText)
MERGE (restData)-[:CALLS]->(restBase)
@endcypher
*/

"use client";

// =====================================================================
// useIdeKernel.ts — ide-kernel REST 客户端 hook (per 2026-11-17 IDE1.0 网页版接入)
//
// star-api-rest /api/v1/ide/* 端点 (routes/ide.rs, ide-kernel 移植 crate):
//  - GET  /api/v1/ide/version                  → 连通性探测 + 内核版本
//  - POST /api/v1/ide/doc/open {path,text?}    → 打开文档 (服务端读盘需 IDE_PROJECT_ROOT)
//  - GET  /api/v1/ide/doc/active               → 活动文档视图
//  - POST /api/v1/ide/doc/{undo,redo,backspace,newline,insert,sync,caret} → 编辑原语
//  - POST /api/v1/ide/outline {path,text?}     → 语义节点树
//  - GET  /api/v1/ide/scm/log?n=               → Patch 提交记录 (只读 git log)
//
// 状态机: 'preview' (未探测) → 'connected' (version 探测成功) / 'error' (探测失败,
// 诚实"未接入")。所有动作失败不回滚状态, 只在 error 字段留消息 — UI 据此降级显示。
// 端点 base: process.env.NEXT_PUBLIC_API_BASE ?? http://localhost:8081
// (与 useTerminalStackWs REST 模式同款约定)。
// =====================================================================

import { useCallback, useEffect, useRef, useState } from "react";

/** 内核文档视图 (与 crates/ide-kernel/src/doc_store.rs DocView 一致)。 */
export interface IdeDocView {
  id: number;
  path: string | null;
  text: string;
  line_count: number;
  caret_row: number;
  caret_col: number;
  line_ending: string;
  encoding: string;
  dirty: boolean;
  has_selection: boolean;
  selected_text: string;
  cursor_count: number;
  can_undo: boolean;
  can_redo: boolean;
}

/** 语义节点 (outline 条目, kind: function/class/method/section/...)。 */
export interface IdeOutlineEntry {
  name: string;
  kind: string;
  line: number;
  depth: number;
}

/** outline 结果 (POST /api/v1/ide/outline)。 */
export interface IdeOutlineResult {
  path: string;
  language: string;
  entries: IdeOutlineEntry[];
  error: string | null;
}

/** Patch 提交记录条目 (GET /api/v1/ide/scm/log)。 */
export interface IdeCommitEntry {
  hash: string;
  short: string;
  author: string;
  time: string;
  message: string;
}

/** 内核连接状态机 (与 useTerminalStackWs 的 TerminalRestMode 同款约定)。 */
export type IdeKernelMode = "preview" | "connected" | "error";

/** useIdeKernel 入参。 */
export interface UseIdeKernelOptions {
  /** 是否探测/连接内核 (false = 完全不发请求, 保持"未接入")。 */
  enabled: boolean;
  /** REST 端点 base (缺省 NEXT_PUBLIC_API_BASE ?? http://localhost:8081)。 */
  restBase?: string;
  /** syncText 防抖毫秒 (缺省 400)。 */
  syncDebounceMs?: number;
}

/** useIdeKernel 返回值。 */
export interface UseIdeKernelResult {
  /** 连接状态机: preview (未探测) / connected / error。 */
  mode: IdeKernelMode;
  /** 内核版本 (探测成功后非空)。 */
  kernelVersion: string | null;
  /** 活动文档视图 (无打开文档时 null)。 */
  activeView: IdeDocView | null;
  /** Patch 提交记录 (scm/log)。 */
  patches: IdeCommitEntry[];
  /** scm/log 的诚实降级原因 (非 git 目录 / git 未安装等)。 */
  scmError: string | null;
  /** 有请求在途。 */
  busy: boolean;
  /** 最近一次动作的失败原因 (null = 无失败)。 */
  error: string | null;
  /** 探测内核连通性 (GET /version)。 */
  probe: () => Promise<boolean>;
  /** 打开文件为新标签 (缺省不带 text, 服务端按 IDE_PROJECT_ROOT 读盘)。 */
  openFile: (path: string, text?: string) => Promise<IdeDocView | null>;
  /** 全文本同步 (防抖; 重置内核 undo 栈, 与"打开文件"同语义)。 */
  syncText: (text: string) => void;
  /** 撤销 / 重做 / 退格 / 换行 / 在光标处插入。 */
  undo: () => Promise<IdeDocView | null>;
  redo: () => Promise<IdeDocView | null>;
  backspace: () => Promise<IdeDocView | null>;
  newline: () => Promise<IdeDocView | null>;
  insert: (text: string) => Promise<IdeDocView | null>;
  /** 语义节点树 (text 缺省时服务端按需读盘, 否则回退本地推导)。 */
  loadOutline: (
    path: string,
    text?: string,
  ) => Promise<IdeOutlineResult | null>;
  /** 拉取 Patch 提交记录 (只读 git log)。 */
  loadPatches: (n?: number) => Promise<void>;
}

const DEFAULT_REST_BASE = "http://localhost:8081";
const IDE_PATH = "/api/v1/ide";

/** 端点 base: NEXT_PUBLIC_API_BASE (可配) → 本地默认 (per 2026-11-16 用户发令)。 */
function restBase(override?: string): string {
  if (override) return override.replace(/\/+$/, "");
  if (typeof process !== "undefined" && process.env.NEXT_PUBLIC_API_BASE) {
    return process.env.NEXT_PUBLIC_API_BASE.replace(/\/+$/, "");
  }
  return DEFAULT_REST_BASE;
}

/** fetch 包装: 非 2xx 抛错 (取后端 RestError.message); 成功解包 {data}。 */
async function restData<T>(base: string, path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${base}${path}`, init);
  if (!res.ok) {
    let message = `ide-rest ${res.status} for ${path}`;
    try {
      const body = (await res.json()) as { error?: { message?: string } };
      if (body?.error?.message) message = body.error.message;
    } catch {
      // 非 JSON 错误体 → 保留状态码消息
    }
    throw new Error(message);
  }
  const body = (await res.json()) as { data?: T };
  return (body?.data ?? null) as T;
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

/** JSON POST body 通用 init。 */
function postJson(body: unknown): RequestInit {
  return {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  };
}

/**
 * React hook: 连接 star-api-rest 的 ide-kernel 端点。
 *
 * enabled=false 时完全不发请求 (保持"未接入"徽标语义, 测试环境无 fetch 也安全);
 * enabled=true 时挂载即探测 GET /api/v1/ide/version, 成功 → 'connected' 并拉取
 * Patch 提交记录, 失败 → 'error' (UI 保持诚实"未接入")。
 */
export function useIdeKernel(options: UseIdeKernelOptions): UseIdeKernelResult {
  const { enabled } = options;
  const base = restBase(options.restBase);
  const syncDebounceMs = options.syncDebounceMs ?? 400;

  const [mode, setMode] = useState<IdeKernelMode>("preview");
  const [kernelVersion, setKernelVersion] = useState<string | null>(null);
  const [activeView, setActiveView] = useState<IdeDocView | null>(null);
  const [patches, setPatches] = useState<IdeCommitEntry[]>([]);
  const [scmError, setScmError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /** 活动文档 ref — 动作回调内读取最新光标位置, 避免闭包过期。 */
  const activeRef = useRef<IdeDocView | null>(null);
  const syncTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  /** 内核可用性守卫: 未启用或测试环境无 fetch 时一律不动作。 */
  const fetchable = useCallback(() => {
    return enabled && typeof fetch === "function";
  }, [enabled]);

  const applyView = useCallback((view: IdeDocView | null) => {
    activeRef.current = view;
    setActiveView(view);
  }, []);

  /** GET /api/v1/ide/version — 连通性探测。 */
  const probe = useCallback(async (): Promise<boolean> => {
    if (!fetchable()) return false;
    try {
      const data = await restData<{ kernel_version: string }>(base, `${IDE_PATH}/version`);
      setKernelVersion(data.kernel_version ?? null);
      setMode("connected");
      setError(null);
      return true;
    } catch (err) {
      setMode("error");
      setError(errorMessage(err));
      return false;
    }
  }, [base, fetchable]);

  /** GET /api/v1/ide/scm/log — Patch 提交记录。 */
  const loadPatches = useCallback(
    async (n: number = 20): Promise<void> => {
      if (!fetchable()) return;
      setBusy(true);
      try {
        const data = await restData<{
          commits: IdeCommitEntry[];
          error: string | null;
        }>(base, `${IDE_PATH}/scm/log?n=${Math.max(1, Math.min(100, n))}`);
        setPatches(data.commits ?? []);
        setScmError(data.error ?? null);
      } catch (err) {
        setPatches([]);
        setScmError(errorMessage(err));
      } finally {
        setBusy(false);
      }
    },
    [base, fetchable],
  );

  /** POST /api/v1/ide/doc/open — 打开文件为新标签。 */
  const openFile = useCallback(
    async (path: string, text?: string): Promise<IdeDocView | null> => {
      if (!fetchable()) return null;
      setBusy(true);
      try {
        const data = await restData<{ view: IdeDocView }>(
          base,
          `${IDE_PATH}/doc/open`,
          postJson(text === undefined ? { path } : { path, text }),
        );
        applyView(data.view);
        setError(null);
        return data.view;
      } catch (err) {
        setError(errorMessage(err));
        return null;
      } finally {
        setBusy(false);
      }
    },
    [base, fetchable, applyView],
  );

  /** 单个编辑原语 POST 的公共骨架 (无 body 或带 body)。 */
  const runOp = useCallback(
    async (path: string, body?: unknown): Promise<IdeDocView | null> => {
      if (!fetchable()) return null;
      setBusy(true);
      try {
        const data = await restData<{ view: IdeDocView }>(
          base,
          `${IDE_PATH}${path}`,
          body === undefined ? { method: "POST" } : postJson(body),
        );
        applyView(data.view);
        setError(null);
        return data.view;
      } catch (err) {
        setError(errorMessage(err));
        return null;
      } finally {
        setBusy(false);
      }
    },
    [base, fetchable, applyView],
  );

  const undo = useCallback(() => runOp("/doc/undo"), [runOp]);
  const redo = useCallback(() => runOp("/doc/redo"), [runOp]);
  const backspace = useCallback(() => runOp("/doc/backspace"), [runOp]);
  const newline = useCallback(() => runOp("/doc/newline"), [runOp]);

  /** 在活动文档光标处插入文本。 */
  const insert = useCallback(
    (text: string): Promise<IdeDocView | null> => {
      const view = activeRef.current;
      if (!view) return Promise.resolve(null);
      return runOp("/doc/insert", {
        row: view.caret_row,
        col: view.caret_col,
        text,
      });
    },
    [runOp],
  );

  /** 全文本同步 (防抖): textarea 整段编辑走这里, 内核持有单一事实来源。 */
  const syncText = useCallback(
    (text: string): void => {
      if (!fetchable() || !activeRef.current) return;
      if (syncTimerRef.current) clearTimeout(syncTimerRef.current);
      syncTimerRef.current = setTimeout(async () => {
        try {
          const data = await restData<{ view: IdeDocView }>(
            base,
            `${IDE_PATH}/doc/sync`,
            postJson({ text }),
          );
          applyView(data.view);
          setError(null);
        } catch (err) {
          setError(errorMessage(err));
        }
      }, syncDebounceMs);
    },
    [base, fetchable, applyView, syncDebounceMs],
  );

  /** POST /api/v1/ide/outline — 语义节点树 (无状态纯计算, 结果直接返回)。 */
  const loadOutline = useCallback(
    async (path: string, text?: string): Promise<IdeOutlineResult | null> => {
      if (!fetchable()) return null;
      try {
        const data = await restData<IdeOutlineResult>(
          base,
          `${IDE_PATH}/outline`,
          postJson(text === undefined ? { path } : { path, text }),
        );
        setError(null);
        return data;
      } catch (err) {
        setError(errorMessage(err));
        return null;
      }
    },
    [base, fetchable],
  );

  // 挂载探测 + 成功后拉 Patch 记录; 卸载清防抖定时器。
  useEffect(() => {
    if (!fetchable()) return;
    let cancelled = false;
    probe().then((ok) => {
      if (!cancelled && ok) {
        void loadPatches();
      }
    });
    return () => {
      cancelled = true;
      if (syncTimerRef.current) clearTimeout(syncTimerRef.current);
    };
  }, [fetchable, probe, loadPatches]);

  return {
    mode,
    kernelVersion,
    activeView,
    patches,
    scmError,
    busy,
    error,
    probe,
    openFile,
    syncText,
    undo,
    redo,
    backspace,
    newline,
    insert,
    loadOutline,
    loadPatches,
  };
}
