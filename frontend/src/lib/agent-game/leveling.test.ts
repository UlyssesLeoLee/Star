// =====================================================================
// leveling.test.ts — 升级曲线 / 死亡 / 复活 / perk 加成
// =====================================================================
// 覆盖:
//   1. computeClaim — 基础 + perk 加成
//   2. applyClaim — xp 累加 / 升级 / 满级
//   3. applyCostSpend — HP 扣血 / 死亡触发
//   4. applyDeath — 保留 50% 金币 / level 清零
//   5. applyRevive — 扣 50 金币 / level 1 / 复活次数 +1
//   6. applyRestart — 不扣币 / 复活无 deaths+1
//   7. perk 加成 — xp_boost / coin_magnet / bounty_hunter / iron_will
//   8. lucky_star 触发 (random01 < 0.01)
// =====================================================================

import { describe, it, expect } from "vitest";
import {
  computeClaim,
  applyClaim,
  applyCostSpend,
  applyDeath,
  applyRevive,
  applyRestart,
  freshGameState,
  xpProgress,
  xpMultiplier,
  coinMagnetBonus,
  bountyMultiplier,
  ironWillMultiplier,
  luckyStarTriggered,
} from "./leveling";
import {
  createInitialGameState,
  MAX_HP, MAX_LEVEL, REVIVE_COST, DEATH_COIN_KEEP_RATIO,
  XP_TO_NEXT_LEVEL,
} from "./types";
import type { WorkItem, AgentSession } from "@/types/ids";

const baseAgent: AgentSession = {
  id: "ag-001",
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  worktree_id: "wt-001",
  agent_kind: "claude-sonnet",
  status: "executing",
  current_step: "test",
  token_usage: { input: 0, output: 0, total: 0 },
  cost_summary: { usd: 0, budget_usd: 5.0 },
  started_at: "2026-09-05T10:00:00Z",
};

const makeWi = (priority: WorkItem["priority"]): WorkItem => ({
  id: "wi-001",
  tenant_id: "ten-acme",
  project_id: "prj-physis",
  key: "PHYSIS-1",
  title: "test",
  description: "",
  kind: "task",
  status: "done",
  priority,
  reporter_id: "usr-001",
  labels: [],
  workflow_id: "wf-default",
  created_at: "2026-09-05T08:00:00Z",
  updated_at: "2026-09-05T08:00:00Z",
});

describe("computeClaim (基础 + perk 加成)", () => {
  it("p0 无 perk: 20 xp + 5 coin", () => {
    const r = computeClaim(makeWi("p0"), [], 0);
    expect(r).toEqual({ xp: 20, coins: 5 });
  });

  it("p1: 10 xp + 3 coin", () => {
    expect(computeClaim(makeWi("p1"), [], 0)).toEqual({ xp: 10, coins: 3 });
  });

  it("p2: 5 xp + 2 coin", () => {
    expect(computeClaim(makeWi("p2"), [], 0)).toEqual({ xp: 5, coins: 2 });
  });

  it("p3: 2 xp + 1 coin", () => {
    expect(computeClaim(makeWi("p3"), [], 0)).toEqual({ xp: 2, coins: 1 });
  });

  it("xp_boost ×1: +25% xp", () => {
    const r = computeClaim(makeWi("p0"), ["xp_boost"], 0);
    expect(r.xp).toBe(25); // 20 * 1.25
    expect(r.coins).toBe(5);
  });

  it("coin_magnet ×2: +2 coin", () => {
    const r = computeClaim(makeWi("p1"), ["coin_magnet", "coin_magnet"], 0);
    expect(r.xp).toBe(10);
    expect(r.coins).toBe(5); // 3 + 2
  });

  it("bounty_hunter ×2 p0: coin × 2.0", () => {
    const r = computeClaim(makeWi("p0"), ["bounty_hunter", "bounty_hunter"], 0);
    expect(r.coins).toBe(10); // 5 * 2.0
  });

  it("bounty_hunter p1: 无加成 (只对 p0)", () => {
    const r = computeClaim(makeWi("p1"), ["bounty_hunter"], 0);
    expect(r.coins).toBe(3);
  });
});

