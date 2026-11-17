/*
@cypher schema=1 source_sha256=5f90e6b00014eec735f62fbe0be582f8da7522ffd11e9c2049c086a8a471f3ca
MERGE (self:File {path:"frontend/src/lib/nema/nema.ts"})
MERGE (parse:Symbol {id:"frontend/src/lib/nema/nema.ts::parseNema",kind:"function"})
MERGE (serialize:Symbol {id:"frontend/src/lib/nema/nema.ts::serializeNema",kind:"function"})
MERGE (gen:Symbol {id:"frontend/src/lib/nema/nema.ts::nemaDocumentForSharedFile",kind:"function"})
MERGE (types:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaDocument",kind:"type"})
MERGE (profile:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaInterpreterProfile",kind:"type"})
MERGE (strict_document:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaStrictDocument",kind:"type"})
MERGE (codec_document:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaCodecDocument",kind:"type"})
MERGE (render_result:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaCodecRenderResult",kind:"type"})
MERGE (codec_span:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaCodecTextSpan",kind:"type"})
MERGE (codec_atom:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaCodecAtomChunk",kind:"type"})
MERGE (codec_chunk:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaCodecSemanticChunk",kind:"type"})
MERGE (codec_mode:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaCodecDomainMode",kind:"type"})
MERGE (codec_edge:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaCodecGraphEdge",kind:"type"})
MERGE (codec_record:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaCodecRecord",kind:"type"})
MERGE (codec_domain:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaCodecDomain",kind:"type"})
MERGE (render_domain:Symbol {id:"frontend/src/lib/nema/nema.ts::NemaCodecRenderedDomain",kind:"type"})
MERGE (strict_id:Symbol {id:"frontend/src/lib/nema/nema.ts::strictId",kind:"function"})
MERGE (task_number:Symbol {id:"frontend/src/lib/nema/nema.ts::taskNumber",kind:"function"})
MERGE (strict_cell:Symbol {id:"frontend/src/lib/nema/nema.ts::strictCell",kind:"function"})
MERGE (strict_cell_text:Symbol {id:"frontend/src/lib/nema/nema.ts::strictCellText",kind:"function"})
MERGE (to_strict:Symbol {id:"frontend/src/lib/nema/nema.ts::nemaDocumentToStrictDocument",kind:"function"})
MERGE (from_strict:Symbol {id:"frontend/src/lib/nema/nema.ts::nemaDocumentFromStrictDocument",kind:"function"})
MERGE (task_view:Symbol {id:"frontend/src/lib/nema/nema.ts::taskViewFromDocument",kind:"function"})
MERGE (strict_task_view:Symbol {id:"frontend/src/lib/nema/nema.ts::taskViewFromStrictDocument",kind:"function"})
MERGE (editor_change:Symbol {id:"frontend/src/lib/nema/nema.ts::nemaStrictDocumentWithEditorChange",kind:"function"})
MERGE (trail_from_strict:Symbol {id:"frontend/src/lib/nema/nema.ts::nemaEditTrailFromStrictDocument",kind:"function"})
MERGE (read_trail:Symbol {id:"frontend/src/lib/nema/nema.ts::readNemaEditTrail",kind:"function"})
MERGE (edit_log_id:Symbol {id:"frontend/src/lib/nema/nema.ts::NEMA_EDIT_LOG_DOMAIN_ID",kind:"constant"})
MERGE (trace_type:Type {id:"frontend/src/lib/nema/nema.ts::NemaEditTraceEntry"})
MERGE (change_input:Type {id:"frontend/src/lib/nema/nema.ts::NemaEditorChangeInput"})
MERGE (list_profiles:Symbol {id:"frontend/src/lib/nema/nema.ts::listNemaInterpreterProfilesWasm",kind:"function"})
MERGE (get_profile:Symbol {id:"frontend/src/lib/nema/nema.ts::getNemaInterpreterProfileWasm",kind:"function"})
MERGE (parse_profile:Symbol {id:"frontend/src/lib/nema/nema.ts::parseNemaWithProfileWasm",kind:"function"})
MERGE (format_profile:Symbol {id:"frontend/src/lib/nema/nema.ts::formatNemaWithProfileWasm",kind:"function"})
MERGE (serialize_profile:Symbol {id:"frontend/src/lib/nema/nema.ts::serializeNemaWithProfileWasm",kind:"function"})
MERGE (parse_legacy:Symbol {id:"frontend/src/lib/nema/nema.ts::parseNemaWasm",kind:"function"})
MERGE (render_legacy:Symbol {id:"frontend/src/lib/nema/nema.ts::renderViewWasm",kind:"function"})
MERGE (self)-[:DEFINES]->(parse)
MERGE (self)-[:DEFINES]->(serialize)
MERGE (self)-[:DEFINES]->(gen)
MERGE (self)-[:DEFINES]->(types)
MERGE (self)-[:DEFINES]->(profile)
MERGE (self)-[:DEFINES]->(strict_document)
MERGE (self)-[:DEFINES]->(codec_document)
MERGE (self)-[:DEFINES]->(render_result)
MERGE (self)-[:DEFINES]->(codec_span)
MERGE (self)-[:DEFINES]->(codec_atom)
MERGE (self)-[:DEFINES]->(codec_chunk)
MERGE (self)-[:DEFINES]->(codec_mode)
MERGE (self)-[:DEFINES]->(codec_edge)
MERGE (self)-[:DEFINES]->(codec_record)
MERGE (self)-[:DEFINES]->(codec_domain)
MERGE (self)-[:DEFINES]->(render_domain)
MERGE (self)-[:DEFINES]->(strict_id)
MERGE (self)-[:DEFINES]->(task_number)
MERGE (self)-[:DEFINES]->(strict_cell)
MERGE (self)-[:DEFINES]->(strict_cell_text)
MERGE (self)-[:DEFINES]->(to_strict)
MERGE (self)-[:DEFINES]->(from_strict)
MERGE (self)-[:DEFINES]->(task_view)
MERGE (self)-[:DEFINES]->(strict_task_view)
MERGE (self)-[:DEFINES]->(editor_change)
MERGE (self)-[:DEFINES]->(trail_from_strict)
MERGE (self)-[:DEFINES]->(read_trail)
MERGE (self)-[:DEFINES]->(edit_log_id)
MERGE (self)-[:DEFINES]->(trace_type)
MERGE (self)-[:DEFINES]->(change_input)
MERGE (self)-[:DEFINES]->(list_profiles)
MERGE (editor_change)-[:USES_TYPE]->(strict_document)
MERGE (trail_from_strict)-[:USES_TYPE]->(strict_document)
MERGE (read_trail)-[:CALLS]->(parse_profile)
MERGE (read_trail)-[:CALLS]->(trail_from_strict)
MERGE (read_trail)-[:USES_TYPE]->(profile)
MERGE (self)-[:DEFINES]->(get_profile)
MERGE (self)-[:DEFINES]->(parse_profile)
MERGE (self)-[:DEFINES]->(format_profile)
MERGE (self)-[:DEFINES]->(serialize_profile)
MERGE (self)-[:DEFINES]->(parse_legacy)
MERGE (self)-[:DEFINES]->(render_legacy)
MERGE (list_profiles)-[:CALLS]->(parse_profile)
MERGE (get_profile)-[:CALLS]->(list_profiles)
MERGE (to_strict)-[:USES_TYPE]->(strict_document)
MERGE (to_strict)-[:CALLS]->(strict_id)
MERGE (to_strict)-[:CALLS]->(strict_cell)
MERGE (from_strict)-[:USES_TYPE]->(strict_document)
MERGE (from_strict)-[:CALLS]->(task_number)
MERGE (from_strict)-[:CALLS]->(strict_cell_text)
MERGE (task_view)-[:USES_TYPE]->(types)
MERGE (strict_task_view)-[:USES_TYPE]->(strict_document)
MERGE (strict_task_view)-[:CALLS]->(strict_cell_text)
MERGE (serialize_profile)-[:CALLS]->(to_strict)
MERGE (serialize_profile)-[:CALLS]->(format_profile)
MERGE (parse_legacy)-[:USES_TYPE]->(codec_document)
MERGE (render_legacy)-[:USES_TYPE]->(render_result)
@endcypher
*/
// =====================================================================
// NEMA UI task projection and legacy format compatibility helpers.
// New profile-aware source parsing/formatting is owned by the Rust codec and
// selected by the exact interpreter profile persisted with each new entry.
// =====================================================================

