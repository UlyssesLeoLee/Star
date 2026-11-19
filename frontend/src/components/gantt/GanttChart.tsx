"use client";

/*
@cypher schema=1 source_sha256=e8cfb839f4edb7c06a87f3c79531dc35f484632882de7b7db9f75f97fca1a647
MERGE (self:File {path:"frontend/src/components/gantt/GanttChart.tsx"})
MERGE (dateUtils:File {path:"frontend/src/lib/gantt/dateUtils.ts"})
MERGE (ganttChart:Function {name:"GanttChart", language:"typescript", visibility:"public"})
MERGE (toUtcCalendarDate:Function {name:"toUtcCalendarDate", language:"typescript", visibility:"public"})
MERGE (addUtcCalendarDays:Function {name:"addUtcCalendarDays", language:"typescript", visibility:"public"})
MERGE (self)-[:IMPORTS]->(dateUtils)
MERGE (ganttChart)-[:CALLS]->(toUtcCalendarDate)
MERGE (ganttChart)-[:CALLS]->(addUtcCalendarDays)
MERGE (openSprintMenu:Symbol {id:"frontend/src/components/gantt/GanttChart.tsx::openSprintMenu",kind:"function"})
MERGE (openMilestoneMenu:Symbol {id:"frontend/src/components/gantt/GanttChart.tsx::openMilestoneMenu",kind:"function"})
MERGE (openWorkItemMenu:Symbol {id:"frontend/src/components/gantt/GanttChart.tsx::openWorkItemMenu",kind:"function"})
MERGE (openRowLabelMenu:Symbol {id:"frontend/src/components/gantt/GanttChart.tsx::openRowLabelMenu",kind:"function"})
MERGE (openBlankMenu:Symbol {id:"frontend/src/components/gantt/GanttChart.tsx::openBlankMenu",kind:"function"})
MERGE (scrollToToday:Symbol {id:"frontend/src/components/gantt/GanttChart.tsx::scrollToToday",kind:"function"})
MERGE (zoomToFit:Symbol {id:"frontend/src/components/gantt/GanttChart.tsx::zoomToFit",kind:"function"})
MERGE (self)-[:DEFINES]->(openSprintMenu)
MERGE (self)-[:DEFINES]->(openMilestoneMenu)
MERGE (self)-[:DEFINES]->(openWorkItemMenu)
MERGE (self)-[:DEFINES]->(openRowLabelMenu)
MERGE (self)-[:DEFINES]->(openBlankMenu)
MERGE (self)-[:DEFINES]->(scrollToToday)
MERGE (self)-[:DEFINES]->(zoomToFit)
MERGE (self)-[:DEFINES]->(ganttChart)
@endcypher
*/

// =====================================================================
// GanttChart — 主容器 (per W2 任务 §1 + design §4.3)
//
// props:
//   sprints, milestones, workItems — 数据
//   dateRange                       — { start, end } ISO 字符串
//   onMilestoneUpdate, onSprintUpdate, onWorkItemMove — 拖动回调
//
// 顶部缩放: week / month / quarter (CSS grid 列宽)
// 左侧 Y 轴: sprint 行 + milestone 行 (per design §4.2)
// X 轴: 日期 header (GanttHeader 渲染)
// 网格线: 浅色虚线 (1px border-line/30) — header 列分隔
// 关键路径 (critical path): progress < 50% 的 milestones 红色 (per design §4.2)
//
// 跨模块联动 (per W2 任务 §3):
//   - milestone onClick -> router.push("/work-item?milestone={id}")
//   - 拖动 milestone 改 due_date -> onMilestoneUpdate -> stub 1s 后 /api/audit
//   - 跨 sprint 拖 work-item -> onWorkItemMove
//
// 浮动窗口 (per 2026-08-31 用户需求):
//   - 右上角 ⛶ 按钮 / 双击图表空白区打开浮动窗口
//   - 浮动窗口内完整甘特图 (同 props, 可拖拽缩放精细操作)
// =====================================================================

