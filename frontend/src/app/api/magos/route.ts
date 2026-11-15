/*
@cypher schema=1 source_sha256=46c36d43405ba3f207abb217b68e98b9ca5f3e7f5cb7d1de02b4caee35c21612
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

// =====================================================================
// POST /api/magos — 审批流 (plan 请求 + protect 检查; 只读组合, 无写动作)
// 审批流 = plan 预览 + 保护规则核对, 用户在前端确认后才由外部流程执行
// =====================================================================
export async function POST(request: Request) {
  let body: { repo?: string; lane?: string; branch?: string } = {};
  try {
    body = (await request.json()) as { repo?: string; lane?: string };
  } catch {
    /* body 缺省 → 默认 repo */
  }
  const repo = body.repo ?? "D:\\Star";
  // plan 是分支级审批 (--branch 必填, 缺省取当前分支 dev)
  const branch = body.branch ?? body.lane ?? "dev";
  const [plan, protect] = await Promise.all([
    runMagos("plan", repo, ["--branch", branch]),
    runMagos("protect-show", repo),
  ]);
  return Response.json({
    ok: plan.ok,
    review: {
      plan: plan.data ?? null,
      plan_error: plan.ok ? null : (plan.error || `plan 无输出 (分支 ${branch} 无待审计划或未注册车道)`),
      protect: protect.data ?? null,
      protect_error: protect.ok ? null : protect.error,
    },
    approve_hint: "审批通过后由外部流程执行 (本代理只读, 不代理 release/override/arm/install)",
  }, { status: plan.ok ? 200 : 502 });
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
