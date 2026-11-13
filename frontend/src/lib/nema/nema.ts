/*
@cypher schema=1 source_sha256=2c0b215f119a2b8f1f85939e1a8dd0783775a9ae991885a7cddf05178d1c020b
MERGE (self:File {path:"frontend/src/lib/nema/nema.ts"})
MERGE (parse:Symbol {id:"frontend/src/lib/nema/nema.ts::parseNema",kind:"function"})
MERGE (serialize:Symbol {id:"frontend/src/lib/nema/nema.ts::serializeNema",kind:"function"})
MERGE (gen:Symbol {id:"frontend/src/lib/nema/nema.ts::nemaDocumentForSharedFile",kind:"function"})
MERGE (types:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaDocument",kind:"type"})
MERGE (self)-[:DEFINES]->(parse)
MERGE (self)-[:DEFINES]->(serialize)
MERGE (self)-[:DEFINES]->(gen)
MERGE (self)-[:DEFINES]->(types)
@endcypher
*/
// =====================================================================
// .NEMA 格式核心 — 文件即任务架构 (per 2026-11-11 用户发令)
// 语法对齐 https://nema-sigma.vercel.app/ 的 .NEMA 规范子集:
//   \!NEMAT1            transport 头 (v1)
//   \?<id> REV <n>      5W1H 节 (What Why How Who When Where 六列)
//   \<id> REV <n>       Table 节 (\# 列名行 + 数据行, ~ = 显式 null)
//   \><id> REV <n>      Graph 节 (A Name Desc 节点 / A>X>B Label Desc 边)
//   前导 N 空格          续写第 N 列 (X_N 续写语义)
// =====================================================================

export const NEMA_TRANSPORT_HEADER = "\\!NEMAT1";

export interface NemaTableSection {
  mode: "table";
  id: number;
  revision: number;
  columns: string[];
  rows: Array<Array<string | null>>;
}

export interface Nema5W1HSection {
  mode: "5w1h";
  id: number;
  revision: number;
  what: string;
  why: string;
  how: string;
  who: string;
  when: string;
  where: string;
}

export interface NemaGraphSection {
  mode: "graph";
  id: number;
  revision: number;
  nodes: Array<{ id: string; name: string; description: string }>;
  edges: Array<{ id: string; from: string; to: string; operator: ">" | "<" | "="; label: string; description: string }>;
}

export type NemaSection = NemaTableSection | Nema5W1HSection | NemaGraphSection;

export interface NemaDocument {
  transport: string;
  sections: NemaSection[];
}

/** 拆 token: 字段由单空格分隔; \空格 转义字面空格; ~ 为显式 null。 */
function splitRow(raw: string): Array<string | null> {
  const tokens: string[] = [];
  let current = "";
  let escaped = false;
  for (const ch of raw.trimEnd()) {
    if (escaped) {
      current += ch === " " ? " " : ch === "~" ? "~" : ch === "\\" ? "\\" : ch;
      escaped = false;
      continue;
    }
    if (ch === "\\") {
      escaped = true;
      continue;
    }
    if (ch === " ") {
      tokens.push(current);
      current = "";
      continue;
    }
    current += ch;
  }
  tokens.push(current);
  return tokens.map((t) => (t === "~" ? null : t));
}

function escapeToken(value: string | null): string {
  if (value === null) return "~";
  return value
    .replace(/\\/g, "\\\\")
    .replace(/~/g, "\\~")
    .replace(/ /g, "\\ ");
}

/** 解析 .NEMA 源码 (规范子集)。 */
export function parseNema(source: string): NemaDocument {
  const lines = source.split(/\r?\n/);
  const sections: NemaSection[] = [];
  let transport = "";
  let section: NemaSection | null = null;

  for (const line of lines) {
    if (line.trim().length === 0) continue;

    if (line.startsWith("\\!")) {
      transport = line.trim();
      continue;
    }

    const tableMatch = /^\\(\d+)\s+REV\s+(\d+)/.exec(line);
    if (tableMatch && !line.startsWith("\\?") && !line.startsWith("\\>")) {
      if (section) sections.push(section);
      section = {
        mode: "table",
        id: Number(tableMatch[1]),
        revision: Number(tableMatch[2]),
        columns: [],
        rows: [],
      };
      continue;
    }

    const wMatch = /^\\\?(\d+)\s+REV\s+(\d+)/.exec(line);
    if (wMatch) {
      if (section) sections.push(section);
      section = {
        mode: "5w1h",
        id: Number(wMatch[1]),
        revision: Number(wMatch[2]),
        what: "",
        why: "",
        how: "",
        who: "",
        when: "",
        where: "",
      };
      continue;
    }

    const gMatch = /^\\>(\d+)\s+REV\s+(\d+)/.exec(line);
    if (gMatch) {
      if (section) sections.push(section);
      section = { mode: "graph", id: Number(gMatch[1]), revision: Number(gMatch[2]), nodes: [], edges: [] };
      continue;
    }

    if (!section) continue;

    const leading = line.length - line.trimStart().length;
    if (leading > 0 && section.mode === "5w1h") {
      // 前导 N 空格 → 续写第 N 列 (1-indexed)
      const sec = section;
      const append = line.trimStart();
      const colIndex = leading - 1;
      const pick = (current: string) => (current.length > 0 ? `${current}${append}` : append);
      switch (colIndex) {
        case 0: sec.what = pick(sec.what); break;
        case 1: sec.why = pick(sec.why); break;
        case 2: sec.how = pick(sec.how); break;
        case 3: sec.who = pick(sec.who); break;
        case 4: sec.when = pick(sec.when); break;
        case 5: sec.where = pick(sec.where); break;
      }
      continue;
    }

    if (section.mode === "5w1h") {
      const sec = section;
      const tokens = splitRow(line);
      sec.what = tokens[0] ?? "";
      sec.why = tokens[1] ?? "";
      sec.how = tokens[2] ?? "";
      sec.who = tokens[3] ?? "";
      sec.when = tokens[4] ?? "";
      sec.where = tokens[5] ?? "";
      continue;
    }

    if (section.mode === "table") {
      const trimmed = line.trimStart();
      if (trimmed.startsWith("\\#")) {
        // 列名行: \# Name Status Detail (原始行判断, 避免 splitRow 把 \# 拆掉)
        const headerRest = trimmed.slice(2);
        section.columns = splitRow(headerRest).map((t) => (t === null ? "" : t));
        continue;
      }
      const tokens = splitRow(line);
      if (section.columns.length === 0) {
        // 自动列名 columnN
        section.columns = tokens.map((_, i) => `column${i + 1}`);
      }
      section.rows.push(tokens.slice(0, Math.max(section.columns.length, tokens.length)));
      continue;
    }

    if (section.mode === "graph") {
      const tokens = splitRow(line);
      if (!tokens[0]) continue;
      const edgeMatch = /^(\w+)>(\w+)>(\w+)$/.exec(tokens[0]);
      const backMatch = /^(\w+)<(\w+)<(\w+)$/.exec(tokens[0]);
      const undirectedMatch = /^(\w+)=(\w+)=(\w+)$/.exec(tokens[0]);
      if (edgeMatch || backMatch || undirectedMatch) {
        const m = edgeMatch ?? backMatch ?? undirectedMatch!;
        const operator = edgeMatch ? ">" : backMatch ? "<" : "=";
        section.edges.push({
          id: m[2],
          from: m[1],
          to: m[3],
          operator,
          label: tokens[1] ?? "",
          description: tokens[2] ?? "",
        });
      } else {
        section.nodes.push({
          id: tokens[0],
          name: tokens[1] ?? tokens[0],
          description: tokens[2] ?? "",
        });
      }
    }
  }

  if (section) sections.push(section);
  return { transport, sections };
}

