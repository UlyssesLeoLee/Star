/*
@cypher schema=1 source_sha256=9257c39fe2e5396b38855b27774f8685f5ecf3104154174a2c627abed062f920
MERGE (self:File {path:"frontend/src/lib/markdown/markdownParser.ts"})
MERGE (fn:Symbol {id:"frontend/src/lib/markdown/markdownParser.ts::markdownToHtml",kind:"function"})
MERGE (esc:Symbol {id:"frontend/src/lib/markdown/markdownParser.ts::escapeHtml",kind:"function"})
MERGE (self)-[:DEFINES]->(fn)
MERGE (self)-[:DEFINES]->(esc)
MERGE (fn)-[:CALLS]->(esc)
@endcypher
*/
// =====================================================================
// markdownParser — 轻量 Markdown → 安全 HTML (WBS 8.4.5)
// 支持: h1-h6 / 加粗 / 斜体 / 行内码 / 围栏代码块 / ul / ol / 链接 / 图片 / 表格 / hr / blockquote
// 安全: 所有文本先 HTML 转义; 原始 HTML 不透传; URL 仅放行 http(s)/mailto/相对/锚点
// =====================================================================

const ESC: Record<string, string> = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" };

export function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ESC[c]);
}

/** 仅放行安全协议; 其余返回 null */
export function sanitizeUrl(url: string): string | null {
  const u = url.trim();
  // 去掉控制字符/空白, 防 "java\tscript:" 绕过
  // eslint-disable-next-line no-control-regex
  const probe = u.replace(/[\u0000- \u007f]/g, "").toLowerCase();
  const m = /^([a-z][a-z0-9+.-]*):/.exec(probe);
  if (m && !["http", "https", "mailto"].includes(m[1])) return null;
  if (probe.startsWith("//")) return null;
  return u;
}

