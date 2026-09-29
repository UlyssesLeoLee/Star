/* CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/worktree/AuthorizedProjectSelector.test.tsx",type:"file",language:"tsx"}),
  (suite:Function {name:"AuthorizedProjectSelector tests",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
  (pagination:Function {name:"authorized Project pagination test",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
  (failure:Function {name:"authorized Project failure test",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
  (identityChange:Function {name:"authorized Project identity change test",type:"function",language:"typescript",visibility:"private",complexity:"moderate"}),
  (selector:Function {name:"AuthorizedProjectSelector",type:"function",language:"tsx",visibility:"public"}),
  (file)-[:CONTAINS]->(suite),(file)-[:CONTAINS]->(pagination),(file)-[:CONTAINS]->(failure),(file)-[:CONTAINS]->(identityChange),(file)-[:USES]->(selector),(suite)-[:CONTAINS]->(pagination),(suite)-[:CONTAINS]->(failure),(suite)-[:CONTAINS]->(identityChange);
*/

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { WorktreeGroupApiClient } from "@/lib/group/worktreeGroupApi";
import { AuthorizedProjectSelector } from "./AuthorizedProjectSelector";

const projectOne = "22222222-2222-4222-8222-222222222222";
const projectTwo = "33333333-3333-4333-8333-333333333333";

afterEach(cleanup);

function renderSelector(listAuthorizedProjects: ReturnType<typeof vi.fn>) {
  const onSelect = vi.fn();
  const onStateChange = vi.fn();
  const api = { listAuthorizedProjects } as unknown as WorktreeGroupApiClient;
  render(
    <AuthorizedProjectSelector
      api={api}
      selectedProjectId=""
      onSelect={onSelect}
      onStateChange={onStateChange}
    />,
  );
  return { onSelect, onStateChange };
}

describe("AuthorizedProjectSelector", () => {
  it("uses only authorized Project rows and loads later pages", async () => {
    const listAuthorizedProjects = vi.fn()
      .mockResolvedValueOnce({
        projects: [{ project_id: projectOne, role: "developer" }],
        limit: 200,
        next_cursor: projectOne,
      })
      .mockResolvedValueOnce({
        projects: [{ project_id: projectTwo, role: "viewer" }],
        limit: 200,
        next_cursor: null,
      });
    const { onSelect } = renderSelector(listAuthorizedProjects);

    await screen.findByRole("option", { name: `Project ${projectOne}` });
    fireEvent.change(screen.getByTestId("worktree-project-selector"), { target: { value: projectOne } });
    expect(onSelect).toHaveBeenCalledWith(projectOne);
    fireEvent.click(screen.getByRole("button", { name: "加载更多 Project" }));
    await screen.findByRole("option", { name: `Project ${projectTwo}` });
    expect(listAuthorizedProjects).toHaveBeenNthCalledWith(1, { limit: 200, cursor: undefined });
    expect(listAuthorizedProjects).toHaveBeenNthCalledWith(2, { limit: 200, cursor: projectOne });
  });

  it("fails closed on directory errors and retries without a local seed fallback", async () => {
    const listAuthorizedProjects = vi.fn()
      .mockRejectedValueOnce(new Error("session_required"))
      .mockResolvedValueOnce({ projects: [], limit: 200, next_cursor: null });
    renderSelector(listAuthorizedProjects);

    expect(await screen.findByRole("alert")).toHaveTextContent("不会回退到本地 seed");
    expect(screen.queryByText(/seed-project/i)).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "重试" }));
    await waitFor(() => expect(screen.getByText("当前用户没有可访问的 Project。")).toBeInTheDocument());
    expect(listAuthorizedProjects).toHaveBeenCalledTimes(2);
  });

  it("hides the previous actor directory immediately when the API session changes", async () => {
    const firstApi = {
      listAuthorizedProjects: vi.fn().mockResolvedValue({
        projects: [{ project_id: projectOne, role: "developer" }], limit: 200, next_cursor: null,
      }),
    } as unknown as WorktreeGroupApiClient;
    let resolveSecond!: (value: unknown) => void;
    const secondApi = {
      listAuthorizedProjects: vi.fn().mockReturnValue(new Promise((resolve) => { resolveSecond = resolve; })),
    } as unknown as WorktreeGroupApiClient;
    const onSelect = vi.fn();
    const onStateChange = vi.fn();
    const props = { selectedProjectId: "", onSelect, onStateChange };
    const view = render(<AuthorizedProjectSelector api={firstApi} {...props} />);

    await screen.findByRole("option", { name: `Project ${projectOne}` });
    view.rerender(<AuthorizedProjectSelector api={secondApi} {...props} />);
    expect(screen.queryByRole("option", { name: `Project ${projectOne}` })).not.toBeInTheDocument();
    expect(screen.getByTestId("worktree-project-selector")).toBeDisabled();

    resolveSecond({ projects: [{ project_id: projectTwo, role: "viewer" }], limit: 200, next_cursor: null });
    await screen.findByRole("option", { name: `Project ${projectTwo}` });
  });
});
