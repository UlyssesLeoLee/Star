/*
@cypher schema=1 source_sha256=pending
MERGE (self:File {path:"frontend/src/lib/markdown/markdownParser.test.ts"})
MERGE (suite:Symbol {id:"frontend/src/lib/markdown/markdownParser.test.ts::markdownParser",kind:"suite"})
MERGE (fn:Symbol {id:"frontend/src/lib/markdown/markdownParser.ts::markdownToHtml",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(fn)
@endcypher
*/
import { describe, it, expect } from "vitest";
import { markdownToHtml as md, escapeHtml, sanitizeUrl } from "./markdownParser";

describe("markdownParser", () => {
  it("headings h1-h6", () => {
    for (let n = 1; n <= 6; n++) expect(md(`${"#".repeat(n)} T`)).toBe(`<h${n}>T</h${n}>`);
    expect(md("####### x")).toContain("<p>");
  });
  it("bold / italic / inline code", () => {
    expect(md("**b** *i* _u_ `c`")).toBe("<p><strong>b</strong> <em>i</em> <em>u</em> <code>c</code></p>");
    expect(md("`<b>`")).toContain("<code>&lt;b&gt;</code>");
  });
  it("fenced code block keeps content literal", () => {
    const h = md("```ts\nconst a = <b>**x**</b>;\n```");
    expect(h).toContain('<code class="language-ts">const a = &lt;b&gt;**x**&lt;/b&gt;;</code>');
    expect(h).not.toContain("<strong>");
  });
  it("unclosed fence runs to end", () => {
    expect(md("```\na\nb")).toContain("a\nb</code>");
  });
  it("unordered and ordered lists", () => {
    expect(md("- a\n- b")).toBe("<ul><li>a</li><li>b</li></ul>");
    expect(md("1. a\n2. b")).toBe("<ol><li>a</li><li>b</li></ol>");
  });
  it("links and images", () => {
    expect(md("[x](https://a.com)")).toContain(
      '<a href="https://a.com" target="_blank" rel="noopener noreferrer nofollow">x</a>'
    );
    expect(md("![alt](/a.png)")).toContain('<img src="/a.png" alt="alt"');
  });
  it("table with alignment", () => {
    const h = md("| a | b |\n|:--|--:|\n| 1 | 2 |");
    expect(h).toContain('<th style="text-align:left">a</th>');
    expect(h).toContain('<td style="text-align:right">2</td>');
  });
  it("hr and blockquote", () => {
    expect(md("---")).toBe("<hr />");
    expect(md("> hi **x**")).toBe("<blockquote><p>hi <strong>x</strong></p></blockquote>");
  });
  it("paragraph merges soft-wrapped lines", () => {
    expect(md("a\nb")).toBe("<p>a b</p>");
  });
  it("CRLF handled", () => {
    expect(md("# a\r\n\r\ntext")).toBe("<h1>a</h1>\n<p>text</p>");
  });
});

describe("markdownParser XSS", () => {
  it("escapes raw html", () => {
    const h = md("<script>alert(1)</script> <img src=x onerror=alert(1)>");
    expect(h).not.toContain("<script");
    expect(h).not.toContain("<img");
    expect(h).toContain("&lt;script&gt;");
  });
  it("blocks javascript: links (incl. whitespace obfuscation)", () => {
    expect(md("[x](javascript:alert(1))")).not.toContain("href");
    expect(md("[x](JaVa\tScRiPt:alert(1))")).not.toContain("href");
    expect(md("[x](data:text/html;base64,AAAA)")).not.toContain("href");
  });
  it("blocks dangerous image src", () => {
    expect(md("![a](javascript:alert(1))")).not.toContain("<img");
  });
  it("escapes attribute-breaking quotes", () => {
    const i = md('![a"onerror="x](/p.png)');
    expect(i).not.toMatch(/alt="a"onerror/);
    expect(i).toContain("&quot;");
  });
  it("escapes html in headings, tables, lists, quotes", () => {
    expect(md("# <b>x</b>")).toBe("<h1>&lt;b&gt;x&lt;/b&gt;</h1>");
    expect(md("| <i> |\n|--|\n| <u> |")).not.toMatch(/<i>|<u>/);
    expect(md("- <s>x")).not.toContain("<s>");
    expect(md("> <s>x")).not.toContain("<s>");
  });
  it("helpers", () => {
    expect(escapeHtml(`<>&"'`)).toBe("&lt;&gt;&amp;&quot;&#39;");
    expect(sanitizeUrl("//evil.com")).toBeNull();
    expect(sanitizeUrl("mailto:a@b.c")).toBe("mailto:a@b.c");
    expect(sanitizeUrl("#sec")).toBe("#sec");
  });
});
