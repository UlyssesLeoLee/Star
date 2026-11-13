/*
@cypher schema=1 source_sha256=838dbc7504a31b89804df1e050ae31cafad1a650ef4654337ac9bd77bb89c5ed
MERGE (self:File {path:"frontend/src/lib/nema/nema.test.ts"})
MERGE (roundtrip:Symbol {id:"frontend/src/lib/nema/nema.test.ts::serialize_parse_roundtrip",kind:"test"})
MERGE (autogen:Symbol {id:"frontend/src/lib/nema/nema.test.ts::nema_auto_generation_requires_shared_editors",kind:"test"})
MERGE (openTask:Symbol {id:"frontend/src/lib/nema/nema.test.ts::open_nema_as_star_task_creates_work_item",kind:"test"})
MERGE (self)-[:DEFINES]->(roundtrip)
MERGE (self)-[:DEFINES]->(autogen)
MERGE (self)-[:DEFINES]->(openTask)
@endcypher
*/
import { describe, it, expect, beforeEach } from "vitest";
import {
  parseNema,
  serializeNema,
  nemaDocumentForSharedFile,
  taskViewFromDocument,
  NEMA_TRANSPORT_HEADER,
} from "./nema";
import { useNemaStore } from "./nemaStore";

describe(".NEMA 格式 — 序列化/解析往返", () => {
  it("5W1H + Table 往返保持字段", () => {
    const doc = nemaDocumentForSharedFile(
      {
        filePath: "docs/design/auth.md",
        title: "BuildLogin",
        editors: ["alice", "bob"],
        purpose: "PreventLeak",
        method: "PairDesign",
        environment: "docs",
      },
      new Date("2026-11-11T08:00:00.000Z"),
    );
    const source = serializeNema(doc);
    expect(source.startsWith(NEMA_TRANSPORT_HEADER)).toBe(true);

    const parsed = parseNema(source);
    expect(parsed.sections.length).toBe(2);

    const five = parsed.sections.find((s) => s.mode === "5w1h");
    expect(five).toBeDefined();
    if (five && five.mode === "5w1h") {
      expect(five.what).toBe("BuildLogin");
      expect(five.why).toBe("PreventLeak");
      expect(five.who).toBe("alice,bob");
    }

    const table = parsed.sections.find((s) => s.mode === "table");
    expect(table).toBeDefined();
    if (table && table.mode === "table") {
      const fileRow = table.rows.find((r) => r[0] === "file");
      expect(fileRow?.[1]).toBe("docs/design/auth.md");
    }
  });

  it("显式 null (~) 与转义往返", () => {
    const source = [
      NEMA_TRANSPORT_HEADER,
      "\\1 REV 1",
      "\\# Name Status Detail",
      "Claude Ready Verified",
      "~ Running Build",
      "Spaced\\ Value OK ~",
    ].join("\n");
    const parsed = parseNema(source);
    const table = parsed.sections.find((s) => s.mode === "table");
    expect(table && table.mode === "table" && table.rows[1][0]).toBeNull();
    expect(table && table.mode === "table" && table.rows[2][0]).toBe("Spaced Value");
    const reserialized = serializeNema(parsed);
    const reparsed = parseNema(reserialized);
    expect(reparsed.sections[0].mode).toBe("table");
  });

  it("taskViewFromDocument 提取文件路径", () => {
    const doc = nemaDocumentForSharedFile(
      { filePath: "src/x.rs", title: "T", editors: ["a", "b"] },
      new Date("2026-11-11T08:00:00.000Z"),
    );
    const view = taskViewFromDocument(doc);
    expect(view.filePath).toBe("src/x.rs");
    expect(view.who).toEqual(["a", "b"]);
  });
});

describe(".NEMA 文件即任务规则", () => {
  beforeEach(() => {
    useNemaStore.setState({ nemaTasks: [] });
  });

  it("编辑者 < 2 不产生 .nema (共同编辑才触发)", () => {
    const entry = useNemaStore
      .getState()
      .registerSharedFile({ filePath: "a.md", title: "Solo", editors: ["alice"] });
    expect(entry).toBeNull();
    expect(useNemaStore.getState().nemaTasks.length).toBe(0);
  });

  it("编辑者 ≥ 2 自动产生 .nema 侧车", () => {
    const entry = useNemaStore.getState().registerSharedFile({
      filePath: "docs/shared.md",
      title: "SharedDoc",
      editors: ["alice", "bob"],
    });
    expect(entry).not.toBeNull();
    expect(entry?.nemaPath).toBe("docs/shared.md.nema");
    expect(entry?.source.startsWith(NEMA_TRANSPORT_HEADER)).toBe(true);
    expect(useNemaStore.getState().nemaTasks.length).toBe(1);
  });

  it("同 path 重复登记幂等复用 (不产生第二条)", () => {
    const first = useNemaStore
      .getState()
      .registerSharedFile({ filePath: "x.md", title: "X", editors: ["a", "b"] });
    const second = useNemaStore
      .getState()
      .registerSharedFile({ filePath: "x.md", title: "X", editors: ["a", "b", "c"] });
    expect(second?.id).toBe(first?.id);
    expect(second?.editors.length).toBe(3);
    expect(useNemaStore.getState().nemaTasks.length).toBe(1);
  });

  it("打开为 Star 任务: 创建 WorkItem 并绑定", () => {
    const entry = useNemaStore.getState().registerSharedFile({
      filePath: "docs/task.md",
      title: "TaskFile",
      editors: ["a", "b"],
    });
    expect(entry).not.toBeNull();
    const workItemId = useNemaStore.getState().openNemaAsStarTask(entry!.id);
    expect(workItemId).toBeTruthy();
    // 绑定已回填
    const bound = useNemaStore.getState().nemaTasks.find((t) => t.id === entry!.id);
    expect(bound?.workItemId).toBe(workItemId);
    // 再次打开幂等 (不重复创建)
    const again = useNemaStore.getState().openNemaAsStarTask(entry!.id);
    expect(again).toBe(workItemId);
  });
});
