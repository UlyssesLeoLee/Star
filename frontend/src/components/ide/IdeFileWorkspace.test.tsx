/*
@cypher schema=1 source_sha256=5f23c8acae4f99c20df093332de089a5a0e27668b48aca4ad327c415b7c66c71
MERGE (self:File {path:"frontend/src/components/ide/IdeFileWorkspace.test.tsx"})
MERGE (target:File {path:"frontend/src/components/ide/IdeFileWorkspace.tsx"})
MERGE (ideFileWorkspace:Symbol {id:"frontend/src/components/ide/IdeFileWorkspace.tsx::IdeFileWorkspace",kind:"component"})
MERGE (deriveSemanticNodes:Symbol {id:"frontend/src/components/ide/IdeFileWorkspace.tsx::deriveSemanticNodes",kind:"function"})
MERGE (buildFileTree:Symbol {id:"frontend/src/components/ide/IdeFileWorkspace.tsx::buildFileTree",kind:"function"})
MERGE (self)-[:IMPORTS]->(target)
MERGE (self)-[:TESTS]->(ideFileWorkspace)
MERGE (self)-[:TESTS]->(deriveSemanticNodes)
MERGE (self)-[:TESTS]->(buildFileTree)
@endcypher
*/

// =====================================================================
// IdeFileWorkspace.test.tsx — IDE1.0 文件编辑工作区组件测试
// 覆盖: 内核状态条 (未接入/已连接 + 诊断/Patch 计数) / 文件树构建与
//       折叠 / 语义节点本地推导与切换 / Patch 记录空态与条目 /
//       IDE1.0 四能力徽标 / 纯函数 helper。
// =====================================================================

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import {
  IdeFileWorkspace,
  deriveSemanticNodes,
  buildFileTree,
  type IdeFileRef,
  type IdePatchEntry,
} from "./IdeFileWorkspace";

const FILES: IdeFileRef[] = [
  {
    id: "f-nema",
    filePath: "src/lib/nema/nemaStore.ts",
    editors: ["alice", "bob"],
    workItemId: "wi-1",
  },
  {
    id: "f-header",
    filePath: "src/components/AppHeader.tsx",
    editors: ["carol"],
  },
  {
    id: "f-readme",
    filePath: "README.md",
    editors: ["dave", "erin"],
  },
];

const PATCHES: IdePatchEntry[] = [
  { id: "p-1", summary: "feat: init workspace", created_at: "2026-11-15T10:00:00Z" },
  { id: "p-2", summary: "fix: semantic derive", created_at: "2026-11-15T10:05:00Z" },
];

