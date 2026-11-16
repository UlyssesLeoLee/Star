// =====================================================================
// frontend/src/components/chat/DmCenterPanel.test.tsx — DM 面板 UI 单测
//   覆盖: 状态条 (未接入/已连接)、新建会话弹窗 (1人→DM, 多人→群聊)、
//   发送消息 (点击/Enter)、未读徽标与点击清零、已读状态标。
// =====================================================================
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { DmCenterPanel } from "./DmCenterPanel";
import { useDmStore, DM_CURRENT_USER_ID, DM_KNOWN_USERS } from "@/lib/chat/dmStore";

const PERSIST_KEY = "star-dm-conversations";

const resetStore = () => {
  localStorage.removeItem(PERSIST_KEY);
  useDmStore.setState({
    conversations: [],
    activeConversationId: null,
    currentUserId: DM_CURRENT_USER_ID,
    realtimeConnected: false,
    gateway: null,
    unsubscribeGateway: null,
  });
};

const others = DM_KNOWN_USERS.filter((u) => u.id !== DM_CURRENT_USER_ID);

beforeEach(resetStore);
afterEach(cleanup);

describe("DmCenterPanel", () => {
  it("默认显示「实时未接入（本地模式）」状态条 + 会话列表容器", () => {
    render(<DmCenterPanel />);
    expect(screen.getByTestId("dm-realtime-status").textContent).toContain("实时未接入");
    expect(screen.getByTestId("dm-conversations")).toBeTruthy();
    expect(screen.getByTestId("dm-new-btn")).toBeTruthy();
    expect(screen.getByTestId("dm-send-input")).toBeDisabled();
  });

  it("setRealtimeConnected(true) → 状态条显示「实时已连接」", () => {
    useDmStore.getState().setRealtimeConnected(true);
    render(<DmCenterPanel />);
    expect(screen.getByTestId("dm-realtime-status").textContent).toContain("实时已连接");
  });

  it("新建会话: 选 1 人 → 私聊 (dm), 列表出现会话项并激活", () => {
    render(<DmCenterPanel />);
    fireEvent.click(screen.getByTestId("dm-new-btn"));
    expect(screen.getByTestId("dm-new-modal")).toBeTruthy();
    expect(screen.getByTestId("dm-new-confirm")).toBeDisabled();

    fireEvent.click(screen.getByTestId(`dm-new-user-${others[0].id}`));
    fireEvent.click(screen.getByTestId("dm-new-confirm"));

    const conv = useDmStore.getState().conversations[0];
    expect(conv.kind).toBe("dm");
    expect(screen.getByTestId(`dm-conversation-${conv.id}`)).toBeTruthy();
    expect(screen.getByTestId("dm-active-title").textContent).toContain(others[0].name);
  });

  it("新建会话: 选 2 人 + 群名 → 群聊", () => {
    render(<DmCenterPanel />);
    fireEvent.click(screen.getByTestId("dm-new-btn"));
    fireEvent.click(screen.getByTestId(`dm-new-user-${others[0].id}`));
    fireEvent.click(screen.getByTestId(`dm-new-user-${others[1].id}`));

    const titleInput = screen.getByTestId("dm-new-group-title");
    fireEvent.change(titleInput, { target: { value: "前端小组" } });
    fireEvent.click(screen.getByTestId("dm-new-confirm"));

    const conv = useDmStore.getState().conversations[0];
    expect(conv.kind).toBe("group");
    expect(conv.title).toBe("前端小组");
    expect(screen.getByText("群聊")).toBeTruthy();
    expect(screen.getByTestId(`dm-conversation-${conv.id}`)).toBeTruthy();
  });

  it("取消按钮关闭弹窗, 不创建会话", () => {
    render(<DmCenterPanel />);
    fireEvent.click(screen.getByTestId("dm-new-btn"));
    fireEvent.click(screen.getByTestId("dm-new-cancel"));
    expect(screen.queryByTestId("dm-new-modal")).toBeNull();
    expect(useDmStore.getState().conversations).toHaveLength(0);
  });

  it("发送消息: 输入 + 点击发送 → 消息流出现 + 输入清空 + 状态标「发送」", () => {
    const id = useDmStore.getState().openOrCreateConversation([others[0].id]);
    render(<DmCenterPanel />);
    fireEvent.click(screen.getByTestId(`dm-conversation-${id}`));

    const input = screen.getByTestId("dm-send-input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "你好 Hera" } });
    expect(screen.getByTestId("dm-send-btn")).toBeEnabled();
    fireEvent.click(screen.getByTestId("dm-send-btn"));

    const conv = useDmStore.getState().conversations.find((c) => c.id === id)!;
    expect(conv.messages).toHaveLength(1);
    expect(screen.getByTestId(`dm-message-${conv.messages[0].id}`).textContent).toContain(
      "你好 Hera",
    );
    expect(screen.getByTestId(`dm-msg-state-${conv.messages[0].id}`).textContent).toBe("发送");
    expect(input.value).toBe("");
  });

  it("Enter 键发送消息", () => {
    const id = useDmStore.getState().openOrCreateConversation([others[0].id]);
    render(<DmCenterPanel />);
    fireEvent.click(screen.getByTestId(`dm-conversation-${id}`));
    const input = screen.getByTestId("dm-send-input");
    fireEvent.change(input, { target: { value: "回车发送" } });
    fireEvent.keyDown(input, { key: "Enter" });
    const conv = useDmStore.getState().conversations.find((c) => c.id === id)!;
    expect(conv.messages).toHaveLength(1);
    expect(conv.messages[0].text).toBe("回车发送");
  });

  it("未选会话时发送按钮禁用; 空输入禁用", () => {
    render(<DmCenterPanel />);
    expect(screen.getByTestId("dm-send-btn")).toBeDisabled();
  });

  it("他人消息到达非活跃会话 → 未读徽标; 点击会话 → 清零并标记已读", () => {
    const peer = others[0];
    const id = useDmStore.getState().openOrCreateConversation([peer.id]);
    useDmStore.getState().setActiveConversation(null);
    useDmStore.getState().receiveFromGateway({
      conversationId: id,
      senderId: peer.id,
      senderName: peer.name,
      text: "在吗?",
      at: new Date().toISOString(),
    });

    render(<DmCenterPanel />);
    expect(screen.getByTestId(`dm-unread-${id}`).textContent).toBe("1");

    fireEvent.click(screen.getByTestId(`dm-conversation-${id}`));
    const conv = useDmStore.getState().conversations.find((c) => c.id === id)!;
    expect(conv.unread).toBe(0);
    expect(conv.messages[0].state).toBe("read");
  });

  it("群聊消息已读回执: 多人已读时显示「N 人已读」", () => {
    const [p1, p2] = others;
    const id = useDmStore.getState().openOrCreateConversation([p1.id, p2.id], "组");
    useDmStore.getState().sendDm(id, "大家好", { id: DM_CURRENT_USER_ID });
    // p1/p2 已读 → readBy 含本端 + 2 人
    const conv = useDmStore.getState().conversations.find((c) => c.id === id)!;
    const msg = conv.messages[0];
    useDmStore.setState({
      conversations: [
        {
          ...conv,
          messages: [{ ...msg, readBy: [DM_CURRENT_USER_ID, p1.id, p2.id] }],
        },
      ],
    });

    render(<DmCenterPanel />);
    expect(screen.getByTestId(`dm-msg-readby-${msg.id}`).textContent).toContain("2 人已读");
  });

  it("会话列表含成员头像首字母与最后消息摘要", () => {
    const peer = others[0];
    const id = useDmStore.getState().openOrCreateConversation([peer.id]);
    useDmStore.getState().sendDm(id, "最后一条消息", { id: DM_CURRENT_USER_ID });

    render(<DmCenterPanel />);
    const item = screen.getByTestId(`dm-conversation-${id}`);
    expect(item.textContent).toContain(peer.name.slice(0, 1).toUpperCase());
    expect(item.textContent).toContain("最后一条消息");
  });

  it("会话列表按 lastActivityAt 降序排列", () => {
    const a = useDmStore.getState().openOrCreateConversation([others[0].id]);
    const b = useDmStore.getState().openOrCreateConversation([others[1].id]);
    // 让 a 更新活动时间 → a 排最前
    useDmStore.getState().sendDm(a, "新消息", { id: DM_CURRENT_USER_ID });

    render(<DmCenterPanel />);
    const items = screen.getAllByTestId(/^dm-conversation-/);
    expect(items[0].getAttribute("data-testid")).toBe(`dm-conversation-${a}`);
    expect(items[1].getAttribute("data-testid")).toBe(`dm-conversation-${b}`);
  });
});
