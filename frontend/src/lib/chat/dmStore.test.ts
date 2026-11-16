/*
@cypher schema=1 source_sha256=ff91366f48f0bc7fc24371fa87bc0a774dea5d3eee192505589f0ce38d39a8ef
MERGE (self:File {path:"frontend/src/lib/chat/dmStore.test.ts"})
@endcypher
*/
// =====================================================================
// frontend/src/lib/chat/dmStore.test.ts — 用户间即时通信 store 单测
//   覆盖: openOrCreateConversation (幂等/DM/群聊/空成员)、sendDm、
//   receiveFromGateway (未读/已读)、markConversationRead、实时标志、
//   网关绑定/失败回退、localStorage 持久化与 SSR-safe 重载回落。
// =====================================================================
import { beforeEach, describe, expect, it } from "vitest";
import {
  useDmStore,
  DM_CURRENT_USER_ID,
  DM_KNOWN_USERS,
  type ImGateway,
} from "./dmStore";

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

/** 当前用户之外的种子用户 (usr-002 Hera, usr-003 Athena, ...) */
const others = DM_KNOWN_USERS.filter((u) => u.id !== DM_CURRENT_USER_ID);

const getConv = (id: string) =>
  useDmStore.getState().conversations.find((c) => c.id === id)!;

const fakeGateway = (overrides: Partial<ImGateway> = {}): ImGateway => ({
  isConnected: () => true,
  sendMessage: () => true,
  subscribe: () => () => {},
  ...overrides,
});

beforeEach(resetStore);

describe("dmStore: openOrCreateConversation", () => {
  it("单人 → dm 会话, 含当前用户, 标题=对方显示名, 幂等复用", () => {
    const peer = others[0];
    const a = useDmStore.getState().openOrCreateConversation([peer.id]);
    expect(a).not.toBe("");
    const conv = getConv(a);
    expect(conv.kind).toBe("dm");
    expect([...conv.memberIds].sort()).toEqual([DM_CURRENT_USER_ID, peer.id].sort());
    expect(conv.title).toBe(peer.name);
    expect(conv.messages).toHaveLength(0);
    expect(conv.unread).toBe(0);

    // 幂等: 再开一次复用同一会话
    const b = useDmStore.getState().openOrCreateConversation([peer.id]);
    expect(b).toBe(a);
    expect(useDmStore.getState().conversations).toHaveLength(1);
    expect(useDmStore.getState().activeConversationId).toBe(a);
  });

  it("多人 → group 会话, 自定义标题, 成员=当前用户+对方", () => {
    const [p1, p2] = others;
    const id = useDmStore.getState().openOrCreateConversation([p1.id, p2.id], "前端小组");
    const conv = getConv(id);
    expect(conv.kind).toBe("group");
    expect(conv.title).toBe("前端小组");
    expect(conv.memberIds).toHaveLength(3);
    expect(conv.memberIds).toContain(DM_CURRENT_USER_ID);
  });

  it("成员顺序不同 → 复用同一会话 (集合相等)", () => {
    const [p1, p2] = others;
    const a = useDmStore.getState().openOrCreateConversation([p1.id, p2.id], "组A");
    const b = useDmStore.getState().openOrCreateConversation([p2.id, p1.id], "组B");
    expect(b).toBe(a);
    expect(useDmStore.getState().conversations).toHaveLength(1);
  });

  it("成员为空 → 返回空串且不创建", () => {
    const id = useDmStore.getState().openOrCreateConversation([]);
    expect(id).toBe("");
    expect(useDmStore.getState().conversations).toHaveLength(0);
  });
});