describe("applyClaim (升级)", () => {
  it("累积到 XP_TO_NEXT_LEVEL[0]=50 升 Lv 2", () => {
    let s = createInitialGameState("ag-001", 5);
    const r1 = applyClaim(s, 20, 5, 0);  // xp 0+20=20
    expect(r1.state.level).toBe(1);
    expect(r1.state.xp).toBe(20);
    const r2 = applyClaim(r1.state, 30, 5, 0);  // xp 20+30=50 = 50 升
    expect(r2.state.level).toBe(2);
    expect(r2.state.xp).toBe(0);
    expect(r2.leveledUp).toBe(true);
    expect(r2.levelsGained).toBe(1);
  });

  it("一次 claim 跨 2 级 (xp=150, Lv 1→3)", () => {
    let s = createInitialGameState("ag-001", 5);
    const r = applyClaim(s, 150, 5, 0);
    expect(r.state.level).toBe(3);
    expect(r.levelsGained).toBe(2);
  });

  it("满级 Lv 10 不再升级, 但 coin 仍累", () => {
    let s = { ...createInitialGameState("ag-001", 5), level: 10, xp: 0 };
    const r = applyClaim(s, 100, 5, 0);
    expect(r.state.level).toBe(10);
    expect(r.state.xp).toBe(0);
    expect(r.state.coins).toBe(5);
    expect(r.isMaxLevel).toBe(true);
  });

  it("死亡时 claim 无效 (gate)", () => {
    let s = { ...createInitialGameState("ag-001", 5), alive: false };
    const r = applyClaim(s, 100, 5, 0);
    expect(r.leveledUp).toBe(false);
    expect(r.state.xp).toBe(0);
  });
});

describe("applyCostSpend (HP 扣血)", () => {
  it("cost 0.5 / budget 5 = 10% = 10 hp", () => {
    const s = createInitialGameState("ag-001", 5);
    const r = applyCostSpend(s, 0.5, 5);
    expect(r.state.hp).toBe(90);
    expect(r.died).toBe(false);
  });

  it("cost 5 / budget 5 = 100% = 100 hp (dead)", () => {
    const s = createInitialGameState("ag-001", 5);
    const r = applyCostSpend(s, 5, 5);
    expect(r.state.hp).toBe(0);
    expect(r.died).toBe(true);
  });

  it("iron_will ×1: HP 损失 -25%", () => {
    const s = { ...createInitialGameState("ag-001", 5), perks: ["iron_will"] };
    const r = applyCostSpend(s, 0.5, 5);  // 0.5/5=0.1, hpLoss = round(0.1*100*0.75) = round(7.5) = 8
    // 100 - 8 = 92
    expect(r.state.hp).toBe(92);
  });

  it("iron_will ×3: HP 损失 -75% (下界 0.25)", () => {
    const s = { ...createInitialGameState("ag-001", 5), perks: ["iron_will", "iron_will", "iron_will"] };
    const r = applyCostSpend(s, 5, 5);  // 100% ratio, 但被压到 25%
    expect(r.died).toBe(false);
    expect(r.state.hp).toBe(75); // 100 - round(100 * 0.25) = 75
  });

  it("死亡时不应用", () => {
    const s = { ...createInitialGameState("ag-001", 5), alive: false };
    const r = applyCostSpend(s, 5, 5);
    expect(r.died).toBe(false);
    expect(r.state.hp).toBe(MAX_HP);
  });
});

describe("applyDeath (死亡)", () => {
  it("保留 50% 金币 (向下取整)", () => {
    const s = { ...createInitialGameState("ag-001", 5), coins: 13, deaths: 0 };
    const r = applyDeath(s);
    expect(r.coins).toBe(6); // floor(13 * 0.5) = 6
    expect(r.alive).toBe(false);
  });

  it("level/xp/perks 清零, HP 满", () => {
    const s = { ...createInitialGameState("ag-001", 5), level: 5, xp: 30, perks: ["xp_boost"], hp: 20 };
    const r = applyDeath(s);
    expect(r.level).toBe(1);
    expect(r.xp).toBe(0);
    expect(r.hp).toBe(MAX_HP);
    expect(r.perks).toEqual([]);
  });

  it("deaths +1", () => {
    const s = { ...createInitialGameState("ag-001", 5), deaths: 3 };
    const r = applyDeath(s);
    expect(r.deaths).toBe(4);
  });

  it("highestLevel 保留 (跨 life 累计)", () => {
    const s = { ...createInitialGameState("ag-001", 5), highestLevel: 7 };
    const r = applyDeath(s);
    expect(r.highestLevel).toBe(7);
  });
});

describe("applyRevive (复活)", () => {
  it("扣 50 金币 + 重置", () => {
    const s = { ...createInitialGameState("ag-001", 5), alive: false, coins: 100, revives: 0 };
    const r = applyRevive(s);
    expect(r.ok).toBe(true);
    expect(r.state.coins).toBe(50);
    expect(r.state.alive).toBe(true);
    expect(r.state.level).toBe(1);
    expect(r.state.xp).toBe(0);
    expect(r.state.hp).toBe(MAX_HP);
    expect(r.state.perks).toEqual([]);
    expect(r.state.revives).toBe(1);
  });

  it("coins < 50 → 拒绝 (insufficient_coins)", () => {
    const s = { ...createInitialGameState("ag-001", 5), alive: false, coins: 30 };
    const r = applyRevive(s);
    expect(r.ok).toBe(false);
    expect(r.reason).toBe("insufficient_coins");
  });

  it("已 alive → 拒绝 (already_alive)", () => {
    const s = createInitialGameState("ag-001", 5);
    const r = applyRevive(s);
    expect(r.ok).toBe(false);
    expect(r.reason).toBe("already_alive");
  });
});

