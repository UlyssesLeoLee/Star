/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/nema/nemaCollaboration.test.ts"})
MERGE (suite_editors:Symbol {id:"frontend/src/lib/nema/nemaCollaboration.test.ts::editor_mutations",kind:"suite"})
MERGE (suite_presence:Symbol {id:"frontend/src/lib/nema/nemaCollaboration.test.ts::presence_honesty",kind:"suite"})
MERGE (store:File {path:"frontend/src/lib/nema/nemaStore.ts"})
MERGE (nema:File {path:"frontend/src/lib/nema/nema.ts"})
MERGE (collab:File {path:"frontend/src/lib/nema/nemaCollaboration.ts"})
MERGE (update_editors:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::updateNemaEditors",kind:"function"})
MERGE (register:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::registerSharedFile",kind:"function"})
MERGE (open:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::openNemaAsStarTask",kind:"function"})
MERGE (presence:Symbol {id:"frontend/src/lib/nema/nemaCollaboration.ts::nemaEditorPresence",kind:"function"})
MERGE (self)-[:DEFINES]->(suite_editors)
MERGE (self)-[:DEFINES]->(suite_presence)
MERGE (self)-[:IMPORTS]->(store)
MERGE (self)-[:IMPORTS]->(nema)
MERGE (self)-[:IMPORTS]->(collab)
MERGE (suite_editors)-[:TESTS]->(update_editors)
MERGE (suite_editors)-[:TESTS]->(register)
MERGE (suite_editors)-[:TESTS]->(open)
MERGE (suite_presence)-[:TESTS]->(presence)
@endcypher
*/
// =====================================================================
// 文件即任务式多 agent 共同编辑测试 (per 2026-11-17 用户发令)
//   全部断言走真实 Rust codec (vitest.setup.ts 的 nema wasm handler),
//   与 nema.test.ts 同源同路径 — 不是 mock。
// =====================================================================
import { describe, it, expect, beforeEach } from "vitest";
import {
  formatNemaWithProfileWasm,
  listNemaInterpreterProfilesWasm,
  NEMA_DEFAULT_NEW_DOCUMENT_REVISION,
  NEMA_EDIT_LOG_DOMAIN_ID,
  NEMA_TRANSPORT_HEADER,
  nemaDocumentForSharedFile,
  nemaEditTrailFromStrictDocument,
  parseNemaWithProfileWasm,
  serializeNema,
  type NemaInterpreterProfile,
  type NemaStrictCell,
} from "./nema";
import { useNemaStore } from "./nemaStore";
import { nemaEditorPresence } from "./nemaCollaboration";
import { useStore as useMainStore } from "@/lib/store";

async function defaultProfile(): Promise<NemaInterpreterProfile> {
  const installed = await listNemaInterpreterProfilesWasm();
  const profile = installed.find(
    (candidate) => candidate.grammar_revision === NEMA_DEFAULT_NEW_DOCUMENT_REVISION,
  );
  if (!profile) throw new Error(`profile not installed: ${NEMA_DEFAULT_NEW_DOCUMENT_REVISION}`);
  return profile;
}

const cellText = (cell: NemaStrictCell | undefined): string =>
  cell?.kind === "value" ? cell.value : "";

describe("共同编辑 — 编辑者增删与 .nema 痕迹", () => {
  beforeEach(() => {
    useNemaStore.setState({ nemaTasks: [] });
    useMainStore.setState({ workItems: [] });
  });

  it("追加编辑者 → .nema 源更新 + 痕迹写入 + strict v2 往返合法", async () => {
    const profile = await defaultProfile();
    const entry = await useNemaStore.getState().registerSharedFile(
      {
        filePath: "docs/shared.md",
        title: "SharedDoc",
        editors: ["alice", "bob"],
        purpose: "测试共同编辑",
        method: "PairDesign",
        environment: "docs",
      },
      profile,
    );
    expect(entry).not.toBeNull();
    const result = await useNemaStore.getState().updateNemaEditors(entry!.id, {
      editors: ["alice", "bob", "carol"],
      actor: "Ulysses",
    });
    if (result.action !== "updated") throw new Error(`expected updated, got ${result.action}`);
    // 编辑者列表更新
    expect(result.entry.editors).toEqual(["alice", "bob", "carol"]);
    // pin 的 interpreterProfile 不漂移
    expect(result.entry.interpreterProfile).toEqual(profile);
    // 仍是 strict v2 输出 (不是 legacy transport 头)
    expect(result.entry.source.startsWith(NEMA_TRANSPORT_HEADER)).toBe(false);
    expect(result.entry.source.startsWith(" \\?1 1")).toBe(true);
    // 真实 Rust codec 往返: 解析成功 + 三个域 (5W1H / 元数据 / 痕迹)
    const parsed = await parseNemaWithProfileWasm(result.entry.source, profile);
    expect(parsed.domains).toHaveLength(3);
    const five = parsed.domains.find((d) => d.mode === "five_w_one_h");
    expect(five).toBeDefined();
    expect(cellText(five?.records[0]?.[3])).toBe("alice,bob,carol");
    // 痕迹域: Field/Value 表, 每条痕迹 = editor / edited_at / change 三行
    const log = parsed.domains.find((d) => d.id === NEMA_EDIT_LOG_DOMAIN_ID);
    expect(log?.mode).toBe("table");
    expect(log?.columns).toEqual(["Field", "Value"]);
    expect(log?.records).toHaveLength(3);
    expect(cellText(log?.records[0]?.[0])).toBe("editor");
    expect(cellText(log?.records[0]?.[1])).toBe("Ulysses");
    expect(cellText(log?.records[1]?.[0])).toBe("edited_at");
    expect(cellText(log?.records[2]?.[0])).toBe("change");
    expect(cellText(log?.records[2]?.[1])).toContain("carol");
    // 痕迹读取 (AST → 结构化)
    const trail = nemaEditTrailFromStrictDocument(parsed);
    expect(trail).toHaveLength(1);
    expect(trail[0].editor).toBe("Ulysses");
    expect(trail[0].change).toContain("加入共同编辑者");
    expect(Number.isNaN(Date.parse(trail[0].edited_at))).toBe(false);
    // 元数据表 editors / editor_count / updated_at 行同步
    const meta = parsed.domains.find((d) => d.id === "2");
    const field = (name: string) =>
      meta?.records.find((r) => cellText(r[0]) === name);
    expect(cellText(field("editors")?.[1])).toBe("alice,bob,carol");
    expect(cellText(field("editor_count")?.[1])).toBe("3");
    expect(cellText(field("updated_at")?.[1])).toBeTruthy();
    // canonical: 同一 profile 重新 format 幂等 (Rust canonical round-trip)
    const reformatted = await formatNemaWithProfileWasm(parsed, profile);
    expect(reformatted).toBe(result.entry.source);
  });

  it("移除编辑者 → 源更新 + 移除痕迹", async () => {
    const profile = await defaultProfile();
    const entry = await useNemaStore.getState().registerSharedFile(
      { filePath: "docs/team.md", title: "TeamDoc", editors: ["alice", "bob", "carol"] },
      profile,
    );
    expect(entry).not.toBeNull();
    const result = await useNemaStore.getState().updateNemaEditors(entry!.id, {
      editors: ["alice", "bob"],
      actor: "Ulysses",
    });
    if (result.action !== "updated") throw new Error(`expected updated, got ${result.action}`);
    expect(result.entry.editors).toEqual(["alice", "bob"]);
    const parsed = await parseNemaWithProfileWasm(result.entry.source, profile);
    const trail = nemaEditTrailFromStrictDocument(parsed);
    expect(trail).toHaveLength(1);
    expect(trail[0].change).toContain("移除共同编辑者");
    expect(trail[0].change).toContain("carol");
    const five = parsed.domains.find((d) => d.mode === "five_w_one_h");
    expect(cellText(five?.records[0]?.[3])).toBe("alice,bob");
  });

  it("编辑者不变 → unchanged, 不追加痕迹", async () => {
    const profile = await defaultProfile();
    const entry = await useNemaStore.getState().registerSharedFile(
      { filePath: "docs/stable.md", title: "StableDoc", editors: ["alice", "bob"] },
      profile,
    );
    expect(entry).not.toBeNull();
    const result = await useNemaStore.getState().updateNemaEditors(entry!.id, {
      editors: ["alice", "bob"],
      actor: "Ulysses",
    });
    expect(result.action).toBe("unchanged");
    const parsed = await parseNemaWithProfileWasm(entry!.source, profile);
    expect(parsed.domains).toHaveLength(2); // 无痕迹域
  });

  it("改动后已绑定 WorkItem 标题/描述回写", async () => {
    const profile = await defaultProfile();
    const entry = await useNemaStore.getState().registerSharedFile(
      { filePath: "docs/task.md", title: "TaskFile", editors: ["alice", "bob"] },
      profile,
    );
    expect(entry).not.toBeNull();
    const workItemId = await useNemaStore.getState().openNemaAsStarTask(entry!.id);
    expect(workItemId).toBeTruthy();
    const result = await useNemaStore.getState().updateNemaEditors(entry!.id, {
      editors: ["alice", "bob", "carol"],
      actor: "Ulysses",
    });
    if (result.action !== "updated") throw new Error(`expected updated, got ${result.action}`);
    const workItem = useMainStore.getState().workItems.find((w) => w.id === workItemId);
    expect(workItem).toBeDefined();
    expect(workItem?.title).toBe("TaskFile");
    expect(workItem?.description).toContain("共同编辑");
    expect(workItem?.description).toContain("carol");
  });

  it("legacy 条目不被静默升级", async () => {
    const legacySource = serializeNema(
      nemaDocumentForSharedFile(
        {
          filePath: "old/legacy.md",
          title: "LegacyDoc",
          editors: ["alice", "bob"],
        },
        new Date("2026-11-10T00:00:00.000Z"),
      ),
    );
    expect(legacySource.startsWith(NEMA_TRANSPORT_HEADER)).toBe(true);
    const profile = await defaultProfile();
    const entry = await useNemaStore
      .getState()
      .importNemaSource("old/legacy.md.nema", legacySource, profile);
    expect(entry.legacyFormat).toBe(true);

    const result = await useNemaStore.getState().updateNemaEditors(entry.id, {
      editors: ["alice", "bob", "carol"],
      actor: "Ulysses",
    });
    expect(result.action).toBe("refused");
    const stored = useNemaStore.getState().nemaTasks.find((t) => t.id === entry.id);
    expect(stored?.source).toBe(legacySource);
    expect(stored?.legacyFormat).toBe(true);
    expect(stored?.editors).toEqual(["alice", "bob"]);
  });
});

describe("编辑者状态 — 诚实来源", () => {
  it("无实时协作连接时绝不显示'正在编辑'", () => {
    const trail = [
      { editor: "alice", edited_at: "2026-11-17T00:00:00.000Z", change: "加入共同编辑者: bob" },
    ];
    const presence = nemaEditorPresence(["alice", "bob"], trail, "alice");
    expect(presence.kind).toBe("unavailable");
    expect(presence.label).toContain("未接入");
  });

  it("无痕迹编辑者显示'未开始' (.nema 痕迹可证)", () => {
    const presence = nemaEditorPresence(["alice", "bob"], [], "alice");
    expect(presence.kind).toBe("not_started");
    expect(presence.label).toBe("未开始");
  });

  it("只有真实 presence 源才能给出 editing/offline", () => {
    const live = (editor: string) => (editor === "alice" ? "editing" : "offline");
    expect(nemaEditorPresence(["alice", "bob"], [], "alice", live).kind).toBe("editing");
    expect(nemaEditorPresence(["alice", "bob"], [], "bob", live).kind).toBe("offline");
    // 未知编辑者 (live 返回 null) 仍落回诚实降级
    const liveUnknown = () => null;
    expect(nemaEditorPresence(["alice"], [], "alice", liveUnknown).kind).toBe("not_started");
  });
});
