"use client";

/* =====================================================================
 * LaneSettingsPage — /lane/[laneId]/settings
 * LANE 专属设置界面：
 *   1. 路径配置：本地 checkout / 虚拟机镜像 / 沙盒路径
 *   2. 统计数据：Worktree 数量 / 任务吞吐 / Agent 占用 / Git 状态聚合
 *   3. Git 操作：批量 pull / push / fetch / rebase / branch 管理
 * ===================================================================== */
import { useEffect, useState, useMemo, useCallback } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import {
  FolderGit2,
  Server,
  Box,
  BarChart2,
  GitBranch,
  GitPullRequest,
  Download,
  Upload,
  RefreshCw,
  Settings,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Terminal,
  Database,
  Zap,
  HardDrive,
  Monitor,
  MoreHorizontal,
  ChevronDown,
  ChevronRight,
  Plus,
  Trash2,
  Edit3,
  Copy,
  Search,
  Filter,
  Clock,
  Users,
  Activity,
  GitCommitHorizontal,
  GitFork,
  Merge,
  ChevronLeft,
} from "lucide-react";
import { clsx } from "clsx";
import { useStore } from "@/lib/store";
import { useNavStore } from "@/lib/nav/navStore";
import { useTranslation } from "@/lib/i18n";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { GuardedLink as LinkG } from "@/components/navigation/GuardedLink";
import { Lane, Worktree, WorktreeStatus } from "@/types/ids";

// =====================================================================
// Types
// =====================================================================
type PathType = "local" | "vm" | "sandbox";

interface LanePathConfig {
  type: PathType;
  path: string;
  isActive: boolean;
  lastSync?: string;
  health?: "healthy" | "degraded" | "unknown";
}

interface LaneStats {
  totalWorktrees: number;
  activeWorktrees: number;
  dirtyWorktrees: number;
  behindWorktrees: number;
  totalAgents: number;
  activeAgents: number;
  totalBranches: number;
  mergedBranches: number;
  openPRs: number;
  lastActivity: string | null;
  throughput: { period: string; completed: number; created: number }[];
}

interface GitOperationResult {
  success: boolean;
  output: string;
  error?: string;
}

const PATH_TYPE_LABELS: Record<PathType, { label: string; icon: React.ReactNode; desc: string }> = {
  local: { label: "本地路径", icon: <HardDrive size={16} />, desc: "本机磁盘上的 Git checkout 目录" },
  vm: { label: "虚拟机镜像", icon: <Monitor size={16} />, desc: "Multipass / VMware / VirtualBox 等虚拟机内的工作区" },
  sandbox: { label: "沙盒路径", icon: <Box size={16} />, desc: "隔离沙盒 (Docker / Firecracker / gVisor 等)" },
};

const WORKTREE_STATUS_TONE: Record<WorktreeStatus, string> = {
  initializing: "bg-info/10 text-info border-info/40",
  cloning: "bg-info/10 text-info border-info/40",
  syncing: "bg-info/10 text-info border-info/40",
  active: "bg-ok/10 text-ok border-ok/40",
  dirty: "bg-warn/10 text-warn border-warn/40",
  behind: "bg-accent/10 text-accent border-accent/40",
  diverged: "bg-err/10 text-err border-err/40",
  conflict: "bg-err/10 text-err border-err/40",
  committing: "bg-info/10 text-info border-info/40",
  pushing: "bg-info/10 text-info border-info/40",
  ci_running: "bg-info/10 text-info border-info/40",
  review_requested: "bg-warn/10 text-warn border-warn/40",
  merged: "bg-ok/10 text-ok border-ok/40",
  closed: "bg-bg-soft text-ink-mute border-line",
  abandoned: "bg-bg-soft text-ink-mute border-line",
  archived: "bg-bg-soft text-ink-mute border-line",
  reverted: "bg-bg-soft text-ink-mute border-line",
};