const INLINE_RE =
  /(`+)([\s\S]+?)\1|!\[([^\]]*)\]\(([^)\s]+)(?:\s+"[^"]*")?\)|\[([^\]]+)\]\(([^)\s]+)(?:\s+"[^"]*")?\)|\*\*([\s\S]+?)\*\*|__([\s\S]+?)__|\*([^*\s][^*]*?)\*|(?<![A-Za-z0-9])_([^_\s][^_]*?)_(?![A-Za-z0-9])/;

function renderInline(raw: string): string {
  let out = "";
  let rest = raw;
  while (rest.length > 0) {
    const m = INLINE_RE.exec(rest);
    if (!m) {
      out += escapeHtml(rest);
      break;
    }
    out += escapeHtml(rest.slice(0, m.index));
    rest = rest.slice(m.index + m[0].length);
    if (m[1] !== undefined) {
      out += `<code>${escapeHtml(m[2].trim())}</code>`;
    } else if (m[4] !== undefined) {
      const src = sanitizeUrl(m[4]);
      out +=
        src === null
          ? escapeHtml(m[0])
          : `<img src="${escapeHtml(src)}" alt="${escapeHtml(m[3])}" loading="lazy" />`;
    } else if (m[6] !== undefined) {
      const href = sanitizeUrl(m[6]);
      out +=
        href === null
          ? renderInline(m[5])
          : `<a href="${escapeHtml(href)}" target="_blank" rel="noopener noreferrer nofollow">${renderInline(m[5])}</a>`;
    } else if (m[7] !== undefined || m[8] !== undefined) {
      out += `<strong>${renderInline(m[7] ?? m[8])}</strong>`;
    } else {
      out += `<em>${renderInline(m[9] ?? m[10])}</em>`;
    }
  }
  return out;
}

const isBlank = (l: string) => /^\s*$/.test(l);
const FENCE = /^\s{0,3}(`{3,}|~{3,})\s*([\w+#.-]*)[^\n]*$/;
const HR = /^\s{0,3}([-*_])(\s*\1){2,}\s*$/;
const HEADING = /^\s{0,3}(#{1,6})\s+(.*?)\s*#*\s*$/;
const UL = /^(\s*)[-*+]\s+(.*)$/;
const OL = /^(\s*)\d+[.)]\s+(.*)$/;
const TABLE_SEP = /^\s*\|?\s*:?-{1,}:?\s*(\|\s*:?-{1,}:?\s*)*\|?\s*$/;

function splitRow(line: string): string[] {
  let s = line.trim();
  if (s.startsWith("|")) s = s.slice(1);
  if (s.endsWith("|")) s = s.slice(0, -1);
  return s.split("|").map((c) => c.trim());
}

function isBlockStart(l: string): boolean {
  return FENCE.test(l) || HR.test(l) || HEADING.test(l) || UL.test(l) || OL.test(l) || /^\s{0,3}>/.test(l);
}

function isFenceClose(l: string, marker: string): boolean {
  const t = l.trim();
  return t.length >= marker.length && t.split("").every((ch) => ch === marker[0]);
}

function renderBlocks(lines: string[]): string {
  const out: string[] = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (isBlank(line)) {
      i++;
      continue;
    }

    const fence = FENCE.exec(line);
    if (fence) {
      const marker = fence[1];
      const lang = fence[2];
      const buf: string[] = [];
      i++;
      while (i < lines.length && !isFenceClose(lines[i], marker)) {
        buf.push(lines[i]);
        i++;
      }
      i++; // 跳过闭合围栏 (未闭合则到文末)
      const cls = lang ? ` class="language-${escapeHtml(lang)}"` : "";
      out.push(`<pre data-md-code="true"><code${cls}>${escapeHtml(buf.join("\n"))}</code></pre>`);
      continue;
    }

    if (HR.test(line)) {
      out.push("<hr />");
      i++;
      continue;
    }

    const h = HEADING.exec(line);
    if (h) {
      const n = h[1].length;
      out.push(`<h${n}>${renderInline(h[2])}</h${n}>`);
      i++;
      continue;
    }

    if (/^\s{0,3}>/.test(line)) {
      const buf: string[] = [];
      while (i < lines.length && /^\s{0,3}>/.test(lines[i])) {
        buf.push(lines[i].replace(/^\s{0,3}>\s?/, ""));
        i++;
      }
      out.push(`<blockquote>${renderBlocks(buf)}</blockquote>`);
      continue;
    }

    if (i + 1 < lines.length && line.includes("|") && TABLE_SEP.test(lines[i + 1]) && lines[i + 1].includes("-")) {
      const head = splitRow(line);
      const aligns = splitRow(lines[i + 1]).map((c) =>
        c.startsWith(":") && c.endsWith(":") ? "center" : c.endsWith(":") ? "right" : c.startsWith(":") ? "left" : ""
      );
      i += 2;
      const rows: string[][] = [];
      while (i < lines.length && !isBlank(lines[i]) && lines[i].includes("|")) {
        rows.push(splitRow(lines[i]));
        i++;
      }
      const cell = (tag: string, c: string, idx: number) =>
        `<${tag}${aligns[idx] ? ` style="text-align:${aligns[idx]}"` : ""}>${renderInline(c)}</${tag}>`;
      out.push(
        `<table><thead><tr>${head.map((c, k) => cell("th", c, k)).join("")}</tr></thead>` +
          `<tbody>${rows.map((r) => `<tr>${head.map((_, k) => cell("td", r[k] ?? "", k)).join("")}</tr>`).join("")}</tbody></table>`
      );
      continue;
    }

    const ul = UL.exec(line);
    const ol = OL.exec(line);
    if (ul || ol) {
      const ordered = !!ol && !ul;
      const re = ordered ? OL : UL;
      const items: string[] = [];
      while (i < lines.length) {
        const m = re.exec(lines[i]);
        if (!m) break;
        let text = m[2];
        i++;
        // 续行 (缩进的非列表行) 并入当前项
        while (
          i < lines.length &&
          !isBlank(lines[i]) &&
          /^\s{2,}\S/.test(lines[i]) &&
          !UL.test(lines[i]) &&
          !OL.test(lines[i])
        ) {
          text += " " + lines[i].trim();
          i++;
        }
        items.push(`<li>${renderInline(text)}</li>`);
        if (i + 1 < lines.length && isBlank(lines[i]) && re.test(lines[i + 1])) i++;
      }
      const tag = ordered ? "ol" : "ul";
      out.push(`<${tag}>${items.join("")}</${tag}>`);
      continue;
    }

    const buf: string[] = [line.trim()];
    i++;
    while (i < lines.length && !isBlank(lines[i]) && !isBlockStart(lines[i])) {
      buf.push(lines[i].trim());
      i++;
    }
    out.push(`<p>${renderInline(buf.join(" "))}</p>`);
  }
  return out.join("\n");
}

/** Markdown → 安全 HTML 字符串 */
export function markdownToHtml(md: string): string {
  return renderBlocks(md.replace(/\r\n?/g, "\n").split("\n"));
}
