import {
  Inbox,
  CheckSquare,
  FolderKanban,
  Bot,
  BarChart3,
  Settings,
  Trello,
  SquareChartGantt,
  LayoutDashboard,
  Users,
  Building2,
  FileText,
  MessageSquare,
  Workflow,
  ShieldCheck,
  Hammer,
  Calendar,
  GitBranch,
  MessageCircleWarning,
  ListTodo,
  GitFork,
  Plug,
  Search,
  Server,
  Boxes,
  History,
  Zap,
  Network,
  Briefcase,
  Key,
  Terminal,
  Sparkles,
  Monitor,
  FolderOpen,
  Smartphone,
  RefreshCw,
  Webhook,
  Code2,
} from "lucide-react";

export type ModuleCategory =
  | "core"
  | "work"
  | "agent"
  | "integration"
  | "system";

export interface ModuleDefinition {
  id: string;
  label: string;
  code: string;
  href: string;
  category: ModuleCategory;
  categoryLabel: string;
  description: string;
  icon: React.ElementType;
  isCore?: boolean;
  count?: number;
}

// =====================================================================
// CATEGORY_STYLES — 5 域分色 (Jira 风格识别度)
//
// Rationale (per 2026-09-02 15:42 JST 拍板):
//   - Jira Software 主页 sidebar 走的是 "彩色 icon tile + 域语义分色"
//   - Lucide 没有 filled 变体, 但可以用 圆角色块 + line icon + 域染色
//     制造等效的"色块底 + 图标" JIRA card 风格
//   - 5 域色: cyan (core) / sky (work) / emerald (agent) / violet
//     (integration) / amber (system), 跟 Atlassian Design System 主色
//     (#0052CC / #36B37E / #6554C0 / #FF8B00) 视觉接近
//   - 同时支持 light/dark mode (light 用 -700 / dark 用 -300/400)
//   - bg + text + border 三组 class, 配 hover / active / glow variant
// =====================================================================
export interface CategoryStyle {
  /** 色块底 (低透明) */
  bg: string;
  /** 色块底 (active 状态, 强透明) */
  bgActive: string;
  /** 前景色 (icon 主色) */
  text: string;
  /** 边框色 */
  border: string;
  /** 边框色 (active 状态, 更亮) */
  borderActive: string;
  /** hover glow shadow (rgba 已硬编码, 跟 Tailwind 默认色匹配) */
  glow: string;
  /** 小圆点 / 徽章色 (1px-4px 元素) */
  dot: string;
  /** 域名称 (i18n key 引用) */
  name: string;
}

