/*
@cypher schema=1 source_sha256=4b94582667a5a1a69f52f7c35d3d4ac7c80a689943c521d8a103d1d60dedc5c6
MERGE (self:File {path:"frontend/src/lib/nema/nemaWatcher.ts"})
MERGE (scan:Function {name:"nemaWatcher::scanNemaFiles",language:"typescript"})
MERGE (hook:Function {name:"nemaWatcher::useNemaWatcher",language:"typescript"})
MERGE (api:File {path:"frontend/src/lib/localFiles.ts"})
MERGE (store:File {path:"frontend/src/lib/nema/nemaStore.ts"})
MERGE (sync:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::syncNemaSource",kind:"function"})
MERGE (profile:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaInterpreterProfile",kind:"type"})
MERGE (self)-[:DEFINES]->(scan)
MERGE (self)-[:DEFINES]->(hook)
MERGE (self)-[:IMPORTS]->(api)
MERGE (self)-[:IMPORTS]->(store)
MERGE (scan)-[:CALLS]->(sync)
MERGE (scan)-[:USES_TYPE]->(profile)
MERGE (hook)-[:CALLS]->(scan)
@endcypher
*/
// =====================================================================
// nemaWatcher — .nema 文件系统监听与任务状态回写 (per WBS 8.4.4)
// 机制: 轮询 workspace-files 只读桥接 API (本地模式) 检测磁盘 .nema 文件
//       变化 → syncNemaSource 导入/更新 → 已绑定任务自动回写标题/描述。
// 纯网页版 (无本地后端) 优雅降级: 扫描返回 backendUnavailable。
// =====================================================================

import { useCallback, useEffect, useRef, useState } from "react";
import { isLocalMode, listWorkspaceFiles, readWorkspaceFile } from "@/lib/localFiles";
import { useNemaStore } from "./nemaStore";
import type { NemaInterpreterProfile } from "./nema";

export interface NemaScanResult {
  /** 扫描到的 .nema 文件数 */
  scanned: number;
  /** 新导入任务数 */
  imported: number;
  /** 内容变化更新数 */
  updated: number;
  /** 后端不可用 (非本地模式 / API 挂) */
  backendUnavailable: boolean;
  /** 扫描错误 */
  errors: string[];
}

/** 单次扫描: 列目录 → 筛 .nema → 读内容 → sync (导入/更新/回写)。 */
export async function scanNemaFiles(
  roots: string[] = [""],
  importProfile?: NemaInterpreterProfile | null,
): Promise<NemaScanResult> {
  const result: NemaScanResult = {
    scanned: 0,
    imported: 0,
    updated: 0,
    backendUnavailable: false,
    errors: [],
  };
  if (!isLocalMode()) {
    result.backendUnavailable = true;
    return result;
  }
  const store = useNemaStore.getState();

  for (const root of roots) {
    const list = await listWorkspaceFiles(root);
    if (list.status !== 200 || !list.body) {
      result.backendUnavailable = true;
      if (list.error) result.errors.push(`${root}: ${list.error}`);
      continue;
    }
    const nemaFiles = list.body.entries.filter(
      (e) => !e.is_dir && e.name.toLowerCase().endsWith(".nema"),
    );
    for (const file of nemaFiles) {
      result.scanned += 1;
      const nemaPath = root ? `${root}/${file.name}` : file.name;
      const read = await readWorkspaceFile(nemaPath);
      if (read.status !== 200 || !read.body) {
        if (read.error) result.errors.push(`${nemaPath}: ${read.error}`);
        continue;
      }
      try {
        const synced = await store.syncNemaSource(nemaPath, read.body.text, importProfile ?? undefined);
        if (synced.action === "imported") result.imported += 1;
        else if (synced.action === "updated") result.updated += 1;
      } catch (error) {
        result.errors.push(`${nemaPath}: ${error instanceof Error ? error.message : String(error)}`);
      }
    }
  }
  return result;
}

export interface NemaWatcherState {
  lastResult: NemaScanResult | null;
  /** 上次扫描时间 */
  lastScanAt: number | null;
  /** 手动触发一次扫描 */
  scanNow: () => void;
  /** 是否在监听中 */
  watching: boolean;
}

const DEFAULT_INTERVAL_MS = 20_000;

/**
 * 轮询式 .nema 监听 hook (per WBS 8.4.4)。
 * 本地模式 + 提供 roots 时启动定时扫描; 非本地模式不启动。
 */
export function useNemaWatcher(
  roots: string[] = [""],
  intervalMs = DEFAULT_INTERVAL_MS,
  importProfile?: NemaInterpreterProfile | null,
): NemaWatcherState {
  const [lastResult, setLastResult] = useState<NemaScanResult | null>(null);
  const [lastScanAt, setLastScanAt] = useState<number | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const rootsKey = roots.join("|");

  const scanNow = useCallback(async () => {
    const result = await scanNemaFiles(roots, importProfile);
    setLastResult(result);
    setLastScanAt(Date.now());
  }, [rootsKey, importProfile]);

  const watching = isLocalMode() && roots.length > 0 && importProfile != null;

  useEffect(() => {
    if (!watching) return;
    void scanNow();
    timerRef.current = setInterval(() => void scanNow(), intervalMs);
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [watching, intervalMs, scanNow]);

  return { lastResult, lastScanAt, scanNow, watching };
}
