/*
@cypher schema=1 source_sha256=bc637144219ad6ccd8d046d72fbd05f6c03c6c1c4f28b6654158245fa1c55e79
MERGE (self:File {path:"frontend/src/app/(app)/work-item/[id]/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/work-item/[id]/page.tsx::WorkItemDetailPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
"use client";

// =====================================================================
// /work-item/[id] — 任务卡子页面 (per 2026-11-14 用户发令: 任务卡子窗口补上)
//   任务卡点击 → 本详情页 (替代旧右侧 Drawer; 聊天走全局右侧聊天栏)
//   结构: 头部 key/标题/状态 → 详情字段 → 描述 → 关联 Worktree → 操作
// =====================================================================

import { use, useMemo } from "react";
import { useRouter } from "next/navigation";
import { useStore } from "@/lib/store";
import { StatusPill } from "@/components/StatusPill";
import {
  ArrowLeft,
  GitBranch,
  MessageSquare,
  User,
  Flag,
  CalendarDays,
} from "lucide-react";
import { useChatStore } from "@/lib/chat/chatStore";

export default function WorkItemDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = use(params);
  const router = useRouter();
  const workItems = useStore((s) => s.workItems);
  const worktrees = useStore((s) => s.worktrees);
  const identities = useStore((s) => s.identities);
  const openTopicForTask = useChatStore((s) => s.openTopicForTask);
  const setActiveTopic = useChatStore((s) => s.setActiveTopic);

  const item = useMemo(() => workItems.find((w) => w.id === id || w.key === id), [workItems, id]);
  const worktree = useMemo(
    () => (item?.worktree_id ? worktrees.find((wt) => wt.id === item.worktree_id) : undefined),
    [worktrees, item],
  );
  const assignee = useMemo(
    () => (item?.assignee_id ? identities.find((i) => i.id === item.assignee_id) : undefined),
    [identities, item],
  );

  if (!item) {
    return (
      <div className="flex h-full items-center justify-center text-ink-mute text-sm" data-testid="work-item-not-found">
        任务卡 {id} 不存在
      </div>
    );
  }

  const openChat = () => {
    const tid = openTopicForTask(item.key, item.title);
    setActiveTopic(tid);
  };

  return (
    <div className="h-full overflow-y-auto p-4 sm:p-6" data-testid="work-item-detail">
      <div className="max-w-4xl mx-auto space-y-4">
        {/* 头部 */}
        <div className="flex items-start gap-3">
          <button
            type="button"
            onClick={() => router.back()}
            data-testid="work-item-back"
            className="mt-0.5 p-1.5 rounded-lg border border-line text-ink-mute hover:text-accent hover:border-accent/50"
          >
            <ArrowLeft size={15} />
          </button>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <span className="font-mono text-xs font-bold text-info">{item.key}</span>
              <StatusPill value={item.kind} size="xs" translateAs="workItemKind" />
              <StatusPill value={item.status} size="xs" translateAs="workItem" />
              <span className="flex items-center gap-1 text-[11px] font-mono font-bold text-warn">
                <Flag size={11} /> {item.priority}
              </span>
            </div>
            <h1 className="text-lg font-bold text-ink mt-1.5" data-testid="work-item-title">{item.title}</h1>
          </div>
          <button
            type="button"
            onClick={openChat}
            data-testid="work-item-open-chat"
            className="flex items-center gap-1.5 text-xs px-3 py-2 rounded-lg bg-accent text-white hover:bg-accent-soft shrink-0"
          >
            <MessageSquare size={13} /> 打开聊天
          </button>
        </div>

        {/* 详情网格 */}
        <div className="bg-bg-card rounded-xl border border-line p-4 grid grid-cols-2 md:grid-cols-4 gap-3">
          <div>
            <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1">负责人</div>
            <div className="text-sm text-ink flex items-center gap-1.5">
              <User size={12} className="text-accent" /> {assignee?.display_name ?? "—"}
            </div>
          </div>
          <div>
            <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1">故事点</div>
            <div className="text-sm text-ink font-mono">{item.story_points ?? "—"}</div>
          </div>
          <div>
            <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1">到期</div>
            <div className="text-sm text-ink flex items-center gap-1.5 font-mono">
              <CalendarDays size={12} className="text-ink-mute" /> {item.due_date?.slice(0, 10) ?? "—"}
            </div>
          </div>
          <div>
            <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1">标签</div>
            <div className="flex flex-wrap gap-1">
              {item.labels.length > 0
                ? item.labels.map((l) => (
                    <span key={l} className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-bg-soft text-ink-dim border border-line/60">{l}</span>
                  ))
                : <span className="text-sm text-ink-dim">—</span>}
            </div>
          </div>
        </div>

        {/* 描述 */}
        <div className="bg-bg-card rounded-xl border border-line p-4">
          <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-2">描述</div>
          <p className="text-sm text-ink leading-relaxed" data-testid="work-item-description">
            {item.description || "无描述"}
          </p>
        </div>

        {/* 关联 Worktree */}
        {worktree && (
          <div className="bg-bg-card rounded-xl border border-line p-4 flex items-center gap-3">
            <GitBranch size={14} className="text-accent shrink-0" />
            <div className="min-w-0 flex-1">
              <div className="text-sm font-semibold text-ink truncate">{worktree.name}</div>
              <div className="text-[10px] font-mono text-ink-mute">{worktree.id} · {worktree.status}</div>
            </div>
            <a
              href={`/worktree/${worktree.id}/group`}
              data-testid="work-item-open-worktree"
              className="text-xs px-2.5 py-1.5 rounded-lg border border-line text-accent hover:bg-accent/10"
            >
              打开 Worktree →
            </a>
          </div>
        )}
      </div>
    </div>
  );
}