export const NEMA_TRANSPORT_HEADER = "\\!NEMAT1";
/** Explicit default for newly created documents; old files retain their pinned revision. */
export const NEMA_DEFAULT_NEW_DOCUMENT_REVISION = "matrix-2026-11-15";

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

/** Immutable identity for one Rust NEMA interpreter and grammar snapshot. */
export interface NemaInterpreterProfile {
  format_id: "nema";
  plugin_id: string;
  grammar_revision: string;
  plugin_version: string;
  abi_version: number;
  ast_schema_version: number;
  grammar_source_sha256: string;
  artifact_sha256: string;
}

export type NemaStrictCell =
  | { kind: "value"; value: string }
  | { kind: "null" }
  | { kind: "empty" }
  | { kind: "missing" };

export type NemaStrictMode = "table" | "five_w_one_h" | "graph";

export interface NemaStrictDomain {
  mode: NemaStrictMode;
  /** Decimal strings preserve Rust u64 identities through JavaScript JSON. */
  id: string;
  revision: string;
  columns: string[] | null;
  explicit_header: boolean;
  declared_width: number | null;
  records: NemaStrictCell[][];
}

/** Rust's lossless AST. The strict profile APIs keep all four cell states. */
export interface NemaStrictDocument {
  domains: NemaStrictDomain[];
}

