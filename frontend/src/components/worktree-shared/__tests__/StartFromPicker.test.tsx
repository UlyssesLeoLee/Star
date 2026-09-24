// frontend/src/components/worktree-shared/__tests__/StartFromPicker.test.tsx
//
// ULYS-228 (ULYS-218.1) — FR-ORCA-009 Start-from Picker modal.
//
// 覆盖:
//   1. 4 选 1 radio 渲染 + 默认选中 repo_base
//   2. 切换 kind → 候选 list 自动切换
//   3. 选中 candidate + confirm → onSelect 回调 (kind/candidate_id/label 都对)
//   4. 取消按钮 → onCancel 回调
//   5. backdrop 点击 → onCancel 回调
//   6. Esc 按键 → onCancel 回调
//   7. fetch 503 / 网络错误 → 显示 error 文案, 但仍可继续
//   8. fetch 空候选 → 显示 empty 文案

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { http, HttpResponse } from "msw";
import { server } from "@/mocks/server";
import { StartFromPicker } from "../StartFromPicker";
import type { PickerCandidatesDto } from "../startFromPickerTypes";

const MOCK_OK: PickerCandidatesDto = {
  github_branches: [
    {
      id: "remote_branch:origin/feat-x",
      label: "origin/feat-x",
      description: "remote",
      kind: "remote_branch",
    },
  ],
  existing_worktrees: [
    {
      id: "local_branch:feat-y",
      label: "feat-y",
      description: "local",
      kind: "local_branch",
    },
  ],
  local_paths: [
    {
      id: "commit_sha:abc1234",
      label: "abc1234 — chore",
      description: "abc1234567890abcdef1234567890abcdef123456",
      kind: "commit_sha",
    },
  ],
  empty: {
    id: "repo_base:origin/main",
    label: "origin/main",
    description: "base ref",
    kind: "repo_base",
  },
};

beforeEach(() => {
  // reset 默认 handler (避免 cross-test 污染)
  server.resetHandlers();
});

describe("StartFromPicker", () => {
  it("renders modal with 4 radios and defaults to repo_base", async () => {
    server.use(
      http.get("/api/v1/worktree-picker/candidates", () =>
        HttpResponse.json(MOCK_OK),
      ),
    );

    render(
      <StartFromPicker repoId="repo-1" onSelect={vi.fn()} onCancel={vi.fn()} />,
    );

    expect(screen.getByTestId("start-from-picker-modal")).toBeInTheDocument();
    expect(
      screen.getByTestId("start-from-radio-repo_base"),
    ).toBeChecked();

    // 等 candidates 加载, 默认 kind=repo_base → 显示 1 个 candidate (MOCK_OK.empty)
    await waitFor(() => {
      const list = screen.getByTestId("start-from-list");
      expect(list.querySelectorAll("button[data-testid='start-from-candidate']").length).toBe(1);
    });
  });

  it("switching kind updates candidate list", async () => {
    server.use(
      http.get("/api/v1/worktree-picker/candidates", () =>
        HttpResponse.json(MOCK_OK),
      ),
    );

    render(
      <StartFromPicker repoId="repo-1" onSelect={vi.fn()} onCancel={vi.fn()} />,
    );

    await waitFor(() => {
      expect(
        screen.getByTestId("start-from-radio-local_branch"),
      ).toBeInTheDocument();
    });

    // 切到 local_branch → 显示 1 个 candidate (MOCK_OK.existing_worktrees[0])
    fireEvent.click(screen.getByTestId("start-from-radio-local_branch"));

    await waitFor(() => {
      const list = screen.getByTestId("start-from-list");
      expect(list.getAttribute("data-kind")).toBe("local_branch");
      expect(
        list.querySelector(
          "button[data-testid='start-from-candidate'][data-candidate-id='local_branch:feat-y']",
        ),
      ).not.toBeNull();
    });
  });

  it("confirms selection with kind + candidate_id + label", async () => {
    server.use(
      http.get("/api/v1/worktree-picker/candidates", () =>
        HttpResponse.json(MOCK_OK),
      ),
    );

    const onSelect = vi.fn();
    render(
      <StartFromPicker
        repoId="repo-1"
        onSelect={onSelect}
        onCancel={vi.fn()}
      />,
    );

    // 切到 local_branch, 选 feat-y, confirm
    fireEvent.click(screen.getByTestId("start-from-radio-local_branch"));
    await waitFor(() => {
      screen.getByTestId("start-from-list");
    });
    const items = screen.getAllByTestId("start-from-candidate");
    fireEvent.click(items[0]);

    fireEvent.click(screen.getByTestId("start-from-confirm"));

    expect(onSelect).toHaveBeenCalledOnce();
    expect(onSelect).toHaveBeenCalledWith({
      kind: "local_branch",
      candidate_id: "local_branch:feat-y",
      label: "feat-y",
    });
  });

  it("calls onCancel when cancel button clicked", () => {
    const onCancel = vi.fn();
    render(
      <StartFromPicker repoId="r1" onSelect={vi.fn()} onCancel={onCancel} />,
    );
    fireEvent.click(screen.getByTestId("start-from-cancel"));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it("calls onCancel when backdrop clicked", () => {
    const onCancel = vi.fn();
    render(
      <StartFromPicker repoId="r1" onSelect={vi.fn()} onCancel={onCancel} />,
    );
    fireEvent.click(screen.getByTestId("start-from-picker-modal"));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it("calls onCancel when Esc pressed", () => {
    const onCancel = vi.fn();
    render(
      <StartFromPicker repoId="r1" onSelect={vi.fn()} onCancel={onCancel} />,
    );
    act(() => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    });
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it("displays error when fetch returns 5xx, but does not throw", async () => {
    server.use(
      http.get("/api/v1/worktree-picker/candidates", () =>
        HttpResponse.json(
          { code: "PICKER.NOT_REGISTERED" },
          { status: 503 },
        ),
      ),
    );

    render(
      <StartFromPicker repoId="r1" onSelect={vi.fn()} onCancel={vi.fn()} />,
    );

    await waitFor(() => {
      expect(screen.getByTestId("start-from-error")).toBeInTheDocument();
    });
    expect(screen.getByTestId("start-from-error").textContent).toMatch(/503/);
  });

  it("shows empty message when candidates list is empty for kind", async () => {
    server.use(
      http.get("/api/v1/worktree-picker/candidates", () =>
        HttpResponse.json({
          github_branches: [],
          existing_worktrees: [],
          local_paths: [],
          empty: null,
        }),
      ),
    );

    render(
      <StartFromPicker repoId="r1" onSelect={vi.fn()} onCancel={vi.fn()} />,
    );

    await waitFor(() => {
      expect(screen.getByTestId("start-from-empty")).toBeInTheDocument();
    });
  });

  it("confirm button is disabled until a candidate is picked", async () => {
    server.use(
      http.get("/api/v1/worktree-picker/candidates", () =>
        HttpResponse.json(MOCK_OK),
      ),
    );

    render(
      <StartFromPicker repoId="r1" onSelect={vi.fn()} onCancel={vi.fn()} />,
    );

    const confirm = screen.getByTestId("start-from-confirm");
    expect(confirm).toBeDisabled();

    // 切到 remote_branch, 选第一项
    fireEvent.click(screen.getByTestId("start-from-radio-remote_branch"));
    await waitFor(() => {
      screen.getByTestId("start-from-list");
    });
    const items = screen.getAllByTestId("start-from-candidate");
    fireEvent.click(items[0]);

    expect(confirm).not.toBeDisabled();
  });
});