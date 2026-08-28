// =====================================================================
// commandBarStore.test.ts — ⌘K 全局命令 store (per design §6 + 任务要求)
// =====================================================================
// 6 个测试 (per 任务要求 ≥2):
//   1. 初始 state: isOpen=false, query="", recent=[]
//   2. open() → isOpen=true + query 清空
//   3. close() → isOpen=false + query 清空
//   4. toggle() 翻转 + 重新打开清 query
//   5. setQuery 改 query
//   6. pushRecent 去重 + cap 5 + 最新在前
//   7. clearRecent 清空
// =====================================================================

import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { useCommandBarStore } from "../commandBarStore";

const resetStore = () => {
  useCommandBarStore.setState({
    isOpen: false,
    query: "",
    recent: [],
  });
};

describe("useCommandBarStore (CommandBar ⌘K, per §6)", () => {
  beforeEach(() => {
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
    resetStore();
  });

  afterEach(() => {
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }
    resetStore();
  });

  it("initial state: isOpen=false, query='', recent=[]", () => {
    const s = useCommandBarStore.getState();
    expect(s.isOpen).toBe(false);
    expect(s.query).toBe("");
    expect(s.recent).toEqual([]);
  });

  it("open(): isOpen=true 且 query 清空 (per §6 open 行为)", () => {
    useCommandBarStore.getState().setQuery("inbox");
    useCommandBarStore.getState().open();
    const s = useCommandBarStore.getState();
    expect(s.isOpen).toBe(true);
    expect(s.query).toBe("");
  });

  it("close(): isOpen=false 且 query 清空 (per §6 Esc 关闭)", () => {
    useCommandBarStore.getState().open();
    useCommandBarStore.getState().setQuery("issues");
    useCommandBarStore.getState().close();
    const s = useCommandBarStore.getState();
    expect(s.isOpen).toBe(false);
    expect(s.query).toBe("");
  });

  it("toggle(): 翻转 isOpen + 重新打开清 query (per §6 ⌘K toggle)", () => {
    // 关闭 → 打开
    useCommandBarStore.setState({ query: "leftover" });
    useCommandBarStore.getState().toggle();
    expect(useCommandBarStore.getState().isOpen).toBe(true);
    expect(useCommandBarStore.getState().query).toBe("");

    // 打开 → 关闭: 保留 query (避免误清输入)
    useCommandBarStore.getState().setQuery("abc");
    useCommandBarStore.getState().toggle();
    expect(useCommandBarStore.getState().isOpen).toBe(false);
    expect(useCommandBarStore.getState().query).toBe("abc");
  });

  it("setQuery(): 改 query 字段", () => {
    useCommandBarStore.getState().setQuery("kanban");
    expect(useCommandBarStore.getState().query).toBe("kanban");
    useCommandBarStore.getState().setQuery("gantt");
    expect(useCommandBarStore.getState().query).toBe("gantt");
  });

  it("pushRecent(): 去重 + 最新在前 + cap 5 (per spec 5 条限制)", () => {
    const { pushRecent } = useCommandBarStore.getState();
    pushRecent({ id: "p1", label: "Inbox",  href: "/inbox",    type: "page", at: 1 });
    pushRecent({ id: "p2", label: "Issues", href: "/issues",   type: "page", at: 2 });
    pushRecent({ id: "p3", label: "Projects", href: "/projects", type: "page", at: 3 });
    pushRecent({ id: "p4", label: "Agents", href: "/agents",   type: "page", at: 4 });
    pushRecent({ id: "p5", label: "Analytics", href: "/analytics", type: "page", at: 5 });
    pushRecent({ id: "p6", label: "Settings", href: "/settings", type: "page", at: 6 });
    // cap 5: p1 应被淘汰
    let recent = useCommandBarStore.getState().recent;
    expect(recent).toHaveLength(5);
    expect(recent[0].id).toBe("p6"); // 最新在前
    expect(recent.find((r) => r.id === "p1")).toBeUndefined();

    // 去重: 再 push p4 → p4 应移到最前
    pushRecent({ id: "p4", label: "Agents", href: "/agents", type: "page", at: 99 });
    recent = useCommandBarStore.getState().recent;
    expect(recent[0].id).toBe("p4");
    expect(recent.filter((r) => r.id === "p4")).toHaveLength(1);
    expect(recent).toHaveLength(5);
  });

  it("clearRecent(): 清空 recent 数组", () => {
    const { pushRecent } = useCommandBarStore.getState();
    pushRecent({ id: "a", label: "A", type: "page", at: 1 });
    pushRecent({ id: "b", label: "B", type: "page", at: 2 });
    expect(useCommandBarStore.getState().recent.length).toBe(2);
    useCommandBarStore.getState().clearRecent();
    expect(useCommandBarStore.getState().recent).toEqual([]);
  });
});
