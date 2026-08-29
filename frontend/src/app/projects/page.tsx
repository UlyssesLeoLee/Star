"use client";

// =====================================================================
// Projects Page — 5-panel project workspace (per ui-redesign-multica-style.md §5.2)
// =====================================================================
// 职责:
//   1. 顶部 project switcher — 切换 selectedProjectId (zustand 本地 state, 不持久化)
//   2. 5 tab 切换: Overview / Board / Timeline / Calendar / Members
//   3. 每个 tab 根据 project_id 过滤 store 数据:
//      - Overview   — project 元信息 + KPI (work-items / agents / last activity)
//      - Board      — Kanban (per project_id 过滤 workItems, 复用 W1 KanbanBoard)
//      - Timeline   — Gantt (per project_id 过滤 sprint/milestone/work-item, 复用 W2 GanttChart)
//      - Calendar   — 月/周视图 (per project_id 过滤 event, 复用 W3 MonthView/WeekView)
//      - Members    — 团队成员 + 角色 (从 workspace.member_ids 推导)
//
// 不做 (per 守门):
//   - 不改 tailwind.config.ts (U5)
//   - 不改 next.config.js (U5)
//   - 不改 frontend/src/app/layout.tsx (U1)
//   - 不写 layout (U1 在做 AppShell)
//   - 不写 SubNav (U2 在做), page 接受 local selectedTab 状态
//   - 不引 dnd-kit (per W1 守门 + 不在 dependencies), 复用 W1 KanbanBoard (HTML5 native)
//
// 已知缺口 (per 缺标比错标 — 8/26 JST 偏好):
//   1. Board 拖动: store.transitionWorkItem 即时改 status + 同步 board.columns,
//      但 board.columns 仍共享 (全项目), 故 Project 切换时 board 会被前一个项目改过
//      持久化影响; 后端 PATCH /work-items/{id}/status D.6+ 接
//   2. Timeline: handleMilestoneUpdate / handleSprintUpdate / handleWorkItemMove
//      走 useStore.setState 同步 workItems/milestones/sprints, 后端 PATCH D.6+ 接
//   3. Calendar: handleEventMove 同 Timeline, 走 useStore.setState
//   4. Members: 角色 (project_admin / developer / viewer) 用 mock 推导
//      (member_count 5+ → project_admin, 1-4 → developer, 0 → viewer)
//      rbac 真接入 Phase I+ 接 backend permission API
//   5. light mode: per design §7 dark-only, 后置
//   6. mobile 响应式: 已写基础 (1280/1024/768), 触屏拖动 Phase Mobile
//
// 数据源 (zustand store):
//   projects, workItems, sprints, milestones, board, identities, workspaces,
//   agentSessions, changeSets, worktrees, pullRequests
// =====================================================================

import { useMemo, useState, useCallback, useEffect } from "react";
import { useSearchParams } from "next/navigation";
import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle, Stat } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Tabs } from "@/components/Tabs";
import { KanbanBoard } from "@/components/board/KanbanBoard";
import { GanttChart } from "@/components/gantt";
import { MonthView } from "@/components/calendar/MonthView";
import { WeekView } from "@/components/calendar/WeekView";
import { CalendarHeader } from "@/components/calendar/CalendarHeader";
import { CalendarLegend } from "@/components/calendar/CalendarLegend";
import { buildEvents } from "@/components/calendar/events";
import {
  FolderTree,
  LayoutDashboard,
  Trello,
  SquareChartGantt,
  CalendarRange,
  Users,
  ChevronRight,
  Activity,
  Clock,
} from "lucide-react";
import { addDays, format, parseISO, differenceInDays } from "date-fns";
import type {
  Board, Project, WorkItem, WorkItemStatus, Identity, Workspace, Sprint, Milestone, Iso8601,
} from "@/types/ids";

