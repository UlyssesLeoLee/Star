/*
@cypher schema=1 source_sha256=f4b0cc6c123d93e600dad1b2b9226489f7706dc377327802fb3bb40ecac73447
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

import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
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
