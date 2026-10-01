// =====================================================================
// search/globalSearch.ts — CommandBar 统一搜索 (per 2026-10-01 OOB 指令)
//   - 参考微信搜索: 顶部 search input + 下方 type filter chips
//   - 不止 modules, 搜 任务/work_items + worktree + canvas + agent + feedback + project
//   - 子串匹配 (中英文不限), label > subLabel score 权重
//   - 高亮命中片段 (mark 字段)
// =====================================================================

import {
  ALL_MODULES,
  type ModuleDefinition,
} from "@/lib/nav/registry";
import {
  workItems,
  worktrees,
  agentSessions,
  canvases,
  feedbacks,
  projects,
  changeSets,
  notifications,
  pullRequests,
  sprints,
  milestones,
  comments,
} from "@/lib/seed";

// ---------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------
export type SearchType =
  | "module"
  | "task"
  | "worktree"
  | "canvas"
  | "agent"
  | "feedback"
  | "project"
  | "sprint"
  | "milestone"
  | "changeSet"
  | "pullRequest"
  | "notification"
  | "comment"
  | "doc";

export interface SearchHit {
  id: string;
  type: SearchType;
  /** 主标题 */
  label: string;
  /** 副标题/上下文 */
  subLabel?: string;
  /** 跳转目标 */
  href: string;
  /** 状态徽章颜色 */
  tone?: "info" | "ok" | "warn" | "err" | "default";
  /** 匹配得分 (高→低) */
  score: number;
  /** 命中片段高亮 (用于 UI 加 <mark>) */
  match?: { field: "label" | "subLabel" | "code" | "tag"; value: string };
}

export interface SearchResult {
  query: string;
  total: number;
  groups: Array<{ type: SearchType; hits: SearchHit[] }>;
}

// ---------------------------------------------------------------------
// Score
// ---------------------------------------------------------------------
const TONE_BY_STATUS: Record<string, SearchHit["tone"]> = {
  in_progress: "info",
  review: "warn",
  done: "ok",
  blocked: "err",
  failed: "err",
  cancelled: "default",
  completed: "ok",
  merged: "ok",
  draft: "default",
  open: "info",
  resolved: "ok",
};

function toneFor(status?: string): SearchHit["tone"] {
  if (!status) return undefined;
  return TONE_BY_STATUS[status];
}

function scoreHit(q: string, fields: Array<{ value: string; weight: number; tag?: string }>): number {
  let total = 0;
  let matchedField: SearchHit["match"];
  for (const f of fields) {
    if (!f.value) continue;
    const idx = f.value.toLowerCase().indexOf(q);
    if (idx < 0) continue;
    // label 越靠前, score 越高
    const positionScore = idx === 0 ? 3 : idx < 5 ? 2 : 1;
    total += f.weight * positionScore;
    if (!matchedField || f.weight > matchedFieldWeight(matchedField)) {
      matchedField = {
        field: f.tag === "doc" ? "subLabel" : f.tag === "tag" ? "tag" : f.tag === "code" ? "code" : "label",
        value: f.value,
      };
    }
  }
  return total;
}

function matchedFieldWeight(m: NonNullable<SearchHit["match"]>): number {
  switch (m.field) {
    case "label": return 10;
    case "code": return 8;
    case "tag": return 3;
    case "subLabel": return 5;
  }
}

// ---------------------------------------------------------------------
// Index (built lazily on first call)
// ---------------------------------------------------------------------
export interface IndexEntry {
  hit: SearchHit;
  haystack: string;
}

let _index: IndexEntry[] | null = null;

