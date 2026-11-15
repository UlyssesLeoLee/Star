/*
@cypher schema=1 source_sha256=c1d16d026a6f4d266d7b79fd9b69cbca968751f11522eb4005ea3feacd5efcd1
MERGE (self:File {path:"frontend/src/components/board/TaskCardCommand.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/board/TaskCardCommand.test.tsx::TaskCardCommand suite",kind:"test_suite"})
MERGE (self)-[:DEFINES]->(suite)
@endcypher
*/
// =====================================================================
// TaskCardCommand.test.tsx — 任务卡命令下达组件测试
// =====================================================================
// 覆盖:
//   1. 渲染: tcc-input / tcc-send / tcc-honest 徽标文案
//   2. Enter 发送: 调 onSubmitCommand(trimmed cmd) + 清空输入 + sent-hint
//   3. 发送按钮: 点击发送 + 空输入 disabled / 不触发
//   4. 命令历史: localStorage star-tcc-<taskId> 持久化 + tcc-history-<i> 渲染
//   5. ↑↓ 调历史: 越界钳制 + 越过最新一条恢复编辑前文本
//   6. 草稿: 输入即落盘 star-tcc-draft-<taskId>, unmount 后重挂恢复
//   7. 交接: onHandoff(target) + 清空输入 + 草稿清除; 未传 onHandoff 不炸
//   8. 提示自动隐藏: 3s 后 tcc-sent-hint 消失
//   9. helpers 纯函数: 10 条上限 / 连续重复去重 / 损坏 JSON 容错
//
// 环境: vitest jsdom 自带 localStorage, 无需 vi.stubGlobal。
// =====================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup, act } from "@testing-library/react";
import {
  TaskCardCommand,
  saveCommandHistory,
  loadCommandHistory,
  saveDraft,
  loadDraft,
  HISTORY_LIMIT,
  HANDOFF_TARGETS,
} from "./TaskCardCommand";

const TASK_ID = "T-1";

function renderCommand(
  props: Partial<Parameters<typeof TaskCardCommand>[0]> = {},
) {
  const defaults = {
    taskId: TASK_ID,
    taskKey: "T-142",
    onSubmitCommand: vi.fn(),
  };
  return render(<TaskCardCommand {...defaults} {...props} />);
}

