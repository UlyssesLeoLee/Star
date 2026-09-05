"use client";

/**
 * SprintBoardView — Sprint 页面 Jira 范式重写 (per 2026-09-05 19:32 JST 拍板)
 *
 * 布局 (神作级, 色彩心理学 + 平面设计 4 律):
 *   - 顶部 action bar: 搜索 + 创建 Sprint 按钮 (主操作, 电光青霓虹)
 *   - 左侧 Backlog (30%): sprint_id 缺省 workItems 列表, 拖出到 Sprint
 *   - 右侧 Sprint 栈 (70%): active 在顶 + planned 可折叠 + completed 折叠
 *   - 每张 Sprint 卡: 头部 (HUD 角标) + 4 列 kanban (todo/doing/review/done) + 操作 (启动/完成/删除/改名)
 *
 * 跨区拖动: @dnd-kit/core DndContext + useDraggable + useDroppable
 * 7 动作: 走 store (createSprint/startSprint/completeSprint/deleteSprint/renameSprint/addToSprint/removeFromSprint)
 */

import { useState, useMemo, useRef, useEffect } from "react";
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  useDraggable,
  useDroppable,
  type DragEndEvent,
  type DragStartEvent,
} from "@dnd-kit/core";
import {
  Calendar,
  Plus,
  Play,
  CheckCircle2,
  Trash2,
  Edit3,
  Search,
  Inbox,
  X,
  ChevronDown,
  ChevronRight,
  GripVertical,
  Target,
  Clock,
  TrendingUp,
  Sparkles,
} from "lucide-react";
import { useStore } from "@/lib/store";
import type { WorkItem, Sprint, Identity } from "@/types/ids";