function buildIndex(): IndexEntry[] {
  const out: IndexEntry[] = [];

  // 模块 (per nav/registry)
  for (const m of ALL_MODULES) {
    const label = m.label;
    const code = m.code;
    const desc = m.description ?? m.categoryLabel ?? "";
    const cat = m.categoryLabel ?? "";
    const hit: SearchHit = {
      id: `module:${m.id}`,
      type: "module",
      label,
      subLabel: desc,
      href: m.href,
      tone: m.isCore ? "info" : "default",
      score: 0,
    };
    out.push({
      hit,
      haystack: [label, code, cat, desc].join(" \n ").toLowerCase(),
    });
  }

  // 项目 (per seed.projects)
  for (const p of projects) {
    const hit: SearchHit = {
      id: `project:${p.id}`,
      type: "project",
      label: p.name,
      subLabel: `${p.key} · ${p.member_count} members`,
      href: `/projects?project_id=${encodeURIComponent(p.id)}`,
      tone: "info",
      score: 0,
    };
    out.push({
      hit,
      haystack: `${p.name} ${p.key}`.toLowerCase(),
    });
  }

  // 任务 / WorkItem (per seed.workItems)
  for (const w of workItems) {
    const key = w.key;
    const title = w.title;
    const desc = w.description ?? "";
    const kind = w.kind;
    const status = w.status;
    const priority = w.priority;
    const labels = (w.labels ?? []).join(" ");
    const hit: SearchHit = {
      id: `task:${w.id}`,
      type: "task",
      label: `${key} ${title}`,
      subLabel: `${kind} · ${priority} · ${status}${labels ? ` · ${labels}` : ""}`,
      href: `/sprint?app=list&selected=${encodeURIComponent(w.id)}`,
      tone: toneFor(status),
      score: 0,
    };
    out.push({
      hit,
      haystack: `${key}\n${title}\n${desc}\n${labels}`.toLowerCase(),
    });
  }

  // Worktree
  for (const t of worktrees) {
    const hit: SearchHit = {
      id: `worktree:${t.id}`,
      type: "worktree",
      label: t.name,
      subLabel: `${t.branch} · ${t.status}`,
      href: `/worktree/${encodeURIComponent(t.id)}`,
      tone: toneFor(t.status),
      score: 0,
    };
    out.push({
      hit,
      haystack: `${t.id} ${t.name} ${t.branch}`.toLowerCase(),
    });
  }

  // Canvas
  for (const c of canvases) {
    const frameTitles = (c.frames ?? []).map((f) => f.title).join(" / ");
    const hit: SearchHit = {
      id: `canvas:${c.id}`,
      type: "canvas",
      label: c.title,
      subLabel: `${frameTitles || "(no frames)"}`,
      href: `/canvas/${encodeURIComponent(c.id)}`,
      tone: "info",
      score: 0,
    };
    out.push({
      hit,
      haystack: `${c.id} ${c.title}\n${frameTitles}`.toLowerCase(),
    });
  }

  // Agent sessions
  for (const a of agentSessions) {
    const hit: SearchHit = {
      id: `agent:${a.id}`,
      type: "agent",
      label: a.name,
      subLabel: `${a.agent_kind} · ${a.status}`,
      href: `/agent-view?session=${encodeURIComponent(a.id)}`,
      tone: toneFor(a.status),
      score: 0,
    };
    out.push({
      hit,
      haystack: `${a.name} ${a.agent_kind}`.toLowerCase(),
    });
  }

  // Feedback
  for (const fb of feedbacks) {
    const hit: SearchHit = {
      id: `feedback:${fb.id}`,
      type: "feedback",
      label: fb.question,
      subLabel: `${fb.category} · ${fb.severity} · ${fb.status}`,
      href: `/inbox?type=feedback&selected=${encodeURIComponent(fb.id)}`,
      tone: toneFor(fb.status),
      score: 0,
    };
    out.push({
      hit,
      haystack: `${fb.question}\n${fb.answer ?? ""}`.toLowerCase(),
    });
  }

  // Sprints
  for (const s of sprints) {
    const hit: SearchHit = {
      id: `sprint:${s.id}`,
      type: "sprint",
      label: s.name,
      subLabel: `goal: ${s.goal ?? ""}`.trim(),
      href: `/sprint?view=sprint&sprint=${encodeURIComponent(s.id)}`,
      tone: "info",
      score: 0,
    };
    out.push({
      hit,
      haystack: `${s.name}\n${s.goal ?? ""}`.toLowerCase(),
    });
  }

  // Milestones
  for (const m of milestones) {
    const hit: SearchHit = {
      id: `milestone:${m.id}`,
      type: "milestone",
      label: m.name,
      subLabel: `${m.progress.toFixed(0)}% · ${m.due_date ?? ""}`.trim(),
      href: `/projects?tab=timeline&milestone=${encodeURIComponent(m.id)}`,
      score: 0,
    };
    out.push({
      hit,
      haystack: `${m.name}`.toLowerCase(),
    });
  }

  // ChangeSets
  for (const cs of changeSets) {
    const hit: SearchHit = {
      id: `changeSet:${cs.id}`,
      type: "changeSet",
      label: cs.title,
      subLabel: `${cs.diff_summary} · ${cs.status}`,
      href: `/sprint?app=cs&cs=${encodeURIComponent(cs.id)}`,
      tone: toneFor(cs.status),
      score: 0,
    };
    out.push({
      hit,
      haystack: `${cs.title}\n${cs.diff_summary}`.toLowerCase(),
    });
  }

  // Pull requests
  for (const pr of pullRequests) {
    const hit: SearchHit = {
      id: `pullRequest:${pr.id}`,
      type: "pullRequest",
      label: `${pr.title} (#${pr.number})`,
      subLabel: `${pr.source_branch} → ${pr.target_branch} · ${pr.status}`,
      href: `/scm?pr=${encodeURIComponent(pr.id)}`,
      tone: toneFor(pr.status),
      score: 0,
    };
    out.push({
      hit,
      haystack: `${pr.title}\n${pr.source_branch}`.toLowerCase(),
    });
  }

  // Notifications
for (const n of notifications) {
  const hit: SearchHit = {
    id: `notification:${n.id}`,
    type: "notification",
    label: n.subject,
    subLabel: n.body.slice(0, 80),
    href: `/inbox?selected=${encodeURIComponent(n.id)}`,
    tone: toneFor(n.status),
    score: 0,
  };
  out.push({
    hit,
    haystack: `${n.subject}\n${n.body}`.toLowerCase(),
  });
}

  // Comments
  for (const c of comments) {
    const hit: SearchHit = {
      id: `comment:${c.id}`,
      type: "comment",
      label: c.body.slice(0, 60),
      subLabel: `comment · ${c.target_kind}/${c.target_id}`,
      href: `/sprint?app=list&target=${encodeURIComponent(c.target_id)}`,
      tone: "default",
      score: 0,
    };
    out.push({
      hit,
      haystack: c.body.toLowerCase(),
    });
  }

  // Docs (frontend/docs/*.md - 列出 V模型 / 设计文档等)
  // 简易 hardcoded 索引 (per 现有 docs structure)
  const docs: Array<{ id: string; label: string; href: string }> = [
    { id: "doc:vmodel", label: "V字モデル Kanban テンプレート", href: "/deliverables/kanban-vmodel-jp/" },
    { id: "doc:agents", label: "AGENTS.md", href: "/AGENTS.md" },
    { id: "doc:automation", label: "automation-design.md", href: "/docs/automation-design.md" },
    { id: "doc:basic-design", label: "basic-design.md", href: "/docs/basic-design.md" },
    { id: "doc:requirements", label: "requirements.md", href: "/docs/requirements.md" },
  ];
  for (const d of docs) {
    const hit: SearchHit = {
      id: d.id,
      type: "doc",
      label: d.label,
      href: d.href,
      tone: "default",
      score: 0,
    };
    out.push({
      hit,
      haystack: d.label.toLowerCase(),
    });
  }

  return out;
}