describe("IdeFileWorkspace", () => {
  it("默认显示编辑内核未接入徽标与 0 计数", () => {
    render(<IdeFileWorkspace files={FILES} />);
    expect(screen.getByTestId("ide-kernel-status").textContent).toContain("未接入");
    expect(screen.getByTestId("ide-diag-count").textContent).toBe("0");
    expect(screen.getByTestId("ide-patch-count").textContent).toBe("0");
  });

  it("内核已连接时显示已连接徽标与真实计数", () => {
    render(<IdeFileWorkspace files={FILES} kernelConnected diagCount={7} patches={PATCHES} />);
    expect(screen.getByTestId("ide-kernel-status").textContent).toContain("已连接");
    expect(screen.getByTestId("ide-diag-count").textContent).toBe("7");
    expect(screen.getByTestId("ide-patch-count").textContent).toBe("2");
  });

  it("按路径段构建文件树: 文件夹可折叠、文件节点带 id testid", () => {
    render(<IdeFileWorkspace files={FILES} />);
    expect(screen.getByTestId("ide-file-tree")).toBeTruthy();
    // 文件叶子节点 (data-testid: ide-file-node-<id>)
    expect(screen.getByTestId("ide-file-node-f-nema")).toBeTruthy();
    expect(screen.getByTestId("ide-file-node-f-header")).toBeTruthy();
    expect(screen.getByTestId("ide-file-node-f-readme")).toBeTruthy();
    // 文件夹节点: src / src/lib / src/lib/nema / src/components
    expect(screen.getByTestId("ide-file-folder-src")).toBeTruthy();
    expect(screen.getByTestId("ide-file-folder-src-lib")).toBeTruthy();
    expect(screen.getByTestId("ide-file-folder-src-lib-nema")).toBeTruthy();
    expect(screen.getByTestId("ide-file-folder-src-components")).toBeTruthy();
    // 文件节点显示编辑人数与任务绑定角标
    const nemaNode = screen.getByTestId("ide-file-node-f-nema");
    expect(nemaNode.textContent).toContain("任务");
    expect(nemaNode.textContent).toContain("2人");
  });

  it("点击文件夹折叠/展开子节点", () => {
    render(<IdeFileWorkspace files={FILES} />);
    const srcFolder = screen.getByTestId("ide-file-folder-src");
    fireEvent.click(srcFolder);
    expect(screen.queryByTestId("ide-file-node-f-nema")).toBeNull();
    expect(screen.queryByTestId("ide-file-node-f-header")).toBeNull();
    fireEvent.click(screen.getByTestId("ide-file-folder-src"));
    expect(screen.getByTestId("ide-file-node-f-nema")).toBeTruthy();
    expect(screen.getByTestId("ide-file-node-f-header")).toBeTruthy();
  });

  it("默认选中第一个文件并推导语义节点 (文件夹=模块, 文件名=函数)", () => {
    render(<IdeFileWorkspace files={FILES} />);
    expect(screen.getByTestId("ide-semantic-panel")).toBeTruthy();
    // 标注本地推导预览
    expect(screen.getByText("本地推导预览")).toBeTruthy();
    // src/lib/nema/nemaStore.ts → 3 模块 + 1 函数
    const nodes = screen.getAllByTestId(/^ide-sem-node-f-nema-\d+$/);
    expect(nodes).toHaveLength(4);
    expect(screen.getByTestId("ide-sem-node-f-nema").textContent).toContain(
      "src/lib/nema/nemaStore.ts",
    );
    expect(nodes[0].textContent).toContain("src");
    expect(nodes[0].textContent).toContain("模块节点");
    expect(nodes[3].textContent).toContain("nemaStore.ts");
    expect(nodes[3].textContent).toContain("函数节点");
  });

  it("点击文件树节点切换语义面板到该文件", () => {
    render(<IdeFileWorkspace files={FILES} />);
    fireEvent.click(screen.getByTestId("ide-file-node-f-readme"));
    expect(screen.getByTestId("ide-sem-node-f-readme")).toBeTruthy();
    // README.md 只有文件名一个函数节点
    const nodes = screen.getAllByTestId(/^ide-sem-node-f-readme-\d+$/);
    expect(nodes).toHaveLength(1);
    expect(nodes[0].textContent).toContain("函数节点");
    // 旧文件的语义节点区已卸载
    expect(screen.queryByTestId("ide-sem-node-f-nema")).toBeNull();
  });

  it("无 Patch 时显示空态文案", () => {
    render(<IdeFileWorkspace files={FILES} />);
    expect(screen.getByTestId("ide-patch-log").textContent).toContain(
      "暂无 Patch 提交 — 编辑内核接入后同步",
    );
  });

  it("有 Patch 时渲染提交记录且不显示空态", () => {
    render(<IdeFileWorkspace files={FILES} patches={PATCHES} />);
    expect(screen.getByTestId("ide-patch-log").textContent).toContain("feat: init workspace");
    expect(screen.getByTestId("ide-patch-log").textContent).toContain("fix: semantic derive");
    expect(screen.getByTestId("ide-patch-log").textContent).not.toContain("暂无 Patch 提交");
  });

  it("渲染 IDE1.0 四个能力徽标", () => {
    render(<IdeFileWorkspace files={FILES} />);
    const caps: Record<string, string> = {
      "ide-cap-file-routing": "文件级路由",
      "ide-cap-semantic-parallel": "语义节点并行",
      "ide-cap-patch-commit": "Patch 级提交",
      "ide-cap-cas-fencing": "CAS+Fencing",
    };
    for (const [testId, label] of Object.entries(caps)) {
      const el = screen.getByTestId(testId);
      expect(el.textContent).toContain(label);
    }
  });

  it("文件列表为空时文件树显示空态且语义面板提示先选择文件", () => {
    render(<IdeFileWorkspace files={[]} />);
    expect(screen.getByTestId("ide-file-tree").textContent).toContain("暂无文件");
    expect(screen.getByTestId("ide-semantic-panel").textContent).toContain(
      "选中左侧文件后展示语义节点",
    );
  });
});

