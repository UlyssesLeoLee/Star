/*
@cypher schema=1 source_sha256=2a5e9d1e934b496b6e25bfd67511f54ef383d8403d0121ddde54053520105ea6
MERGE (self:File {path:"frontend/src/lib/nema/nemaWatcher.test.ts"})
MERGE (suite:Symbol {id:"frontend/src/lib/nema/nemaWatcher.test.ts::nemaWatcher",kind:"suite"})
MERGE (watcher:File {path:"frontend/src/lib/nema/nemaWatcher.ts"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:IMPORTS]->(watcher)
MERGE (suite)-[:TESTS]->(watcher)
@endcypher
*/
// per WBS 8.4.4: .nema 文件系统监听与任务状态回写测试
import { describe, it, expect, beforeEach, vi } from "vitest";
import { scanNemaFiles } from "./nemaWatcher";
import { useNemaStore } from "./nemaStore";
import { useStore as useMainStore } from "@/lib/store";
import { listWorkspaceFiles, readWorkspaceFile } from "@/lib/localFiles";
import { nemaDocumentForSharedFile, serializeNema } from "./nema";

vi.mock("@/lib/localFiles", () => ({
  isLocalMode: () => true,
  listWorkspaceFiles: vi.fn(),
  readWorkspaceFile: vi.fn(),
}));

const mockedList = vi.mocked(listWorkspaceFiles);
const mockedRead = vi.mocked(readWorkspaceFile);

function makeNemaSource(title: string): string {
  return serializeNema(
    nemaDocumentForSharedFile(
      {
        filePath: "docs/example.md",
        title,
        editors: ["alice", "bob"],
        purpose: "测试",
        method: "PairDesign",
        environment: "docs",
      },
      new Date("2026-11-13T08:00:00.000Z"),
    ),
  );
}

const SAMPLE_NEMA = makeNemaSource("测试任务");
const SAMPLE_NEMA_V2 = makeNemaSource("测试任务 v2");

function mockApi(files: Record<string, string>) {
  mockedList.mockResolvedValue({
    status: 200,
    body: {
      root: "",
      path: "",
      entries: Object.keys(files).map((name) => ({ name, is_dir: false, size_bytes: 100 })),
    },
  });
  mockedRead.mockImplementation(async (path: string) => ({
    status: 200,
    body: { path, text: files[path] ?? "" },
  }));
}

describe("nemaWatcher (WBS 8.4.4)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useNemaStore.setState({ nemaTasks: [] });
  });

  it("A. 新 .nema 文件扫描导入", async () => {
    mockApi({ "notes.nema": SAMPLE_NEMA });
    const result = await scanNemaFiles([""]);
    expect(result.scanned).toBe(1);
    expect(result.imported).toBe(1);
    expect(useNemaStore.getState().nemaTasks).toHaveLength(1);
  });

  it("B. 内容变化更新 (source 不同)", async () => {
    mockApi({ "notes.nema": SAMPLE_NEMA });
    await scanNemaFiles([""]);
    mockApi({ "notes.nema": SAMPLE_NEMA_V2 });
    const result = await scanNemaFiles([""]);
    expect(result.updated).toBe(1);
    expect(useNemaStore.getState().nemaTasks[0].source).toContain("v2");
  });

  it("C. 内容未变不重复更新", async () => {
    mockApi({ "notes.nema": SAMPLE_NEMA });
    await scanNemaFiles([""]);
    mockApi({ "notes.nema": SAMPLE_NEMA });
    const result = await scanNemaFiles([""]);
    expect(result.updated).toBe(0);
    expect(result.imported).toBe(0);
    expect(useNemaStore.getState().nemaTasks).toHaveLength(1);
  });

  it("D. 已绑定任务的 .nema 变化回写 workItem 标题", async () => {
    mockApi({ "notes.nema": SAMPLE_NEMA });
    await scanNemaFiles([""]);
    const entry = useNemaStore.getState().nemaTasks[0];
    const workItemId = useNemaStore.getState().openNemaAsStarTask(entry.id);
    expect(workItemId).toBeTruthy();

    mockApi({ "notes.nema": SAMPLE_NEMA_V2 });
    await scanNemaFiles([""]);

    const main = useMainStore.getState();
    const workItem = main.workItems.find((w) => w.id === workItemId);
    expect(workItem?.title).toBe("测试任务 v2");
  });

  it("E. 非 .nema 文件被忽略", async () => {
    mockApi({ "notes.md": SAMPLE_NEMA, "a.txt": "x" });
    const result = await scanNemaFiles([""]);
    expect(result.scanned).toBe(0);
    expect(useNemaStore.getState().nemaTasks).toHaveLength(0);
  });

  it("F. 后端不可用时 graceful 返回", async () => {
    mockedList.mockResolvedValue({ status: 0, body: null, error: "LOCAL_BACKEND_UNREACHABLE" });
    const result = await scanNemaFiles([""]);
    expect(result.backendUnavailable).toBe(true);
  });
});