export const CATEGORY_STYLES: Record<ModuleCategory, CategoryStyle> = {
  // Core — cyan (保留现有 accent 色系, 主品牌)
  core: {
    bg: "bg-cyan-500/10 dark:bg-cyan-400/10",
    bgActive: "bg-cyan-500/20 dark:bg-cyan-400/20",
    text: "text-cyan-700 dark:text-cyan-300",
    border: "border-cyan-500/30 dark:border-cyan-400/30",
    borderActive: "border-cyan-500/60 dark:border-cyan-400/60",
    glow: "shadow-[0_0_10px_rgba(34,211,238,0.35)]",
    dot: "bg-cyan-500 dark:bg-cyan-400",
    name: "Core",
  },
  // Work — sky (Jira Software 蓝)
  work: {
    bg: "bg-sky-500/10 dark:bg-sky-400/10",
    bgActive: "bg-sky-500/20 dark:bg-sky-400/20",
    text: "text-sky-700 dark:text-sky-300",
    border: "border-sky-500/30 dark:border-sky-400/30",
    borderActive: "border-sky-500/60 dark:border-sky-400/60",
    glow: "shadow-[0_0_10px_rgba(56,189,248,0.35)]",
    dot: "bg-sky-500 dark:bg-sky-400",
    name: "Work",
  },
  // Agent — emerald (Jira 活跃绿 #36B37E)
  agent: {
    bg: "bg-emerald-500/10 dark:bg-emerald-400/10",
    bgActive: "bg-emerald-500/20 dark:bg-emerald-400/20",
    text: "text-emerald-700 dark:text-emerald-300",
    border: "border-emerald-500/30 dark:border-emerald-400/30",
    borderActive: "border-emerald-500/60 dark:border-emerald-400/60",
    glow: "shadow-[0_0_10px_rgba(52,211,153,0.35)]",
    dot: "bg-emerald-500 dark:bg-emerald-400",
    name: "Agent",
  },
  // Integration — violet (Confluence 紫 #6554C0)
  integration: {
    bg: "bg-violet-500/10 dark:bg-violet-400/10",
    bgActive: "bg-violet-500/20 dark:bg-violet-400/20",
    text: "text-violet-700 dark:text-violet-300",
    border: "border-violet-500/30 dark:border-violet-400/30",
    borderActive: "border-violet-500/60 dark:border-violet-400/60",
    glow: "shadow-[0_0_10px_rgba(167,139,250,0.35)]",
    dot: "bg-violet-500 dark:bg-violet-400",
    name: "Integration",
  },
  // System — amber (Jira 警示橙 #FF8B00)
  system: {
    bg: "bg-amber-500/10 dark:bg-amber-400/10",
    bgActive: "bg-amber-500/20 dark:bg-amber-400/20",
    text: "text-amber-700 dark:text-amber-300",
    border: "border-amber-500/30 dark:border-amber-400/30",
    borderActive: "border-amber-500/60 dark:border-amber-400/60",
    glow: "shadow-[0_0_10px_rgba(251,191,36,0.35)]",
    dot: "bg-amber-500 dark:bg-amber-400",
    name: "System",
  },
};

/** 取域色卡 (缺省回退 core, 避免 undefined 错误) */
export function getCategoryStyles(category: ModuleCategory): CategoryStyle {
  return CATEGORY_STYLES[category] ?? CATEGORY_STYLES.core;
}

