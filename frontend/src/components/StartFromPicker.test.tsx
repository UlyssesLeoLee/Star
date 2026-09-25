// =====================================================================
// StartFromPicker.test.tsx — Start-from Picker 4 选 1 UI 测试 (per ULYS-194 §6)
//
// 覆盖:
//   - 4 tab 切换
//   - 候选列表渲染
//   - 选中态 → confirm 按钮 enabled
//   - onConfirm 回调触发 + 传 PickerCandidate
//   - 空 category 兜底
//   - loading / error 状态
//   - toPickerKind / kindToTab 工具函数
// =====================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup, waitFor } from "@testing-library/react";
import {
  StartFromPicker,
  fetchPickerCandidates,
  toPickerKind,
  kindToTab,
  type PickerCandidatesResponse,
  type PickerCandidate,
} from "./StartFromPicker";

// --- fixtures ---

const gh: PickerCandidate = {
  id: "remote:origin/main",
  label: "★ main",
  description: "origin/main @ abc1234",
  kind: "remote_branch",
};
const gh2: PickerCandidate = {
  id: "remote:origin/feat-x",
  label: "feat-x",
  description: "origin/feat-x @ def5678",
  kind: "remote_branch",
};
const ex: PickerCandidate = {
  id: "wt:11111111-2222-3333-4444-555555555555",
  label: "main",
  description: "Existing worktree on branch 'main'",
  kind: "local_branch",
};
const lp: PickerCandidate = {
  id: "local:/tmp/repo-a",
  label: "repo-a",
  description: "/tmp/repo-a",
  kind: "repo_base",
};
const em: PickerCandidate = {
  id: "empty:repo",
  label: "Empty / Start from scratch",
  description: "No base ref",
  kind: "commit_sha",
};

const FIXTURE: PickerCandidatesResponse = {
  github_branches: [gh, gh2],
  existing_worktrees: [ex],
  local_paths: [lp],
  empty: em,
};

// --- kind 工具 ---

describe("PickerKind helpers", () => {
  it("toPickerKind maps all backend variants", () => {
    expect(toPickerKind("remote_branch")).toBe("remote_branch");
    expect(toPickerKind("local_branch")).toBe("local_branch");
    expect(toPickerKind("repo_base")).toBe("repo_base");
    expect(toPickerKind("commit_sha")).toBe("commit_sha");
    expect(toPickerKind("unknown")).toBeNull();
  });

  it("kindToTab round-trips", () => {
    expect(kindToTab("remote_branch")).toBe("github");
    expect(kindToTab("local_branch")).toBe("existing");
    expect(kindToTab("repo_base")).toBe("local");
    expect(kindToTab("commit_sha")).toBe("empty");
  });
});

// --- 组件渲染 ---

