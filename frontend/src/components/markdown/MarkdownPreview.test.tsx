/*
@cypher schema=1 source_sha256=pending
MERGE (self:File {path:"frontend/src/components/markdown/MarkdownPreview.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/markdown/MarkdownPreview.test.tsx::MarkdownPreview",kind:"suite"})
MERGE (c:Symbol {id:"frontend/src/components/markdown/MarkdownPreview.tsx::MarkdownPreview",kind:"component"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(c)
@endcypher
*/
import { describe, it, expect, afterEach, vi } from "vitest";
import { render, screen, fireEvent, cleanup, waitFor } from "@testing-library/react";
import { I18nProvider } from "@/lib/i18n";
import { MarkdownPreview } from "./MarkdownPreview";

afterEach(cleanup);
const r = (src: string) =>
  render(
    <I18nProvider initialLanguage="zh-CN">
      <MarkdownPreview source={src} />
    </I18nProvider>
  );

describe("MarkdownPreview", () => {
  it("renders heading", () => {
    r("# 标题");
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("标题");
    expect(screen.getByTestId("markdown-preview")).toBeTruthy();
  });
  it("renders code block with copy button and copies text", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });
    r("```js\nlet a = 1;\n```");
    expect(screen.getByTestId("markdown-preview-body").querySelector("pre code")?.textContent).toBe("let a = 1;");
    fireEvent.click(screen.getByTestId("markdown-preview-copy-0"));
    await waitFor(() => expect(writeText).toHaveBeenCalledWith("let a = 1;"));
    await waitFor(() => expect(screen.getByTestId("markdown-preview-copy-0").textContent).toBe("已复制"));
  });
  it("renders table", () => {
    r("| a | b |\n|--|--|\n| 1 | 2 |");
    const body = screen.getByTestId("markdown-preview-body");
    expect(body.querySelectorAll("th").length).toBe(2);
    expect(body.querySelector("td")?.textContent).toBe("1");
  });
  it("does not render raw html", () => {
    r("<script>window.__x=1</script><b>hi</b>");
    const body = screen.getByTestId("markdown-preview-body");
    expect(body.querySelector("script")).toBeNull();
    expect(body.querySelector("b")).toBeNull();
  });
});