describe("TaskCardCommand", () => {
  beforeEach(() => {
    cleanup();
    localStorage.clear();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  // ---- 1. 基础渲染 ----
  it("renders input / send / honest badge", () => {
    renderCommand();
    expect(screen.getByTestId("tcc-input")).toBeInTheDocument();
    expect(screen.getByTestId("tcc-send")).toBeInTheDocument();
    expect(screen.getByTestId("tcc-honest")).toHaveTextContent(
      "命令执行未接入 — 内容同步至聊天话题, 宿主 agent 网关接入后真实执行",
    );
    // 交接区默认目标 = Ulysses
    expect(screen.getByTestId("tcc-handoff-target")).toHaveValue("Ulysses");
  });

  // ---- 2. Enter 发送 ----
  it("sends trimmed command on Enter, clears input, shows hint, records history", () => {
    const onSubmitCommand = vi.fn();
    renderCommand({ onSubmitCommand });

    const input = screen.getByTestId("tcc-input");
    fireEvent.change(input, { target: { value: "  star run  " } });
    fireEvent.keyDown(input, { key: "Enter" });

    expect(onSubmitCommand).toHaveBeenCalledTimes(1);
    expect(onSubmitCommand).toHaveBeenCalledWith("star run");
    expect(input).toHaveValue("");
    expect(screen.getByTestId("tcc-sent-hint")).toHaveTextContent(
      "已发送到右侧聊天",
    );
    // 历史立即渲染, 最新一条在最上
    expect(screen.getByTestId("tcc-history-0")).toHaveTextContent("star run");
  });

  // ---- 3. 发送按钮 / 空输入 ----
  it("sends via send button and disables it on empty input", () => {
    const onSubmitCommand = vi.fn();
    renderCommand({ onSubmitCommand });

    const button = screen.getByTestId("tcc-send");
    expect(button).toBeDisabled();

    const input = screen.getByTestId("tcc-input");
    fireEvent.change(input, { target: { value: "git status" } });
    expect(button).toBeEnabled();

    fireEvent.click(button);
    expect(onSubmitCommand).toHaveBeenCalledWith("git status");
    expect(input).toHaveValue("");
  });

  it("does not send on Enter with empty input", () => {
    const onSubmitCommand = vi.fn();
    renderCommand({ onSubmitCommand });

    fireEvent.keyDown(screen.getByTestId("tcc-input"), { key: "Enter" });
    expect(onSubmitCommand).not.toHaveBeenCalled();
    expect(screen.queryByTestId("tcc-sent-hint")).not.toBeInTheDocument();
  });

  // ---- 4. 命令历史 ----
  it("persists history in localStorage under star-tcc-<taskId> and renders items", () => {
    saveCommandHistory(TASK_ID, "cmd-b");
    saveCommandHistory(TASK_ID, "cmd-a"); // 最新在前

    expect(localStorage.getItem(`star-tcc-${TASK_ID}`)).toBe(
      JSON.stringify(["cmd-a", "cmd-b"]),
    );

    renderCommand();
    const list = screen.getByTestId("tcc-history");
    expect(list).toBeInTheDocument();
    expect(screen.getByTestId("tcc-history-0")).toHaveTextContent("cmd-a");
    expect(screen.getByTestId("tcc-history-1")).toHaveTextContent("cmd-b");
  });

  it("clicking a history item refills the input", () => {
    saveCommandHistory(TASK_ID, "old-cmd");
    renderCommand();

    fireEvent.click(screen.getByTestId("tcc-history-0"));
    expect(screen.getByTestId("tcc-input")).toHaveValue("old-cmd");
  });

  // ---- 5. ↑↓ 调历史 ----
  it("navigates history with ArrowUp / ArrowDown and restores pre-history text", () => {
    saveCommandHistory(TASK_ID, "cmd-2");
    saveCommandHistory(TASK_ID, "cmd-1"); // ["cmd-1", "cmd-2"]
    renderCommand();

    const input = screen.getByTestId("tcc-input");
    fireEvent.change(input, { target: { value: "draft text" } });

    fireEvent.keyDown(input, { key: "ArrowUp" });
    expect(input).toHaveValue("cmd-1");
    fireEvent.keyDown(input, { key: "ArrowUp" });
    expect(input).toHaveValue("cmd-2");
    // 已到最旧一条, 再按 ↑ 钳制不动
    fireEvent.keyDown(input, { key: "ArrowUp" });
    expect(input).toHaveValue("cmd-2");

    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(input).toHaveValue("cmd-1");
    // 越过最新一条 → 恢复进入历史前正在编辑的文本
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(input).toHaveValue("draft text");
  });

  // ---- 6. 草稿 ----
  it("saves draft on typing and restores it on remount", () => {
    const { unmount } = renderCommand();
    const input = screen.getByTestId("tcc-input");
    fireEvent.change(input, { target: { value: "半条命令" } });

    expect(screen.getByTestId("tcc-draft")).toBeInTheDocument();
    expect(localStorage.getItem(`star-tcc-draft-${TASK_ID}`)).toBe("半条命令");

    unmount();
    expect(screen.queryByTestId("tcc-input")).not.toBeInTheDocument();

    // 重开组件: 草稿恢复
    renderCommand();
    const restored = screen.getByTestId("tcc-input");
    expect(restored).toHaveValue("半条命令");
    expect(screen.getByTestId("tcc-draft")).toBeInTheDocument();
  });

  it("clears draft after send", () => {
    renderCommand();
    const input = screen.getByTestId("tcc-input");
    fireEvent.change(input, { target: { value: "temp" } });
    expect(localStorage.getItem(`star-tcc-draft-${TASK_ID}`)).toBe("temp");

    fireEvent.keyDown(input, { key: "Enter" });
    expect(localStorage.getItem(`star-tcc-draft-${TASK_ID}`)).toBeNull();
  });

  // ---- 7. 交接 ----
  it("calls onHandoff with selected target and clears input", () => {
    const onHandoff = vi.fn();
    renderCommand({ onHandoff });

    const input = screen.getByTestId("tcc-input");
    fireEvent.change(input, { target: { value: "unfinished cmd" } });

    fireEvent.change(screen.getByTestId("tcc-handoff-target"), {
      target: { value: "Agent A1" },
    });
    fireEvent.click(screen.getByTestId("tcc-handoff-btn"));

    expect(onHandoff).toHaveBeenCalledTimes(1);
    expect(onHandoff).toHaveBeenCalledWith("Agent A1");
    expect(input).toHaveValue("");
    expect(localStorage.getItem(`star-tcc-draft-${TASK_ID}`)).toBeNull();
  });

  it("handoff without onHandoff prop does not throw", () => {
    renderCommand(); // onHandoff 未传
    fireEvent.click(screen.getByTestId("tcc-handoff-btn"));
    expect(screen.getByTestId("tcc-input")).toHaveValue("");
  });

  // ---- 8. 提示自动隐藏 ----
  it("auto-hides the sent hint after 3s", () => {
    vi.useFakeTimers();
    renderCommand();

    const input = screen.getByTestId("tcc-input");
    fireEvent.change(input, { target: { value: "cmd" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(screen.getByTestId("tcc-sent-hint")).toBeInTheDocument();

    act(() => {
      vi.advanceTimersByTime(3000);
    });
    expect(screen.queryByTestId("tcc-sent-hint")).not.toBeInTheDocument();
  });

  // ---- 9. helpers 纯函数 ----
  it("caps history at HISTORY_LIMIT entries (newest first)", () => {
    for (let i = 0; i < 12; i++) saveCommandHistory(TASK_ID, `c-${i}`);
    const list = loadCommandHistory(TASK_ID);
    expect(list).toHaveLength(HISTORY_LIMIT);
    expect(list[0]).toBe("c-11");
    expect(list[HISTORY_LIMIT - 1]).toBe("c-2");
  });

  it("dedupes consecutive duplicate commands", () => {
    saveCommandHistory(TASK_ID, "same");
    saveCommandHistory(TASK_ID, "same");
    expect(loadCommandHistory(TASK_ID)).toEqual(["same"]);
  });

  it("tolerates corrupt JSON and missing keys", () => {
    expect(loadCommandHistory("missing")).toEqual([]);
    localStorage.setItem("star-tcc-corrupt", "{not json");
    expect(loadCommandHistory("corrupt")).toEqual([]);
    localStorage.setItem("star-tcc-arr", JSON.stringify([1, "ok", null]));
    expect(loadCommandHistory("arr")).toEqual(["ok"]);
  });

  it("round-trips draft and removes it on empty text", () => {
    expect(loadDraft("d-1")).toBe("");
    saveDraft("d-1", "hello draft");
    expect(loadDraft("d-1")).toBe("hello draft");
    expect(localStorage.getItem("star-tcc-draft-d-1")).toBe("hello draft");
    saveDraft("d-1", "");
    expect(localStorage.getItem("star-tcc-draft-d-1")).toBeNull();
    expect(loadDraft("d-1")).toBe("");
  });

  it("exposes HANDOFF_TARGETS as Ulysses / Agent A1 / Agent B2", () => {
    expect(HANDOFF_TARGETS).toEqual(["Ulysses", "Agent A1", "Agent B2"]);
  });
});
