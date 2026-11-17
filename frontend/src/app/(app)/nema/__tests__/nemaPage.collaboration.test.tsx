/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/(app)/nema/__tests__/nemaPage.collaboration.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/app/(app)/nema/__tests__/nemaPage.collaboration.test.tsx::nema_page_collaboration",kind:"suite"})
MERGE (page:File {path:"frontend/src/app/(app)/nema/page.tsx"})
MERGE (store:File {path:"frontend/src/lib/nema/nemaStore.ts"})
MERGE (register:Symbol {id:"frontend/src/lib/nema/nemaStore.ts::registerSharedFile",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:IMPORTS]->(page)
MERGE (self)-[:IMPORTS]->(store)
MERGE (suite)-[:TESTS]->(page)
MERGE (suite)-[:TESTS]->(register)
@endcypher
*/
// =====================================================================
// nema 页面共同编辑区块测试 (per 2026-11-17 用户发令)
//   - 区块存在 + 实时协作未接入的诚实标注 (StatusNotice disconnected)
//   - 编辑者状态绝不显示"正在编辑" (无真实 presence 源)
// =====================================================================
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import NemaPage from "../page";
import { I18nProvider } from "@/lib/i18n";
import { useNemaStore } from "@/lib/nema/nemaStore";
import { useStore as useMainStore } from "@/lib/store";
import {
  listNemaInterpreterProfilesWasm,
  NEMA_DEFAULT_NEW_DOCUMENT_REVISION,
} from "@/lib/nema/nema";

function renderPage() {
  return render(<I18nProvider initialLanguage="zh-CN"><NemaPage /></I18nProvider>);
}

describe("nema 页面共同编辑区块", () => {
  beforeEach(() => {
    useNemaStore.setState({ nemaTasks: [] });
    useMainStore.setState({ identities: [], workItems: [] });
  });

  it("渲染共同编辑区块 + 实时协作未接入诚实标注", async () => {
    renderPage();
    expect(screen.getByTestId("nema-collaboration")).toBeInTheDocument();
    const notice = await screen.findByTestId("status-notice-disconnected");
    expect(notice.textContent).toContain("实时协作未接入");
    expect(screen.getByTestId("nema-collaboration-empty")).toBeInTheDocument();
  });

  it("登记后的编辑者显示'未开始'而非'正在编辑' (无真实 presence 源)", async () => {
    const installed = await listNemaInterpreterProfilesWasm();
    const profile = installed.find(
      (p) => p.grammar_revision === NEMA_DEFAULT_NEW_DOCUMENT_REVISION,
    );
    if (!profile) throw new Error(`profile not installed: ${NEMA_DEFAULT_NEW_DOCUMENT_REVISION}`);
    const entry = await useNemaStore.getState().registerSharedFile(
      { filePath: "docs/collab.md", title: "CollabDoc", editors: ["alice", "bob"] },
      profile,
    );
    expect(entry).not.toBeNull();

    renderPage();
    expect(await screen.findByTestId("nema-editor-alice")).toBeInTheDocument();
    expect(screen.getByTestId("nema-presence-alice").textContent).toBe("未开始");
    expect(screen.getByTestId("nema-presence-bob").textContent).toBe("未开始");
    expect(screen.queryByText("正在编辑")).toBeNull();
    expect(screen.getByTestId("nema-remove-editor-alice")).toBeInTheDocument();
    expect(screen.getByTestId("nema-add-editor")).toBeInTheDocument();
    expect(screen.getByTestId("nema-edit-trail")).toBeInTheDocument();
  });
});
