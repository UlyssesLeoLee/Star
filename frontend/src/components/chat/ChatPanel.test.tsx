// =====================================================================
// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/chat/ChatPanel.test.tsx — ULYS-98-W1.2
// =====================================================================
// W1 验收 (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 1.2"):
// "1 unit test (ChatPanel.test.tsx)"
//
// 在 jsdom 下:
// 1. fetch 是 mock 的 — 测试 BFF 不可达时 local stub 回退 ("AI 暂未接入")
// 2. fetch 模拟 200 — 测试 happy path 写入 assistant 消息
// 3. Ctrl+L 切换 isOpen (模拟 keydown)
// 4. Enter 触发 send (InputBox 通过 onSend)
// =====================================================================
import {
  describe,
  it,
  expect,
  vi,
  beforeEach,
  afterEach,
} from "vitest";
import {
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "@testing-library/react";
import { useState } from "react";

// next/dynamic 在 vitest/jsdom 下不会真的 chunk-split, 但 ssr:false 仍会渲染 placeholder
vi.mock("next/dynamic", () => ({
  default: (
    loader: () => Promise<{ default: React.ComponentType<unknown> }>,
  ) => {
    let CachedComp: React.ComponentType<unknown> | null = null;
    const Wrapper = (props: unknown) => {
      if (!CachedComp) {
        // 同步解析 loader 抛出的 Promise (jsdom 中同步行为)
        void loader().then((m) => {
          CachedComp = m.default;
        });
        return null;
      }
      const Comp = CachedComp;
      return <Comp {...(props as object)} />;
    };
    Wrapper.displayName = "DynamicLoaderStub";
    return Wrapper;
  },
}));

import ChatPanel from "./ChatPanel";

const mockFetch = vi.fn();
beforeEach(() => {
  mockFetch.mockReset();
  // @ts-expect-error - global fetch assignment for jsdom
  global.fetch = mockFetch;
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("ChatPanel (ULYS-98-W1.2)", () => {
  it("renders the toggle button + drawer in closed-by-default state", () => {
    render(<ChatPanel defaultOpen={false} />);
    const toggle = screen.getByTestId("chat-toggle");
    expect(toggle).toBeTruthy();
    const panel = screen.getByTestId("chat-panel");
    expect(panel.getAttribute("data-open")).toBe("false");
  });

  it("toggles open state on Ctrl+L keydown", () => {
    render(<ChatPanel defaultOpen={false} />);
    const panel = screen.getByTestId("chat-panel");
    expect(panel.getAttribute("data-open")).toBe("false");

    fireEvent.keyDown(window, { key: "l", ctrlKey: true });

    waitFor(() => {
      expect(panel.getAttribute("data-open")).toBe("true");
    });
  });

  it("falls back to local stub ('AI 暂未接入') when BFF fetch fails", async () => {
    mockFetch.mockRejectedValueOnce(new Error("network down"));

    render(<ChatPanel defaultOpen={true} />);

    // Open via the toggle so panel mounts.
    const textarea = screen.getByTestId("chat-input-textarea");
    fireEvent.change(textarea, { target: { value: "hello world" } });
    fireEvent.click(screen.getByTestId("chat-input-send"));

    await waitFor(() => {
      expect(screen.getAllByTestId("message-bubble-user").length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByTestId("message-bubble-assistant").length).toBeGreaterThanOrEqual(1);
    });

    const assistantBubbles = screen.getAllByTestId("message-bubble-assistant");
    expect(
      assistantBubbles.some((b) => b.textContent?.includes("AI 暂未接入")),
    ).toBe(true);

    // BFF was attempted once.
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  it("renders the happy-path assistant reply when BFF returns 200", async () => {
    const now = new Date().toISOString();
    mockFetch.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => ({
        session_id: "00000000-0000-0000-0000-000000000001",
        message_id: "00000000-0000-0000-0000-000000000abc",
        assistant_content: "AI 暂未接入",
        created_at: now,
      }),
    });

    render(<ChatPanel defaultOpen={true} />);
    fireEvent.change(screen.getByTestId("chat-input-textarea"), {
      target: { value: "hi" },
    });
    fireEvent.click(screen.getByTestId("chat-input-send"));

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledTimes(1);
    });

    const assistantBubbles = screen.getAllByTestId(
      "message-bubble-assistant",
    );
    expect(assistantBubbles.length).toBeGreaterThanOrEqual(1);
    expect(
      assistantBubbles.some((b) => b.textContent?.includes("AI 暂未接入")),
    ).toBe(true);
  });
});