describe("applyRestart (重开, 不扣币)", () => {
  it("重置 level/xp/perks, HP 满, 不扣币, 不增 deaths", () => {
    const s = { ...createInitialGameState("ag-001", 5), level: 5, xp: 30, perks: ["xp_boost"], hp: 0, alive: false, coins: 100, deaths: 2 };
    const r = applyRestart(s);
    expect(r.level).toBe(1);
    expect(r.xp).toBe(0);
    expect(r.hp).toBe(MAX_HP);
    expect(r.perks).toEqual([]);
    expect(r.alive).toBe(true);
    expect(r.coins).toBe(100);  // 不扣
    expect(r.deaths).toBe(2);    // 不增
  });
});

describe("freshGameState (从 agent 初始化)", () => {
  it("agent_id 透传, 默认 Lv 1 / HP 满", () => {
    const s = freshGameState(baseAgent);
    expect(s.agentId).toBe("ag-001");
    expect(s.level).toBe(1);
    expect(s.xp).toBe(0);
    expect(s.coins).toBe(0);
    expect(s.hp).toBe(MAX_HP);
    expect(s.alive).toBe(true);
    expect(s.deaths).toBe(0);
  });
});

describe("xpProgress (UI 显示)", () => {
  it("Lv 1 xp 25/50 → ratio 0.5", () => {
    const s = { ...createInitialGameState("ag-001", 5), level: 1, xp: 25 };
    const p = xpProgress(s);
    expect(p.current).toBe(25);
    expect(p.need).toBe(50);
    expect(p.ratio).toBe(0.5);
  });

  it("满级 ratio = 1", () => {
    const s = { ...createInitialGameState("ag-001", 5), level: 10 };
    const p = xpProgress(s);
    expect(p.ratio).toBe(1);
  });
});

describe("perk 加成 helper", () => {
  it("xpMultiplier 0 stack = 1.0", () => {
    expect(xpMultiplier([])).toBe(1);
  });

  it("xpMultiplier 1 stack = 1.25", () => {
    expect(xpMultiplier(["xp_boost"])).toBe(1.25);
  });

  it("xpMultiplier 4 stack (cap) = 2.0", () => {
    expect(xpMultiplier(["xp_boost", "xp_boost", "xp_boost", "xp_boost"])).toBe(2);
  });

  it("coinMagnetBonus 跟 stack 数相等", () => {
    expect(coinMagnetBonus([])).toBe(0);
    expect(coinMagnetBonus(["coin_magnet"])).toBe(1);
    expect(coinMagnetBonus(["coin_magnet", "coin_magnet"])).toBe(2);
  });

  it("bountyMultiplier p0 + bounty_hunter ×1 = 1.5", () => {
    expect(bountyMultiplier(["bounty_hunter"], "p0")).toBe(1.5);
  });

  it("bountyMultiplier p1 = 1 (无加成)", () => {
    expect(bountyMultiplier(["bounty_hunter"], "p1")).toBe(1);
  });

  it("ironWillMultiplier 0 stack = 1.0, 3 stack = 0.25", () => {
    expect(ironWillMultiplier([])).toBe(1);
    expect(ironWillMultiplier(["iron_will", "iron_will", "iron_will"])).toBe(0.25);
  });

  it("luckyStarTriggered 0.005 触发 (< 0.01)", () => {
    expect(luckyStarTriggered(["lucky_star"], 0.005)).toBe(true);
  });

  it("luckyStarTriggered 0.5 不触发", () => {
    expect(luckyStarTriggered(["lucky_star"], 0.5)).toBe(false);
  });

  it("luckyStarTriggered 没 lucky_star perk 不触发", () => {
    expect(luckyStarTriggered([], 0.001)).toBe(false);
  });
});

describe("常量 sanity", () => {
  it("MAX_HP = 100", () => expect(MAX_HP).toBe(100));
  it("MAX_LEVEL = 10", () => expect(MAX_LEVEL).toBe(10));
  it("REVIVE_COST = 50", () => expect(REVIVE_COST).toBe(50));
  it("DEATH_COIN_KEEP_RATIO = 0.5", () => expect(DEATH_COIN_KEEP_RATIO).toBe(0.5));
  it("XP_TO_NEXT_LEVEL 9 个值 + Infinity (Lv 9→10 + Lv 10 满级)", () => {
    expect(XP_TO_NEXT_LEVEL).toHaveLength(10);
    expect(XP_TO_NEXT_LEVEL[8]).toBe(2500);
    expect(XP_TO_NEXT_LEVEL[9]).toBe(Infinity);
  });
});