/** 序列化 .NEMA 文档 (规范子集)。 */
export function serializeNema(doc: NemaDocument): string {
  const out: string[] = [];
  const header = doc.transport || NEMA_TRANSPORT_HEADER;
  out.push(header);
  for (const s of doc.sections) {
    if (s.mode === "5w1h") {
      out.push(`\\?${s.id} REV ${s.revision}`);
      out.push(
        [s.what, s.why, s.how, s.who, s.when, s.where].map((v) => escapeToken(v || "~")).join(" "),
      );
    } else if (s.mode === "table") {
      out.push(`\\${s.id} REV ${s.revision}`);
      out.push(`\\# ${s.columns.map(escapeToken).join(" ")}`);
      for (const row of s.rows) {
        out.push(row.map(escapeToken).join(" "));
      }
    } else {
      out.push(`\\>${s.id} REV ${s.revision}`);
      for (const n of s.nodes) {
        out.push(`${n.id} ${escapeToken(n.name)} ${escapeToken(n.description)}`);
      }
      for (const e of s.edges) {
        out.push(`${e.from}${e.operator}${e.id}${e.operator}${e.to} ${escapeToken(e.label)} ${escapeToken(e.description)}`);
      }
    }
  }
  return out.join("\n");
}

export interface SharedFileInput {
  /** 共同编辑的文件路径 (e.g. "docs/design/auth.md") */
  filePath: string;
  /** 文件标题 (What) */
  title: string;
  /** 共同编辑者标识 (2+ 触发 .nema 自动生成) */
  editors: string[];
  /** 编辑原因/动机 (Why) */
  purpose?: string;
  /** 产出方式 (How) */
  method?: string;
  /** 编辑发生环境 (Where) */
  environment?: string;
}

/** 为共同编辑的文件生成 .nema 任务文档 (5W1H + 文件元数据 Table)。 */
export function nemaDocumentForSharedFile(input: SharedFileInput, now: Date = new Date()): NemaDocument {
  const five: Nema5W1HSection = {
    mode: "5w1h",
    id: 1,
    revision: 1,
    what: input.title,
    why: input.purpose || "SharedEdit",
    how: input.method || "CollaborativeEdit",
    who: input.editors.join(","),
    when: now.toISOString(),
    where: input.environment || input.filePath,
  };
  const meta: NemaTableSection = {
    mode: "table",
    id: 2,
    revision: 1,
    columns: ["Field", "Value"],
    rows: [
      ["file", input.filePath],
      ["editors", input.editors.join(",")],
      ["editor_count", String(input.editors.length)],
      ["created_at", now.toISOString()],
    ],
  };
  return { transport: NEMA_TRANSPORT_HEADER, sections: [five, meta] };
}

/** 从 .nema 文档提取任务视图字段 (打开为 Star 任务用)。 */
export function taskViewFromDocument(doc: NemaDocument): {
  title: string;
  what: string;
  why: string;
  how: string;
  who: string[];
  when: string;
  where: string;
  filePath: string;
} {
  const five = doc.sections.find((s): s is Nema5W1HSection => s.mode === "5w1h");
  const table = doc.sections.find((s): s is NemaTableSection => s.mode === "table");
  let filePath = "";
  if (table) {
    for (const row of table.rows) {
      if (row[0] === "file" && row[1]) filePath = row[1];
    }
  }
  return {
    title: five?.what || "NEMA Task",
    what: five?.what || "",
    why: five?.why || "",
    how: five?.how || "",
    who: (five?.who ?? "").split(",").filter(Boolean),
    when: five?.when || "",
    where: five?.where || "",
    filePath,
  };
}