describe("StartFromPicker", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  afterEach(() => {
    cleanup();
  });

  it("renders 4 tabs with counts", () => {
    render(
      <StartFromPicker
        repoId="r-1"
        onConfirm={() => {}}
        initialCandidates={FIXTURE}
      />,
    );

    expect(screen.getByTestId("start-from-picker-tab-github")).toBeTruthy();
    expect(screen.getByTestId("start-from-picker-tab-existing")).toBeTruthy();
    expect(screen.getByTestId("start-from-picker-tab-local")).toBeTruthy();
    expect(screen.getByTestId("start-from-picker-tab-empty")).toBeTruthy();

    // counts
    expect(screen.getByTestId("start-from-picker-tab-github-count").textContent).toBe("2");
    expect(screen.getByTestId("start-from-picker-tab-existing-count").textContent).toBe("1");
    expect(screen.getByTestId("start-from-picker-tab-local-count").textContent).toBe("1");
    // empty 永远 1 个, 但只 1 个 chip 也算
    expect(screen.getByTestId("start-from-picker-tab-empty-count").textContent).toBe("1");
  });

  it("shows github branch candidates by default", () => {
    render(
      <StartFromPicker
        repoId="r-1"
        onConfirm={() => {}}
        initialCandidates={FIXTURE}
      />,
    );

    expect(screen.getByTestId(`start-from-picker-option-${gh.id}`)).toBeTruthy();
    expect(screen.getByTestId(`start-from-picker-option-${gh2.id}`)).toBeTruthy();
  });

  it("switches tab and shows correct category", () => {
    render(
      <StartFromPicker
        repoId="r-1"
        onConfirm={() => {}}
        initialCandidates={FIXTURE}
      />,
    );

    fireEvent.click(screen.getByTestId("start-from-picker-tab-existing"));
    expect(screen.getByTestId(`start-from-picker-option-${ex.id}`)).toBeTruthy();

    fireEvent.click(screen.getByTestId("start-from-picker-tab-local"));
    expect(screen.getByTestId(`start-from-picker-option-${lp.id}`)).toBeTruthy();

    fireEvent.click(screen.getByTestId("start-from-picker-tab-empty"));
    expect(screen.getByTestId(`start-from-picker-option-${em.id}`)).toBeTruthy();
  });

  it("shows empty list message when category is empty", () => {
    const partial: PickerCandidatesResponse = {
      github_branches: [],
      existing_worktrees: [],
      local_paths: [],
      empty: em,
    };
    render(
      <StartFromPicker
        repoId="r-1"
        onConfirm={() => {}}
        initialCandidates={partial}
      />,
    );

    fireEvent.click(screen.getByTestId("start-from-picker-tab-github"));
    expect(screen.getByTestId("start-from-picker-empty-list")).toBeTruthy();
  });

  it("confirm disabled until selection made, enabled after", () => {
    render(
      <StartFromPicker
        repoId="r-1"
        onConfirm={() => {}}
        initialCandidates={FIXTURE}
      />,
    );

    const btn = screen.getByTestId("start-from-picker-confirm") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);

    fireEvent.click(screen.getByTestId(`start-from-picker-option-${gh.id}`));
    expect(btn.disabled).toBe(false);
  });

  it("onConfirm fires with selected PickerCandidate", () => {
    const onConfirm = vi.fn();
    render(
      <StartFromPicker
        repoId="r-1"
        onConfirm={onConfirm}
        initialCandidates={FIXTURE}
      />,
    );

    fireEvent.click(screen.getByTestId(`start-from-picker-option-${gh.id}`));
    fireEvent.click(screen.getByTestId("start-from-picker-confirm"));

    expect(onConfirm).toHaveBeenCalledWith(gh);
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("onCancel fires when cancel clicked", () => {
    const onCancel = vi.fn();
    render(
      <StartFromPicker
        repoId="r-1"
        onConfirm={() => {}}
        onCancel={onCancel}
        initialCandidates={FIXTURE}
      />,
    );

    fireEvent.click(screen.getByTestId("start-from-picker-cancel"));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("switching tab clears selection", () => {
    render(
      <StartFromPicker
        repoId="r-1"
        onConfirm={() => {}}
        initialCandidates={FIXTURE}
      />,
    );

    fireEvent.click(screen.getByTestId(`start-from-picker-option-${gh.id}`));
    expect(screen.getByTestId("start-from-picker-selected")).toBeTruthy();

    fireEvent.click(screen.getByTestId("start-from-picker-tab-existing"));
    expect(screen.queryByTestId("start-from-picker-selected")).toBeNull();
  });

  it("shows loading state during fetch", () => {
    // 没传 initialCandidates → 应进入 loading 状态
    render(
      <StartFromPicker repoId="r-1" onConfirm={() => {}} />,
    );
    expect(screen.getByTestId("start-from-picker-loading")).toBeTruthy();
  });

  it("shows error state when fetcher throws", async () => {
    const failingFetcher = vi.fn().mockRejectedValue(new Error("network fail"));
    render(
      <StartFromPicker
        repoId="r-1"
        onConfirm={() => {}}
        fetcher={failingFetcher}
      />,
    );

    await waitFor(() => {
      expect(screen.getByTestId("start-from-picker-error")).toBeTruthy();
    });
    expect(failingFetcher).toHaveBeenCalledWith(
      expect.objectContaining({ repoId: "r-1" }),
    );
  });

  it("selected option gets aria-selected=true", () => {
    render(
      <StartFromPicker
        repoId="r-1"
        onConfirm={() => {}}
        initialCandidates={FIXTURE}
      />,
    );

    const option = screen.getByTestId(`start-from-picker-option-${gh.id}`);
    expect(option.getAttribute("aria-selected")).toBe("false");

    fireEvent.click(option);
    expect(option.getAttribute("aria-selected")).toBe("true");
  });

  it("active tab gets aria-selected=true", () => {
    render(
      <StartFromPicker
        repoId="r-1"
        onConfirm={() => {}}
        initialCandidates={FIXTURE}
        initialTab="existing"
      />,
    );

    const githubTab = screen.getByTestId("start-from-picker-tab-github");
    const existingTab = screen.getByTestId("start-from-picker-tab-existing");
    expect(githubTab.getAttribute("aria-selected")).toBe("false");
    expect(existingTab.getAttribute("aria-selected")).toBe("true");
  });

  it("fetchPickerCandidates hits /api/v1/worktree-picker/candidates", async () => {
    // mock global fetch
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      statusText: "OK",
      json: async () => FIXTURE,
    });
    vi.stubGlobal("fetch", mockFetch);

    const result = await fetchPickerCandidates({ repoId: "r-1" });
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining("/api/v1/worktree-picker/candidates?repo_id=r-1"),
      expect.objectContaining({ credentials: "include" }),
    );
    expect(result).toEqual(FIXTURE);

    vi.unstubAllGlobals();
  });
});