// =====================================================================
// 工件卡 (可拖动)
// =====================================================================
function DraggableWorkItemCard({
  item,
  identities,
  onClick,
}: {
  item: WorkItem;
  identities: Identity[];
  onClick?: (id: string) => void;
}) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: item.id,
    data: { type: "workItem", workItemId: item.id, fromSprintId: item.sprint_id },
  });
  const assignee = identities.find((i) => i.id === item.assignee_id);
  return (
    <div
      ref={setNodeRef}
      {...attributes}
      {...listeners}
      onClick={() => onClick?.(item.id)}
      data-testid={`sprint-card-${item.id}`}
      className={`group anime-panel p-3 cursor-grab active:cursor-grabbing transition-all ${
        isDragging ? "opacity-30" : "lift-on-hover"
      }`}
    >
      <div className="flex items-start gap-2">
        <GripVertical className="w-3 h-3 text-ink-mute opacity-0 group-hover:opacity-100 transition-opacity mt-0.5 shrink-0" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="font-mono text-[10px] text-info font-semibold">{item.key}</span>
            <span
              className={`font-mono text-[9px] px-1.5 py-0.5 rounded font-bold ${
                item.priority === "p0" ? "text-err" :
                item.priority === "p1" ? "text-warn" :
                item.priority === "p2" ? "text-info" : "text-ink-mute"
              }`}
            >
              {item.priority.toUpperCase()}
            </span>
            {item.story_points != null && (
              <span className="anime-hud-tag text-[9px] ml-auto">{item.story_points} SP</span>
            )}
          </div>
          <div className="text-[12px] text-ink-DEFAULT leading-snug line-clamp-2">{item.title}</div>
          {assignee && (
            <div className="flex items-center gap-1 mt-1.5 text-[10px] text-ink-mute font-mono">
              <span className="inline-block w-3 h-3 rounded-full bg-accent/30 border border-accent/50" />
              {assignee.display_name}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// =====================================================================
// Drop zone
// =====================================================================
function DroppableSprintColumn({
  sprintId,
  status,
  children,
  label,
  count,
}: {
  sprintId: string;
  status: "todo" | "in_progress" | "review" | "done";
  children: React.ReactNode;
  label: string;
  count: number;
}) {
  const id = `sprint-${sprintId}-${status}`;
  const { setNodeRef, isOver } = useDroppable({ id, data: { sprintId, status, type: "column" } });
  const tone =
    status === "todo" ? "var(--color-text-mute)" :
    status === "in_progress" ? "var(--color-primary)" :
    status === "review" ? "var(--color-accent-violet)" :
    "var(--ok-DEFAULT)";
  return (
    <div
      ref={setNodeRef}
      data-testid={`sprint-col-${sprintId}-${status}`}
      className="rounded-md p-2 min-h-[120px] border transition-all"
      style={{
        background: isOver ? "color-mix(in srgb, var(--color-primary) 10%, transparent)" : "var(--bg-soft)",
        borderColor: isOver ? "var(--color-primary)" : "var(--color-border-subtle)",
        borderStyle: isOver ? "solid" : "dashed",
      }}
    >
      <div className="flex items-center justify-between mb-1.5">
        <div className="flex items-center gap-1.5">
          <span className="w-1.5 h-1.5 rounded-full" style={{ background: tone, boxShadow: `0 0 6px ${tone}` }} />
          <span className="text-[10px] font-mono uppercase tracking-wider font-semibold" style={{ color: tone }}>
            {label}
          </span>
        </div>
        <span className="text-[10px] text-ink-mute font-mono">{count}</span>
      </div>
      <div className="space-y-1.5">{children}</div>
    </div>
  );
}

// =====================================================================
// Backlog (整个左列是 1 个 drop zone, 拖到 backlog = removeFromSprint)
// =====================================================================
function BacklogDropZone({ children, count }: { children: React.ReactNode; count: number }) {
  const { setNodeRef, isOver } = useDroppable({ id: "backlog", data: { type: "backlog" } });
  return (
    <div
      ref={setNodeRef}
      data-testid="sprint-backlog"
      className="rounded-lg p-3 border-2 transition-all"
      style={{
        background: isOver ? "color-mix(in srgb, var(--color-secondary) 10%, transparent)" : "var(--bg-card)",
        borderColor: isOver ? "var(--color-secondary)" : "var(--color-border-subtle)",
        borderStyle: isOver ? "solid" : "dashed",
      }}
    >
      <div className="flex items-center gap-2 mb-3">
        <Inbox className="w-4 h-4" style={{ color: "var(--color-secondary)" }} />
        <h3 className="text-sm font-bold text-ink-DEFAULT">Backlog</h3>
        <span className="anime-hud-tag ml-auto">{count}</span>
      </div>
      <p className="text-[10px] text-ink-mute font-mono mb-3">
        拖动卡片到右侧 Sprint 加入计划
      </p>
      <div className="space-y-2 max-h-[calc(100vh-360px)] overflow-y-auto pr-1">{children}</div>
    </div>
  );
}

// =====================================================================
// Sprint 卡片 (右侧一张)
// =====================================================================
function SprintCard({
  sprint,
  items,
  identities,
  onStart,
  onComplete,
  onDelete,
  onRename,
}: {
  sprint: Sprint;
  items: WorkItem[];
  identities: Identity[];
  onStart: () => void;
  onComplete: () => void;
  onDelete: () => void;
  onRename: (name: string, goal: string) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [editName, setEditName] = useState(sprint.name);
  const [editGoal, setEditGoal] = useState(sprint.goal);

  const statusTone =
    sprint.status === "active" ? "var(--color-primary)" :
    sprint.status === "completed" ? "var(--ok-DEFAULT)" :
    sprint.status === "cancelled" ? "var(--err-DEFAULT)" :
    "var(--color-accent-violet)";
  const statusLabel =
    sprint.status === "active" ? "進行中" :
    sprint.status === "completed" ? "已完成" :
    sprint.status === "cancelled" ? "已取消" : "計劃中";

  const committedPts = items.reduce((sum, w) => sum + (w.story_points ?? 0), 0);
  const donePts = items.filter((w) => w.status === "done").reduce((sum, w) => sum + (w.story_points ?? 0), 0);
  const progressPct = committedPts > 0 ? Math.round((donePts / committedPts) * 100) : 0;
  const capacityUsedPct = sprint.capacity_points > 0 ? Math.round((committedPts / sprint.capacity_points) * 100) : 0;

  const cols = [
    { status: "todo" as const, label: "TODO" },
    { status: "in_progress" as const, label: "DOING" },
    { status: "review" as const, label: "REVIEW" },
    { status: "done" as const, label: "DONE" },
  ];

  return (
    <div
      data-testid={`sprint-block-${sprint.id}`}
      className="anime-panel anime-chamfer p-4 mb-3"
      style={{ borderLeft: `3px solid ${statusTone}` }}
    >
      {/* 头部 */}
      <div className="flex items-start justify-between gap-3 mb-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <Target className="w-3.5 h-3.5 shrink-0" style={{ color: statusTone }} />
            {editing ? (
              <input
                data-testid={`sprint-rename-input-${sprint.id}`}
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
                className="text-sm font-bold bg-bg-soft border border-line rounded px-2 py-0.5 flex-1"
                autoFocus
              />
            ) : (
              <h3 className="text-sm font-bold text-ink-DEFAULT truncate">{sprint.name}</h3>
            )}
            <span
              className="anime-hud-tag text-[9px]"
              style={{ color: statusTone, borderColor: `color-mix(in srgb, ${statusTone} 40%, transparent)`, background: `color-mix(in srgb, ${statusTone} 12%, transparent)` }}
            >
              {statusLabel}
            </span>
            {sprint.status === "active" && <span className="w-1.5 h-1.5 rounded-full pulse-ok" style={{ background: statusTone, boxShadow: `0 0 8px ${statusTone}` }} />}
          </div>
          {editing ? (
            <input
              value={editGoal}
              onChange={(e) => setEditGoal(e.target.value)}
              className="text-[11px] text-ink-dim bg-bg-soft border border-line rounded px-2 py-0.5 w-full"
              placeholder="Sprint goal"
            />
          ) : (
            sprint.goal && <p className="text-[11px] text-ink-dim line-clamp-1">{sprint.goal}</p>
          )}
          <div className="flex items-center gap-3 mt-1.5 text-[10px] text-ink-mute font-mono">
            <span className="flex items-center gap-1"><Clock className="w-2.5 h-2.5" />{sprint.start_date.slice(0, 10)} → {sprint.end_date.slice(0, 10)}</span>
            <span>·</span>
            <span>{items.length} cards</span>
            <span>·</span>
            <span style={{ color: capacityUsedPct > 100 ? "var(--err-DEFAULT)" : "inherit" }}>{committedPts}/{sprint.capacity_points} SP ({capacityUsedPct}%)</span>
          </div>
        </div>

        {/* 操作按钮 */}
        <div className="flex items-center gap-1 shrink-0">
          {editing ? (
            <>
              <button
                onClick={() => { onRename(editName, editGoal); setEditing(false); }}
                className="btn-primary text-[10px] py-1"
                data-testid={`sprint-rename-save-${sprint.id}`}
              >
                <CheckCircle2 className="w-3 h-3" />
              </button>
              <button onClick={() => { setEditName(sprint.name); setEditGoal(sprint.goal); setEditing(false); }} className="btn text-[10px] py-1">
                <X className="w-3 h-3" />
              </button>
            </>
          ) : (
            <>
              {sprint.status === "planned" && (
                <button onClick={onStart} className="btn-primary text-[10px] py-1" data-testid={`sprint-start-${sprint.id}`}>
                  <Play className="w-3 h-3" /> 启动
                </button>
              )}
              {sprint.status === "active" && (
                <button onClick={onComplete} className="btn-primary text-[10px] py-1" data-testid={`sprint-complete-${sprint.id}`} style={{ background: "var(--ok-DEFAULT)", borderColor: "var(--ok-DEFAULT)" }}>
                  <CheckCircle2 className="w-3 h-3" /> 完成
                </button>
              )}
              {sprint.status !== "active" && sprint.status !== "completed" && (
                <button onClick={() => setEditing(true)} className="btn text-[10px] py-1" data-testid={`sprint-edit-${sprint.id}`}>
                  <Edit3 className="w-3 h-3" />
                </button>
              )}
              {(sprint.status === "planned" || sprint.status === "cancelled") && (
                <button onClick={onDelete} className="btn text-[10px] py-1 hover:text-err" data-testid={`sprint-delete-${sprint.id}`}>
                  <Trash2 className="w-3 h-3" />
                </button>
              )}
            </>
          )}
        </div>
      </div>

      {/* 进度条 (active 才显示) */}
      {sprint.status === "active" && (
        <div className="mb-3">
          <div className="flex items-center justify-between mb-1">
            <span className="text-[10px] text-ink-mute font-mono flex items-center gap-1">
              <TrendingUp className="w-2.5 h-2.5" />完成度
            </span>
            <span className="text-[10px] font-mono font-bold" style={{ color: statusTone }}>{progressPct}% ({donePts}/{committedPts} SP)</span>
          </div>
          <div className="h-1.5 rounded-full overflow-hidden" style={{ background: "var(--color-surface-elevated)" }}>
            <div
              className="h-full rounded-full transition-all"
              style={{
                width: `${progressPct}%`,
                background: `linear-gradient(90deg, var(--color-primary), var(--ok-DEFAULT))`,
                boxShadow: "0 0 6px color-mix(in srgb, var(--color-primary) 50%, transparent)",
              }}
            />
          </div>
        </div>
      )}

      {/* 4 列 kanban */}
      <div className="grid grid-cols-4 gap-2">
        {cols.map((c) => (
          <DroppableSprintColumn
            key={c.status}
            sprintId={sprint.id}
            status={c.status}
            label={c.label}
            count={items.filter((w) => w.status === c.status).length}
          >
            {items.filter((w) => w.status === c.status).map((w) => (
              <DraggableWorkItemCard key={w.id} item={w} identities={identities} />
            ))}
          </DroppableSprintColumn>
        ))}
      </div>
    </div>
  );
}

// =====================================================================
// 创建 Sprint Dialog
// =====================================================================
function CreateSprintDialog({ open, onClose, onCreate }: { open: boolean; onClose: () => void; onCreate: (input: { name: string; goal: string; start_date: string; end_date: string; capacity_points: number }) => void }) {
  const [name, setName] = useState("");
  const [goal, setGoal] = useState("");
  const [start, setStart] = useState(new Date().toISOString().slice(0, 10));
  const [end, setEnd] = useState(new Date(Date.now() + 14 * 24 * 60 * 60 * 1000).toISOString().slice(0, 10));
  const [capacity, setCapacity] = useState(40);
  if (!open) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4" style={{ background: "rgba(0,0,0,0.6)" }} onClick={onClose}>
      <div className="anime-panel anime-chamfer max-w-md w-full p-6" onClick={(e) => e.stopPropagation()} data-testid="sprint-create-dialog">
        <div className="flex items-center gap-2 mb-4">
          <Sparkles className="w-4 h-4" style={{ color: "var(--color-primary)" }} />
          <h3 className="text-title font-bold">创建新 Sprint</h3>
        </div>
        <div className="space-y-3">
          <div>
            <label className="block text-[10px] text-ink-mute font-mono uppercase tracking-wider mb-1">名称</label>
            <input data-testid="sprint-create-name" value={name} onChange={(e) => setName(e.target.value)} className="w-full bg-bg-soft border border-line rounded px-3 py-1.5 text-sm" placeholder="Sprint 25 — …" />
          </div>
          <div>
            <label className="block text-[10px] text-ink-mute font-mono uppercase tracking-wider mb-1">目标 (Goal)</label>
            <input data-testid="sprint-create-goal" value={goal} onChange={(e) => setGoal(e.target.value)} className="w-full bg-bg-soft border border-line rounded px-3 py-1.5 text-sm" placeholder="Sprint 目标" />
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="block text-[10px] text-ink-mute font-mono uppercase tracking-wider mb-1">开始日期</label>
              <input data-testid="sprint-create-start" type="date" value={start} onChange={(e) => setStart(e.target.value)} className="w-full bg-bg-soft border border-line rounded px-3 py-1.5 text-sm font-mono" />
            </div>
            <div>
              <label className="block text-[10px] text-ink-mute font-mono uppercase tracking-wider mb-1">结束日期</label>
              <input data-testid="sprint-create-end" type="date" value={end} onChange={(e) => setEnd(e.target.value)} className="w-full bg-bg-soft border border-line rounded px-3 py-1.5 text-sm font-mono" />
            </div>
          </div>
          <div>
            <label className="block text-[10px] text-ink-mute font-mono uppercase tracking-wider mb-1">容量 (SP)</label>
            <input data-testid="sprint-create-capacity" type="number" min={1} max={500} value={capacity} onChange={(e) => setCapacity(parseInt(e.target.value) || 40)} className="w-full bg-bg-soft border border-line rounded px-3 py-1.5 text-sm font-mono" />
          </div>
        </div>
        <div className="flex items-center justify-end gap-2 mt-5">
          <button onClick={onClose} className="btn text-xs">取消</button>
          <button
            data-testid="sprint-create-submit"
            onClick={() => {
              if (!name.trim()) return;
              onCreate({ name: name.trim(), goal: goal.trim(), start_date: new Date(start).toISOString(), end_date: new Date(end).toISOString(), capacity_points: capacity });
              setName(""); setGoal(""); onClose();
            }}
            className="btn-primary text-xs"
          >
            <Plus className="w-3 h-3" /> 创建
          </button>
        </div>
      </div>
    </div>
  );
}

// =====================================================================
// 完成 Sprint Dialog (move_uncompleted_to 选择)
// =====================================================================
function CompleteSprintDialog({ open, sprint, onClose, onComplete }: { open: boolean; sprint: Sprint | null; onClose: () => void; onComplete: (move: 'backlog' | 'new_sprint', newSprint?: { name: string; goal: string; start_date: string; end_date: string; capacity_points: number }) => void }) {
  const [move, setMove] = useState<'backlog' | 'new_sprint'>('backlog');
  const [name, setName] = useState("");
  const [goal, setGoal] = useState("");
  const [capacity, setCapacity] = useState(40);
  if (!open || !sprint) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4" style={{ background: "rgba(0,0,0,0.6)" }} onClick={onClose}>
      <div className="anime-panel anime-chamfer max-w-md w-full p-6" onClick={(e) => e.stopPropagation()} data-testid="sprint-complete-dialog">
        <div className="flex items-center gap-2 mb-4">
          <CheckCircle2 className="w-4 h-4" style={{ color: "var(--ok-DEFAULT)" }} />
          <h3 className="text-title font-bold">完成 Sprint: {sprint.name}</h3>
        </div>
        <p className="text-[12px] text-ink-dim mb-3">未完成卡片如何处理?</p>
        <div className="space-y-2 mb-4">
          <label className="flex items-center gap-2 p-3 rounded border cursor-pointer" style={{ borderColor: move === 'backlog' ? "var(--color-primary)" : "var(--color-border)" }}>
            <input type="radio" checked={move === 'backlog'} onChange={() => setMove('backlog')} />
            <div>
              <div className="text-sm font-semibold">回 Backlog</div>
              <div className="text-[10px] text-ink-mute font-mono">所有未完成卡片 sprint_id 置空</div>
            </div>
          </label>
          <label className="flex items-center gap-2 p-3 rounded border cursor-pointer" style={{ borderColor: move === 'new_sprint' ? "var(--color-primary)" : "var(--color-border)" }}>
            <input type="radio" checked={move === 'new_sprint'} onChange={() => setMove('new_sprint')} />
            <div>
              <div className="text-sm font-semibold">新建 Sprint</div>
              <div className="text-[10px] text-ink-mute font-mono">未完成卡片自动加入新 Sprint</div>
            </div>
          </label>
        </div>
        {move === 'new_sprint' && (
          <div className="space-y-2 mb-4 pl-6 border-l-2" style={{ borderColor: "var(--color-accent-violet)" }}>
            <input data-testid="sprint-complete-newsprint-name" value={name} onChange={(e) => setName(e.target.value)} className="w-full bg-bg-soft border border-line rounded px-3 py-1.5 text-sm" placeholder="新 Sprint 名称" />
            <input value={goal} onChange={(e) => setGoal(e.target.value)} className="w-full bg-bg-soft border border-line rounded px-3 py-1.5 text-sm" placeholder="Goal" />
            <input type="number" min={1} max={500} value={capacity} onChange={(e) => setCapacity(parseInt(e.target.value) || 40)} className="w-full bg-bg-soft border border-line rounded px-3 py-1.5 text-sm font-mono" placeholder="Capacity SP" />
          </div>
        )}
        <div className="flex items-center justify-end gap-2">
          <button onClick={onClose} className="btn text-xs">取消</button>
          <button
            data-testid="sprint-complete-submit"
            onClick={() => {
              onComplete(move, move === 'new_sprint' ? { name: name.trim() || "Sprint (续)", goal: goal.trim(), start_date: new Date().toISOString(), end_date: new Date(Date.now() + 14 * 86400000).toISOString(), capacity_points: capacity } : undefined);
              onClose();
            }}
            className="btn-primary text-xs"
            style={{ background: "var(--ok-DEFAULT)", borderColor: "var(--ok-DEFAULT)" }}
          >
            <CheckCircle2 className="w-3 h-3" /> 完成
          </button>
        </div>
      </div>
    </div>
  );
}

// =====================================================================
// 主组件
// =====================================================================
export function SprintBoardView({ onSelect }: { onSelect?: (id: string) => void }) {
  const sprints = useStore((s) => s.sprints);
  const workItems = useStore((s) => s.workItems);
  const identities = useStore((s) => s.identities);
  const createSprint = useStore((s) => s.createSprint);
  const startSprint = useStore((s) => s.startSprint);
  const completeSprint = useStore((s) => s.completeSprint);
  const deleteSprint = useStore((s) => s.deleteSprint);
  const renameSprint = useStore((s) => s.renameSprint);
  const addToSprint = useStore((s) => s.addToSprint);
  const removeFromSprint = useStore((s) => s.removeFromSprint);

  const [search, setSearch] = useState("");
  const [showCreate, setShowCreate] = useState(false);
  const [completeSprintId, setCompleteSprintId] = useState<string | null>(null);
  const [showCompleted, setShowCompleted] = useState(false);
  const [activeDragId, setActiveDragId] = useState<string | null>(null);

  // 过滤
  const filtered = useMemo(() => {
    if (!search.trim()) return workItems;
    const q = search.toLowerCase();
    return workItems.filter((w) => w.title.toLowerCase().includes(q) || w.key.toLowerCase().includes(q));
  }, [workItems, search]);

  const backlogItems = useMemo(() => filtered.filter((w) => !w.sprint_id), [filtered]);

  const activeSprints = useMemo(() => sprints.filter((s) => s.status === "active"), [sprints]);
  const plannedSprints = useMemo(() => sprints.filter((s) => s.status === "planned"), [sprints]);
  const completedSprints = useMemo(() => sprints.filter((s) => s.status === "completed" || s.status === "cancelled"), [sprints]);

  // 拖动 sensors
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }));

  // 拖动处理
  const handleDragStart = (e: DragStartEvent) => {
    setActiveDragId(e.active.id as string);
  };
  const handleDragEnd = (e: DragEndEvent) => {
    setActiveDragId(null);
    const { active, over } = e;
    if (!over) return;
    const workItemId = active.id as string;
    const item = workItems.find((w) => w.id === workItemId);
    if (!item) return;
    const overData = over.data.current as { type?: string; sprintId?: string; status?: string } | undefined;
    if (overData?.type === "backlog") {
      // 拖到 Backlog = remove
      if (item.sprint_id) removeFromSprint(workItemId);
    } else if (overData?.sprintId) {
      // 拖到某 Sprint 列
      if (item.sprint_id !== overData.sprintId) {
        addToSprint(overData.sprintId, workItemId);
      }
    }
  };

  // DragOverlay
  const draggingItem = activeDragId ? workItems.find((w) => w.id === activeDragId) : null;

  // 关闭 dialog 用的 ref
  const completeSprintTarget = completeSprintId ? sprints.find((s) => s.id === completeSprintId) ?? null : null;

  return (
    <DndContext sensors={sensors} onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
      <div className="flex flex-col h-full" data-testid="sprint-board-view">
        {/* 顶部 action bar */}
        <div className="flex items-center gap-2 mb-4">
          <div className="anime-panel flex-1 flex items-center gap-2 px-3 py-1.5">
            <Search className="w-3.5 h-3.5 text-ink-mute" />
            <input
              data-testid="sprint-search"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="搜索 work-item (title / key)..."
              className="bg-transparent flex-1 text-sm focus:outline-none placeholder:text-ink-mute"
            />
            {search && (
              <button onClick={() => setSearch("")} className="text-ink-mute hover:text-ink">
                <X className="w-3 h-3" />
              </button>
            )}
          </div>
          <button
            data-testid="sprint-create-button"
            onClick={() => setShowCreate(true)}
            className="btn-primary text-xs shrink-0"
          >
            <Plus className="w-3.5 h-3.5" /> 创建 Sprint
          </button>
        </div>

        {/* 主区: 左 Backlog / 右 Sprint 栈 */}
        <div className="grid grid-cols-12 gap-4 flex-1 min-h-0">
          {/* 左侧 Backlog */}
          <div className="col-span-12 lg:col-span-4">
            <BacklogDropZone count={backlogItems.length}>
              {backlogItems.length === 0 ? (
                <div className="text-center text-ink-mute italic py-6 text-xs">无 Backlog 卡片</div>
              ) : (
                backlogItems.map((w) => <DraggableWorkItemCard key={w.id} item={w} identities={identities} onClick={onSelect} />)
              )}
            </BacklogDropZone>
          </div>

          {/* 右侧 Sprint 栈 */}
          <div className="col-span-12 lg:col-span-8 overflow-y-auto pr-1" data-testid="sprint-list">
            {/* Active 永远展开在顶 */}
            {activeSprints.map((sp) => (
              <SprintCard
                key={sp.id}
                sprint={sp}
                items={filtered.filter((w) => w.sprint_id === sp.id)}
                identities={identities}
                onStart={() => startSprint(sp.id)}
                onComplete={() => setCompleteSprintId(sp.id)}
                onDelete={() => deleteSprint(sp.id)}
                onRename={(name, goal) => renameSprint(sp.id, name, goal)}
              />
            ))}

            {/* Planned 展开 */}
            {plannedSprints.length > 0 && (
              <div className="mb-3">
                <h4 className="text-[10px] text-ink-mute font-mono uppercase tracking-wider mb-2 flex items-center gap-1">
                  <ChevronDown className="w-3 h-3" /> 計劃中 ({plannedSprints.length})
                </h4>
                {plannedSprints.map((sp) => (
                  <SprintCard
                    key={sp.id}
                    sprint={sp}
                    items={filtered.filter((w) => w.sprint_id === sp.id)}
                    identities={identities}
                    onStart={() => startSprint(sp.id)}
                    onComplete={() => setCompleteSprintId(sp.id)}
                    onDelete={() => deleteSprint(sp.id)}
                    onRename={(name, goal) => renameSprint(sp.id, name, goal)}
                  />
                ))}
              </div>
            )}

            {/* Completed 折叠 */}
            {completedSprints.length > 0 && (
              <div className="mb-3">
                <button
                  onClick={() => setShowCompleted((s) => !s)}
                  className="text-[10px] text-ink-mute font-mono uppercase tracking-wider mb-2 flex items-center gap-1 hover:text-ink"
                  data-testid="sprint-completed-toggle"
                >
                  {showCompleted ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
                  {showCompleted ? "收起" : "展开"} 已完成 ({completedSprints.length})
                </button>
                {showCompleted && completedSprints.map((sp) => (
                  <SprintCard
                    key={sp.id}
                    sprint={sp}
                    items={filtered.filter((w) => w.sprint_id === sp.id)}
                    identities={identities}
                    onStart={() => startSprint(sp.id)}
                    onComplete={() => setCompleteSprintId(sp.id)}
                    onDelete={() => deleteSprint(sp.id)}
                    onRename={(name, goal) => renameSprint(sp.id, name, goal)}
                  />
                ))}
              </div>
            )}

            {sprints.length === 0 && (
              <div className="anime-panel p-13 text-center">
                <Target className="w-10 h-10 mx-auto mb-3 text-ink-mute" />
                <h3 className="text-title font-bold mb-1">尚無 Sprint</h3>
                <p className="text-sm text-ink-dim mb-4">创建第一个 Sprint 开始计划</p>
                <button onClick={() => setShowCreate(true)} className="btn-primary text-sm">
                  <Plus className="w-4 h-4" /> 创建 Sprint
                </button>
              </div>
            )}
          </div>
        </div>

        {/* Dialogs */}
        <CreateSprintDialog
          open={showCreate}
          onClose={() => setShowCreate(false)}
          onCreate={(input) => createSprint(input)}
        />
        <CompleteSprintDialog
          open={!!completeSprintId}
          sprint={completeSprintTarget}
          onClose={() => setCompleteSprintId(null)}
          onComplete={(move, newSprint) => {
            if (completeSprintId) completeSprint(completeSprintId, move, newSprint);
          }}
        />

        {/* Drag overlay */}
        <DragOverlay>
          {draggingItem ? (
            <div className="anime-panel p-3 opacity-90 rotate-2 shadow-2xl" style={{ boxShadow: "var(--shadow-lg)" }}>
              <div className="font-mono text-[10px] text-info">{draggingItem.key}</div>
              <div className="text-[12px] text-ink-DEFAULT line-clamp-1">{draggingItem.title}</div>
            </div>
          ) : null}
        </DragOverlay>
      </div>
    </DndContext>
  );
}
