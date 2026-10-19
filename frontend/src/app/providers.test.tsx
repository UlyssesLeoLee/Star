/*
@cypher schema=1 source_sha256=c9a535aed94be6f272ed10fae424ec0c339312142251c72e62fa4d9b7a54e890
MERGE (self:File {path:"frontend/src/app/providers.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/app/providers.test.tsx::Providers host session composition tests",kind:"suite"})
MERGE (probe:Symbol {id:"frontend/src/app/providers.test.tsx::SessionProbe",kind:"function"})
MERGE (makeSession:Symbol {id:"frontend/src/app/providers.test.tsx::session",kind:"function"})
MERGE (sessionCase:Symbol {id:"frontend/src/app/providers.test.tsx::shares the injected host session case",kind:"test"})
MERGE (missingCase:Symbol {id:"frontend/src/app/providers.test.tsx::keeps Group unavailable without a host session case",kind:"test"})
MERGE (providers:Symbol {id:"frontend/src/app/providers.tsx::Providers",kind:"function"})
MERGE (groupProvider:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::WorktreeGroupApiProvider",kind:"function"})
MERGE (api:Symbol {id:"frontend/src/lib/group/worktreeGroupApi.ts::WorktreeGroupApiClient.getGroupContext",kind:"method"})
MERGE (runHook:Symbol {id:"frontend/src/lib/run/runDirectorySession.tsx::useRunDirectory",kind:"function"})
MERGE (groupHook:Symbol {id:"frontend/src/lib/group/groupProjection.tsx::useWorktreeGroupApi",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:DEFINES]->(probe)
MERGE (self)-[:DEFINES]->(makeSession)
MERGE (suite)-[:DEFINES]->(sessionCase)
MERGE (suite)-[:DEFINES]->(missingCase)
MERGE (sessionCase)-[:CALLS]->(providers)
MERGE (sessionCase)-[:CALLS]->(api)
MERGE (sessionCase)-[:TESTS]->(groupProvider)
MERGE (probe)-[:CALLS]->(api)
MERGE (probe)-[:CALLS]->(runHook)
MERGE (probe)-[:CALLS]->(groupHook)
MERGE (sessionCase)-[:CALLS]->(providers)
MERGE (missingCase)-[:CALLS]->(providers)
MERGE (missingCase)-[:TESTS]->(groupProvider)
@endcypher
*/

import { useEffect, type ReactNode } from "react";
import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { Providers } from "./providers";
import { useWorktreeGroupApi } from "@/lib/group/groupProjection";
import { useRunDirectory, type RunDirectoryHostSession } from "@/lib/run/runDirectorySession";
import type { WorktreeGroupApiClient } from "@/lib/group/worktreeGroupApi";

vi.mock("@/components/theme/ThemeProvider", () => ({
  ThemeProvider: ({ children }: { children: ReactNode }) => children,
}));

let latestGroupClient: WorktreeGroupApiClient | null = null;

function SessionProbe() {
  const runSession = useRunDirectory();
  const groupClient = useWorktreeGroupApi();
  latestGroupClient = groupClient;

  useEffect(() => {
    if (groupClient) void groupClient.getGroupContext("worktree-1");
  }, [groupClient]);

  return (
    <>
      <output data-testid="run-session-status">{runSession.status}</output>
      <output data-testid="group-session-status">{groupClient ? "ready" : "missing"}</output>
    </>
  );
}

function session(generation: string, getAccessToken: RunDirectoryHostSession["getAccessToken"], fetcher: typeof fetch): RunDirectoryHostSession {
  return { generation, getAccessToken, fetcher };
}

describe("Providers host session composition", () => {
  it("shares the injected host session callback and replaces clients when generation changes", async () => {
    latestGroupClient = null;
    let token = "host-token-1";
    const getAccessToken = vi.fn(async () => token);
    const fetcher = vi.fn<typeof fetch>().mockImplementation(async () => new Response(JSON.stringify({ ok: true }), { status: 200 }));
    const hostSession = session("host-generation-1", getAccessToken, fetcher);
    const { rerender } = render(<Providers runDirectorySession={hostSession}><SessionProbe /></Providers>);

    expect(screen.getByTestId("run-session-status")).toHaveTextContent("ready");
    expect(screen.getByTestId("group-session-status")).toHaveTextContent("ready");
    await waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
    expect(getAccessToken).toHaveBeenCalledTimes(1);
    expect(fetcher.mock.calls[0][0]).toBe("/api/v1/worktrees/worktree-1/group-context");
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe("Bearer host-token-1");
    const firstClient = latestGroupClient;

    token = "host-token-2";
    rerender(<Providers runDirectorySession={session("host-generation-2", getAccessToken, fetcher)}><SessionProbe /></Providers>);
    await waitFor(() => expect(fetcher).toHaveBeenCalledTimes(2));
    expect(latestGroupClient).not.toBe(firstClient);
    expect(new Headers(fetcher.mock.calls[1][1]?.headers).get("Authorization")).toBe("Bearer host-token-2");
  });

  it("keeps Group unavailable when the embedding host has not supplied a session", () => {
    latestGroupClient = null;
    render(<Providers><SessionProbe /></Providers>);

    expect(screen.getByTestId("run-session-status")).toHaveTextContent("missing");
    expect(screen.getByTestId("group-session-status")).toHaveTextContent("missing");
  });
});