export function getSearchIndex(): IndexEntry[] {
  if (_index) return _index;
  _index = buildIndex();
  return _index;
}

// ---------------------------------------------------------------------
// Public search API
// ---------------------------------------------------------------------
export interface SearchOptions {
  /** "all" = 全部, 否则限定到指定 type */
  filter?: SearchType | "all";
  /** 上限 (默认 50) */
  limit?: number;
  /** 按类型分组 (默认 true, 微信式) */
  group?: boolean;
}

export function searchAll(query: string, options: SearchOptions = {}): SearchResult {
  const q = (query ?? "").trim().toLowerCase();
  const filter = options.filter ?? "all";
  const limit = options.limit ?? 50;
  const wantGroup = options.group ?? true;
  if (!q) return { query, total: 0, groups: [] };

  const idx = getSearchIndex();
  const hits: SearchHit[] = [];
  for (const entry of idx) {
    if (filter !== "all" && entry.hit.type !== filter) continue;
    const idxFound = entry.haystack.indexOf(q);
    if (idxFound < 0) continue;
    // 给当前 entry 打分
    const weighted = entry.haystack
      .split("\n")
      .map((line, i) => ({ value: line, weight: [10, 8, 6, 4, 3, 2][i] ?? 1 }));
    const hit: SearchHit = { ...entry.hit, score: scoreHit(q, weighted) };
    hits.push(hit);
  }
  hits.sort((a, b) => b.score - a.score);
  const top = hits.slice(0, limit);
  if (!wantGroup) {
    return { query, total: hits.length, groups: [{ type: "module", hits: top }] };
  }
  // 按 type 分组, 每组按 score desc
  const byType = new Map<SearchType, SearchHit[]>();
  for (const h of top) {
    const arr = byType.get(h.type) ?? [];
    arr.push(h);
    byType.set(h.type, arr);
  }
  // type 顺序: 模块 → 任务 → Worktree → Canvas → Agent → 反馈 → Project → Sprint → Milestone → ChangeSet → PullRequest → Notification → Comment → Doc
  const order: SearchType[] = [
    "module", "task", "worktree", "canvas", "agent", "feedback",
    "project", "sprint", "milestone", "changeSet", "pullRequest",
    "notification", "comment", "doc",
  ];
  const groups: Array<{ type: SearchType; hits: SearchHit[] }> = [];
  for (const t of order) {
    const arr = byType.get(t);
    if (arr && arr.length) groups.push({ type: t, hits: arr });
  }
  // 兜底: 有遗漏 type
  for (const [t, arr] of byType.entries()) {
    if (!order.includes(t) && arr.length) groups.push({ type: t, hits: arr });
  }
  return { query, total: hits.length, groups };
}

// 单个类型标签 (中文)
export const TYPE_LABEL: Record<SearchType, string> = {
  module: "模块",
  task: "任务",
  worktree: "Worktree",
  canvas: "画布",
  agent: "Agent",
  feedback: "反馈",
  project: "项目",
  sprint: "Sprint",
  milestone: "里程碑",
  changeSet: "ChangeSet",
  pullRequest: "Pull Request",
  notification: "通知",
  comment: "评论",
  doc: "文档",
};

export const TYPE_FILTERS: Array<{ id: SearchType | "all"; label: string }> = [
  { id: "all", label: "全部" },
  { id: "module", label: TYPE_LABEL.module },
  { id: "task", label: TYPE_LABEL.task },
  { id: "worktree", label: TYPE_LABEL.worktree },
  { id: "canvas", label: TYPE_LABEL.canvas },
  { id: "agent", label: TYPE_LABEL.agent },
  { id: "feedback", label: TYPE_LABEL.feedback },
  { id: "project", label: TYPE_LABEL.project },
  { id: "doc", label: TYPE_LABEL.doc },
];
