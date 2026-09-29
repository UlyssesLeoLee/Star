/* CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/worktree/ProjectWorktreeLifecycleControls.test.tsx",type:"file",language:"typescript"}),
  (suite:Function {name:"ProjectWorktreeLifecycleControls tests",type:"function",visibility:"private",complexity:"moderate"}),
  (createFlow:Function {name:"accepts Worktree create receipt and refreshes Project Index",type:"function",visibility:"private",complexity:"moderate"}),
  (providerFailure:Function {name:"keeps lifecycle writes closed when provider is unavailable",type:"function",visibility:"private",complexity:"simple"}),
  (unsafeProjection:Function {name:"rejects repository projections with undeclared host path fields",type:"function",visibility:"private",complexity:"simple"}),
  (unsafeName:Function {name:"rejects repository display names that resemble host paths",type:"function",visibility:"private",complexity:"simple"}),
  (makeClient:Function {name:"makeWorktreeLifecycleClient",type:"function",visibility:"private",complexity:"moderate"}),
  (jsonResponse:Function {name:"jsonResponse",type:"function",visibility:"private",complexity:"simple"});
MATCH (file:File {name:"frontend/src/app/worktree/ProjectWorktreeLifecycleControls.test.tsx"});
CREATE (file)-[:CONTAINS]->(suite),(suite)-[:CONTAINS]->(createFlow),(suite)-[:CONTAINS]->(providerFailure),(suite)-[:CONTAINS]->(unsafeProjection),(suite)-[:CONTAINS]->(unsafeName),(suite)-[:CONTAINS]->(makeClient),(suite)-[:CONTAINS]->(jsonResponse),
       (createFlow)-[:CALLS]->(makeClient),(providerFailure)-[:CALLS]->(makeClient),(unsafeProjection)-[:CALLS]->(makeClient),(unsafeName)-[:CALLS]->(makeClient),(createFlow)-[:CALLS]->(jsonResponse),(providerFailure)-[:CALLS]->(jsonResponse),(unsafeProjection)-[:CALLS]->(jsonResponse),(unsafeName)-[:CALLS]->(jsonResponse);
*/

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ProjectWorktreeLifecycleControls } from "./ProjectWorktreeLifecycleControls";
import { WorktreeGroupApiClient } from "@/lib/group/worktreeGroupApi";

const projectId = "11111111-1111-4111-8111-111111111111";
const repositoryId = "22222222-2222-4222-8222-222222222222";
const operationId = "33333333-3333-4333-8333-333333333333";
const worktreeId = "44444444-4444-4444-8444-444444444444";
const correlationId = "55555555-5555-4555-8555-555555555555";
const idempotencyId = "66666666-6666-4666-8666-666666666666";

describe("ProjectWorktreeLifecycleControls", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("accepts a matching create receipt and refreshes the Project Index", async () => {
    const { api, fetcher } = makeClient(async (url, init) => {
      if (url.endsWith("/worktree-repositories")) {
        return jsonResponse({
          project_id: projectId,
          repositories: [{ repository_id: repositoryId, name: "star-core", default_branch: "main" }],
        });
      }
      const body = JSON.parse(String(init?.body)) as { branch: string; correlation_id: string };
      return jsonResponse({
        operation_id: operationId,
        worktree_id: worktreeId,
        project_id: projectId,
        repository_id: repositoryId,
        branch: body.branch,
        state: "provisioning",
        accepted_at: "2026-09-30T00:00:00Z",
        correlation_id: body.correlation_id,
      }, 202);
    });
    const onAccepted = vi.fn();
    vi.stubGlobal("crypto", { randomUUID: vi.fn().mockReturnValueOnce(correlationId).mockReturnValueOnce(idempotencyId) });

    render(<ProjectWorktreeLifecycleControls api={api} projectId={projectId} role="developer" onAccepted={onAccepted} />);
    expect(await screen.findByLabelText("Project Repository")).toHaveValue(repositoryId);
    fireEvent.click(screen.getByRole("button", { name: "创建 Worktree" }));

    await waitFor(() => expect(onAccepted).toHaveBeenCalledTimes(1));
    expect(await screen.findByRole("status")).toHaveTextContent(worktreeId);
    const createCall = fetcher.mock.calls.find(([input, init]) => String(input).endsWith("/worktrees") && init?.method === "POST");
    expect(createCall).toBeDefined();
    expect(new Headers(createCall?.[1]?.headers).get("Idempotency-Key")).toBe(`worktree-${idempotencyId}`);
    expect(JSON.parse(String(createCall?.[1]?.body))).toMatchObject({
      repository_id: repositoryId,
      branch: "main",
      base_ref: "main",
      correlation_id: correlationId,
    });
  });

  it("keeps lifecycle writes closed when the trusted provider is unavailable", async () => {
    const { api, fetcher } = makeClient(async () => jsonResponse({ error: { code: "worktree_lifecycle_unavailable" } }, 503));

    render(<ProjectWorktreeLifecycleControls api={api} projectId={projectId} role="developer" onAccepted={vi.fn()} />);

    expect(await screen.findByRole("alert")).toHaveTextContent("创建和导入保持关闭");
    expect(screen.queryByRole("button", { name: "创建 Worktree" })).not.toBeInTheDocument();
    expect(fetcher.mock.calls.every(([, init]) => init?.method !== "POST")).toBe(true);
  });

  it("rejects repository projections with undeclared host path fields", async () => {
    const { api, fetcher } = makeClient(async () => jsonResponse({
      project_id: projectId,
      repositories: [{ repository_id: repositoryId, name: "star-core", default_branch: "main", checkout_path: "C:\\private" }],
    }));

    render(<ProjectWorktreeLifecycleControls api={api} projectId={projectId} role="developer" onAccepted={vi.fn()} />);

    expect(await screen.findByRole("alert")).toHaveTextContent("包含无效字段");
    expect(screen.queryByRole("button", { name: "创建 Worktree" })).not.toBeInTheDocument();
    expect(fetcher.mock.calls.every(([, init]) => init?.method !== "POST")).toBe(true);
  });

  it("rejects repository display names that resemble host paths", async () => {
    const { api } = makeClient(async () => jsonResponse({
      project_id: projectId,
      repositories: [{ repository_id: repositoryId, name: "C:\\private\\repository", default_branch: "main" }],
    }));

    render(<ProjectWorktreeLifecycleControls api={api} projectId={projectId} role="developer" onAccepted={vi.fn()} />);

    expect(await screen.findByRole("alert")).toHaveTextContent("包含无效字段");
    expect(screen.queryByRole("button", { name: "创建 Worktree" })).not.toBeInTheDocument();
  });
});

function makeClient(respond: (url: string, init?: RequestInit) => Promise<Response>) {
  const fetcher = vi.fn<typeof fetch>((input, init) => respond(String(input), init));
  const api = new WorktreeGroupApiClient(async () => "user-jwt", fetcher);
  return { api, fetcher };
}

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), { status, headers: { "Content-Type": "application/json" } });
}
