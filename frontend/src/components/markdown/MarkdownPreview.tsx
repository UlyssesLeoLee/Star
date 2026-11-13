/*
@cypher schema=1 source_sha256=218be3c4b982c8fcb9d9cc6e7e435d41c405206aaea5f73ca3d235c14dfc5d75
MERGE (self:File {path:"frontend/src/components/markdown/MarkdownPreview.tsx"})
MERGE (c:Symbol {id:"frontend/src/components/markdown/MarkdownPreview.tsx::MarkdownPreview",kind:"component"})
MERGE (p:Symbol {id:"frontend/src/lib/markdown/markdownParser.ts::markdownToHtml",kind:"function"})
MERGE (self)-[:DEFINES]->(c)
MERGE (c)-[:CALLS]->(p)
@endcypher
*/
"use client";
import { useMemo, useRef, useState } from "react";
import { markdownToHtml } from "@/lib/markdown/markdownParser";
import { useTranslation } from "@/lib/i18n";

export interface MarkdownPreviewProps {
  source: string;
  className?: string;
  "data-testid"?: string;
}

const STYLE = `
.md-preview{color:var(--cel-ink-soft,inherit);line-height:1.65;font-size:14px;word-break:break-word}
.md-preview h1,.md-preview h2,.md-preview h3,.md-preview h4,.md-preview h5,.md-preview h6{font-weight:700;margin:1.2em 0 .5em;line-height:1.3}
.md-preview h1{font-size:1.6em}.md-preview h2{font-size:1.35em}.md-preview h3{font-size:1.15em}
.md-preview p{margin:.6em 0}
.md-preview a{color:var(--cel-cyan,#00f0ff);text-decoration:underline}
.md-preview ul{list-style:disc;padding-left:1.5em;margin:.6em 0}
.md-preview ol{list-style:decimal;padding-left:1.5em;margin:.6em 0}
.md-preview blockquote{border-left:3px solid var(--cel-cyan,#00f0ff);padding:.1em 1em;margin:.8em 0;opacity:.85}
.md-preview hr{border:0;border-top:1px solid var(--cel-ink,#444);margin:1.2em 0;opacity:.5}
.md-preview img{max-width:100%}
.md-preview code{background:var(--bg-3,rgba(127,127,127,.2));padding:.1em .35em;border-radius:3px;font-family:ui-monospace,monospace;font-size:.9em}
.md-preview pre{background:var(--bg-2,rgba(0,0,0,.4));border:1px solid var(--cel-ink,#444);border-radius:4px;padding:.8em 1em;overflow:auto;margin:.8em 0}
.md-preview pre code{background:transparent;padding:0}
.md-preview table{border-collapse:collapse;margin:.8em 0;display:block;overflow:auto}
.md-preview th,.md-preview td{border:1px solid var(--cel-ink,#444);padding:.35em .7em}
.md-preview th{background:var(--bg-3,rgba(127,127,127,.2))}
.md-code-copy{font-size:11px;padding:2px 8px;border:1px solid var(--cel-ink,#444);border-radius:3px;background:var(--bg-3,#222);color:inherit;cursor:pointer}
`;

/** 站内 Markdown 预览 (WBS 8.4.5). HTML 由 markdownParser 转义生成, 无原始 HTML 透传. */
export function MarkdownPreview({
  source,
  className,
  "data-testid": testId = "markdown-preview",
}: MarkdownPreviewProps) {
  const { t } = useTranslation();
  const html = useMemo(() => markdownToHtml(source), [source]);
  const rootRef = useRef<HTMLDivElement>(null);
  const [copiedIdx, setCopiedIdx] = useState<number | null>(null);

  // 复制按钮在解析产物之外渲染 (不向 HTML 注入脚本), 按序号对应第 N 个代码块
  const codeBlocks = useMemo(() => (html.match(/<pre data-md-code="true">/g) ?? []).length, [html]);

  const onCopy = async (idx: number) => {
    const pre = rootRef.current?.querySelectorAll("pre[data-md-code]")[idx];
    const text = pre?.textContent ?? "";
    try {
      await navigator.clipboard.writeText(text);
      setCopiedIdx(idx);
      setTimeout(() => setCopiedIdx((c) => (c === idx ? null : c)), 1500);
    } catch {
      /* 剪贴板不可用: 静默 */
    }
  };

  return (
    <div className={className} data-testid={testId}>
      <style>{STYLE}</style>
      <div
        ref={rootRef}
        className="md-preview"
        data-testid={`${testId}-body`}
        dangerouslySetInnerHTML={{ __html: html }}
      />
      {codeBlocks > 0 && (
        <div data-testid={`${testId}-copy-bar`} style={{ display: "flex", gap: 6, flexWrap: "wrap", marginTop: 6 }}>
          {Array.from({ length: codeBlocks }, (_, idx) => (
            <button
              key={idx}
              type="button"
              className="md-code-copy"
              data-testid={`${testId}-copy-${idx}`}
              onClick={() => void onCopy(idx)}
            >
              {copiedIdx === idx ? t.markdownPreview.copied : `${t.markdownPreview.copy} #${idx + 1}`}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

export default MarkdownPreview;