describe("deriveSemanticNodes", () => {
  it("文件夹段=模块节点, 文件名段=函数节点", () => {
    expect(deriveSemanticNodes("src/lib/store.ts")).toEqual([
      { label: "src", kind: "module" },
      { label: "lib", kind: "module" },
      { label: "store.ts", kind: "function" },
    ]);
  });

  it("无路径分隔符时只有函数节点", () => {
    expect(deriveSemanticNodes("README.md")).toEqual([
      { label: "README.md", kind: "function" },
    ]);
  });
});

describe("buildFileTree", () => {
  it("共享前缀目录只出现一次", () => {
    const tree = buildFileTree(FILES);
    const src = tree.find((n) => n.kind === "dir" && n.name === "src");
    expect(src).toBeTruthy();
    const rootFiles = tree.filter((n) => n.kind === "file");
    expect(rootFiles).toHaveLength(1); // README.md 在根层
  });

  it("目录排在文件前", () => {
    const tree = buildFileTree([
      { id: "a", filePath: "z.ts", editors: [] },
      { id: "b", filePath: "src/a.ts", editors: [] },
    ]);
    expect(tree[0].kind).toBe("dir");
    expect(tree[1].kind).toBe("file");
  });
});

// =====================================================================
// 内核已连接模式 (per 2026-11-17 IDE1.0 网页版接入) — mock fetch
// =====================================================================

/** 模拟内核 REST 响应 (hook 只用 ok/status/json)。 */
function jsonBody(body: unknown, status = 200) {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: async () => body,
  };
}

const DOC_VIEW = {
  id: 1,
  path: "src/lib/nema/nemaStore.ts",
  text: "fn main() {}\n",
  line_count: 1,
  caret_row: 0,
  caret_col: 0,
  line_ending: "Unix (LF)",
  encoding: "UTF-8",
  dirty: false,
  has_selection: false,
  selected_text: "",
  cursor_count: 1,
  can_undo: true,
  can_redo: false,
};

/** 按端点路由的 mock fetch。 */
function mockIdeFetch() {
  const mockFetch = vi.fn(async (input: unknown, init?: RequestInit) => {
    const url = String(input);
    if (url.includes("/api/v1/ide/version")) {
      return jsonBody({ data: { kernel_version: "0.1.0", service: "ide-kernel" } });
    }
    if (url.includes("/api/v1/ide/doc/open")) {
      const body = init?.body ? (JSON.parse(String(init.body)) as { path?: string; text?: string }) : {};
      return jsonBody({
        data: {
          view: {
            ...DOC_VIEW,
            path: body.path ?? DOC_VIEW.path,
            text: body.text ?? DOC_VIEW.text,
          },
        },
      });
    }
    if (url.includes("/api/v1/ide/doc/undo")) {
      return jsonBody({ data: { view: { ...DOC_VIEW, can_undo: false, can_redo: true } } });
    }
    if (url.includes("/api/v1/ide/doc/sync")) {
      const body = init?.body ? (JSON.parse(String(init.body)) as { text?: string }) : {};
      return jsonBody({ data: { view: { ...DOC_VIEW, text: body.text ?? "", dirty: true } } });
    }
    if (url.includes("/api/v1/ide/outline")) {
      return jsonBody({
        data: {
          path: "x.rs",
          language: "Rust",
          entries: [
            { name: "main", kind: "function", line: 0, depth: 0 },
            { name: "helper", kind: "function", line: 2, depth: 0 },
          ],
          error: null,
        },
      });
    }
    if (url.includes("/api/v1/ide/scm/log")) {
      return jsonBody({
        data: {
          commits: [
            {
              hash: "a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4",
              short: "a1b2c3d4",
              author: "tester",
              time: "2026-11-17T00:00:00Z",
              message: "feat: kernel wiring",
            },
          ],
          error: null,
        },
      });
    }
    return jsonBody({ data: {} }, 404);
  });
  // @ts-expect-error - global fetch assignment for jsdom
  global.fetch = mockFetch;
  return mockFetch;
}

