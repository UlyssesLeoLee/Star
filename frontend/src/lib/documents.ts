/*
@cypher schema=1 source_sha256=2026-11-11-workitem-documents-v1
MERGE (self:File {path:"frontend/src/lib/documents.ts"})
MERGE (resolve:Symbol {id:"documents::resolveDocumentHref",kind:"function"})
MERGE (open:Symbol {id:"documents::openWorkItemDocument",kind:"function"})
MERGE (badge:Symbol {id:"documents::documentKindLabel",kind:"function"})
MERGE (self)-[:DEFINES]->(resolve)
MERGE (self)-[:DEFINES]->(open)
MERGE (self)-[:DEFINES]->(badge)
@endcypher
*/
// 任务卡关联文档打开逻辑 (per 2026-11-11 用户发令: 一卡多文档, 点击直接打开)
// 仓库内路径 → GitHub dev blob; http(s) → 直接打开。

import type { WorkItemDocument } from "@/types/ids";

const GITHUB_REPO_BLOB = "https://github.com/UlyssesLeoLee/Star/blob/dev";

/** 计算文档点击目标 href。 */
export function resolveDocumentHref(doc: Pick<WorkItemDocument, "path">): string {
  const path = doc.path.trim();
  if (/^https?:\/\//i.test(path)) return path;
  return `${GITHUB_REPO_BLOB}/${path.replace(/^\/+/, "")}`;
}

/** 点击直接打开 (新标签页)。 */
export function openWorkItemDocument(doc: Pick<WorkItemDocument, "path">): void {
  const href = resolveDocumentHref(doc);
  if (typeof window !== "undefined") {
    window.open(href, "_blank", "noopener,noreferrer");
  }
}

export const DOCUMENT_KIND_META: Record<WorkItemDocument["kind"], { label: string; tone: string }> = {
  document: { label: "文档", tone: "text-cyan-300 border-cyan-400/40" },
  review: { label: "审查", tone: "text-amber-300 border-amber-400/40" },
  approval: { label: "批准", tone: "text-lime-300 border-lime-400/40" },
  evidence: { label: "证据", tone: "text-emerald-300 border-emerald-400/40" },
  link: { label: "链接", tone: "text-blue-300 border-blue-400/40" },
};
