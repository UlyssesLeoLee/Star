// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/mocks/handlers/annotations.ts
//
// MSW handler for /api/v2/annotations/* (per FR-ORCA-035 §11 v1.0, ULYS-211)。
//
// 这是 crates/agent-bridge/src/annotate.rs `AnnotationRegistry` 的**行为等价
// mock**: 同样的 6 个 API、同样的校验顺序、同样的 3 个 error variant、
// 同样的 `to_prompt_fragment` 文本格式。后端补 axum 路由后删掉本文件即可,
// 前端代码一行不用改。
//
// 与 Rust 侧的两处刻意保持一致:
//   1. `register_agent_run` 未注册的 run → AgentRunNotFound (故 mock 只认
//      DEMO_AGENT_RUN_ID, 不做"见到就注册", 否则该错误分支永远测不到)
//   2. `list_for_file` 遍历 HashMap, 顺序不保证 → mock 也不排序, 排序由
//      前端 store 兜底 (见 lib/hooks/use-annotations.ts)

import { http, HttpResponse } from "msw";

import type {
  AnnotationErrorBody,
  CreateAnnotationRequest,
  DiffAnnotation,
  FeedToAgentResponse,
} from "@/lib/api/annotations";
import { DEMO_AGENT_RUN_ID } from "@/mocks/data/annotations";

const BASE = "/api/v2/annotations";

// ── in-memory registry (mirrors AnnotationRegistry) ──
let annotations: DiffAnnotation[] = [];
const knownAgentRuns = new Set<string>([DEMO_AGENT_RUN_ID]);
let seq = 0;

/** 测试用: 清空 registry 并复位已注册 run */
export function __resetAnnotationMock(): void {
  annotations = [];
  knownAgentRuns.clear();
  knownAgentRuns.add(DEMO_AGENT_RUN_ID);
  seq = 0;
}

/** 测试用: 额外注册 agent run (mirrors `register_agent_run`) */
export function __registerAgentRunMock(agentRunId: string): void {
  knownAgentRuns.add(agentRunId);
}

/** 测试用: 直读当前 registry */
export function __annotationMockSnapshot(): readonly DiffAnnotation[] {
  return annotations;
}

function err(
  status: number,
  code: AnnotationErrorBody["code"],
  message: string,
) {
  return HttpResponse.json<AnnotationErrorBody>({ code, message }, { status });
}

/** mirrors `DiffAnnotation::to_prompt_fragment` (annotate.rs) */
function toPromptFragment(a: DiffAnnotation): string {
  return (
    `[Diff Annotation]\n` +
    `File: ${a.file_path}\n` +
    `Lines: ${a.line_range.start}-${a.line_range.end}\n` +
    `By: ${a.author.kind}\n` +
    `At: ${a.created_at}\n` +
    `Body: ${a.body}\n`
  );
}

export const annotationsHandlers = [
  // ── POST /annotations/feed (必须排在 /:id 之前, MSW 取首个匹配) ──
  http.post(`${BASE}/feed`, ({ request }) => {
    const agentRunId = new URL(request.url).searchParams.get("agent_run_id") ?? "";
    if (!knownAgentRuns.has(agentRunId)) {
      return err(404, "agent_run_not_found", `agent run not found: ${agentRunId}`);
    }
    // list_for_agent_run: 按 add 顺序 (created_at ASC)
    const list = annotations.filter((a) => a.agent_run_id === agentRunId);
    const body: FeedToAgentResponse = {
      agent_run_id: agentRunId,
      prompt_fragment:
        list.length === 0
          ? null
          : list.map(toPromptFragment).join("\n---\n\n"),
    };
    return HttpResponse.json(body, { status: 200 });
  }),

  // ── POST /annotations (add) ──
  http.post(BASE, async ({ request }) => {
    const req = (await request.json()) as CreateAnnotationRequest;

    // 校验顺序照抄 annotate.rs::add
    if (!req.file_path) return err(400, "invalid_field", "file_path 不能为空");
    if (!req.body) return err(400, "invalid_field", "body 不能为空");
    if (!knownAgentRuns.has(req.agent_run_id)) {
      return err(404, "agent_run_not_found", `agent run not found: ${req.agent_run_id}`);
    }
    if (req.line_range.start === 0) {
      return err(400, "invalid_field", "line_range.start must be >= 1");
    }
    if (req.line_range.start > req.line_range.end) {
      return err(
        400,
        "invalid_field",
        `line_range.start (${req.line_range.start}) > end (${req.line_range.end})`,
      );
    }

    seq += 1;
    const created: DiffAnnotation = {
      id: `ann-mock-${seq}`,
      file_path: req.file_path,
      line_range: req.line_range,
      body: req.body,
      agent_run_id: req.agent_run_id,
      // seq 进毫秒位: 保证同一测试内 created_at 严格递增 (Date.now() 精度不够)
      created_at: new Date(Date.now() + seq).toISOString(),
      author: req.author,
    };
    annotations.push(created);
    return HttpResponse.json(created, { status: 201 });
  }),

  // ── GET /annotations?agent_run_id= | ?file_path= ──
  http.get(BASE, ({ request }) => {
    const q = new URL(request.url).searchParams;
    const agentRunId = q.get("agent_run_id");
    const filePath = q.get("file_path");

    if (agentRunId) {
      if (!knownAgentRuns.has(agentRunId)) {
        return err(404, "agent_run_not_found", `agent run not found: ${agentRunId}`);
      }
      return HttpResponse.json(
        annotations.filter((a) => a.agent_run_id === agentRunId),
      );
    }
    if (filePath !== null) {
      return HttpResponse.json(annotations.filter((a) => a.file_path === filePath));
    }
    return err(400, "invalid_field", "agent_run_id 或 file_path 必填");
  }),

  // ── GET /annotations/:id ──
  http.get(`${BASE}/:id`, ({ params }) => {
    const found = annotations.find((a) => a.id === params.id);
    return found
      ? HttpResponse.json(found)
      : err(404, "not_found", `annotation not found: ${params.id}`);
  }),

  // ── DELETE /annotations/:id ──
  http.delete(`${BASE}/:id`, ({ params }) => {
    const before = annotations.length;
    annotations = annotations.filter((a) => a.id !== params.id);
    return before === annotations.length
      ? err(404, "not_found", `annotation not found: ${params.id}`)
      : new HttpResponse(null, { status: 204 });
  }),
];