describe("IdeFileWorkspace 内核已连接 (probeKernel)", () => {
  let mockFetch: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    mockFetch = mockIdeFetch();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("探测成功 → 已连接徽标 + 内核版本 + 内核 Patch 记录", async () => {
    render(<IdeFileWorkspace files={FILES} probeKernel />);
    await waitFor(() => {
      expect(screen.getByTestId("ide-kernel-status").textContent).toContain("已连接");
    });
    expect(screen.getByTestId("ide-kernel-version").textContent).toContain("ide-kernel 0.1.0");
    await waitFor(() => {
      expect(screen.getByTestId("ide-patch-log").textContent).toContain("feat: kernel wiring");
    });
    expect(screen.getByTestId("ide-patch-count").textContent).toBe("1");
    expect(screen.getByTestId("ide-editor-panel")).toBeTruthy();
  });

  it("点击文件节点 → POST doc/open 载入内核会话并渲染 textarea", async () => {
    render(<IdeFileWorkspace files={FILES} probeKernel />);
    await waitFor(() => {
      expect(screen.getByTestId("ide-kernel-status").textContent).toContain("已连接");
    });
    fireEvent.click(screen.getByTestId("ide-file-node-f-readme"));
    await waitFor(() => {
      expect(screen.getByTestId("ide-editor-textarea").value).toBe("fn main() {}\n");
    });
    const openCalls = mockFetch.mock.calls.filter(([input]) =>
      String(input).includes("/api/v1/ide/doc/open"),
    );
    expect(openCalls.length).toBeGreaterThanOrEqual(1);
  });

  it("textarea 修改 → 防抖 POST /doc/sync", async () => {
    render(<IdeFileWorkspace files={FILES} probeKernel />);
    await waitFor(() => {
      expect(screen.getByTestId("ide-kernel-status").textContent).toContain("已连接");
    });
    fireEvent.click(screen.getByTestId("ide-file-node-f-readme"));
    await waitFor(() => {
      expect(screen.getByTestId("ide-editor-textarea")).toBeTruthy();
    });
    fireEvent.change(screen.getByTestId("ide-editor-textarea"), {
      target: { value: "fn main() {}\n// edited" },
    });
    await waitFor(() => {
      const syncCalls = mockFetch.mock.calls.filter(([input]) =>
        String(input).includes("/api/v1/ide/doc/sync"),
      );
      expect(syncCalls.length).toBeGreaterThanOrEqual(1);
    });
  });

  it("撤销按钮 → POST /doc/undo (can_undo 时可用)", async () => {
    render(<IdeFileWorkspace files={FILES} probeKernel />);
    await waitFor(() => {
      expect(screen.getByTestId("ide-kernel-status").textContent).toContain("已连接");
    });
    fireEvent.click(screen.getByTestId("ide-file-node-f-readme"));
    await waitFor(() => {
      expect(screen.getByTestId("ide-editor-textarea")).toBeTruthy();
    });
    fireEvent.click(screen.getByTestId("ide-op-undo"));
    await waitFor(() => {
      const undoCalls = mockFetch.mock.calls.filter(([input]) =>
        String(input).includes("/api/v1/ide/doc/undo"),
      );
      expect(undoCalls.length).toBeGreaterThanOrEqual(1);
    });
  });

  it("已打开文档带正文 → 内核 outline 语义节点 (真实内核, 非本地推导)", async () => {
    render(<IdeFileWorkspace files={FILES} probeKernel />);
    await waitFor(() => {
      expect(screen.getByTestId("ide-kernel-status").textContent).toContain("已连接");
    });
    // f-nema 路径与 mock open 返回的 view.path 一致 → outline 带正文
    fireEvent.click(screen.getByTestId("ide-file-node-f-nema"));
    await waitFor(() => {
      expect(screen.getByText("内核 outline")).toBeTruthy();
    });
    const nodes = screen.getAllByTestId(/^ide-outline-node-f-nema-\d+$/);
    expect(nodes).toHaveLength(2);
    expect(nodes[0].textContent).toContain("main");
  });
});
