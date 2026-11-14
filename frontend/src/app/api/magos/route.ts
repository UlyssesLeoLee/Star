/*
@cypher schema=1 source_sha256=50465b8c7c62551b9fb314415070f91e7fc8b7cea808158ee0aaa776f347d432
MERGE (self:File {path:"frontend/src/app/api/magos/route.ts"})
MERGE (handler:Symbol {id:"frontend/src/app/api/magos/route.ts::GET",kind:"function"})
MERGE (self)-[:DEFINES]->(handler)
@endcypher
*/
// =====================================================================
// /api/magos — MAGOS lane/worktree 管理只读代理
// 后端 = 本地 MAGOS (E:\MAGOS\scripts\magos_guard.py) 只读子命令:
//   list / plan / doctor / version / protect-show
// per INTEGRATION-MAP-GITHUB-REPOS-001 §2.2 (2026-11-14 用户授权深度集成)
// 安全: 白名单只读命令, 绝不代理 write 类 (release/override/arm/install...)
// =====================================================================

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

const MAGOS_SCRIPT = process.env.MAGOS_GUARD_SCRIPT ?? "E:\\MAGOS\\scripts\\magos_guard.py";
const READ_ONLY_COMMANDS = new Set(["list", "plan", "doctor", "version", "protect-show"]);

async function runMagos(cmd: string, repo: string, extra: string[] = []) {
  try {
    const { stdout } = await execFileAsync("python", [MAGOS_SCRIPT, cmd, "--repo", repo, "--json", ...extra], {
      timeout: 30_000,
      windowsHide: true,
    });
    return { ok: true, data: stdout ? JSON.parse(stdout) : null };
  } catch (err) {
    const e = err as { stderr?: string; code?: number };
    return { ok: false, error: (e.stderr ?? String(err)).slice(0, 500) };
  }
}

export async function GET(request: Request) {
  const url = new URL(request.url);
  const cmd = url.searchParams.get("cmd") ?? "list";
  if (!READ_ONLY_COMMANDS.has(cmd)) {
    return Response.json({ ok: false, error: `read-only commands only: ${[...READ_ONLY_COMMANDS].join("/")}` }, { status: 403 });
  }
  const repo = url.searchParams.get("repo") ?? "D:\\Star";
  const extra = url.searchParams.getAll("arg");

  const result = await runMagos(cmd, repo, extra);
  return Response.json(result, { status: result.ok ? 200 : 502 });
}