// =====================================================================
// PathConfigCard — 单个路径配置卡片
// =====================================================================
interface PathConfigCardProps {
  config: LanePathConfig;
  index: number;
  onUpdate: (index: number, updates: Partial<LanePathConfig>) => void;
  onRemove: (index: number) => void;
  onTest: (index: number) => void;
}

function PathConfigCard({ config, index, onUpdate, onRemove, onTest }: PathConfigCardProps) {
  const { t } = useTranslation();
  const typeInfo = PATH_TYPE_LABELS[config.type];
  const [editing, setEditing] = useState(false);
  const [editPath, setEditPath] = useState(config.path);

  return (
    <div className="anime-panel anime-chamfer p-4">
      <div className="flex items-start justify-between gap-4 mb-3">
        <div className="flex items-center gap-3 flex-1 min-w-0">
          <div className={clsx("size-10 rounded-lg grid place-items-center shrink-0", config.type === "local" && "bg-accent/10", config.type === "vm" && "bg-info/10", config.type === "sandbox" && "bg-warn/10")}>
            {typeInfo.icon}
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <span className="font-semibold text-ink">{typeInfo.label}</span>
              <StatusPill value={config.isActive ? "active" : "inactive"} size="xs" />
              <StatusPill value={config.health ?? "unknown"} size="xs" />
            </div>
            <p className="text-[11px] text-ink-mute mt-0.5">{typeInfo.desc}</p>
            {!editing && config.path && (
              <div className="mt-1 flex items-center gap-2 text-[11px] font-mono text-ink-dim truncate max-w-[300px]">
                <FolderGit2 size={11} />
                <span>{config.path}</span>
                {config.lastSync && <span className="ml-2">最后同步: {config.lastSync}</span>}
              </div>
            )}
            {editing && (
              <div className="mt-2 flex gap-2">
                <input
                  value={editPath}
                  onChange={(e) => setEditPath(e.target.value)}
                  placeholder="输入路径 (如: D:\\Workspaces\\lane-main)"
                  className="flex-1 bg-bg-soft border border-line rounded px-2 py-1.5 text-xs"
                />
                <button onClick={() => { onUpdate(index, { path: editPath, isActive: true }); setEditing(false); }} className="btn-primary text-xs">保存</button>
                <button onClick={() => { setEditPath(config.path); setEditing(false); }} className="btn text-xs">取消</button>
              </div>
            )}
          </div>
        </div>
        <div className="flex items-center gap-1 shrink-0">
          {!editing && config.path && (
            <button onClick={() => setEditing(true)} className="btn text-xs" title="编辑路径"><Edit3 size={12} /></button>
          )}
          {!editing && (
            <button onClick={() => onTest(index)} className="btn text-xs" title="测试连接"><Terminal size={12} /></button>
          )}
          {config.path && (
            <button onClick={() => navigator.clipboard.writeText(config.path)} className="btn text-xs" title="复制路径"><Copy size={12} /></button>
          )}
          <button onClick={() => onRemove(index)} className="btn text-xs hover:text-err" title="移除"><Trash2 size={12} /></button>
        </div>
      </div>
      {config.path && !editing && config.lastSync && (
        <div className="text-[10px] text-ink-mute font-mono">最后同步: {config.lastSync}</div>
      )}
    </div>
  );
}

// =====================================================================
// StatsCard — 统计卡片
// =====================================================================
interface StatsCardProps {
  label: string;
  value: string | number;
  icon: React.ReactNode;
  trend?: { value: number; label: string };
  tone?: "accent" | "ok" | "warn" | "info" | "err";
}