function strictId(value: number): string {
  if (!Number.isSafeInteger(value) || value < 1) {
    throw new Error("NEMA domain IDs and revisions must be positive safe integers in the task view");
  }
  return String(value);
}

function taskNumber(value: string): number {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed < 1 || String(parsed) !== value) {
    throw new Error(`NEMA identifier ${value} cannot be represented by the task view`);
  }
  return parsed;
}

function strictCell(value: string | null | undefined): NemaStrictCell {
  if (value === null) return { kind: "null" };
  if (value === undefined) return { kind: "missing" };
  if (value.length === 0) return { kind: "empty" };
  return { kind: "value", value };
}

function strictCellText(cell: NemaStrictCell): string | null | undefined {
  switch (cell.kind) {
    case "value": return cell.value;
    case "null": return null;
    case "empty": return "";
    case "missing": return undefined;
  }
}

/** Converts the task view model to the Rust-owned, lossless profile AST. */
export function nemaDocumentToStrictDocument(document: NemaDocument): NemaStrictDocument {
  return {
    domains: document.sections.map((section) => {
      const identity = { id: strictId(section.id), revision: strictId(section.revision) };
      if (section.mode === "5w1h") {
        return {
          ...identity,
          mode: "five_w_one_h",
          columns: null,
          explicit_header: false,
          declared_width: null,
          records: [[section.what, section.why, section.how, section.who, section.when, section.where].map(strictCell)],
        };
      }
      if (section.mode === "table") {
        return {
          ...identity,
          mode: "table",
          columns: [...section.columns],
          explicit_header: true,
          declared_width: null,
          records: section.rows.map((row) =>
            section.columns.map((_, index) => strictCell(index < row.length ? row[index] : undefined)),
          ),
        };
      }

      const records: NemaStrictCell[][] = section.nodes.map((node) => [
        strictCell(node.id), strictCell(node.name), strictCell(node.description),
      ]);
      for (const edge of section.edges) {
        const token = `${edge.from}${edge.operator}${edge.id}${edge.operator}${edge.to}`;
        records.push([strictCell(token), strictCell(edge.label), strictCell(edge.description)]);
      }
      return {
        ...identity,
        mode: "graph",
        columns: null,
        explicit_header: false,
        declared_width: null,
        records,
      };
    }),
  };
}