export const ALL_MODULES: ModuleDefinition[] = [
  // ── Core Workspaces ──────────────────────────────────────────────
  {
    id: "inbox",
    label: "Inbox",
    code: "01",
    href: "/inbox",
    category: "core",
    categoryLabel: "Core Workspace",
    description: "通知中心、@提及与审计流聚合工作台",
    icon: Inbox,
    isCore: true,
    count: 3,
  },
  {
    id: "issues",
    label: "Issues",
    code: "02",
    href: "/issues",
    category: "core",
    categoryLabel: "Core Workspace",
    description: "工作项与任务看板 / 树形全景视图",
    icon: CheckSquare,
    isCore: true,
  },
  {
    id: "projects",
    label: "Projects",
    code: "03",
    href: "/projects",
    category: "core",
    categoryLabel: "Core Workspace",
    description: "多面板项目工作区 (Kanban / Timeline / Backlog / Agents / Worktrees)",
    icon: FolderKanban,
    isCore: true,
  },
  {
    id: "agents",
    label: "Agents",
    code: "04",
    href: "/agents",
    category: "core",
    categoryLabel: "Core Workspace",
    description: "智能 Agent 运行状态、编排、会话与执行日志",
    icon: Bot,
    isCore: true,
  },
  {
    id: "analytics",
    label: "Analytics",
    code: "05",
    href: "/analytics",
    category: "core",
    categoryLabel: "Core Workspace",
    description: "工程效能大盘、燃尽图与遥测指标统计",
    icon: BarChart3,
    isCore: true,
  },
  {
    id: "settings",
    label: "Settings",
    code: "06",
    href: "/settings",
    category: "core",
    categoryLabel: "Core Workspace",
    description: "租户全局配置、团队成员、安全与权限管理",
    icon: Settings,
    isCore: true,
  },
  {
    id: "remote",
    label: "Remote Control",
    code: "M",
    href: "/remote",
    category: "core",
    categoryLabel: "Core Workspace",
    description: "手机端远程连接 desktop / terminal / files (per 2026-09-01 PHASE-MOBILE-PWA v0.2)",
    icon: Smartphone,
    isCore: true,
  },

  // ── Tactical Views ───────────────────────────────────────────────
  {
    id: "kanban",
    label: "Kanban Board",
    code: "KB",
    href: "/projects?tab=kanban",
    category: "work",
    categoryLabel: "Work Management",
    description: "4 态泳道即时拖拽任务看板",
    icon: Trello,
  },
  {
    id: "timeline",
    label: "Timeline & Gantt",
    code: "TL",
    href: "/projects?tab=timeline",
    category: "work",
    categoryLabel: "Work Management",
    description: "甘特图排期、里程碑与日历时间线联动",
    icon: SquareChartGantt,
  },
  {
    id: "backlog",
    label: "Backlog",
    code: "BL",
    href: "/projects?tab=backlog",
    category: "work",
    categoryLabel: "Work Management",
    description: "需求待办列表与优先级排期池",
    icon: LayoutDashboard,
  },

  // ── Agent & Worktree Tools ───────────────────────────────────────
  {
    id: "agent-windows",
    label: "Agent Windows",
    code: "WIN",
    href: "/agent-windows",
    category: "agent",
    categoryLabel: "Worktree / Agent",
    description: "Agent 多终端并行任务执行窗口",
    icon: Sparkles,
  },
  {
    id: "worktree",
    label: "Worktree Manager",
    code: "WT",
    href: "/worktree",
    category: "agent",
    categoryLabel: "Worktree / Agent",
    description: "Git Worktree 隔离分支工作流与状态机",
    icon: GitBranch,
  },
  {
    id: "validation",
    label: "Validation Test",
    code: "VAL",
    href: "/validation",
    category: "agent",
    categoryLabel: "Worktree / Agent",
    description: "自动化用例验证与断言测试套件",
    icon: ShieldCheck,
  },
  {
    id: "context",
    label: "Context Graph",
    code: "CTX",
    href: "/context",
    category: "agent",
    categoryLabel: "Worktree / Agent",
    description: "Agent 上下文知识图谱与决策包",
    icon: ListTodo,
  },
  {
    id: "feedback",
    label: "Agent Feedback",
    code: "FB",
    href: "/feedback",
    category: "agent",
    categoryLabel: "Worktree / Agent",
    description: "人机协作反馈回路与异常预警",
    icon: MessageCircleWarning,
  },

  // ── Work Management Modules ──────────────────────────────────────
  {
    id: "workflow",
    label: "Workflow Engine",
    code: "WF",
    href: "/workflow",
    category: "work",
    categoryLabel: "Work Management",
    description: "自动化工作流编排与状态流转规则",
    icon: Workflow,
  },
  {
    id: "development",
    label: "Development Hub",
    code: "DEV",
    href: "/development",
    category: "work",
    categoryLabel: "Work Management",
    description: "变更集 ChangeSet 与分支代码协同",
    icon: Hammer,
  },
  {
    id: "planning",
    label: "Planning Hub",
    code: "PLN",
    href: "/planning",
    category: "work",
    categoryLabel: "Work Management",
    description: "Sprint 迭代规划与容量负荷评估",
    icon: Calendar,
  },

  // ── Integrations & Security ──────────────────────────────────────
  {
    id: "scm",
    label: "SCM Provider",
    code: "SCM",
    href: "/scm",
    category: "integration",
    categoryLabel: "Integration & Security",
    description: "Git VCS 仓库与 Pull Request 追踪",
    icon: GitFork,
  },
  {
    id: "integration",
    label: "MCP Integrations",
    code: "MCP",
    href: "/integration",
    category: "integration",
    categoryLabel: "Integration & Security",
    description: "第三方工具链与 MCP 服务连接网关",
    icon: Plug,
  },
  {
    id: "api-keys",
    label: "API Keys & Secrets",
    code: "KEY",
    href: "/settings/api-keys",
    category: "integration",
    categoryLabel: "Integration & Security",
    description: "凭据密钥保管库与模型 API Key 配置",
    icon: Key,
  },
  {
    id: "developer-console",
    label: "Developer Console",
    code: "DEV",
    href: "/settings/developer",
    category: "integration",
    categoryLabel: "Integration & Security",
    description: "Developer API (REST + MCP) + 出站 Webhook 管理 (per 2026-09-02 14:06 JST 拍板)",
    icon: Code2,
  },
  {
    id: "webhooks",
    label: "Webhooks",
    code: "WHK",
    href: "/settings/developer?tab=webhooks",
    category: "integration",
    categoryLabel: "Integration & Security",
    description: "出站 Webhook endpoint + 5 域预置 vendor 模板 (Slack/Teams/Discord)",
    icon: Webhook,
  },
  {
    id: "cli-profiles",
    label: "CLI Profiles",
    code: "CLI",
    href: "/settings/cli-profiles",
    category: "integration",
    categoryLabel: "Integration & Security",
    description: "命令行环境配置与凭据模板",
    icon: Terminal,
  },
  {
    id: "search",
    label: "Universal Search",
    code: "SRC",
    href: "/search",
    category: "integration",
    categoryLabel: "Integration & Security",
    description: "全域跨模块语义检索与索引",
    icon: Search,
  },

  // ── System & Meta ────────────────────────────────────────────────
  {
    id: "local-runtime",
    label: "Local Runtime",
    code: "RUN",
    href: "/local-runtime",
    category: "system",
    categoryLabel: "System & Meta",
    description: "本地容器执行时与守护进程控制台",
    icon: Server,
  },
  {
    id: "audit",
    label: "Audit Log",
    code: "AUD",
    href: "/audit",
    category: "system",
    categoryLabel: "System & Meta",
    description: "不可篡改操作合规审计日志",
    icon: History,
  },
  {
    id: "automation",
    label: "Automation Triggers",
    code: "AUT",
    href: "/automation",
    category: "system",
    categoryLabel: "System & Meta",
    description: "事件驱动自动化与 Webhook 调度",
    icon: Zap,
  },
  {
    id: "relation",
    label: "Entity Relations",
    code: "REL",
    href: "/relation",
    category: "system",
    categoryLabel: "System & Meta",
    description: "跨域实体依赖网状图谱",
    icon: Network,
  },
  {
    id: "permission",
    label: "RBAC Permissions",
    code: "SEC",
    href: "/permission",
    category: "system",
    categoryLabel: "System & Meta",
    description: "细粒度角色权限策略矩阵",
    icon: ShieldCheck,
  },
  {
    id: "identity",
    label: "Identity & Members",
    code: "IDN",
    href: "/identity",
    category: "system",
    categoryLabel: "System & Meta",
    description: "成员账号、组织架构与鉴权身份",
    icon: Users,
  },
  {
    id: "tenant",
    label: "Tenant Admin",
    code: "TNT",
    href: "/tenant",
    category: "system",
    categoryLabel: "System & Meta",
    description: "多租户隔离与资源配额控制",
    icon: Building2,
  },
  // ── Refactor Sweep (per 2026-09-02 10:41 JST 拍板) ──
  // 分批对已完成任务做重构, 5 状态 todo/doing/testing/review/done 看板,
  // 走完一轮回到 todo, 累计轮次. 跟 Kanban 一样可自定义列.
  {
    id: "refactor",
    label: "Refactor Sweep",
    code: "RFS",
    href: "/refactor",
    category: "work",
    categoryLabel: "Work Management",
    description: "分批次重构已完成任务 · Jira 风格 todo→done 循环, 列可自定义",
    icon: RefreshCw,
  },
];

export const MODULE_MAP = new Map<string, ModuleDefinition>(
  ALL_MODULES.map((m) => [m.id, m])
);