function StatsCard({ label, value, icon, trend, tone = "accent" }: StatsCardProps) {
  const toneClass = {
    accent: "text-accent border-accent/40 bg-accent/10",
    ok: "text-ok border-ok/40 bg-ok/10",
    warn: "text-warn border-warn/40 bg-warn/10",
    info: "text-info border-info/40 bg-info/10",
    err: "text-err border-err/40 bg-err/10",
  }[tone];
  return (
    <div className={clsx("anime-panel anime-chamfer p-4 flex flex-col gap-2", toneClass)}>
      <div className="flex items-center justify-between">
        <span className="text-[10px] font-mono uppercase tracking-wider font-semibold">{label}</span>
        <span className="text-ink-mute">{icon}</span>
      </div>
      <div className="text-3xl font-black text-ink">{value}</div>
      {trend && (
        <div className={clsx("text-[10px] font-mono", trend.value >= 0 ? "text-ok" : "text-err")}>
          {trend.value >= 0 ? "▲" : "▼"} {Math.abs(trend.value)}% {trend.label}
        </div>
      )}
    </div>
  );
}

// =====================================================================
// WorktreeTable — Worktree 列表表格
// =====================================================================
interface WorktreeTableProps {
  worktrees: Worktree[];
  onGitOp: (worktreeId: string, op: string) => void;
}

