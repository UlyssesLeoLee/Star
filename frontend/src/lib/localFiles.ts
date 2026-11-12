/*
@cypher schema=1 source_sha256=826a3e035f976970bfd673b028e019b280af107c43341a6d688fa131baea44ce
MERGE (self:File {path:"frontend/src/lib/localFiles.ts"})
MERGE (isLocal:Symbol {id:"frontend/src/lib/localFiles.ts::isLocalMode",kind:"function"})
MERGE (list:Symbol {id:"frontend/src/lib/localFiles.ts::listWorkspaceFiles",kind:"function"})
MERGE (read:Symbol {id:"frontend/src/lib/localFiles.ts::readWorkspaceFile",kind:"function"})
MERGE (self)-[:DEFINES]->(isLocal)
MERGE (self)-[:DEFINES]->(list)
MERGE (self)-[:DEFINES]->(read)
@endcypher
*/
// =====================================================================
// 本地文件夹桥接客户端 (per 2026-11-11 用户发令: 网页版关联本地文件夹)
// 本地模式 = 浏览器可达 127.0.0.1 后端 (localhost dev 或本机打开线上页面 +
// 本地后端运行中)。线上无本地后端时优雅降级 (返回 null / 错误码)。
// =====================================================================

export interface WorkspaceFileEntry {
  name: string;
  is_dir: boolean;
  size_bytes: number | null;
}

export interface WorkspaceFileList {
  root: string;
  path: string;
  entries: WorkspaceFileEntry[];
}

export interface WorkspaceFileRead {
  path: string;
  text: string;
}

/** 本地模式判定: localhost / 127.0.0.1 / 局域网私有地址。 */
export function isLocalMode(): boolean {
  if (typeof window === "undefined") return false;
  const host = window.location.hostname;
  return (
    host === "localhost" ||
    host === "127.0.0.1" ||
    host.startsWith("192.168.") ||
    host.startsWith("10.") ||
    /^172\.(1[6-9]|2\d|3[01])\./.test(host)
  );
}

async function apiGet<T>(path: string): Promise<{ status: number; body: T | null; error?: string }> {
  try {
    const res = await fetch(path, { headers: { Accept: "application/json" } });
    if (!res.ok) {
      let code = `HTTP_${res.status}`;
      try {
        code = (JSON.parse(await res.text()) as { error?: string }).error ?? code;
      } catch {
        /* not json */
      }
      return { status: res.status, body: null, error: code };
    }
    return { status: res.status, body: (await res.json()) as T };
  } catch {
    return { status: 0, body: null, error: "LOCAL_BACKEND_UNREACHABLE" };
  }
}

/** 列举白名单根目录下 path 的文件/目录 (path 相对根, 空串 = 根)。 */
export async function listWorkspaceFiles(
  path = "",
): Promise<{ status: number; body: WorkspaceFileList | null; error?: string }> {
  const query = path ? `?path=${encodeURIComponent(path)}` : "";
  return apiGet<WorkspaceFileList>(`/api/v1/workspace-files${query}`);
}

/** 读取白名单内文本文件 (≤4 MiB; 二进制返回 UNSUPPORTED_MEDIA_TYPE)。 */
export async function readWorkspaceFile(
  path: string,
): Promise<{ status: number; body: WorkspaceFileRead | null; error?: string }> {
  return apiGet<WorkspaceFileRead>(`/api/v1/workspace-files/read?path=${encodeURIComponent(path)}`);
}
