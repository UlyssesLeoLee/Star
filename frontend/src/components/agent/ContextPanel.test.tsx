/*
@cypher schema=1 source_sha256=5947df2dfc908ee6a4cc71f1e7e2f6956e5acbcdb5bfd11c99f804437fb573a2
MERGE (self:File {path:"frontend/src/components/agent/ContextPanel.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/agent/ContextPanel.test.tsx::ContextPanel",kind:"suite"})
MERGE (target:File {path:"frontend/src/components/agent/ContextPanel.tsx"})
MERGE (self)-[:IMPORTS]->(target)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(target)
@endcypher
*/
import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ContextPanel, estimateTokens } from "@/components/agent/ContextPanel";
import type { ContextPanelMessage } from "@/components/agent/ContextPanel";

const msg = (
  id: string,
  role: string,
  content: string,
  timestamp: string,
): ContextPanelMessage => ({ id, role, content, timestamp });

/** 用本地时间构造 ISO 串, 保证 formatTime 在任何测试机 TZ 下都渲染回预期 HH:MM */
const isoAt = (h: number, m: number) => new Date(2026, 10, 15, h, m).toISOString();

const twoMessages: ContextPanelMessage[] = [
  msg("m1", "human", "abcd", isoAt(10, 0)),
  msg("m2", "agent", "abcdefgh", isoAt(10, 5)),
];

/** 只匹配消息行 testid (排除 ctx-msg-count) */
const MSG_ROW_RE = /^ctx-msg-(?!count$)/;