// ---- 5 tab 定义 (per design §5.2) ----
type ProjectsTabId = "overview" | "board" | "timeline" | "calendar" | "members";
const TAB_ITEMS: Array<{ id: ProjectsTabId; label: string; icon: React.ReactNode }> = [
  { id: "overview", label: "Overview", icon: <LayoutDashboard size={12} /> },
  { id: "board",    label: "Board",    icon: <Trello size={12} /> },
  { id: "timeline", label: "Timeline", icon: <SquareChartGantt size={12} /> },
  { id: "calendar", label: "Calendar", icon: <CalendarRange size={12} /> },
  { id: "members",  label: "Members",  icon: <Users size={12} /> },
];

// Kanban 4 列 (per W1 KANBAN_COLUMNS, 内联避免循环 import)
const KANBAN_COLUMNS: WorkItemStatus[] = ["todo", "in_progress", "review", "done"];

// ---- 角色 mock 推导 (per 已知缺口 #4) ----
// member_count >= 5 → project_admin, 1-4 → developer, 0 → viewer
function deriveRole(project: Project): "project_admin" | "developer" | "viewer" {
  if (project.member_count >= 5) return "project_admin";
  if (project.member_count >= 1) return "developer";
  return "viewer";
}

export default function ProjectsPage() {
  // ---- store 订阅 (zustand selectors) ----
  const projects = useStore((s) => s.projects);
  const workItems = useStore((s) => s.workItems);
  const sprints = useStore((s) => s.sprints);
  const milestones = useStore((s) => s.milestones);
  const board = useStore((s) => s.board);
  const relations = useStore((s) => s.relations);
  const identities = useStore((s) => s.identities);
  const workspaces = useStore((s) => s.workspaces);
  const agentSessions = useStore((s) => s.agentSessions);
  // Board 列编辑 (per 2026-08-29 18:52 JST 拍板)
  const addBoardColumn = useStore((s) => s.addBoardColumn);
  const removeBoardColumn = useStore((s) => s.removeBoardColumn);
  const renameBoardColumn = useStore((s) => s.renameBoardColumn);
  const changeSets = useStore((s) => s.changeSets);
  const worktrees = useStore((s) => s.worktrees);
  const repositories = useStore((s) => s.repositories);
  const pullRequests = useStore((s) => s.pullRequests);
  const transitionWorkItem = useStore((s) => s.transitionWorkItem);
  const transitionMilestone = useStore((s) => s.transitionMilestone);
  const transitionSprint = useStore((s) => s.transitionSprint);

  // ---- local state ----
  const [selectedProjectId, setSelectedProjectId] = useState<string>(
    () => projects[0]?.id ?? "",
  );
  const [tab, setTab] = useState<ProjectsTabId>("overview");
  // 同步 URL ?tab=X 到 local state (per 2026-08-29 17:42 JST 修 next.config.js redirect 后, redirect 给 tab=timeline 但 page 默认 tab=overview, 必须 useSearchParams 同步)
  const searchParams = useSearchParams();
  useEffect(() => {
    const tabParam = searchParams.get("tab");
    if (tabParam && ["overview", "board", "timeline", "calendar", "members"].includes(tabParam)) {
      setTab(tabParam as ProjectsTabId);
    }
  }, [searchParams]);
  const [calendarView, setCalendarView] = useState<"month" | "week">("month");
  const [calendarCursor, setCalendarCursor] = useState<{ year: number; month: number }>(() => {
    const now = new Date();
    return { year: now.getFullYear(), month: now.getMonth() };
  });
  const [weekStart, setWeekStart] = useState<Date>(() => {
    const now = new Date();
    now.setHours(0, 0, 0, 0);
    return now;
  });

  // ---- 选中的 project + project-scoped 数据 (per project_id 过滤) ----
  const selectedProject = useMemo<Project | null>(
    () => projects.find((p) => p.id === selectedProjectId) ?? projects[0] ?? null,
    [projects, selectedProjectId],
  );

  const projectWorkItems = useMemo(
    () => workItems.filter((w) => w.project_id === selectedProjectId),
    [workItems, selectedProjectId],
  );
  const projectSprints = useMemo(
    () => sprints.filter((s) => s.project_id === selectedProjectId),
    [sprints, selectedProjectId],
  );
  const projectMilestones = useMemo(
    () => milestones.filter((m) => m.project_id === selectedProjectId),
    [milestones, selectedProjectId],
  );
  // 任务依赖关系 (per MS Project task link, 2026-08-29 17:33 JST)
  // 过滤: from_id / to_id 是本项目的 work_item / sprint / milestone
  const projectRelations = useMemo(
    () => {
      const wiIds = new Set(projectWorkItems.map((w) => w.id));
      const sprintIds = new Set(projectSprints.map((s) => s.id));
      const msIds = new Set(projectMilestones.map((m) => m.id));
      const known = new Set([...wiIds, ...sprintIds, ...msIds]);
      return relations.filter(
        (r) => known.has(r.from_id) || known.has(r.to_id),
      );
    },
    [relations, projectWorkItems, projectSprints, projectMilestones],
  );
  const projectWorkspaces = useMemo(
    () => workspaces.filter((w) => w.project_id === selectedProjectId),
    [workspaces, selectedProjectId],
  );
  const projectMemberIds = useMemo(() => {
    const set = new Set<string>();
    for (const ws of projectWorkspaces) for (const mid of ws.member_ids) set.add(mid);
    return Array.from(set);
  }, [projectWorkspaces]);
  const projectMembers = useMemo(
    () => identities.filter((u) => projectMemberIds.includes(u.id)),
    [identities, projectMemberIds],
  );

  // ---- Board 拖动 transition: ProjectBoard 复用 W1 模式 (transitionWorkItem + 同步 board.columns) ----
  const handleBoardTransition = useCallback(
    (workItemId: string, toStatus: WorkItemStatus) => {
      // 校验: 仅处理当前 project 的 work-item
      const w = workItems.find((x) => x.id === workItemId);
      if (!w || w.project_id !== selectedProjectId) return;

      // 1) 走 store 状态机
      transitionWorkItem(workItemId, toStatus);

      // 2) 同步 board.columns (per W1 pattern)
      useStore.setState((s) => {
        const fromCol = s.board.columns.find((c) => c.work_item_ids.includes(workItemId));
        const toCol = s.board.columns.find((c) => c.status === toStatus);
        if (!fromCol || !toCol) return s;
        if (fromCol.status === toCol.status) return s;
        return {
          board: {
            ...s.board,
            columns: s.board.columns.map((c) => {
              if (c.status === fromCol.status) {
                return { ...c, work_item_ids: c.work_item_ids.filter((id) => id !== workItemId) };
              }
              if (c.status === toCol.status) {
                return { ...c, work_item_ids: [...c.work_item_ids, workItemId] };
              }
              return c;
            }),
          },
        };
      });
    },
    [workItems, selectedProjectId, transitionWorkItem],
  );

  // ---- ProjectBoard: project 过滤的虚拟 board (per project_id) ----
  const projectBoard = useMemo<Board>(() => {
    // 复制 seed board 结构, 过滤 work_item_ids
    return {
      ...board,
      project_id: selectedProjectId,
      name: `${selectedProject?.key ?? "Project"} Board`,
      columns: board.columns.map((col) => ({
        ...col,
        work_item_ids: col.work_item_ids.filter((id) =>
          projectWorkItems.some((w) => w.id === id),
        ),
      })),
    };
  }, [board, selectedProject, selectedProjectId, projectWorkItems]);

  // ---- ProjectCalendar: project 过滤的 events ----
  const projectEvents = useMemo(
    () => buildEvents(projectSprints, projectMilestones, projectWorkItems),
    [projectSprints, projectMilestones, projectWorkItems],
  );

  // ---- ProjectTimeline: Gantt date range ----
  const ganttDateRange = useMemo(() => {
    const all: Date[] = [
      ...projectSprints.flatMap((s) => [parseISO(s.start_date), parseISO(s.end_date)]),
      ...projectMilestones.map((m) => parseISO(m.due_date)),
    ];
    if (all.length === 0) {
      const today = new Date();
      return { start: format(today, "yyyy-MM-dd"), end: format(addDays(today, 60), "yyyy-MM-dd") };
    }
    const min = all.reduce((a, b) => (a < b ? a : b));
    const max = all.reduce((a, b) => (a > b ? a : b));
    const start = addDays(min, -7);
    const end = addDays(max, 7);
    if (differenceInDays(end, start) > 180) {
      return { start: format(start, "yyyy-MM-dd"), end: format(addDays(start, 180), "yyyy-MM-dd") };
    }
    return { start: format(start, "yyyy-MM-dd"), end: format(end, "yyyy-MM-dd") };
  }, [projectSprints, projectMilestones]);

  // ---- Gantt handlers (per 已知缺口 #2: 走 useStore.setState, 后端 PATCH D.6+) ----
  const handleMilestoneUpdate = useCallback((id: string, newDueDate: string) => {
    transitionMilestone(id, newDueDate);
  }, [transitionMilestone]);
  const handleSprintUpdate = useCallback((id: string, newStart: string, newEnd: string) => {
    transitionSprint(id, newStart, newEnd);
  }, [transitionSprint]);
  const handleWorkItemMove = useCallback((workItemId: string, newSprintId: string) => {
    useStore.setState((s) => ({
      workItems: s.workItems.map((w) =>
        w.id === workItemId
          ? { ...w, sprint_id: newSprintId, updated_at: new Date().toISOString() }
          : w,
      ),
    }));
  }, []);

  // ---- Calendar handler ----
  const handleCalendarEventMove = useCallback((eventId: string, newDate: string) => {
    const iso = `${newDate}T00:00:00.000Z`;
    // 1) 找是否 milestone
    const ms = useStore.getState().milestones.find((m) => m.id === eventId);
    if (ms) {
      transitionMilestone(eventId, iso);
      return;
    }
    // 2) 否则视为 work-item (改 due_date)
    useStore.setState((s) => ({
      workItems: s.workItems.map((w) =>
        w.id === eventId
          ? { ...w, due_date: iso, updated_at: new Date().toISOString() }
          : w,
      ),
    }));
  }, [transitionMilestone]);

  // ---- ProjectOverview KPI ----
  const kpis = useMemo(() => {
    const openCount = projectWorkItems.filter(
      (w) => w.status !== "done" && w.status !== "wontfix",
    ).length;
    const closedCount = projectWorkItems.filter(
      (w) => w.status === "done" || w.status === "wontfix",
    ).length;
    const activeAgents = agentSessions.filter(
      (a) => a.project_id === selectedProjectId &&
             !["completed", "failed", "cancelled"].includes(a.status),
    ).length;
    const cs = changeSets.filter((c) => c.project_id === selectedProjectId);
    const wt = worktrees.filter((w) => w.project_id === selectedProjectId);
    // PR 没有 project_id, 通过 repository.project_id 推导
    const projectRepoIds = new Set(
      repositories.filter((r) => r.project_id === selectedProjectId).map((r) => r.id),
    );
    const prs = pullRequests.filter((p) => projectRepoIds.has(p.repository_id));
    const lastActivityTs = [
      ...projectWorkItems.map((w) => w.updated_at),
      ...cs.map((c) => c.created_at),
      ...wt.map((w) => w.last_event_at),
    ].sort().pop();
    return {
      open: openCount,
      closed: closedCount,
      activeAgents,
      changeSets: cs.length,
      worktrees: wt.length,
      pullRequests: prs.length,
      lastActivityTs,
    };
  }, [projectWorkItems, agentSessions, changeSets, worktrees, repositories, pullRequests, selectedProjectId]);

  // ---- E2E hook: 暴露 selectedProject + 切换函数 (供自动化测试) ----
  useEffect(() => {
    if (typeof window === "undefined") return;
    (window as unknown as Record<string, unknown>).__projectsApi = {
      selectedProjectId,
      setSelectedProjectId,
      setTab,
      getProjectBoard: () => projectBoard,
      getProjectWorkItems: () => projectWorkItems,
      getProjectEvents: () => projectEvents,
      getProjectMembers: () => projectMembers,
    };
  }, [selectedProjectId, projectBoard, projectWorkItems, projectEvents, projectMembers]);

  // ---- owner identity ----
  const ownerIdentity = useMemo<Identity | null>(
    () => identities.find((u) => u.id === selectedProject?.owner_id) ?? null,
    [identities, selectedProject],
  );

  if (!selectedProject) {
    return (
      <div className="max-w-7xl">
        <PageHeader
          title="Projects"
          icon={<FolderTree className="text-accent" size={20} />}
          subtitle="(no projects available)"
        />
      </div>
    );
  }

  return (
    <div className="max-w-7xl" data-testid="projects-page">
      <PageHeader
        title="Projects"
        subtitle="多面板项目工作区 — Overview / Board / Timeline / Calendar / Members 5 tab 聚合。"
        icon={<FolderTree className="text-accent" size={20} />}
        track="D"
        count={`${projects.length} projects`}
      />

      {/* ---- Project switcher (顶部, sticky) ---- */}
      <ProjectSwitcher
        projects={projects}
        selectedId={selectedProjectId}
        onSelect={setSelectedProjectId}
      />

      {/* ---- Tabs ---- */}
      <Tabs
        active={tab}
        onChange={(id) => setTab(id as ProjectsTabId)}
        items={TAB_ITEMS.map((t) => ({
          id: t.id,
          label: t.label,
          icon: t.icon,
          badge:
            t.id === "overview" ? undefined :
            t.id === "board" ? projectWorkItems.length :
            t.id === "timeline" ? (projectSprints.length + projectMilestones.length) :
            t.id === "calendar" ? projectEvents.length :
            t.id === "members" ? projectMembers.length :
            undefined,
        }))}
      />

      {/* ---- Tab content ---- */}
      {tab === "overview" && (
        <ProjectOverview
          project={selectedProject}
          owner={ownerIdentity}
          workspaces={projectWorkspaces}
          kpis={kpis}
        />
      )}

      {tab === "board" && (
        <div data-testid="projects-board-tab">
          <KanbanBoard
            board={projectBoard}
            workItems={projectWorkItems}
            identities={identities}
            onTransition={handleBoardTransition}
            onAddColumn={addBoardColumn}
            onRemoveColumn={removeBoardColumn}
            onRenameColumn={renameBoardColumn}
          />
          <div className="mt-3 text-[10px] text-ink-mute font-mono">
            列对应状态: {KANBAN_COLUMNS.join(" / ")} — 拖动卡片触发 transitionWorkItem (走 store 状态机)
          </div>
          {/* 已知缺口 #1 提示 */}
          <div className="mt-1 text-[10px] text-ink-mute font-mono">
            ⚠ Board 拖动改 status 走 store 状态机 + 同步 board.columns; 后端 PATCH /work-items/{`{id}`}/status 持久化 D.6+ 接
          </div>
        </div>
      )}

      {tab === "timeline" && (
        <div data-testid="projects-timeline-tab">
          <GanttChart
            sprints={projectSprints}
            milestones={projectMilestones}
            workItems={projectWorkItems}
            relations={projectRelations}
            dateRange={ganttDateRange}
            onMilestoneUpdate={handleMilestoneUpdate}
            onSprintUpdate={handleSprintUpdate}
            onWorkItemMove={handleWorkItemMove}
          />
          {/* 已知缺口 #2 提示 */}
          <div className="mt-3 text-[10px] text-ink-mute font-mono">
            ⚠ 拖动 milestone / sprint 改 due_date / 起止, 走 store + useStore.setState; 后端 PATCH D.6+ 接
          </div>
        </div>
      )}

      {tab === "calendar" && (
        <div data-testid="projects-calendar-tab" className="space-y-3">
          <CalendarHeader
            year={calendarCursor.year}
            month={calendarCursor.month}
            weekStart={weekStart}
            view={calendarView}
            onPrev={() => {
              if (calendarView === "month") {
                const d = new Date(calendarCursor.year, calendarCursor.month - 1, 1);
                setCalendarCursor({ year: d.getFullYear(), month: d.getMonth() });
              } else {
                const d = new Date(weekStart);
                d.setDate(d.getDate() - 7);
                setWeekStart(d);
              }
            }}
            onNext={() => {
              if (calendarView === "month") {
                const d = new Date(calendarCursor.year, calendarCursor.month + 1, 1);
                setCalendarCursor({ year: d.getFullYear(), month: d.getMonth() });
              } else {
                const d = new Date(weekStart);
                d.setDate(d.getDate() + 7);
                setWeekStart(d);
              }
            }}
            onToday={() => {
              const now = new Date();
              setCalendarCursor({ year: now.getFullYear(), month: now.getMonth() });
              const w = new Date(now);
              w.setHours(0, 0, 0, 0);
              setWeekStart(w);
            }}
            onViewChange={setCalendarView}
            userTimezone={Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC"}
          />

          {calendarView === "month" ? (
            <MonthView
              year={calendarCursor.year}
              month={calendarCursor.month}
              events={projectEvents}
              onEventMove={handleCalendarEventMove}
              onMonthChange={(y, m) => setCalendarCursor({ year: y, month: m })}
            />
          ) : (
            <WeekView
              startDate={weekStart}
              events={projectEvents}
              onEventMove={handleCalendarEventMove}
              userTimezone={Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC"}
            />
          )}

          <CalendarLegend />
          {/* 已知缺口 #3 提示 */}
          <div className="text-[10px] text-ink-mute font-mono">
            ⚠ 拖 work-item / milestone 改 due_date 走 useStore.setState; 后端 PATCH D.6+ 接
          </div>
        </div>
      )}

      {tab === "members" && (
        <ProjectMembers
          project={selectedProject}
          members={projectMembers}
          workspaces={projectWorkspaces}
          ownerId={selectedProject.owner_id}
        />
      )}
    </div>
  );
}

// =====================================================================
// ProjectSwitcher — 顶部项目切换 (multica 风格 chip row)
// =====================================================================
function ProjectSwitcher({
  projects, selectedId, onSelect,
}: {
  projects: Project[];
  selectedId: string;
  onSelect: (id: string) => void;
}) {
  return (
    <div
      data-testid="project-switcher"
      className="flex items-center gap-2 mb-4 overflow-x-auto"
      role="tablist"
      aria-label="Project switcher"
    >
      {projects.map((p) => {
        const active = p.id === selectedId;
        return (
          <button
            key={p.id}
            type="button"
            data-testid={`project-switcher-${p.id}`}
            onClick={() => onSelect(p.id)}
            className={
              "group flex items-center gap-2 px-3 py-2 rounded border transition-colors whitespace-nowrap " +
              (active
                ? "border-accent/60 bg-accent/10 text-ink"
                : "border-line bg-bg-soft/40 text-ink-dim hover:border-accent/40 hover:text-ink")
            }
          >
            <span className={"font-mono text-[10px] " + (active ? "text-accent" : "text-ink-mute")}>
              {p.key}
            </span>
            <span className="text-sm font-medium">{p.name}</span>
            <span className="text-[10px] text-ink-mute font-mono">
              {p.member_count} members
            </span>
            {active && <ChevronRight size={12} className="text-accent" />}
          </button>
        );
      })}
    </div>
  );
}

// =====================================================================
// ProjectOverview — 元信息 + KPI (per task §1)
// =====================================================================
function ProjectOverview({
  project, owner, workspaces, kpis,
}: {
  project: Project;
  owner: Identity | null;
  workspaces: Workspace[];
  kpis: {
    open: number;
    closed: number;
    activeAgents: number;
    changeSets: number;
    worktrees: number;
    pullRequests: number;
    lastActivityTs: Iso8601 | undefined;
  };
}) {
  const role = deriveRole(project);
  const lastActivity = kpis.lastActivityTs
    ? new Date(kpis.lastActivityTs)
    : null;
  return (
    <div data-testid="projects-overview-tab" className="space-y-5">
      {/* ---- Metadata card ---- */}
      <div className="card">
        <div className="flex items-start justify-between gap-4 mb-3">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <span className="font-mono text-info">{project.key}</span>
              <h2 className="text-lg font-semibold">{project.name}</h2>
              <StatusPill value={project.visibility} size="xs" />
            </div>
            <p className="text-sm text-ink-dim">
              {project.member_count} members · created {new Date(project.created_at).toLocaleDateString()} · tenant {project.tenant_id}
            </p>
          </div>
          <div className="text-right text-[11px] text-ink-mute font-mono space-y-0.5">
            <div>project_id: <span className="text-ink-dim">{project.id}</span></div>
            <div>role (mock): <span className="text-accent">{role}</span></div>
          </div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
          <div>
            <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1">Owner</div>
            <div className="flex items-center gap-2">
              <span className="inline-flex items-center justify-center w-7 h-7 rounded-full bg-accent/15 border border-accent/30 text-accent text-xs font-mono">
                {(owner?.display_name ?? project.owner_id).slice(0, 2).toUpperCase()}
              </span>
              <div>
                <div className="text-sm font-medium">{owner?.display_name ?? project.owner_id}</div>
                {owner && <div className="text-[10px] text-ink-mute font-mono">{owner.email}</div>}
              </div>
            </div>
          </div>
          <div>
            <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1">Workspaces</div>
            <div className="space-y-1">
              {workspaces.length === 0 ? (
                <div className="text-xs text-ink-mute italic">(no workspaces)</div>
              ) : (
                workspaces.map((ws) => (
                  <div key={ws.id} className="flex items-center gap-2 text-xs">
                    <StatusPill value={ws.kind} size="xs" />
                    <span>{ws.name}</span>
                    <span className="text-ink-mute font-mono">{ws.member_ids.length} members</span>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>

      {/* ---- KPI cards ---- */}
      <div>
        <SectionTitle><Activity size={11} className="inline mr-1" /> KPIs</SectionTitle>
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
          <Stat label="Open Issues"    value={kpis.open}        tone="info" hint={`${kpis.closed} closed`} />
          <Stat label="Active Agents"  value={kpis.activeAgents} hint="agent sessions" tone="ok" />
          <Stat label="Worktrees"      value={kpis.worktrees}   tone="info" />
          <Stat label="ChangeSets"     value={kpis.changeSets}  tone="default" />
          <Stat label="Pull Requests"  value={kpis.pullRequests} tone="warn" />
          <Stat
            label="Last Activity"
            value={lastActivity ? format(lastActivity, "MM-dd HH:mm") : "—"}
            hint={lastActivity ? relativeTime(lastActivity) : "no activity"}
            tone="default"
          />
        </div>
      </div>

      {/* ---- Recent work-items (mini list) ---- */}
      <RecentWorkItems projectId={project.id} />
    </div>
  );
}

// =====================================================================
// RecentWorkItems — 最近更新的 work-items (per project_id 过滤)
// =====================================================================
function RecentWorkItems({ projectId }: { projectId: string }) {
  const workItems = useStore((s) => s.workItems);
  const recent = useMemo(
    () =>
      workItems
        .filter((w) => w.project_id === projectId)
        .slice()
        .sort((a, b) => (a.updated_at < b.updated_at ? 1 : -1))
        .slice(0, 8),
    [workItems, projectId],
  );
  return (
    <div>
      <SectionTitle><Clock size={11} className="inline mr-1" /> Recent Work-items</SectionTitle>
      <div className="card">
        {recent.length === 0 ? (
          <div className="text-xs text-ink-mute italic">(no work-items)</div>
        ) : (
          <table className="table">
            <thead>
              <tr>
                <th>Key</th>
                <th>Title</th>
                <th>Status</th>
                <th>Priority</th>
                <th>Updated</th>
              </tr>
            </thead>
            <tbody>
              {recent.map((w) => (
                <tr key={w.id} data-testid={`recent-wi-${w.id}`}>
                  <td className="font-mono text-info text-xs">{w.key}</td>
                  <td className="text-xs line-clamp-1 max-w-md">{w.title}</td>
                  <td><StatusPill value={w.status} size="xs" /></td>
                  <td><StatusPill value={w.priority} size="xs" /></td>
                  <td className="font-mono text-[10px] text-ink-mute">
                    {relativeTime(new Date(w.updated_at))}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}

// =====================================================================
// ProjectMembers — 团队成员 + 角色 (per task §5)
// =====================================================================
function ProjectMembers({
  project, members, workspaces, ownerId,
}: {
  project: Project;
  members: Identity[];
  workspaces: Workspace[];
  ownerId: string;
}) {
  const role = deriveRole(project);
  // 每个成员的 "角色" = 在该项目下涉及的 workspace.kind (scratch/shared/archived)
  // 简化: 多数 workspace kind 决定主角色, 多个时逗号拼接
  return (
    <div data-testid="projects-members-tab" className="space-y-3">
      <div className="card">
        <div className="flex items-center justify-between mb-2">
          <div>
            <div className="text-sm font-semibold">{project.name} — Members</div>
            <div className="text-[10px] text-ink-mute font-mono">
              {members.length} members · {workspaces.length} workspaces · derived role: {role}
            </div>
          </div>
          {/* 已知缺口 #4 提示 */}
          <span className="pill border-warn/40 text-warn bg-warn/10 text-[10px] font-mono">
            ⚠ role mock
          </span>
        </div>
        {members.length === 0 ? (
          <div className="text-xs text-ink-mute italic">(no members in this project)</div>
        ) : (
          <table className="table">
            <thead>
              <tr>
                <th>Member</th>
                <th>Provider</th>
                <th>Status</th>
                <th>Workspaces</th>
                <th>Role (mock)</th>
                <th>Last login</th>
              </tr>
            </thead>
            <tbody>
              {members.map((m) => {
                const userWorkspaces = workspaces.filter((w) => w.member_ids.includes(m.id));
                const isOwner = m.id === ownerId;
                // 角色推导: owner → project_admin, 跨多个 workspace 的 → developer, 单一 → viewer
                const userRole: string = isOwner
                  ? "project_admin"
                  : userWorkspaces.length > 1
                    ? "developer"
                    : "viewer";
                return (
                  <tr key={m.id} data-testid={`member-${m.id}`}>
                    <td>
                      <div className="flex items-center gap-2">
                        <span className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-accent/15 border border-accent/30 text-accent text-[10px] font-mono">
                          {m.display_name.slice(0, 2).toUpperCase()}
                        </span>
                        <div>
                          <div className="text-sm">{m.display_name}</div>
                          <div className="text-[10px] text-ink-mute font-mono">{m.email}</div>
                        </div>
                      </div>
                    </td>
                    <td className="text-xs">
                      <StatusPill value={m.provider} size="xs" />
                    </td>
                    <td><StatusPill value={m.status} size="xs" /></td>
                    <td className="text-[11px] text-ink-dim">
                      {userWorkspaces.length === 0 ? (
                        <span className="italic text-ink-mute">(none)</span>
                      ) : (
                        userWorkspaces.map((w) => w.name).join(", ")
                      )}
                    </td>
                    <td><StatusPill value={userRole} size="xs" /></td>
                    <td className="font-mono text-[10px] text-ink-mute">
                      {m.last_login_at ? new Date(m.last_login_at).toLocaleDateString() : "—"}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>
      {/* 已知缺口 #4 详细说明 */}
      <div className="text-[10px] text-ink-mute font-mono">
        ⚠ 角色 (project_admin / developer / viewer) 用 mock 推导: owner → admin, 跨多 workspace → developer, 单一 → viewer
        真实 rbac 接入 Phase I+ 接 backend permission API (per docs/frontend/design/... §RBAC)
      </div>
    </div>
  );
}

// =====================================================================
// relativeTime — "3 minutes ago" / "2 days ago" helper
// =====================================================================
function relativeTime(d: Date): string {
  const diff = Date.now() - d.getTime();
  if (diff < 60_000) return "just now";
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`;
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`;
  if (diff < 30 * 86_400_000) return `${Math.floor(diff / 86_400_000)}d ago`;
  return d.toLocaleDateString();
}