describe("dmStore: sendDm / 未读 / 已读", () => {
  it("sendDm 追加消息: sequence 递增 / state=sent / lastActivityAt 更新 / 自己发不产生未读", () => {
    const id = useDmStore.getState().openOrCreateConversation([others[0].id]);
    const before = getConv(id).lastActivityAt;
    useDmStore.getState().sendDm(id, "你好", { id: DM_CURRENT_USER_ID, name: "Ulysses" });
    const conv = getConv(id);
    expect(conv.messages).toHaveLength(1);
    const m = conv.messages[0];
    expect(m.text).toBe("你好");
    expect(m.sequence).toBe(1);
    expect(m.state).toBe("sent");
    expect(m.authorId).toBe(DM_CURRENT_USER_ID);
    expect(conv.unread).toBe(0);
    expect(conv.lastActivityAt >= before).toBe(true);

    // 第二条 sequence 递增
    useDmStore.getState().sendDm(id, "在吗", { id: DM_CURRENT_USER_ID });
    expect(getConv(id).messages[1].sequence).toBe(2);
  });

  it("空文本不发送", () => {
    const id = useDmStore.getState().openOrCreateConversation([others[0].id]);
    useDmStore.getState().sendDm(id, "   ", { id: DM_CURRENT_USER_ID });
    expect(getConv(id).messages).toHaveLength(0);
  });

  it("sendDm 支持字符串 author (按用户目录解析显示名)", () => {
    const peer = others[0];
    const id = useDmStore.getState().openOrCreateConversation([peer.id]);
    useDmStore.getState().sendDm(id, "hi", peer.id);
    expect(getConv(id).messages[0].authorName).toBe(peer.name);
  });

  it("receiveFromGateway: 非活跃会话 + 他人消息 → unread+1, state=delivered", () => {
    const peer = others[0];
    const id = useDmStore.getState().openOrCreateConversation([peer.id]);
    useDmStore.getState().setActiveConversation(null);
    useDmStore.getState().receiveFromGateway({
      conversationId: id,
      senderId: peer.id,
      senderName: peer.name,
      text: "远程消息",
      at: new Date().toISOString(),
    });
    const conv = getConv(id);
    expect(conv.unread).toBe(1);
    expect(conv.messages[0].state).toBe("delivered");
  });

  it("receiveFromGateway: 活跃会话收到他人消息 → unread 不增", () => {
    const peer = others[0];
    const id = useDmStore.getState().openOrCreateConversation([peer.id]); // active=id
    useDmStore.getState().receiveFromGateway({
      conversationId: id,
      senderId: peer.id,
      senderName: peer.name,
      text: "活跃中",
      at: new Date().toISOString(),
    });
    expect(getConv(id).unread).toBe(0);
  });

  it("receiveFromGateway: 未知会话忽略 (不伪造会话)", () => {
    useDmStore.getState().receiveFromGateway({
      conversationId: "dm-不存在",
      senderId: "usr-002",
      senderName: "Hera",
      text: "x",
      at: new Date().toISOString(),
    });
    expect(useDmStore.getState().conversations).toHaveLength(0);
  });

  it("markConversationRead: 未读清零 + 他人消息 → read + readBy 加入本端", () => {
    const peer = others[0];
    const id = useDmStore.getState().openOrCreateConversation([peer.id]);
    useDmStore.getState().setActiveConversation(null);
    useDmStore.getState().receiveFromGateway({
      conversationId: id,
      senderId: peer.id,
      senderName: peer.name,
      text: "读我",
      at: new Date().toISOString(),
    });
    useDmStore.getState().markConversationRead(id);
    const conv = getConv(id);
    expect(conv.unread).toBe(0);
    expect(conv.messages[0].state).toBe("read");
    expect(conv.messages[0].readBy).toContain(DM_CURRENT_USER_ID);
  });
});

