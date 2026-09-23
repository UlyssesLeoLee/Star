// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/__tests__/DiffAnnotator.test.tsx
//
// DiffAnnotator 渲染 + 交互单测 (per ULYS-211 / FR-ORCA-035)。
// 与 e2e/diff-annotation.spec.ts 覆盖同一条 add → list → feed_to_agent 链路,
// 区别是这里跑 jsdom + MSW, 不依赖浏览器。

import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { DiffAnnotator } from "@/components/DiffAnnotator";
import { useAnnotationStore } from "@/lib/hooks/use-annotations";
import {
  DEMO_AGENT_RUN_ID,
  DEMO_DIFF_LINES,
  DEMO_FILE_PATH,
} from "@/mocks/data/annotations";
import { __resetAnnotationMock } from "@/mocks/handlers/annotations";

// ws endpoint 尚未落地 (见 lib/api/annotations.ts 文件头), jsdom 里不要真连
class NoopWebSocket {
  static readonly CLOSED = 3;
  readyState = NoopWebSocket.CLOSED;
  onmessage: unknown = null;
  onerror: unknown = null;
  close() {
    /* no-op */
  }
}

beforeEach(() => {
  vi.stubGlobal("WebSocket", NoopWebSocket);
  __resetAnnotationMock();
  useAnnotationStore.getState().reset();
});

function renderAnnotator() {
  return render(
    <DiffAnnotator
      filePath={DEMO_FILE_PATH}
      agentRunId={DEMO_AGENT_RUN_ID}
      lines={DEMO_DIFF_LINES}
    />,
  );
}

/** 打开第 line 行的 composer, 填 body, 提交 */
async function dropComment(
  user: ReturnType<typeof userEvent.setup>,
  line: number,
  body: string,
) {
  await user.click(screen.getByTestId(`annotation-gutter-${line}`));
  await user.type(screen.getByTestId("annotation-body-input"), body);
  await user.click(screen.getByTestId("annotation-submit"));
}

describe("DiffAnnotator", () => {
  it("渲染 diff, 新文件侧有行号的行才可标注", async () => {
    renderAnnotator();
    await waitFor(() => expect(screen.getByTestId("annotation-empty")).toBeInTheDocument());

    // added / context 行有 gutter
    expect(screen.getByTestId("annotation-gutter-15")).toBeInTheDocument();
    expect(screen.getByTestId("annotation-gutter-12")).toBeInTheDocument();
    // removed 行 + hunk 头在新文件侧没有行号 → 不可标注 (annotate.rs 要求 1-based)
    const gutters = screen.getAllByRole("button", { name: /添加标注/ });
    const annotatable = DEMO_DIFF_LINES.filter((l) => l.newLine !== null);
    expect(gutters).toHaveLength(annotatable.length);
  });

  it("add → 列表出现该条, 行上出现角标, 计数更新", async () => {
    const user = userEvent.setup();
    renderAnnotator();
    await waitFor(() => expect(screen.getByTestId("annotation-empty")).toBeInTheDocument());

    await dropComment(user, 15, "now_v7 要确认 uuid feature 打开了");

    const list = await screen.findByTestId("annotation-list");
    const items = await within(list).findAllByTestId("annotation-item");
    expect(items).toHaveLength(1);
    expect(items[0]).toHaveTextContent("now_v7 要确认 uuid feature 打开了");
    expect(items[0]).toHaveTextContent("L15");
    expect(screen.getByTestId("annotation-marker-15")).toHaveTextContent("1");
    expect(screen.getByTestId("annotation-count")).toHaveTextContent("1 条标注");
    // 提交后 composer 收起
    expect(screen.queryByTestId("annotation-composer")).not.toBeInTheDocument();
  });

  it("shift+点击扩选 → 半开区间覆盖整段, 每行都打角标", async () => {
    const user = userEvent.setup();
    renderAnnotator();
    await waitFor(() => expect(screen.getByTestId("annotation-empty")).toBeInTheDocument());

    await user.click(screen.getByTestId("annotation-gutter-15"));
    await user.keyboard("{Shift>}");
    await user.click(screen.getByTestId("annotation-gutter-17"));
    await user.keyboard("{/Shift}");

    expect(screen.getByTestId("composer-range")).toHaveTextContent("行 15-17");

    await user.type(screen.getByTestId("annotation-body-input"), "这三行合并成一个 helper");
    await user.click(screen.getByTestId("annotation-submit"));

    const item = await screen.findByTestId("annotation-item");
    expect(item).toHaveTextContent("L15-17");
    for (const line of [15, 16, 17]) {
      expect(screen.getByTestId(`annotation-marker-${line}`)).toBeInTheDocument();
    }
    // 半开区间 end 不含 → 第 18 行不该被标
    expect(screen.queryByTestId("annotation-marker-18")).not.toBeInTheDocument();
  });

  it("空 body 时提交按钮禁用; 取消按钮收起 composer", async () => {
    const user = userEvent.setup();
    renderAnnotator();
    await waitFor(() => expect(screen.getByTestId("annotation-empty")).toBeInTheDocument());

    await user.click(screen.getByTestId("annotation-gutter-12"));
    expect(screen.getByTestId("annotation-submit")).toBeDisabled();

    await user.type(screen.getByTestId("annotation-body-input"), "   ");
    expect(screen.getByTestId("annotation-submit")).toBeDisabled();

    await user.click(screen.getByTestId("annotation-cancel"));
    expect(screen.queryByTestId("annotation-composer")).not.toBeInTheDocument();
  });

  it("feed_to_agent: 空列表时禁用, 有标注后产出 prompt fragment", async () => {
    const user = userEvent.setup();
    renderAnnotator();
    await waitFor(() => expect(screen.getByTestId("annotation-empty")).toBeInTheDocument());
    expect(screen.getByTestId("feed-to-agent-button")).toBeDisabled();

    await dropComment(user, 15, "这里要处理 None");
    await user.click(await screen.findByTestId("annotation-item")); // 不影响状态, 仅确保已落地
    await user.click(screen.getByTestId("feed-to-agent-button"));

    const fragment = await screen.findByTestId("prompt-fragment");
    expect(fragment).toHaveTextContent("[Diff Annotation]");
    expect(fragment).toHaveTextContent(`File: ${DEMO_FILE_PATH}`);
    expect(fragment).toHaveTextContent("Lines: 15-16");
    expect(fragment).toHaveTextContent("这里要处理 None");
  });

  it("删除标注后列表回到空态", async () => {
    const user = userEvent.setup();
    renderAnnotator();
    await waitFor(() => expect(screen.getByTestId("annotation-empty")).toBeInTheDocument());

    await dropComment(user, 15, "先记一笔");
    await screen.findByTestId("annotation-item");

    await user.click(screen.getByTestId("annotation-delete"));
    await waitFor(() => expect(screen.getByTestId("annotation-empty")).toBeInTheDocument());
    expect(screen.queryByTestId("annotation-marker-15")).not.toBeInTheDocument();
  });

  it("后端拒绝时把 6-field error 的 message 显示出来", async () => {
    const user = userEvent.setup();
    // 未注册的 agent_run → agent_run_not_found
    render(
      <DiffAnnotator
        filePath={DEMO_FILE_PATH}
        agentRunId="22222222-2222-2222-2222-222222222222"
        lines={DEMO_DIFF_LINES}
      />,
    );
    await dropComment(user, 15, "会被拒");

    const err = await screen.findByTestId("annotation-error");
    expect(err).toHaveTextContent(/agent run not found/);
  });
});