describe("ContextPanel 上下文面板 (per 2026-11-15 需求)", () => {
  describe("estimateTokens helper (chars/4)", () => {
    it("空串 → 0", () => {
      expect(estimateTokens("")).toBe(0);
    });
    it("4 字符 → 1 token", () => {
      expect(estimateTokens("abcd")).toBe(1);
    });
    it("5 字符 → 2 tokens (向上取整)", () => {
      expect(estimateTokens("abcde")).toBe(2);
    });
    it("中文按字符数计 (chars, 非 bytes)", () => {
      expect(estimateTokens("你好世界")).toBe(1);
    });
  });

  describe("① 头部", () => {
    it("显示 token 估算 (各消息内容 chars/4 之和) 与消息条数", () => {
      render(<ContextPanel messages={twoMessages} />);
      // "abcd"=1 + "abcdefgh"=2 → 3
      expect(screen.getByTestId("ctx-token-count").textContent).toBe("3");
      expect(screen.getByTestId("ctx-msg-count").textContent).toBe("2");
    });
  });

  describe("② 消息列表", () => {
    it("按时间倒序渲染, 每条带 data-testid ctx-msg-<id>", () => {
      render(<ContextPanel messages={twoMessages} />);
      const items = screen.getAllByTestId(MSG_ROW_RE).map((el) => el.dataset.testid);
      expect(items).toEqual(["ctx-msg-m2", "ctx-msg-m1"]); // m2 更新, 在上
    });

    it("每条显示 role / 内容摘要 / 时间", () => {
      render(<ContextPanel messages={twoMessages} />);
      const row = screen.getByTestId("ctx-msg-m1");
      expect(row.textContent).toContain("human");
      expect(row.textContent).toContain("abcd");
      expect(row.textContent).toContain("10:00");
    });

    it("内容超过 60 字截断为前 60 字 + …", () => {
      const long = "a".repeat(61);
      render(<ContextPanel messages={[msg("m1", "agent", long, isoAt(10, 0))]} />);
      const row = screen.getByTestId("ctx-msg-m1");
      expect(row.textContent).toContain("a".repeat(60) + "…");
      expect(row.textContent).not.toContain(long);
    });

    it("无消息时显示占位文案", () => {
      render(<ContextPanel messages={[]} />);
      expect(screen.getByTestId("ctx-msg-count").textContent).toBe("0");
      expect(screen.getByTestId("ctx-token-count").textContent).toBe("0");
      expect(screen.queryAllByTestId(MSG_ROW_RE)).toHaveLength(0);
      expect(screen.getByText("暂无消息")).toBeTruthy();
    });
  });

  describe("③ 手动压缩按钮", () => {
    it("传入 onCompact 时点击触发回调", () => {
      const onCompact = vi.fn();
      render(<ContextPanel messages={twoMessages} onCompact={onCompact} />);
      fireEvent.click(screen.getByTestId("ctx-compact"));
      expect(onCompact).toHaveBeenCalledTimes(1);
    });

    it("未传 onCompact 时按钮禁用 + 诚实 title", () => {
      render(<ContextPanel messages={twoMessages} />);
      const btn = screen.getByTestId("ctx-compact") as HTMLButtonElement;
      expect(btn.disabled).toBe(true);
      expect(btn.title).toBe("压缩服务未接入 — 宿主 agent 网关接入后启用");
    });
  });

  describe("④ 压缩状态条 (内部 state)", () => {
    it("初始不显示状态条", () => {
      render(<ContextPanel messages={twoMessages} onCompact={vi.fn()} />);
      expect(screen.queryByTestId("ctx-compact-state")).toBeNull();
    });

    it("压缩中 → 完成 (async onCompact)", async () => {
      let resolve!: () => void;
      const onCompact = vi.fn(
        () =>
          new Promise<void>((r) => {
            resolve = r;
          }),
      );
      render(<ContextPanel messages={twoMessages} onCompact={onCompact} />);
      fireEvent.click(screen.getByTestId("ctx-compact"));
      expect(screen.getByTestId("ctx-compact-state").textContent).toBe("压缩中 — 请稍候");
      resolve();
      await waitFor(() =>
        expect(screen.getByTestId("ctx-compact-state").textContent).toBe("压缩完成"),
      );
    });

    it("同步 onCompact → 直接完成", async () => {
      render(<ContextPanel messages={twoMessages} onCompact={vi.fn()} />);
      fireEvent.click(screen.getByTestId("ctx-compact"));
      await waitFor(() =>
        expect(screen.getByTestId("ctx-compact-state").textContent).toBe("压缩完成"),
      );
    });

    it("onCompact 失败 → 失败态", async () => {
      const onCompact = vi.fn(() => Promise.reject(new Error("boom")));
      render(<ContextPanel messages={twoMessages} onCompact={onCompact} />);
      fireEvent.click(screen.getByTestId("ctx-compact"));
      await waitFor(() =>
        expect(screen.getByTestId("ctx-compact-state").textContent).toBe("压缩失败 — 请重试"),
      );
    });

    it("压缩中忽略重复点击", async () => {
      let resolve!: () => void;
      const onCompact = vi.fn(
        () =>
          new Promise<void>((r) => {
            resolve = r;
          }),
      );
      render(<ContextPanel messages={twoMessages} onCompact={onCompact} />);
      const btn = screen.getByTestId("ctx-compact");
      fireEvent.click(btn);
      fireEvent.click(btn);
      expect(onCompact).toHaveBeenCalledTimes(1);
      resolve();
      await waitFor(() =>
        expect(screen.getByTestId("ctx-compact-state").textContent).toBe("压缩完成"),
      );
    });
  });

  describe("⑤ 底部诚实徽标 + 关闭按钮", () => {
    it("显示本地估算诚实徽标", () => {
      render(<ContextPanel messages={twoMessages} />);
      expect(screen.getByTestId("ctx-honest-badge").textContent).toBe(
        "本地上下文估算 — 真实 token 由 agent 运行时提供",
      );
    });

    it("传入 onClose 时点击关闭按钮触发回调", () => {
      const onClose = vi.fn();
      render(<ContextPanel messages={twoMessages} onClose={onClose} />);
      fireEvent.click(screen.getByTestId("ctx-close"));
      expect(onClose).toHaveBeenCalledTimes(1);
    });

    it("未传 onClose 时关闭按钮禁用", () => {
      render(<ContextPanel messages={twoMessages} />);
      expect((screen.getByTestId("ctx-close") as HTMLButtonElement).disabled).toBe(true);
    });
  });
});