import { useMemo, useState, useCallback, useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { useRouter } from "next/navigation";
import { differenceInDays, parseISO } from "date-fns";
import toast from "react-hot-toast";
import { addUtcCalendarDays, toUtcCalendarDate } from "@/lib/gantt/dateUtils";
import type { Sprint, Milestone, WorkItem, WorkItemStatus, SprintStatus } from "@/types/ids";
import { GanttBar, type GanttBarStatus } from "./GanttBar";
import { GanttHeader } from "./GanttHeader";
import { GanttLegend } from "./GanttLegend";
import { useTranslation, interpolate } from "@/lib/i18n";
import { ContextMenu, type ContextMenuItem } from "../context-menu/ContextMenu";
import { useContextMenu } from "../context-menu/useContextMenu";
import { CONTEXT_MENU_MESSAGES, uiLangOf, type UiLang } from "../context-menu/messages";
import { copyText } from "../context-menu/clipboard";
import { transitionMilestone, transitionWorkItemSprint } from "./ganttActions";

/** 甘特右键菜单文案 (三语; 不碰共享 dictionary, 按 useTranslation().language 取值) */
interface GanttMenuLabels {
  viewSprint: string;
  viewMilestone: string;
  openDetail: string;
  copyKey: string;
  changeDueDate: string;
  goToToday: string;
  moveToSprint: string;
  noOtherSprint: string;
  collapse: string;
  expand: string;
  collapseUnavailable: string;
  zoomToFitAll: string;
  dueDateRecorded: string;
  sprintMoveRecorded: string;
  copied: string;
}

const GANTT_MENU_LABELS: Record<UiLang, GanttMenuLabels> = {
  "zh-CN": {
    viewSprint: "查看 sprint",
    viewMilestone: "查看里程碑",
    openDetail: "打开详情",
    copyKey: "复制 key",
    changeDueDate: "改截止日期",
    goToToday: "回到今天",
    moveToSprint: "移到 sprint",
    noOtherSprint: "没有其他 sprint 可选",
    collapse: "折叠",
    expand: "展开",
    collapseUnavailable: "折叠/展开状态待接入",
    zoomToFitAll: "缩放到全部",
    dueDateRecorded: "已记录: 里程碑改期待服务端接入 (当前可直接拖动里程碑条形改期)",
    sprintMoveRecorded: "已记录: 任务移动待服务端接入 (当前可拖拽任务条跨 sprint)",
    copied: "已复制到剪贴板",
  },
  en: {
    viewSprint: "View sprint",
    viewMilestone: "View milestone",
    openDetail: "Open detail",
    copyKey: "Copy key",
    changeDueDate: "Change due date",
    goToToday: "Go to today",
    moveToSprint: "Move to sprint",
    noOtherSprint: "No other sprint available",
    collapse: "Collapse",
    expand: "Expand",
    collapseUnavailable: "Collapse/expand state not wired yet",
    zoomToFitAll: "Zoom to fit all",
    dueDateRecorded: "Recorded: milestone due-date change awaits backend (drag the milestone bar to reschedule now)",
    sprintMoveRecorded: "Recorded: work-item move awaits backend (drag the bar across sprints now)",
    copied: "Copied to clipboard",
  },
  ja: {
    viewSprint: "スプリントを表示",
    viewMilestone: "マイルストーンを表示",
    openDetail: "詳細を開く",
    copyKey: "キーをコピー",
    changeDueDate: "期日を変更",
    goToToday: "今日へ移動",
    moveToSprint: "スプリントへ移動",
    noOtherSprint: "他のスプリントがありません",
    collapse: "折りたたむ",
    expand: "展開",
    collapseUnavailable: "折りたたみ/展開は未接続",
    zoomToFitAll: "全体を表示",
    dueDateRecorded: "記録済み: 期日変更はバックエンド接続待ち (バーをドラッグして今は変更可能)",
    sprintMoveRecorded: "記録済み: タスク移動はバックエンド接続待ち (バーをドラッグして移動可能)",
    copied: "クリップボードにコピーしました",
  },
};

export type ZoomLevel = "week" | "month" | "quarter";

export interface GanttChartProps {
  sprints: Sprint[];
  milestones: Milestone[];
  workItems: WorkItem[];
  /** 任务依赖关系 (per MS Project task link), 用于渲染 SVG 箭头 + 检测拖动冲突 */
  relations?: import("@/types/ids").Relation[];
  dateRange: { start: string; end: string };
  onMilestoneUpdate?: (id: string, newDueDate: string) => void;
  onSprintUpdate?: (id: string, newStart: string, newEnd: string) => void;
  onWorkItemMove?: (workItemId: string, newSprintId: string) => void;
}

const PX_PER_DAY: Record<ZoomLevel, number> = {
  week: 60,    // 1 day = 60px
  month: 20,   // 1 day = 20px
  quarter: 8,  // 1 day = 8px
};

const ZOOM_ORDER: ZoomLevel[] = ["week", "month", "quarter"];

function sprintStatusToBarStatus(s: SprintStatus): GanttBarStatus {
  // sprint 状态映射 (sprint 用同一套颜色)
  return s;
}

function workItemStatusToBarStatus(s: WorkItemStatus): GanttBarStatus {
  return s;
}

export function GanttChart(props: GanttChartProps) {
  const {
    sprints, milestones, workItems, relations = [], dateRange,
    onMilestoneUpdate, onSprintUpdate, onWorkItemMove,
  } = props;

  const router = useRouter();
  // Default zoom "week" (60 px/day)
  const [zoom, setZoom] = useState<ZoomLevel>("week");
  // Floating modal state (per 2026-08-31 user request)
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [modalZoom, setModalZoom] = useState<ZoomLevel>("week");
  // Sync modalZoom to base zoom when modal opens
  const openModal = useCallback(() => {
    setModalZoom(zoom);
    setIsModalOpen(true);
  }, [zoom]);
  const closeModal = useCallback(() => setIsModalOpen(false), []);
  // Close modal on Escape key
  useEffect(() => {
    if (!isModalOpen) return;
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") closeModal(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [isModalOpen, closeModal]);
  useEffect(() => {
    if (typeof window === "undefined") return;
    const saved = window.localStorage.getItem("star.gantt.zoom");
    if (saved && (saved === "week" || saved === "month" || saved === "quarter")) {
      setZoom(saved as ZoomLevel);
    }
  }, []);
  useEffect(() => {
    if (typeof window === "undefined") return;
    window.localStorage.setItem("star.gantt.zoom", zoom);
  }, [zoom]);
  const pxPerDay = PX_PER_DAY[zoom];

  const start = parseISO(toUtcCalendarDate(dateRange.start));
  const end = parseISO(toUtcCalendarDate(dateRange.end));
  const totalDays = Math.max(1, differenceInDays(end, start) + 1);
  const totalWidth = totalDays * pxPerDay;

  // critical path: progress < 50% 的 milestones (self define per design §4.2)
  const criticalIds = useMemo(() => {
    return new Set(milestones.filter((m) => m.progress < 0.5).map((m) => m.id));
  }, [milestones]);

  // 任务依赖 link 渲染 (per MS Project task link 设计, 2026-08-29 17:33 JST)
  // 计算每条 work_item->work_item blocks/relates_to/duplicates 关系的源点 / 终点坐标
  // 行高: sprint row 12px (h-12), milestone row 10px (h-10), 行间距 8px
  const ROW = { sprint: 48, ms: 40, gap: 8 } as const;
  const rowOffsets = useMemo(() => {
    // 按 id 找行, sprint 用 sprint_id, milestone 用 milestone row index
    // 但 workItem link 可能跨 sprint, 行高用 rowIndex 算
    const wiById = new Map(workItems.map((w) => [w.id, w]));
    const sprintById = new Map(sprints.map((s) => [s.id, s]));
    const sprintIdx = new Map(sprints.map((s, i) => [s.id, i]));
    const msIdx = new Map(milestones.map((m, i) => [m.id, i]));

    // 对每个 workItem, 它在 gantt 的 y 坐标 = sprint 行 (因为 wi 是 sprint 的子元素, 但实际
    // 渲染在 sprint 行内顶部, 这里用 sprint row top)
    // 但 wi 有 sprint_id 也有 start_date/end_date, 跨 sprint 拖动时用 sprint 索引
    const chartStart = parseISO(toUtcCalendarDate(dateRange.start));

    const result: Record<string, { x: number; y: number; end_x: number; rowTop: number }> = {};
    for (const sp of sprints) {
      const i = sprintIdx.get(sp.id) ?? 0;
      const top = i * (ROW.sprint + ROW.gap);
      const spStart = parseISO(toUtcCalendarDate(sp.start_date));
      const spEnd = parseISO(addUtcCalendarDays(sp.end_date, 1));
      result[sp.id] = {
        x: Math.max(0, differenceInDays(spStart, chartStart)) * pxPerDay,
        y: top + ROW.sprint / 2,
        end_x: Math.max(0, differenceInDays(spEnd, chartStart)) * pxPerDay,
        rowTop: top,
      };
    }
    for (const ms of milestones) {
      const i = msIdx.get(ms.id) ?? 0;
      const top = sprints.length * (ROW.sprint + ROW.gap) + ROW.gap + i * (ROW.ms + ROW.gap);
      const msDate = parseISO(toUtcCalendarDate(ms.due_date));
      result[ms.id] = {
        x: Math.max(0, differenceInDays(msDate, chartStart)) * pxPerDay,
        y: top + ROW.ms / 2,
        end_x: result[ms.id]?.x ?? 0,
        rowTop: top,
      };
    }
    return result;
  }, [workItems, sprints, milestones, dateRange.start, pxPerDay]);

  // SVG link 列表: 从 relations 过滤 work_item->work_item + work_item->milestone + milestone->work_item
  // 用 FS (Finish-to-Start) 风格箭头
  const links = useMemo(() => {
    const out: Array<{
      id: string;
      fromId: string;
      toId: string;
      kind: string;
      fromX: number;
      fromY: number;
      toX: number;
      toY: number;
    }> = [];
    for (const r of relations) {
      if (r.kind === "parent_of" || r.kind === "cloned_from") continue; // 父级关系, 不画箭头
      const from = rowOffsets[r.from_id];
      const to = rowOffsets[r.to_id];
      if (!from || !to) continue;
      out.push({
        id: r.id,
        fromId: r.from_id,
        toId: r.to_id,
        kind: r.kind,
        fromX: from.end_x,  // 箭头从 from.end 出发 (FS 风格)
        fromY: from.y,
        toX: to.x,         // 箭头指向 to.start
        toY: to.y,
      });
    }
    return out;
  }, [relations, rowOffsets]);

  // 拖动冲突检测 (per MS Project FS link): workItem 新 start 不能早于所有 predecessor.end
  // (per 2026-08-29 17:33 JST 增强, 避免 dependency 冲突)
  // 注: useTranslation hook 在外面拿, 这里直接调 t.gantt.conflictPredecessor
  // 整个 hook 链需要 ref 因为 conflict message 依赖语言 (per 2026-08-31 v0.3)
  const { t: i18nT, language } = useTranslation();
  const lang = uiLangOf(language);
  const L = GANTT_MENU_LABELS[lang];
  const msg = CONTEXT_MENU_MESSAGES[lang];
  const checkWorkItemConflict = useCallback(
    (workItemId: string, newStart: string, _newEnd: string): string | null => {
      // 找 workItem 的所有 predecessor
      const preds = relations.filter(
        (r) => r.to_id === workItemId && (r.kind === "blocks" || r.kind === "relates_to"),
      );
      if (preds.length === 0) return null;
      for (const p of preds) {
        // predecessor 是 work_item, 找它的 sprint (wi.start_date = sp.start_date, wi.end_date = sp.end_date)
        const wi = workItems.find((w) => w.id === p.from_id);
        if (!wi || !wi.sprint_id) continue;
        const sp = sprints.find((s) => s.id === wi.sprint_id);
        if (!sp) continue;
        if (newStart < sp.end_date) {
          return interpolate(i18nT.gantt.conflictPredecessor, {
            key: wi.key || wi.id,
            name: sp.name,
            date: sp.end_date,
          });
        }
      }
      return null;
    },
    [relations, workItems, sprints, i18nT.gantt.conflictPredecessor],
  );

  // work items 按 sprint_id 分组 (跨 sprint 拖动)
  const workItemsBySprint = useMemo(() => {
    const map: Record<string, WorkItem[]> = {};
    for (const wi of workItems) {
      if (wi.sprint_id) {
        (map[wi.sprint_id] ||= []).push(wi);
      }
    }
    return map;
  }, [workItems]);

  const handleSprintClick = useCallback(
    (sprintId: string) => {
      router.push(`/work-item?sprint=${sprintId}`);
    },
    [router],
  );

  const handleMilestoneClick = useCallback(
    (milestoneId: string) => {
      router.push(`/work-item?milestone=${milestoneId}`);
    },
    [router],
  );

  // =====================================================================
  // 右键情景菜单 (per canvas-ux 任务 §E)
  // - stub 动作 (transitionMilestone / transitionWorkItemSprint) 只记录不冒充写入,
  //   触发后给诚实 toast; 纯前端动作 (查看/复制/定位/缩放) 直接完成。
  // - 折叠/展开无现有 state → disabled + 原因, 报告列父代理待办。
  // =====================================================================
  const { menu, openMenu, closeMenu } = useContextMenu();
  const timelineRef = useRef<HTMLDivElement>(null);

  /** 时间线滚动到今天 (纯前端) */
  const scrollToToday = useCallback(() => {
    const container = timelineRef.current;
    if (!container) return;
    const offsetPx = Math.max(0, differenceInDays(new Date(), start)) * pxPerDay;
    container.scrollLeft = Math.max(0, offsetPx - 60);
  }, [start, pxPerDay]);

  /** 缩放到全部: 选一个 pxPerDay ≤ 容器宽/总天数的缩放级别 (纯前端) */
  const zoomToFit = useCallback(() => {
    const container = timelineRef.current;
    const width = container?.clientWidth ?? 1200;
    const level = ZOOM_ORDER.find((z) => PX_PER_DAY[z] * totalDays <= width) ?? "quarter";
    setZoom(level);
  }, [totalDays]);

  /** 复制 key/id 到剪贴板 + 诚实 toast */
  const copyKeyWithToast = useCallback(async (key: string) => {
    const ok = await copyText(key);
    if (ok) toast.success(L.copied);
    else toast.error(msg.copyFailed);
  }, [L, msg]);

  /** sprint 条右键: 查看 / 复制 key (Sprint 类型无 key 字段 → 有则取, 无则回退 id) */
  const openSprintMenu = useCallback((e: React.MouseEvent, sp: Sprint) => {
    e.preventDefault();
    e.stopPropagation();
    const key = (sp as Sprint & { key?: string }).key ?? sp.id;
    const items: ContextMenuItem[] = [
      { id: "view-sprint", label: L.viewSprint, onSelect: () => handleSprintClick(sp.id) },
      { id: "copy-sprint-key", label: L.copyKey, onSelect: () => void copyKeyWithToast(key) },
    ];
    openMenu(e, items);
  }, [L, handleSprintClick, copyKeyWithToast, openMenu]);

  /** 里程碑条右键: 查看 / 复制 key / 改截止日期 (stub + 诚实 toast) / 定位到今天 */
  const openMilestoneMenu = useCallback((e: React.MouseEvent, ms: Milestone) => {
    e.preventDefault();
    e.stopPropagation();
    const key = (ms as Milestone & { key?: string }).key ?? ms.id;
    const items: ContextMenuItem[] = [
      { id: "view-milestone", label: L.viewMilestone, onSelect: () => handleMilestoneClick(ms.id) },
      { id: "copy-milestone-key", label: L.copyKey, onSelect: () => void copyKeyWithToast(key) },
      {
        id: "change-due-date",
        label: L.changeDueDate,
        onSelect: () => {
          transitionMilestone(ms.id, ms.due_date);
          toast(L.dueDateRecorded);
        },
      },
      { id: "go-to-today", label: L.goToToday, onSelect: () => scrollToToday() },
    ];
    openMenu(e, items);
  }, [L, handleMilestoneClick, copyKeyWithToast, scrollToToday, openMenu]);

  /** 任务条 (work-item mini bar) 右键: 打开详情 / 复制 key / 移到各 sprint (stub + 诚实 toast) */
  const openWorkItemMenu = useCallback((e: React.MouseEvent, wi: WorkItem) => {
    e.preventDefault();
    e.stopPropagation();
    const others = sprints.filter((s) => s.id !== wi.sprint_id);
    const moveItems: ContextMenuItem[] = others.length === 0
      ? [{ id: "move-to-sprint-none", label: `${L.moveToSprint} — ${L.noOtherSprint}`, disabled: true, disabledReason: L.noOtherSprint }]
      : others.map((s) => ({
          id: `move-to-sprint-${s.id}`,
          label: `${L.moveToSprint} → ${s.name}`,
          onSelect: () => {
            transitionWorkItemSprint(wi.id, s.id);
            toast(L.sprintMoveRecorded);
          },
        }));
    const items: ContextMenuItem[] = [
      { id: "open-work-item", label: L.openDetail, onSelect: () => router.push(`/work-item?selected=${wi.id}`) },
      { id: "copy-work-item-key", label: L.copyKey, onSelect: () => void copyKeyWithToast(wi.key ?? wi.id) },
      { id: "sep-move", label: "", separator: true },
      ...moveItems,
    ];
    openMenu(e, items);
  }, [L, sprints, router, copyKeyWithToast, openMenu]);

  /** 行标签右键: 折叠/展开无现有 state → disabled + 诚实原因 (父代理待办) */
  const openRowLabelMenu = useCallback((e: React.MouseEvent, kind: "sprint" | "milestone", targetId: string) => {
    e.preventDefault();
    e.stopPropagation();
    const items: ContextMenuItem[] = [
      kind === "sprint"
        ? { id: "view-sprint", label: L.viewSprint, onSelect: () => handleSprintClick(targetId) }
        : { id: "view-milestone", label: L.viewMilestone, onSelect: () => handleMilestoneClick(targetId) },
      { id: "sep-collapse", label: "", separator: true },
      { id: "collapse-row", label: L.collapse, disabled: true, disabledReason: L.collapseUnavailable },
      { id: "expand-row", label: L.expand, disabled: true, disabledReason: L.collapseUnavailable },
    ];
    openMenu(e, items);
  }, [L, handleSprintClick, handleMilestoneClick, openMenu]);

  /** 甘特空白右键: 缩放到全部 / 回到今天 (纯前端) */
  const openBlankMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    // bar / work-item 条上不冒泡到空白菜单 (它们的 handler 已 stopPropagation, 这里兜底)
    const target = e.target as HTMLElement | null;
    if (target?.closest("[data-gantt-bar], [data-work-item-id], [data-row-label-kind]")) return;
    const items: ContextMenuItem[] = [
      { id: "zoom-to-fit-all", label: L.zoomToFitAll, onSelect: () => zoomToFit() },
      { id: "go-to-today", label: L.goToToday, onSelect: () => scrollToToday() },
    ];
    openMenu(e, items);
  }, [L, zoomToFit, scrollToToday, openMenu]);

  return (
    <>
    <div className="card overflow-hidden" data-testid="gantt-chart" data-zoom={zoom}>
      {/* Toolbar */}
      <div className="flex items-center justify-between mb-3 gap-2 flex-wrap">
        <div className="flex items-center gap-2">
          <span className="text-[10px] uppercase tracking-wider text-ink-mute">{i18nT.gantt.zoom}</span>
          {ZOOM_ORDER.map((z) => (
            <button
              key={z}
              type="button"
              onClick={() => setZoom(z)}
              className={`px-2 py-1 text-[10px] font-mono rounded border transition-colors ${
                zoom === z
                  ? "border-accent bg-accent/10 text-accent"
                  : "border-line text-ink-dim hover:border-ink-dim"
              }`}
              data-zoom-button={z}
              data-active={zoom === z ? "true" : "false"}
            >
              {z}
            </button>
          ))}
          <span className="ml-2 text-[10px] text-ink-mute font-mono">
            {interpolate(i18nT.gantt.zoomUnit, {
              totalDays,
              pxPerDay,
              totalWidth,
            })}
          </span>
          {links.length > 0 && (
            <span
              data-testid="gantt-link-count"
              className="ml-2 text-[10px] font-mono text-accent border border-accent/40 bg-accent/10 px-1.5 py-0.5 rounded"
              title={interpolate(i18nT.gantt.linkCountTitle, { count: links.length })}
            >
              {interpolate(links.length === 1 ? i18nT.gantt.linkCountSingular : i18nT.gantt.linkCount, {
                count: links.length,
              })}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <GanttLegend />
          {/* 展开浮动窗口按钮 (per 2026-08-31 user request) */}
          <button
            type="button"
            data-testid="gantt-expand-btn"
            onClick={openModal}
            title={i18nT.gantt.expandTitle}
            className="flex items-center justify-center w-7 h-7 rounded border border-line text-ink-dim hover:border-accent hover:text-accent hover:bg-accent/10 transition-colors text-[14px]"
          >
            {i18nT.gantt.expandModal}
          </button>
        </div>
      </div>

      <div className="flex border border-line rounded overflow-hidden bg-bg-soft/30">
        {/* Y-axis (fixed 200px) */}
        <div
          className="w-[200px] flex-shrink-0 border-r border-line bg-bg-soft/50 select-none"
          data-testid="gantt-yaxis"
        >
          <div className="h-10 border-b border-line bg-bg-soft/80 flex items-center px-2 text-[10px] uppercase tracking-wider text-ink-mute">
            {i18nT.gantt.sprintsHeader}
          </div>
          {sprints.map((sp) => (
            <div
              key={`y-s-${sp.id}`}
              data-row-label-kind="sprint"
              data-row-label-id={sp.id}
              className="h-12 px-2 py-1 border-b border-line/50 flex flex-col justify-center cursor-pointer hover:bg-bg-soft/60"
              onClick={() => handleSprintClick(sp.id)}
              onContextMenu={(e) => openRowLabelMenu(e, "sprint", sp.id)}
            >
              <div className="text-[11px] font-medium text-ink truncate">{sp.name}</div>
              <div className="text-[9px] text-ink-mute truncate">{sp.status}</div>
            </div>
          ))}
          <div className="h-2 bg-bg-soft/80" />
          <div className="h-7 border-b border-line bg-bg-soft/80 flex items-center px-2 text-[10px] uppercase tracking-wider text-ink-mute">
            {i18nT.gantt.milestonesHeader}
          </div>
          {milestones.map((ms) => {
            const critical = criticalIds.has(ms.id);
            return (
              <div
                key={`y-m-${ms.id}`}
                data-row-label-kind="milestone"
                data-row-label-id={ms.id}
                data-row-critical={critical ? "true" : "false"}
                className="h-10 px-2 py-1 border-b border-line/50 flex flex-col justify-center cursor-pointer hover:bg-bg-soft/60"
                onClick={() => handleMilestoneClick(ms.id)}
                onContextMenu={(e) => openRowLabelMenu(e, "milestone", ms.id)}
              >
                <div className={`text-[11px] font-medium truncate ${critical ? "text-err" : "text-ink"}`}>
                  ◆ {ms.name}
                </div>
                <div className="text-[9px] text-ink-mute truncate">
                  {Math.round(ms.progress * 100)}% · due {ms.due_date.slice(5)}
                </div>
              </div>
            );
          })}
        </div>

        {/* Timeline area (scrollable) — 双击空白区打开浮动窗口 */}
        <div
          ref={timelineRef}
          className="flex-1 overflow-x-auto"
          data-testid="gantt-timeline"
          onDoubleClick={(e) => {
            // Only open if dbl-click landed on the bg row, not on a bar/label
            const target = e.target as HTMLElement;
            const isBar = target.closest("[data-gantt-bar]") || target.closest("[data-work-item-id]");
            if (!isBar) openModal();
          }}
          onContextMenu={openBlankMenu}
        >
          <div style={{ width: totalWidth, minWidth: "100%" }}>
            <GanttHeader
              start={start}
              totalDays={totalDays}
              pxPerDay={pxPerDay}
              zoom={zoom}
              onContextMenu={(e) => {
                // 列头右键: 折叠/展开无现有 state → disabled + 诚实原因
                e.preventDefault();
                openMenu(e, [
                  { id: "collapse-header", label: L.collapse, disabled: true, disabledReason: L.collapseUnavailable },
                  { id: "expand-header", label: L.expand, disabled: true, disabledReason: L.collapseUnavailable },
                ]);
              }}
            />

            {/* Sprint rows */}
            {sprints.map((sp) => {
              const wiList = workItemsBySprint[sp.id] ?? [];
              return (
                <div
                  key={`tl-s-${sp.id}`}
                  data-row-kind="sprint"
                  data-row-id={sp.id}
                  className="h-12 border-b border-line/50 relative bg-bg-card/30"
                >
                  <GanttBar
                    item={{
                      id: sp.id,
                      label: sp.name,
                      status: sprintStatusToBarStatus(sp.status),
                    }}
                    startDate={toUtcCalendarDate(sp.start_date)}
                    endDate={toUtcCalendarDate(sp.end_date)}
                    dateRangeStart={toUtcCalendarDate(dateRange.start)}
                    pxPerDay={pxPerDay}
                    variant="sprint"
                    onCheckConflict={undefined /* sprint 不检查 predecessor, 见 workItem 渲染分支 */}
                    onClick={() => handleSprintClick(sp.id)}
                    onContextMenu={(e) => openSprintMenu(e, sp)}
                    onDragEnd={(newStart, newEnd) => {
                      onSprintUpdate?.(sp.id, newStart, newEnd);
                    }}
                  />
                  {/* work items: 在 sprint 行底部小条 (供跨 sprint 拖) */}
                  {wiList.slice(0, 4).map((wi, i) => (
                    <div
                      key={`wi-${wi.id}`}
                      data-work-item-id={wi.id}
                      data-work-item-status={wi.status}
                      draggable
                      onDragStart={(e) => {
                        e.dataTransfer.setData("text/work-item-id", wi.id);
                        e.dataTransfer.effectAllowed = "move";
                      }}
                      onContextMenu={(e) => openWorkItemMenu(e, wi)}
                      className="absolute h-3 text-[8px] text-white px-1 cursor-grab rounded-sm select-none"
                      style={{
                        top: 32,
                        left: 6 + i * 90,
                        width: 84,
                        backgroundColor:
                          wi.status === "done" ? "#3fb950" :
                          wi.status === "in_progress" ? "#2f81f7" :
                          wi.status === "blocked" ? "#f85149" :
                          wi.status === "review" ? "#d29922" : "#6e7681",
                        lineHeight: "12px",
                      }}
                      title={`${wi.key} — ${wi.title} (拖到其他 sprint 改 sprint_id)`}
                    >
                      {wi.key}
                    </div>
                  ))}
                </div>
              );
            })}

            {/* Spacer between sprints and milestones */}
            <div className="h-2 bg-bg-soft/60" />

            {/* Milestone rows */}
            {milestones.map((ms) => {
              const critical = criticalIds.has(ms.id);
              return (
                <div
                  key={`tl-m-${ms.id}`}
                  data-row-kind="milestone"
                  data-row-id={ms.id}
                  data-row-critical={critical ? "true" : "false"}
                  className="h-10 border-b border-line/50 relative bg-bg-card/30"
                  onDragOver={(e) => {
                    e.preventDefault();
                    e.dataTransfer.dropEffect = "move";
                  }}
                  onDrop={(e) => {
                    e.preventDefault();
                    const wiId = e.dataTransfer.getData("text/work-item-id");
                    if (wiId && onWorkItemMove) {
                      // milestone 行接 work-item drop = 把 wi 关联到 milestone (TODO: 真实 attr)
                      onWorkItemMove(wiId, ms.id);
                    }
                  }}
                >
                  <GanttBar
                    item={{
                      id: ms.id,
                      label: ms.name,
                      status: critical ? "blocked" : "done",
                    }}
                    startDate={toUtcCalendarDate(ms.due_date)}
                    endDate={toUtcCalendarDate(ms.due_date)}
                    dateRangeStart={toUtcCalendarDate(dateRange.start)}
                    pxPerDay={pxPerDay}
                    variant="milestone"
                    isCritical={critical}
                    onClick={() => handleMilestoneClick(ms.id)}
                    onContextMenu={(e) => openMilestoneMenu(e, ms)}
                    onDragEnd={(newStart) => {
                      onMilestoneUpdate?.(ms.id, newStart);
                    }}
                  />
                </div>
              );
            })}

            {/* SVG link 渲染层 (per MS Project task link, 2026-08-29 17:33 JST) */}
            <svg
              data-testid="gantt-link-layer"
              width={totalWidth}
              height={(sprints.length * (ROW.sprint + ROW.gap)) + ROW.gap + (milestones.length * (ROW.ms + ROW.gap))}
              style={{ position: "absolute", top: 0, left: 0, pointerEvents: "none" }}
            >
              <defs>
                {/* 箭头 marker (per MS Project FS link) */}
                <marker
                  id="gantt-arrow-blocks"
                  viewBox="0 0 8 8"
                  refX="6"
                  refY="4"
                  markerWidth="6"
                  markerHeight="6"
                  orient="auto-start-reverse"
                >
                  <path d="M 0 0 L 8 4 L 0 8 z" fill="#f85149" />
                </marker>
                <marker
                  id="gantt-arrow-relates"
                  viewBox="0 0 8 8"
                  refX="6"
                  refY="4"
                  markerWidth="6"
                  markerHeight="6"
                  orient="auto-start-reverse"
                >
                  <path d="M 0 0 L 8 4 L 0 8 z" fill="#6e7681" />
                </marker>
                <marker
                  id="gantt-arrow-duplicates"
                  viewBox="0 0 8 8"
                  refX="6"
                  refY="4"
                  markerWidth="6"
                  markerHeight="6"
                  orient="auto-start-reverse"
                >
                  <path d="M 0 0 L 8 4 L 0 8 z" fill="#d29922" />
                </marker>
              </defs>
              {links.map((l) => {
                // 绘制 L 型折线: 从 (fromX, fromY) 水平到中点, 然后垂直到 toY, 水平到 toX
                const midX = (l.fromX + l.toX) / 2;
                const color = l.kind === "blocks" ? "#f85149" :
                              l.kind === "duplicates" ? "#d29922" : "#6e7681";
                const marker = l.kind === "blocks" ? "url(#gantt-arrow-blocks)" :
                              l.kind === "duplicates" ? "url(#gantt-arrow-duplicates)" :
                              "url(#gantt-arrow-relates)";
                return (
                  <g key={l.id} data-link-id={l.id} data-link-kind={l.kind}>
                    <path
                      d={`M ${l.fromX} ${l.fromY} L ${midX} ${l.fromY} L ${midX} ${l.toY} L ${l.toX} ${l.toY}`}
                      fill="none"
                      stroke={color}
                      strokeWidth="1.5"
                      strokeDasharray={l.kind === "relates_to" ? "4 2" : "none"}
                      markerEnd={marker}
                    />
                  </g>
                );
              })}
            </svg>
          </div>
        </div>
      </div>
    </div>

    {/* ── Floating Gantt Modal (portal) ── */}
    {isModalOpen && typeof document !== "undefined" && createPortal(
      <div
        role="dialog"
        aria-modal="true"
        aria-label="甘特图详细视图"
        data-testid="gantt-float-modal"
        className="fixed inset-0 z-[9999] flex items-center justify-center"
        style={{ backdropFilter: "blur(8px)", WebkitBackdropFilter: "blur(8px)", backgroundColor: "rgba(6,8,14,0.72)" }}
        onClick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
      >
        <div
          className="anime-panel relative flex flex-col"
          style={{
            width: "92vw",
            height: "86vh",
            maxWidth: 1600,
            borderRadius: "var(--radius-xl, 16px)",
            boxShadow: "0 0 48px rgba(0,240,255,0.18), 0 8px 48px rgba(0,0,0,0.7)",
            overflow: "hidden",
          }}
          onClick={(e) => e.stopPropagation()}
        >
          {/* Modal title bar */}
          <div className="flex items-center justify-between px-4 py-2 border-b border-line bg-bg-soft/60 flex-shrink-0">
            <div className="flex items-center gap-3">
              <span className="text-xs font-bold text-accent tracking-wide uppercase">甘特图 · 详细视图</span>
              <span className="text-[10px] text-ink-mute">{sprints.length} sprints · {milestones.length} milestones</span>
              {/* Modal zoom controls */}
              <div className="flex items-center gap-1 ml-3">
                <span className="text-[10px] uppercase tracking-wider text-ink-mute">Zoom</span>
                {ZOOM_ORDER.map((z) => (
                  <button
                    key={z}
                    type="button"
                    onClick={() => setModalZoom(z)}
                    className={`px-2 py-0.5 text-[10px] font-mono rounded border transition-colors ${
                      modalZoom === z
                        ? "border-accent bg-accent/10 text-accent"
                        : "border-line text-ink-dim hover:border-ink-dim"
                    }`}
                  >
                    {z}
                  </button>
                ))}
              </div>
            </div>
            <button
              type="button"
              data-testid="gantt-modal-close"
              onClick={closeModal}
              title="关闭 (Esc)"
              className="flex items-center justify-center w-7 h-7 rounded border border-line text-ink-dim hover:border-err hover:text-err hover:bg-err/10 transition-colors text-[13px] font-bold"
            >
              ✕
            </button>
          </div>

          {/* Modal body — scrollable Gantt */}
          <div className="flex-1 overflow-hidden flex flex-col min-h-0">
            {(() => {
              const mPxPerDay = PX_PER_DAY[modalZoom];
              const mTotalWidth = totalDays * mPxPerDay;
              return (
                <div className="flex flex-1 border-0 overflow-hidden" style={{ minHeight: 0 }}>
                  {/* Y-axis */}
                  <div className="w-[220px] flex-shrink-0 border-r border-line bg-bg-soft/50 overflow-y-auto">
                    <div className="h-10 border-b border-line bg-bg-soft/80 flex items-center px-2 text-[10px] uppercase tracking-wider text-ink-mute sticky top-0">
                      Sprints
                    </div>
                    {sprints.map((sp) => (
                      <div
                        key={`my-s-${sp.id}`}
                        className="h-12 px-2 py-1 border-b border-line/50 flex flex-col justify-center cursor-pointer hover:bg-bg-soft/60"
                        onClick={() => handleSprintClick(sp.id)}
                      >
                        <div className="text-[11px] font-medium text-ink truncate">{sp.name}</div>
                        <div className="text-[9px] text-ink-mute truncate">{sp.status}</div>
                      </div>
                    ))}
                    <div className="h-2 bg-bg-soft/80" />
                    <div className="h-7 border-b border-line bg-bg-soft/80 flex items-center px-2 text-[10px] uppercase tracking-wider text-ink-mute sticky top-10">
                      Milestones
                    </div>
                    {milestones.map((ms) => {
                      const critical = criticalIds.has(ms.id);
                      return (
                        <div
                          key={`my-m-${ms.id}`}
                          className="h-10 px-2 py-1 border-b border-line/50 flex flex-col justify-center cursor-pointer hover:bg-bg-soft/60"
                          onClick={() => handleMilestoneClick(ms.id)}
                        >
                          <div className={`text-[11px] font-medium truncate ${critical ? "text-err" : "text-ink"}`}>
                            ◆ {ms.name}
                          </div>
                          <div className="text-[9px] text-ink-mute truncate">
                            {Math.round(ms.progress * 100)}% · due {ms.due_date.slice(5)}
                          </div>
                        </div>
                      );
                    })}
                  </div>

                  {/* Timeline */}
                  <div className="flex-1 overflow-auto" data-testid="gantt-modal-timeline">
                    <div style={{ width: mTotalWidth, minWidth: "100%" }}>
                      <GanttHeader start={start} totalDays={totalDays} pxPerDay={mPxPerDay} zoom={modalZoom} />
                      {sprints.map((sp) => {
                        const wiList = workItemsBySprint[sp.id] ?? [];
                        return (
                          <div
                            key={`mtl-s-${sp.id}`}
                            data-row-kind="sprint"
                            data-row-id={sp.id}
                            className="h-12 border-b border-line/50 relative bg-bg-card/30"
                          >
                            <GanttBar
                              item={{ id: sp.id, label: sp.name, status: sprintStatusToBarStatus(sp.status) }}
                              startDate={toUtcCalendarDate(sp.start_date)}
                              endDate={toUtcCalendarDate(sp.end_date)}
                              dateRangeStart={toUtcCalendarDate(dateRange.start)}
                              pxPerDay={mPxPerDay}
                              variant="sprint"
                              onCheckConflict={undefined}
                              onClick={() => handleSprintClick(sp.id)}
                              onDragEnd={(newStart, newEnd) => onSprintUpdate?.(sp.id, newStart, newEnd)}
                            />
                            {wiList.slice(0, 4).map((wi, i) => (
                              <div
                                key={`mwi-${wi.id}`}
                                data-work-item-id={wi.id}
                                draggable
                                onDragStart={(e) => {
                                  e.dataTransfer.setData("text/work-item-id", wi.id);
                                  e.dataTransfer.effectAllowed = "move";
                                }}
                                className="absolute h-3 text-[8px] text-white px-1 cursor-grab rounded-sm select-none"
                                style={{
                                  top: 32, left: 6 + i * 90, width: 84,
                                  backgroundColor:
                                    wi.status === "done" ? "#3fb950" :
                                    wi.status === "in_progress" ? "#2f81f7" :
                                    wi.status === "blocked" ? "#f85149" :
                                    wi.status === "review" ? "#d29922" : "#6e7681",
                                  lineHeight: "12px",
                                }}
                                title={`${wi.key} — ${wi.title}`}
                              >
                                {wi.key}
                              </div>
                            ))}
                          </div>
                        );
                      })}
                      <div className="h-2 bg-bg-soft/60" />
                      {milestones.map((ms) => {
                        const critical = criticalIds.has(ms.id);
                        return (
                          <div
                            key={`mtl-m-${ms.id}`}
                            data-row-kind="milestone"
                            data-row-id={ms.id}
                            className="h-10 border-b border-line/50 relative bg-bg-card/30"
                          >
                            <GanttBar
                              item={{ id: ms.id, label: ms.name, status: critical ? "blocked" : "done" }}
                              startDate={toUtcCalendarDate(ms.due_date)}
                              endDate={toUtcCalendarDate(ms.due_date)}
                              dateRangeStart={toUtcCalendarDate(dateRange.start)}
                              pxPerDay={mPxPerDay}
                              variant="milestone"
                              isCritical={critical}
                              onClick={() => handleMilestoneClick(ms.id)}
                              onDragEnd={(newStart) => { onMilestoneUpdate?.(ms.id, newStart); }}
                            />
                          </div>
                        );
                      })}
                      {/* SVG links */}
                      <svg
                        width={mTotalWidth}
                        height={(sprints.length * (ROW.sprint + ROW.gap)) + ROW.gap + (milestones.length * (ROW.ms + ROW.gap))}
                        style={{ position: "absolute", top: 0, left: 0, pointerEvents: "none" }}
                      >
                        <defs>
                          <marker id="mgantt-arrow-blocks" viewBox="0 0 8 8" refX="6" refY="4" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                            <path d="M 0 0 L 8 4 L 0 8 z" fill="#f85149" />
                          </marker>
                          <marker id="mgantt-arrow-relates" viewBox="0 0 8 8" refX="6" refY="4" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                            <path d="M 0 0 L 8 4 L 0 8 z" fill="#6e7681" />
                          </marker>
                          <marker id="mgantt-arrow-duplicates" viewBox="0 0 8 8" refX="6" refY="4" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                            <path d="M 0 0 L 8 4 L 0 8 z" fill="#d29922" />
                          </marker>
                        </defs>
                        {links.map((l) => {
                          const fromX = (rowOffsets[l.fromId]?.end_x ?? 0) / pxPerDay * mPxPerDay;
                          const toX = (rowOffsets[l.toId]?.x ?? 0) / pxPerDay * mPxPerDay;
                          const fromY = rowOffsets[l.fromId]?.y ?? 0;
                          const toY = rowOffsets[l.toId]?.y ?? 0;
                          const midX = (fromX + toX) / 2;
                          const color = l.kind === "blocks" ? "#f85149" : l.kind === "duplicates" ? "#d29922" : "#6e7681";
                          const marker = l.kind === "blocks" ? "url(#mgantt-arrow-blocks)" : l.kind === "duplicates" ? "url(#mgantt-arrow-duplicates)" : "url(#mgantt-arrow-relates)";
                          return (
                            <g key={`ml-${l.id}`}>
                              <path
                                d={`M ${fromX} ${fromY} L ${midX} ${fromY} L ${midX} ${toY} L ${toX} ${toY}`}
                                fill="none" stroke={color} strokeWidth="1.5"
                                strokeDasharray={l.kind === "relates_to" ? "4 2" : "none"}
                                markerEnd={marker}
                              />
                            </g>
                          );
                        })}
                      </svg>
                    </div>
                  </div>
                </div>
              );
            })()}
          </div>
        </div>
      </div>,
      document.body
    )}

    {/* 共享右键菜单 (portal 到 body) */}
    <ContextMenu state={menu} onClose={closeMenu} />
    </>
  );
}

// (helper removed — 改用 date-fns format 避免 toISOString 时区漂移)
