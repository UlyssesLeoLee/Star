// frontend/src/components/worktree-shared/__tests__/SidebarWorktreesCard.test.tsx
//
// ULYS-228 (ULYS-218.1) — FR-ORCA-011 AC-2 sidebar "hidden worktrees" card.
//
// 覆盖:
//   1. 默认折叠 (data-expanded=false), 显示 "(N)" count
//   2. 展开后列出 external worktree rows
//   3. 选 row → Import 按钮 → 调 POST → 成功后 row 从 list 消失
//   4. scan 失败 → 显示 error 文案
//   5. empty 时显示 "无 hidden worktree" 文案
//   6. 切换 onCount callback 透传 (sidebar 折叠态用)

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/server";
import { SidebarWorktreesCard } from "../SidebarWorktreesCard";

const SCAN_OK = {
  items: [
    {
      path: "C:/wt/ext-1",
      head_commit: "1234567890abcdef1234567890abcdef12345678",
      branch: "ext-1",
      is_managed: false,
    },
    {
      path: "C:/wt/ext-2",
      head_commit: "fedcba0987654321fedcba0987654321fedcba09",
      branch: null,
      is_managed: false,
    },
  ],
};

const SCAN_EMPTY = { items: [] };

beforeEach(() => {
  server.resetHandlers();
});

describe("SidebarWorktreesCard", () => {
  it("renders collapsed by default with count", async () => {
    server.use(
      http.get("/api/v1/worktrees/external-import/scan", () =>
        HttpResponse.json(SCAN_OK),
      ),
    );

    render(<SidebarWorktreesCard repoId="r1" />);

    const card = screen.getByTestId("sidebar-worktrees-card");
    expect(card.getAttribute("data-expanded")).toBe("false");

    await waitFor(() => {
      expect(card.textContent).toMatch(/Hidden worktrees \(2\)/);
    });
  });

  it("expands to show external worktree rows", async () => {
    server.use(
      http.get("/api/v1/worktrees/external-import/scan", () =>
        HttpResponse.json(SCAN_OK),
      ),
    );

    render(<SidebarWorktreesCard repoId="r1" />);

    await waitFor(() => {
      screen.getByTestId("sidebar-worktrees-toggle");
    });

    fireEvent.click(screen.getByTestId("sidebar-worktrees-toggle"));

    await waitFor(() => {
      const rows = screen.getAllByTestId("sidebar-worktrees-row");
      expect(rows.length).toBe(2);
    });

    expect(
      screen.getByTestId("sidebar-worktrees-card").getAttribute("data-expanded"),
    ).toBe("true");
  });

  it("filters out is_managed=true items from scan", async () => {
    server.use(
      http.get("/api/v1/worktrees/external-import/scan", () =>
        HttpResponse.json({
          items: [
            { ...SCAN_OK.items[0] },
            {
              path: "C:/wt/managed",
              head_commit: "0000000000000000000000000000000000000000",
              branch: "managed",
              is_managed: true,
            },
          ],
        }),
      ),
    );

    render(<SidebarWorktreesCard repoId="r1" />);

    await waitFor(() => {
      expect(screen.getByTestId("sidebar-worktrees-card").textContent).toMatch(
        /Hidden worktrees \(1\)/,
      );
    });
  });

  it("Import button calls POST and removes row", async () => {
    let postedBody: unknown = null;
    server.use(
      http.get("/api/v1/worktrees/external-import/scan", () =>
        HttpResponse.json(SCAN_OK),
      ),
      http.post("/api/v1/worktrees/external-import", async ({ request }) => {
        postedBody = await request.json();
        return HttpResponse.json({
          worktree_id: "wt-new",
          state: "Provisioning",
          at: new Date().toISOString(),
        });
      }),
    );

    render(<SidebarWorktreesCard repoId="r1" />);

    await waitFor(() => {
      screen.getByTestId("sidebar-worktrees-toggle");
    });
    fireEvent.click(screen.getByTestId("sidebar-worktrees-toggle"));

    await waitFor(() => {
      screen.getAllByTestId("sidebar-worktrees-row");
    });

    const importBtn = screen.getAllByTestId("sidebar-worktrees-import")[0];
    fireEvent.click(importBtn);

    await waitFor(() => {
      expect(postedBody).toEqual({ repo_id: "r1", path: "C:/wt/ext-1" });
    });
  });

  it("shows error message when scan fails", async () => {
    server.use(
      http.get("/api/v1/worktrees/external-import/scan", () =>
        HttpResponse.json({ code: "INTERNAL" }, { status: 500 }),
      ),
    );

    render(<SidebarWorktreesCard repoId="r1" />);

    fireEvent.click(screen.getByTestId("sidebar-worktrees-toggle"));

    await waitFor(() => {
      expect(screen.getByTestId("sidebar-worktrees-error")).toBeInTheDocument();
    });
  });

  it("shows empty message when scan returns 0 items", async () => {
    server.use(
      http.get("/api/v1/worktrees/external-import/scan", () =>
        HttpResponse.json(SCAN_EMPTY),
      ),
    );

    render(<SidebarWorktreesCard repoId="r1" />);

    fireEvent.click(screen.getByTestId("sidebar-worktrees-toggle"));

    await waitFor(() => {
      expect(screen.getByTestId("sidebar-worktrees-empty")).toBeInTheDocument();
    });
  });

  it("passes count to onCount callback when scan completes", async () => {
    server.use(
      http.get("/api/v1/worktrees/external-import/scan", () =>
        HttpResponse.json(SCAN_OK),
      ),
    );

    const onCount = vi.fn();
    render(<SidebarWorktreesCard repoId="r1" onCount={onCount} />);

    await waitFor(() => {
      expect(onCount).toHaveBeenCalledWith(2);
    });
  });
});