/** Builds a lossy task-view projection; the original profile AST remains authoritative. */
export function nemaDocumentFromStrictDocument(document: NemaStrictDocument): NemaDocument {
  const sections: NemaSection[] = [];
  for (const domain of document.domains) {
    const id = taskNumber(domain.id);
    const revision = taskNumber(domain.revision);
    if (domain.mode === "five_w_one_h") {
      const row = domain.records[0] ?? [];
      const field = (index: number) => strictCellText(row[index] ?? { kind: "missing" }) ?? "";
      sections.push({
        mode: "5w1h", id, revision,
        what: field(0), why: field(1), how: field(2), who: field(3), when: field(4), where: field(5),
      });
      continue;
    }
    if (domain.mode === "table") {
      const columns = domain.columns ?? [];
      const rows = domain.records.map((row) =>
        columns.map((_, index) => strictCellText(row[index] ?? { kind: "missing" }) ?? null),
      );
      sections.push({ mode: "table", id, revision, columns, rows });
      continue;
    }

    const nodes: NemaGraphSection["nodes"] = [];
    const edges: NemaGraphSection["edges"] = [];
    for (const row of domain.records) {
      const token = strictCellText(row[0] ?? { kind: "missing" });
      if (!token) throw new Error("NEMA graph records require a nonempty graph token");
      const edge = /^([A-Za-z][A-Za-z0-9_.:-]*)([><=])([A-Za-z][A-Za-z0-9_.:-]*)([><=])([A-Za-z][A-Za-z0-9_.:-]*)$/.exec(token);
      if (edge && edge[2] === edge[4]) {
        edges.push({
          from: edge[1], id: edge[3], to: edge[5], operator: edge[2] as ">" | "<" | "=",
          label: strictCellText(row[1] ?? { kind: "missing" }) ?? "",
          description: strictCellText(row[2] ?? { kind: "missing" }) ?? "",
        });
      } else {
        nodes.push({
          id: token,
          name: strictCellText(row[1] ?? { kind: "missing" }) ?? token,
          description: strictCellText(row[2] ?? { kind: "missing" }) ?? "",
        });
      }
    }
    sections.push({ mode: "graph", id, revision, nodes, edges });
  }
  return { transport: NEMA_TRANSPORT_HEADER, sections };
}

export interface NemaCodecTextSpan {
  start_byte: number;
  end_byte: number;
  line: number;
  column: number;
}

export interface NemaCodecAtomChunk {
  text: string;
  opaque_dict_index: number | null;
}

export interface NemaCodecSemanticChunk {
  raw: string;
  span: NemaCodecTextSpan;
  atoms: NemaCodecAtomChunk[];
}

export type NemaCodecDomainMode = "table" | "5w1h" | "graph";

export interface NemaCodecGraphEdge {
  id: string;
  from: string;
  to: string;
  direction: "directed" | "undirected";
  raw_operator: string;
}

export interface NemaCodecRecord {
  span: NemaCodecTextSpan;
  cells: NemaCodecSemanticChunk[][];
  explicit_tail_empty: boolean;
  graph_node_id: string | null;
  graph_edge: NemaCodecGraphEdge | null;
}

export interface NemaCodecDomain {
  id: string;
  revision: number;
  mode: NemaCodecDomainMode;
  header_span: NemaCodecTextSpan;
  explicit_header: boolean;
  explicit_header_span: NemaCodecTextSpan | null;
  columns: string[];
  records: NemaCodecRecord[];
  is_v12: boolean;
}

/** Legacy codec AST returned by `parseNema`; use the profile API for strict grammar ASTs. */
export interface NemaCodecDocument {
  span: NemaCodecTextSpan;
  dictionary: string[];
  dictionary_spans: NemaCodecTextSpan[];
  domains: NemaCodecDomain[];
}

export type NemaCodecRenderedDomain =
  | {
      id: string;
      revision: number;
      mode: "5w1h";
      fields: Record<"What" | "Why" | "How" | "Who" | "When" | "Where", string>;
    }
  | {
      id: string;
      revision: number;
      mode: "table";
      columns: string[];
      records: Array<Record<string, string>>;
    }
  | {
      id: string;
      revision: number;
      mode: "graph";
      nodes: Array<{ id: string; name: string; description: string; extensions?: Record<string, string> }>;
      edges: Array<{
        id: string;
        source: string;
        target: string;
        direction: string;
        name: string;
        description: string;
        extensions?: Record<string, string>;
      }>;
    };