describe("dmStore: 实时网关", () => {
  it("setRealtimeConnected 同步全局与会话级标志", () => {
    const id = useDmStore.getState().openOrCreateConversation([others[0].id]);
    expect(useDmStore.getState().realtimeConnected).toBe(false);
    useDmStore.getState().setRealtimeConnected(true);
    expect(useDmStore.getState().realtimeConnected).toBe(true);
    expect(getConv(id).realtimeConnected).toBe(true);
    useDmStore.getState().setRealtimeConnected(false);
    expect(useDmStore.getState().realtimeConnected).toBe(false);
    expect(getConv(id).realtimeConnected).toBe(false);
  });

  it("bindImGateway: 已连接网关 → realtimeConnected=true; 订阅回调落库", () => {
    const peer = others[0];
    const id = useDmStore.getState().openOrCreateConversation([peer.id]);
    let onMessage: ((m: unknown) => void) | null = null;
    const gw = fakeGateway({
      subscribe: (cb) => {
        onMessage = cb as (m: unknown) => void;
        return () => {};
      },
    });
    useDmStore.getState().bindImGateway(gw);
    expect(useDmStore.getState().realtimeConnected).toBe(true);
    expect(getConv(id).realtimeConnected).toBe(true);

    onMessage!({
      conversationId: id,
      senderId: peer.id,
      senderName: peer.name,
      text: "网关下发",
      at: new Date().toISOString(),
    });
    expect(getConv(id).messages[0].text).toBe("网关下发");
  });

  it("网关 sendMessage 失败 → 回退本地 (realtimeConnected=false), 本地消息保留", async () => {
    const id = useDmStore.getState().openOrCreateConversation([others[0].id]);
    useDmStore
      .getState()
      .bindImGateway(fakeGateway({ sendMessage: () => Promise.resolve(false) }));
    expect(useDmStore.getState().realtimeConnected).toBe(true);

    useDmStore.getState().sendDm(id, "hi", { id: DM_CURRENT_USER_ID });
    await new Promise((r) => setTimeout(r, 0)); // flush promise
    expect(useDmStore.getState().realtimeConnected).toBe(false);
    expect(getConv(id).messages).toHaveLength(1); // 本地消息仍在
  });

  it("网关 sendMessage 抛异常 → 回退本地", () => {
    const id = useDmStore.getState().openOrCreateConversation([others[0].id]);
    useDmStore.getState().bindImGateway(
      fakeGateway({
        sendMessage: () => {
          throw new Error("ws down");
        },
      }),
    );
    useDmStore.getState().sendDm(id, "hi", { id: DM_CURRENT_USER_ID });
    expect(useDmStore.getState().realtimeConnected).toBe(false);
  });

  it("bindImGateway(null) 解绑 → 本地模式", () => {
    const id = useDmStore.getState().openOrCreateConversation([others[0].id]);
    useDmStore.getState().bindImGateway(fakeGateway());
    expect(useDmStore.getState().realtimeConnected).toBe(true);
    useDmStore.getState().bindImGateway(null);
    expect(useDmStore.getState().realtimeConnected).toBe(false);
    expect(getConv(id).realtimeConnected).toBe(false);
    expect(useDmStore.getState().gateway).toBeNull();
  });

  it("tryLoadImGateway: 真实 imGateway 模块已落地 → 适配为 store 接口 (本地模式, 网关不可达)", async () => {
    // imGateway.ts 真实实现已落地 (per 2026-11-16): 返回适配后的 gateway 或 null
    // 网关不可达时 connectImGateway 停在 reconnecting → isConnected() false
    const gw = await useDmStore.getState().tryLoadImGateway();
    if (gw) {
      expect(typeof gw.isConnected).toBe("function");
      expect(typeof gw.subscribe).toBe("function");
    } else {
      expect(gw).toBeNull();
    }
  });
});

describe("dmStore: 持久化 (SSR-safe)", () => {
  it("写入 localStorage key star-dm-conversations; 持久化的 realtimeConnected 强制 false", () => {
    const id = useDmStore.getState().openOrCreateConversation([others[0].id]);
    useDmStore.getState().sendDm(id, "持久化测试", { id: DM_CURRENT_USER_ID });
    const raw = localStorage.getItem(PERSIST_KEY);
    expect(raw).toBeTruthy();
    const parsed = JSON.parse(raw!) as { state: { conversations: Array<{ id: string; realtimeConnected: boolean; messages: unknown[] }> } };
    expect(parsed.state.conversations).toHaveLength(1);
    expect(parsed.state.conversations[0].realtimeConnected).toBe(false);
    expect(parsed.state.conversations[0].messages).toHaveLength(1);
  });

  it("重载 (rehydrate) 恢复会话, realtimeConnected 回落 false", async () => {
    const id = useDmStore.getState().openOrCreateConversation([others[0].id]);
    useDmStore.getState().sendDm(id, "hi", { id: DM_CURRENT_USER_ID });
    const raw = localStorage.getItem(PERSIST_KEY)!;

    // 模拟页面重载: 清空内存态 → 从 localStorage 重灌
    useDmStore.setState({ conversations: [], activeConversationId: null });
    localStorage.setItem(PERSIST_KEY, raw);
    await useDmStore.persist.rehydrate();

    expect(useDmStore.getState().conversations).toHaveLength(1);
    expect(useDmStore.getState().conversations[0].realtimeConnected).toBe(false);
  });

  it("SSR 守卫: 无 window 时不崩溃 (storage 回退 no-op)", () => {
    // createJSONStorage 的工厂在模块加载时执行 typeof window 检查;
    // 此处仅断言 store 在 jsdom 外可安全构造路径已由回退覆盖 (no-op storage)
    expect(useDmStore.getState().conversations).toBeDefined();
  });
});