function WorktreeTable({ worktrees, onGitOp }: WorktreeTableProps) {
  if (worktrees.length === 0) {
    return <div className="anime-panel anime-chamfer p-8 text-center text-ink-mute">该 LANE 下暂无 Worktree</div>;
  }
  return (
    <div className="anime-panel anime-chamfer overflow-hidden">
      <table className="table w-full">
        <thead>
          <tr>
            <th className="w-8"></th>
            <th>Worktree</th>
            <th>Branch</th>
            <th>Status</th>
            <th>Agent</th>
            <th>Runtime</th>
            <th>Last Event</th>
            <th className="w-48">Git 操作</th>
          </tr>
        </thead>
        <tbody>
          {worktrees.map((wt) => (
            <tr key={wt.id} className="hover:bg-bg-soft/50">
              <td><GitBranch size={12} className="text-ink-mute" /></td>
              <td className="font-mono text-xs text-ink">{wt.name}</td>
              <td className="font-mono text-xs text-info">{wt.branch}</td>
              <td><StatusPill value={wt.status} size="xs" /></td>
              <td className="text-[11px] text-ink-dim">{wt.agent_session_id ? "绑定" : "—"}</td>
              <td className="text-[11px] text-ink-dim">{wt.local_runtime_id ? "绑定" : "—"}</td>
              <td className="text-[10px] text-ink-mute font-mono">{wt.last_event_at?.slice(0, 16).replace("T", " ")}</td>
              <td>
                <div className="flex items-center gap-1">
                  <button onClick={() => onGitOp(wt.id, "pull")} className="btn text-[10px] px-2 py-1" title="Pull"><Download size={10} /></button>
                  <button onClick={() => onGitOp(wt.id, "push")} className="btn text-[10px] px-2 py-1" title="Push"><Upload size={10} /></button>
                  <button onClick={() => onGitOp(wt.id, "fetch")} className="btn text-[10px] px-2 py-1" title="Fetch"><RefreshCw size={10} /></button>
                  <button onClick={() => onGitOp(wt.id, "status")} className="btn text-[10px] px-2 py-1" title="Status"><GitCommitHorizontal size={10} /></button>
                  <button onClick={() => onGitOp(wt.id, "branch")} className="btn text-[10px] px-2 py-1" title="分支管理"><GitFork size={10} /></button>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// =====================================================================
// GitOperationModal — Git 操作结果弹窗
// =====================================================================
interface GitOperationModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  result: GitOperationResult | null;
  loading: boolean;
}

function GitOperationModal({ open, onClose, title, result, loading }: GitOperationModalProps) {
  if (!open) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60" onClick={onClose}>
      <div className="anime-panel anime-chamfer max-w-2xl w-full max-h-[70vh] flex flex-col" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between p-4 border-b border-line">
          <h3 className="font-semibold">{title}</h3>
          <button onClick={onClose} className="text-ink-mute hover:text-ink"><XCircle size={20} /></button>
        </div>
        <div className="flex-1 overflow-auto p-4">
          {loading ? (
            <div className="flex items-center justify-center h-32"><RefreshCw className="animate-spin text-accent" size={24} /></div>
          ) : result ? (
            <pre className={clsx("bg-bg p-3 rounded text-[10px] font-mono max-h-96 overflow-auto", result.success ? "text-ok" : "text-err")}>
              {result.output || (result.error ? `Error: ${result.error}` : "无输出")}
            </pre>
          ) : (
            <div className="text-center text-ink-mute py-8">准备执行…</div>
          )}
        </div>
        <div className="p-4 border-t border-line flex justify-end gap-2">
          <button onClick={onClose} className="btn" disabled={loading}>关闭</button>
          {result && !result.success && (
            <button onClick={() => navigator.clipboard.writeText(result.output || result.error || "")} className="btn-primary text-xs"><Copy size={12} /> 复制错误</button>
          )}
        </div>
      </div>
    </div>
  );
}

// =====================================================================
// Main Page
// =====================================================================
export default function LaneSettingsPage() {
  const { t } = useTranslation();
  const router = useRouter();
  const params = useParams();
  const laneId = params.laneId as string;

  // Store
  const lanes = useStore((s) => s.lanes);
  const worktrees = useStore((s) => s.worktrees);
  const agentSessions = useStore((s) => s.agentSessions);
  const projects = useStore((s) => s.projects);
  const identities = useStore((s) => s.identities);

  // Lane data
  const lane = lanes.find((l) => l.id === laneId);
  const project = lane ? projects.find((p) => p.id === lane.project_id) : null;
  const laneWorktrees = worktrees.filter((w) => w.lane_id === laneId);
  const laneAgents = agentSessions.filter((a) => laneWorktrees.some((w) => w.id === a.worktree_id));

  // State
  const [activeTab, setActiveTab] = useState<"paths" | "stats" | "git">("paths");
  const [pathConfigs, setPathConfigs] = useState<LanePathConfig[]>([
    { type: "local", path: "", isActive: false },
    { type: "vm", path: "", isActive: false },
    { type: "sandbox", path: "", isActive: false },
  ]);
  const [gitModal, setGitModal] = useState<{ open: boolean; title: string; op: string; wtId: string } | null>(null);
  const [gitResult, setGitResult] = useState<GitOperationResult | null>(null);
  const [gitLoading, setGitLoading] = useState(false);

  // Computed stats
  const stats = useMemo((): LaneStats => {
    const wt = laneWorktrees;
    const activeWt = wt.filter((w) => w.status === "active");
    const dirtyWt = wt.filter((w) => w.status === "dirty" || w.status === "diverged" || w.status === "conflict");
    const behindWt = wt.filter((w) => w.status === "behind");
    const agents = laneAgents;
    const activeAgents = agents.filter((a) => a.status === "executing" || a.status === "planning");
    // Mock branch/PR data (would come from git binding API)
    const branches = new Set(wt.map((w) => w.branch)).size;
    return {
      totalWorktrees: wt.length,
      activeWorktrees: activeWt.length,
      dirtyWorktrees: dirtyWt.length,
      behindWorktrees: behindWt.length,
      totalAgents: agents.length,
      activeAgents: activeAgents.length,
      totalBranches: branches,
      mergedBranches: wt.filter((w) => w.status === "merged").length,
      openPRs: wt.filter((w) => w.pr_id).length,
      lastActivity: wt.length > 0 ? wt.reduce((max, w) => w.last_event_at > max ? w.last_event_at : max, "") : null,
      throughput: [
        { period: "最近 7 天", completed: 12, created: 15 },
        { period: "最近 30 天", completed: 48, created: 52 },
        { period: "本季度", completed: 156, created: 168 },
      ],
    };
  }, [laneWorktrees, laneAgents]);

  // Handlers
  const updatePathConfig = useCallback((index: number, updates: Partial<LanePathConfig>) => {
    setPathConfigs((prev) => prev.map((c, i) => i === index ? { ...c, ...updates } : c));
  }, []);

  const removePathConfig = useCallback((index: number) => {
    setPathConfigs((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const testPathConfig = useCallback(async (index: number) => {
    const config = pathConfigs[index];
    if (!config.path) return;
    // Simulate test connection
    setPathConfigs((prev) => prev.map((c, i) => i === index ? { ...c, health: "healthy", lastSync: new Date().toISOString().slice(0, 16).replace("T", " ") } : c));
  }, [pathConfigs]);

  const addPathConfig = useCallback(() => {
    setPathConfigs((prev) => [...prev, { type: "local", path: "", isActive: false }]);
  }, []);

  const executeGitOp = useCallback(async (wtId: string, op: string) => {
    const wt = worktrees.find((w) => w.id === wtId);
    if (!wt) return;
    // Find the active path config for display
    const activePath = pathConfigs.find(c => c.isActive && c.path);
    setGitModal({ open: true, title: `${op.toUpperCase()} — ${wt.name}`, op, wtId });
    setGitLoading(true);
    setGitResult(null);
    // Simulate git operation
    setTimeout(() => {
      const success = Math.random() > 0.1;
      setGitResult({
        success,
        output: success
          ? `已在 ${activePath?.path || wt.name} 执行 git ${op}\n${op === "pull" ? "Already up to date." : op === "push" ? "Everything up-to-date" : "操作完成"}`
          : `Error: 权限不足或路径不存在 (${activePath?.path || "未配置路径"})`,
        error: success ? undefined : "模拟错误",
      });
      setGitLoading(false);
    }, 1500);
  }, [worktrees, pathConfigs]);

  const closeGitModal = useCallback(() => {
    setGitModal(null);
    setGitResult(null);
    setGitLoading(false);
  }, []);

  // Breadcrumbs
  const breadcrumbs = [
    { label: "Worktree", href: "/worktree" },
    project && { label: project.name, href: `/worktree?project_id=${project.id}` },
    lane && { label: lane.name, href: `#` },
    { label: "LANE 设置", href: `#` },
  ].filter(Boolean) as { label: string; href: string }[];

  if (!lane) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="anime-panel anime-chamfer p-8 text-center">
          <AlertTriangle className="w-12 h-12 mx-auto mb-4 text-warn" />
          <h2 className="text-xl font-bold mb-2">找不到 LANE</h2>
          <p className="text-ink-mute">ID: {laneId}</p>
          <LinkG href="/worktree" className="mt-4 inline-btn btn-primary"><ChevronLeft size={14} /> 返回 Worktree</LinkG>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full bg-bg">
      <GitOperationModal
        open={gitModal?.open ?? false}
        onClose={closeGitModal}
        title={gitModal?.title ?? ""}
        result={gitResult}
        loading={gitLoading}
      />

      <main className="flex-1 min-w-0 overflow-y-auto p-4 lg:p-6 space-y-6">
        {/* Breadcrumbs */}
        <nav className="flex items-center gap-1.5 text-xs text-ink-dim flex-wrap" aria-label="面包屑">
          {breadcrumbs.map((b, i) => (
            <span key={b.href} className="flex items-center gap-1.5">
              {i > 0 && <ChevronRight size={12} className="text-ink-mute" />}
              {b.href !== "#" ? <LinkG href={b.href} className="hover:text-ink">{b.label}</LinkG> : <span className="font-semibold text-ink">{b.label}</span>}
            </span>
          ))}
        </nav>

        {/* Page Header */}
        <PageHeader
          title={t.pageTitles["/lane/settings"]?.title ?? "LANE 设置"}
          subtitle={`${lane.name} · ${project?.name ?? "未知项目"} — 路径配置、统计聚合与 Git 批量操作`}
          icon={<Settings className="text-accent" size={20} />}
          count={laneWorktrees.length}
        />

        {/* Tabs */}
        <div className="flex items-center gap-1 border-b border-line" role="tablist" aria-label="LANE 设置标签页">
          {[
            { id: "paths", label: "路径配置", icon: <FolderGit2 size={14} /> },
            { id: "stats", label: "统计数据", icon: <BarChart2 size={14} /> },
            { id: "git", label: "Git 操作", icon: <GitBranch size={14} /> },
          ].map((tab) => (
            <button
              key={tab.id}
              role="tab"
              aria-selected={activeTab === tab.id}
              onClick={() => setActiveTab(tab.id as typeof activeTab)}
              className={clsx(
                "px-4 py-2 text-sm font-semibold border-b-2 -mb-px transition-colors flex items-center gap-2",
                activeTab === tab.id
                  ? "border-accent text-accent"
                  : "border-transparent text-ink-dim hover:text-ink"
              )}
            >
              {tab.icon}
              {tab.label}
            </button>
          ))}
        </div>

        {/* ===== Tab: Paths ===== */}
        {activeTab === "paths" && (
          <div className="space-y-6">
            <SectionTitle>路径配置 <span className="text-xs font-normal text-ink-mute ml-2">({pathConfigs.filter(c => c.path).length}/{pathConfigs.length} 已配置)</span></SectionTitle>

            <div className="grid gap-4 lg:grid-cols-3">
              {pathConfigs.map((config, i) => (
                <PathConfigCard
                  key={i}
                  config={config}
                  index={i}
                  onUpdate={updatePathConfig}
                  onRemove={removePathConfig}
                  onTest={testPathConfig}
                />
              ))}
            </div>

            {pathConfigs.every(c => !c.path) && (
              <div className="anime-panel anime-chamfer p-8 text-center border-dashed border-line">
                <FolderGit2 className="w-12 h-12 mx-auto mb-3 text-ink-mute" />
                <h3 className="font-semibold mb-1">尚未配置任何路径</h3>
                <p className="text-ink-mute text-sm mb-4">为 LANE 添加本地、虚拟机或沙盒路径，启用 Git 同步与统计聚合</p>
                <button onClick={addPathConfig} className="btn-primary"><Plus size={14} /> 添加首个路径</button>
              </div>
            )}

            {pathConfigs.some(c => c.path) && (
              <div className="anime-panel anime-chamfer p-4">
                <div className="flex items-center justify-between">
                  <span className="font-semibold">已配置路径</span>
                  <button onClick={addPathConfig} className="btn text-xs"><Plus size={12} /> 新增路径</button>
                </div>
              </div>
            )}
          </div>
        )}

        {/* ===== Tab: Stats ===== */}
        {activeTab === "stats" && (
          <div className="space-y-6">
            <SectionTitle>统计聚合</SectionTitle>

            {/* Overview Cards */}
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <StatsCard label="Worktree 总数" value={stats.totalWorktrees} icon={<FolderGit2 size={20} />} tone="accent" trend={{ value: 2, label: "较上周" }} />
              <StatsCard label="活跃 Worktree" value={stats.activeWorktrees} icon={<Activity size={20} />} tone="ok" trend={{ value: 5, label: "较上周" }} />
              <StatsCard label="脏/冲突" value={stats.dirtyWorktrees} icon={<AlertTriangle size={20} />} tone={stats.dirtyWorktrees > 0 ? "warn" : "ok"} />
              <StatsCard label="落后远端" value={stats.behindWorktrees} icon={<Clock size={20} />} tone={stats.behindWorktrees > 0 ? "warn" : "ok"} />
              <StatsCard label="Agent 总数" value={stats.totalAgents} icon={<Users size={20} />} tone="info" />
              <StatsCard label="活跃 Agent" value={stats.activeAgents} icon={<Zap size={20} />} tone="ok" />
              <StatsCard label="分支总数" value={stats.totalBranches} icon={<GitBranch size={20} />} tone="accent" />
              <StatsCard label="已合并分支" value={stats.mergedBranches} icon={<Merge size={20} />} tone="ok" />
              <StatsCard label="打开 PR" value={stats.openPRs} icon={<GitPullRequest size={20} />} tone="info" />
            </div>

            {/* Throughput Chart (Mock) */}
            <div className="anime-panel anime-chamfer p-4">
              <h3 className="font-semibold mb-3">吞吐趋势</h3>
              <div className="space-y-2">
                {stats.throughput.map((t) => (
                  <div key={t.period} className="flex items-center gap-4">
                    <span className="w-28 text-[11px] font-mono text-ink-mute">{t.period}</span>
                    <div className="flex-1 h-6 bg-bg-soft rounded overflow-hidden relative">
                      <div className="h-full bg-ok rounded-l" style={{ width: `${Math.max(5, (t.completed / (t.completed + t.created)) * 100)}%` }} />
                      <div className="absolute inset-0 flex items-center justify-between px-2 text-[9px] font-mono text-ink">
                        <span>完成 {t.completed}</span>
                        <span>新建 {t.created}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Last Activity */}
            {stats.lastActivity && (
              <div className="anime-panel anime-chamfer p-4">
                <div className="flex items-center gap-2 text-sm">
                  <Clock size={14} className="text-ink-mute" />
                  <span className="font-semibold">最近活动</span>
                  <span className="text-ink-dim font-mono ml-auto">{stats.lastActivity.slice(0, 16).replace("T", " ")}</span>
                </div>
              </div>
            )}
          </div>
        )}

        {/* ===== Tab: Git ===== */}
        {activeTab === "git" && (
          <div className="space-y-6">
            <SectionTitle>Git 批量操作</SectionTitle>

            <div className="anime-panel anime-chamfer p-4 mb-4">
              <div className="flex items-center justify-between flex-wrap gap-3">
                <div className="flex items-center gap-3">
                  <span className="text-xs font-mono text-ink-mute">{laneWorktrees.length} 个 Worktree</span>
                  <button onClick={() => laneWorktrees.forEach(w => executeGitOp(w.id, "fetch"))} className="btn-primary text-xs" disabled={laneWorktrees.length === 0 || gitLoading}>
                    <RefreshCw size={12} /> 全部 Fetch
                  </button>
                  <button onClick={() => laneWorktrees.forEach(w => executeGitOp(w.id, "pull"))} className="btn text-xs" disabled={laneWorktrees.length === 0 || gitLoading}>
                    <Download size={12} /> 全部 Pull
                  </button>
                  <button onClick={() => laneWorktrees.forEach(w => executeGitOp(w.id, "push"))} className="btn text-xs" disabled={laneWorktrees.length === 0 || gitLoading}>
                    <Upload size={12} /> 全部 Push
                  </button>
                  <button onClick={() => laneWorktrees.forEach(w => executeGitOp(w.id, "status"))} className="btn text-xs" disabled={laneWorktrees.length === 0 || gitLoading}>
                    <GitCommitHorizontal size={12} /> 全部 Status
                  </button>
                </div>
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    placeholder="按名称/分支过滤…"
                    className="bg-bg-soft border border-line rounded px-2 py-1.5 text-xs w-48"
                  />
                  <button className="btn text-xs"><Filter size={12} /></button>
                </div>
              </div>
            </div>

            <WorktreeTable worktrees={laneWorktrees} onGitOp={executeGitOp} />
          </div>
        )}

        {/* Back Link */}
        <div className="pt-4 border-t border-line">
          <LinkG href={project ? `/worktree?project_id=${project.id}` : "/worktree"} className="btn text-xs">
            <ChevronLeft size={12} /> 返回 Worktree 汇总
          </LinkG>
        </div>
      </main>
    </div>
  );
}