export interface NemaCodecRenderResult {
  view: NemaCodecRenderedDomain[];
  revision: number;
  hash: string;
  explicit_header: boolean;
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

/** Legacy TypeScript parser for persisted pre-profile task entries. */
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

/** Legacy TypeScript serializer retained for old stored entries and their compatibility tests. */
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

export interface NemaTaskView {
  title: string;
  what: string;
  why: string;
  how: string;
  who: string[];
  when: string;
  where: string;
  filePath: string;
}

/** 从 legacy/task view 文档提取字段。 */
export function taskViewFromDocument(doc: NemaDocument): NemaTaskView {
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

/** Extracts task fields directly from the Rust AST without narrowing u64 IDs to JS numbers. */
export function taskViewFromStrictDocument(doc: NemaStrictDocument): NemaTaskView {
  const five = doc.domains.find((domain) => domain.mode === "five_w_one_h");
  const row = five?.records[0] ?? [];
  const text = (cell: NemaStrictCell | undefined): string =>
    cell?.kind === "value" ? cell.value : "";
  const table = doc.domains.find((domain) => domain.mode === "table");
  const fileColumn = table?.columns?.indexOf("Field") ?? -1;
  const valueColumn = table?.columns?.indexOf("Value") ?? -1;
  let filePath = "";
  if (table && fileColumn >= 0 && valueColumn >= 0) {
    for (const record of table.records) {
      if (text(record[fileColumn]) === "file") filePath = text(record[valueColumn]);
    }
  }
  const what = text(row[0]);
  return {
    title: what || "NEMA Task",
    what,
    why: text(row[1]),
    how: text(row[2]),
    who: text(row[3]).split(",").filter(Boolean),
    when: text(row[4]),
    where: text(row[5]),
    filePath,
  };
}


// =====================================================================
// 多人共同编辑痕迹 (per 2026-11-17 用户发令: 文件即任务式多 agent 共同编辑)
//   - .nema 侧车用 strict v2 合法结构记录"谁在什么时候编辑了什么":
//     独立 Field/Value 表域 (id = NEMA_EDIT_LOG_DOMAIN_ID), 每次编辑者
//     变更追加 editor / edited_at / change 三行。
//   - 改写全部发生在严格 AST (NemaStrictDocument) 上, 不经过 legacy
//     NemaDocument 投影 (那是 lossy 的, 会丢 Cell 四态)。Rust format
//     函数会做 canonical round-trip 校验, 非法输出直接抛错。
// =====================================================================

/** 严格 AST 里标准 5W1H 任务头的域 id (decimal string, 与 Rust u64 一致)。 */
export const NEMA_FIVE_W_ONE_H_DOMAIN_ID = "1";
/** 严格 AST 里标准 Field/Value 元数据表的域 id。 */
export const NEMA_META_TABLE_DOMAIN_ID = "2";
/** 严格 AST 里多人编辑痕迹表的域 id。 */
export const NEMA_EDIT_LOG_DOMAIN_ID = "3";
export const NEMA_FIELD_COLUMN = "Field";
export const NEMA_VALUE_COLUMN = "Value";

/** 一条共同编辑痕迹: 谁 (editor) 在什么时候 (edited_at) 改了什么 (change)。 */
export interface NemaEditTraceEntry {
  editor: string;
  edited_at: string;
  change: string;
}

export interface NemaEditorChangeInput {
  editors: string[];
  trace: NemaEditTraceEntry;
  /** ISO-8601, 写入元数据表 updated_at 行。 */
  at: string;
}

/** 空串必须落 Empty 态 — Rust emit 会拒绝 Value("") ("use Empty for empty value")。 */
function strictValueCell(value: string): NemaStrictCell {
  return value.length === 0 ? { kind: "empty" } : { kind: "value", value };
}

function isFieldValueTable(domain: NemaStrictDomain): boolean {
  const columns = domain.columns;
  return (
    columns != null &&
    columns.length === 2 &&
    columns[0] === NEMA_FIELD_COLUMN &&
    columns[1] === NEMA_VALUE_COLUMN
  );
}

/**
 * 在严格 AST 上直接改写共同编辑者并追加一条编辑痕迹:
 *   - 5W1H 域: who 单元格重写为新编辑者列表;
 *   - Field/Value 元数据表: editors / editor_count 行重写, updated_at 行补写;
 *   - 编辑痕迹表 (id=3): 不存在则创建, 追加 editor / edited_at / change 三行。
 * 输出必须经 formatNemaWithProfileWasm 走 Rust canonical round-trip 校验。
 */
export function nemaStrictDocumentWithEditorChange(
  document: NemaStrictDocument,
  change: NemaEditorChangeInput,
): NemaStrictDocument {
  const cloneCell = (cell: NemaStrictCell): NemaStrictCell =>
    cell.kind === "value" ? { kind: "value", value: cell.value } : { ...cell };
  const domains = document.domains.map((domain) => ({
    ...domain,
    records: domain.records.map((record) => record.map(cloneCell)),
  }));

  const five = domains.find((domain) => domain.mode === "five_w_one_h");
  if (!five) throw new Error("NEMA 文档缺少 5W1H 域, 无法改写共同编辑者");
  const headerRow = five.records[0];
  if (!headerRow) throw new Error("NEMA 5W1H 域缺少记录");
  while (headerRow.length < 6) headerRow.push({ kind: "missing" });
  headerRow[3] = strictValueCell(change.editors.join(","));

  const meta = domains.find(
    (domain) =>
      domain.mode === "table" &&
      isFieldValueTable(domain) &&
      domain.id !== NEMA_EDIT_LOG_DOMAIN_ID,
  );
  if (!meta) throw new Error("NEMA 文档缺少 Field/Value 元数据表, 无法安全改写编辑者");
  const cellText = (cell: NemaStrictCell | undefined): string =>
    cell?.kind === "value" ? cell.value : "";
  let editorsRowFound = false;
  let countRowFound = false;
  let updatedAtRowFound = false;
  const updated = meta.records.map((record) => {
    const field = cellText(record[0]);
    if (field === "editors") {
      editorsRowFound = true;
      record[1] = strictValueCell(change.editors.join(","));
    } else if (field === "editor_count") {
      countRowFound = true;
      record[1] = strictValueCell(String(change.editors.length));
    } else if (field === "updated_at") {
      updatedAtRowFound = true;
      record[1] = strictValueCell(change.at);
    }
    return record;
  });
  if (!editorsRowFound) {
    updated.push([strictValueCell("editors"), strictValueCell(change.editors.join(","))]);
  }
  if (!countRowFound) {
    updated.push([
      strictValueCell("editor_count"),
      strictValueCell(String(change.editors.length)),
    ]);
  }
  if (!updatedAtRowFound) {
    updated.push([strictValueCell("updated_at"), strictValueCell(change.at)]);
  }
  meta.records = updated;

  let log = domains.find((domain) => domain.id === NEMA_EDIT_LOG_DOMAIN_ID);
  if (log) {
    if (log.mode !== "table" || !isFieldValueTable(log)) {
      throw new Error(
        `NEMA 域 id=${NEMA_EDIT_LOG_DOMAIN_ID} 已被非 Field/Value 表占用, 无法追加编辑痕迹`,
      );
    }
  } else {
    log = {
      mode: "table",
      id: NEMA_EDIT_LOG_DOMAIN_ID,
      revision: "1",
      columns: [NEMA_FIELD_COLUMN, NEMA_VALUE_COLUMN],
      explicit_header: true,
      declared_width: null,
      records: [],
    };
    domains.push(log);
  }
  log.records.push(
    [strictValueCell("editor"), strictValueCell(change.trace.editor)],
    [strictValueCell("edited_at"), strictValueCell(change.trace.edited_at)],
    [strictValueCell("change"), strictValueCell(change.trace.change)],
  );
  return { domains };
}

/** 从严格 AST 读取编辑痕迹 (痕迹表内 editor / edited_at / change 三行一组)。 */
export function nemaEditTrailFromStrictDocument(
  document: NemaStrictDocument,
): NemaEditTraceEntry[] {
  const log = document.domains.find((domain) => domain.id === NEMA_EDIT_LOG_DOMAIN_ID);
  if (!log || log.mode !== "table" || !isFieldValueTable(log)) return [];
  const text = (cell: NemaStrictCell | undefined): string =>
    cell?.kind === "value" ? cell.value : "";
  const entries: NemaEditTraceEntry[] = [];
  let pending: { editor?: string; edited_at?: string } = {};
  for (const record of log.records) {
    const field = text(record[0]);
    const value = text(record[1]);
    if (field === "editor") {
      pending = { editor: value || undefined };
    } else if (field === "edited_at" && pending.editor) {
      pending = { ...pending, edited_at: value || undefined };
    } else if (field === "change" && pending.editor && pending.edited_at) {
      entries.push({ editor: pending.editor, edited_at: pending.edited_at, change: value });
      pending = {};
    }
  }
  return entries;
}

/**
 * 用 pin 的严格 profile 解析 .nema 源并读取编辑痕迹。
 * 无 profile (legacy 条目) 或解析失败 → null (不伪造痕迹)。
 */
export async function readNemaEditTrail(
  source: string,
  profile: NemaInterpreterProfile | null | undefined,
): Promise<NemaEditTraceEntry[] | null> {
  if (!profile) return null;
  try {
    const document = await parseNemaWithProfileWasm(source, profile);
    return nemaEditTrailFromStrictDocument(document);
  } catch {
    return null;
  }
}

// =====================================================================
// WASM 渐进增强通道 (per 2026-11-14: nema-codec-wasm 前端对接)
//   wasm 加载成功 → 解析走 Rust 实现; 失败/不可用 → 回退上方 JS 实现
// =====================================================================

let wasmReady: Promise<typeof import("./wasm/nema_codec_wasm")> | null = null;

/** 预加载 wasm codec (幂等; 失败后不再重试, 静默回退 JS) */
export function loadNemaWasm(): Promise<boolean> {
  if (wasmReady === null) {
    wasmReady = import("./wasm/nema_codec_wasm")
      .then(async (mod) => {
        const wasmUrl = new URL("./wasm/nema_codec_wasm_bg.wasm", import.meta.url);
        const res = await fetch(wasmUrl.toString());
        if (!res.ok) throw new Error(`wasm fetch ${res.status}`);
        mod.initSync({ module: await res.arrayBuffer() });
        return mod;
      })
      .catch((e) => {
        console.warn("[nema] wasm codec unavailable, JS fallback:", e);
        wasmReady = null;
        throw e;
      });
  }
  return wasmReady.then(() => true).catch(() => false);
}

/** Parses through the legacy Rust codec and returns its actual span-preserving AST shape. */
export async function parseNemaWasm(source: string): Promise<NemaCodecDocument> {
  const mod = await wasmReady;
  if (!mod) throw new Error("wasm not loaded");
  return JSON.parse(mod.parseNema(source)) as NemaCodecDocument;
}

/** Returns the legacy Rust renderer's structured view and metadata. */
export async function renderViewWasm(source: string): Promise<NemaCodecRenderResult> {
  const mod = await wasmReady;
  if (!mod) throw new Error("wasm not loaded");
  return JSON.parse(mod.renderView(source)) as NemaCodecRenderResult;
}

/** Lists Rust interpreter profiles. The caller must pin one; no latest lookup exists. */
export async function listNemaInterpreterProfilesWasm(): Promise<NemaInterpreterProfile[]> {
  const mod = await wasmReady;
  if (!mod) throw new Error("wasm not loaded");
  return JSON.parse(mod.listNemaInterpreterProfiles()) as NemaInterpreterProfile[];
}

/** Selects a pinned revision by exact identity; it never resolves a moving `latest` alias. */
export async function getNemaInterpreterProfileWasm(
  grammarRevision: string,
): Promise<NemaInterpreterProfile> {
  const profiles = await listNemaInterpreterProfilesWasm();
  const profile = profiles.find((candidate) => candidate.grammar_revision === grammarRevision);
  if (!profile) throw new Error(`NEMA interpreter profile is not installed: ${grammarRevision}`);
  return profile;
}

/** Parses with the exact Rust interpreter profile selected by the caller. */
export async function parseNemaWithProfileWasm(
  source: string,
  profile: NemaInterpreterProfile,
): Promise<NemaStrictDocument> {
  const mod = await wasmReady;
  if (!mod) throw new Error("wasm not loaded");
  return JSON.parse(
    mod.parseNemaWithProfile(JSON.stringify(profile), source),
  ) as NemaStrictDocument;
}

/** Formats through Rust and rejects output that fails same-profile round-trip validation. */
export async function formatNemaWithProfileWasm(
  document: NemaStrictDocument,
  profile: NemaInterpreterProfile,
): Promise<string> {
  const mod = await wasmReady;
  if (!mod) throw new Error("wasm not loaded");
  return mod.formatNemaWithProfile(JSON.stringify(profile), JSON.stringify(document));
}

/** Formats the UI task projection through the selected native Rust interpreter profile. */
export async function serializeNemaWithProfileWasm(
  document: NemaDocument,
  profile: NemaInterpreterProfile,
): Promise<string> {
  return formatNemaWithProfileWasm(nemaDocumentToStrictDocument(document), profile);
}
