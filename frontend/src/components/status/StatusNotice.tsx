/*
@cypher schema=1 source_sha256=8603b9245ea6ebae107afbfb973b0aea5bd2b426bffa07a41cc5c89806fcfafb
MERGE (self:File {path:"frontend/src/components/status/StatusNotice.tsx"})
MERGE (component:Symbol {id:"frontend/src/components/status/StatusNotice.tsx::StatusNotice",kind:"component"})
MERGE (infer:Symbol {id:"frontend/src/components/status/StatusNotice.tsx::inferNoticeState",kind:"function"})
MERGE (noticeState:Type {id:"frontend/src/components/status/StatusNotice.tsx::NoticeState"})
MERGE (self)-[:DEFINES]->(component)
MERGE (self)-[:DEFINES]->(infer)
MERGE (self)-[:DEFINES]->(noticeState)
@endcypher
*/

// =====================================================================
// StatusNotice — 统一状态提示组件族
//   七态: loading(加载中) / empty(空数据) / disconnected(未连接) /
//         unauthorized(无权限) / failed(失败) / recovering(恢复中) /
//         success(操作成功)
//   - 每态自带中文"下一步可执行"提示文案 (hint, 可覆盖)
//   - data-testid 规范: status-notice-<state>
//   - 视觉沿用项目 token: bg-bg-card / border-line / text-ink-mute /
//     bg-err / bg-warn / bg-ok / bg-info, 小字号密集 UI 风格
//   - inferNoticeState(err, hasData) 由错误与数据状态推断七态之一
// =====================================================================

import { clsx } from "clsx";
import {
  AlertCircle,
  CheckCircle2,
  Inbox,
  Loader2,
  RefreshCw,
  ShieldAlert,
  WifiOff,
  type LucideIcon,
} from "lucide-react";

export type NoticeState =
  | "loading"
  | "empty"
  | "disconnected"
  | "unauthorized"
  | "failed"
  | "recovering"
  | "success";

export const NOTICE_STATES: readonly NoticeState[] = [
  "loading",
  "empty",
  "disconnected",
  "unauthorized",
  "failed",
  "recovering",
  "success",
] as const;

interface NoticeTone {
  /** 默认中文标题 */
  title: string;
  /** 用户可执行的下一步提示 (中文) */
  hint: string;
  icon: LucideIcon;
  /** 图标容器底色 */
  iconWrap: string;
  /** 图标 / 标题强调色 */
  accent: string;
  /** 密集模式徽标配色 */
  badge: string;
}

const NOTICE_TONES: Record<NoticeState, NoticeTone> = {
  loading: {
    title: "加载中",
    hint: "数据加载中，请稍候；若长时间无响应，请刷新或重试当前操作。",
    icon: Loader2,
    iconWrap: "bg-info/10",
    accent: "text-info",
    badge: "border-info/30 bg-info/5 text-info",
  },
  empty: {
    title: "暂无数据",
    hint: "当前没有可显示的数据。请调整筛选条件，或新建第一条记录。",
    icon: Inbox,
    iconWrap: "bg-bg-soft",
    accent: "text-ink-mute",
    badge: "border-line text-ink-mute",
  },
  disconnected: {
    title: "未连接",
    hint: "后端服务尚未连接。请确认服务已启动；连接恢复后会自动重试。",
    icon: WifiOff,
    iconWrap: "bg-warn/10",
    accent: "text-warn",
    badge: "border-warn/30 bg-warn/5 text-warn",
  },
  unauthorized: {
    title: "无权限",
    hint: "当前会话未获授权。请重新登录，或向管理员申请访问权限后再试。",
    icon: ShieldAlert,
    iconWrap: "bg-err/10",
    accent: "text-err",
    badge: "border-err/30 bg-err/5 text-err",
  },
  failed: {
    title: "失败",
    hint: "操作失败。请点击重试；若持续失败，请查看日志或联系技术支持。",
    icon: AlertCircle,
    iconWrap: "bg-err/10",
    accent: "text-err",
    badge: "border-err/30 bg-err/5 text-err",
  },
  recovering: {
    title: "恢复中",
    hint: "系统正在自动重连恢复，请稍候；无需重复操作，恢复完成后自动继续。",
    icon: RefreshCw,
    iconWrap: "bg-warn/10",
    accent: "text-warn",
    badge: "border-warn/30 bg-warn/5 text-warn",
  },
  success: {
    title: "操作成功",
    hint: "操作已完成。请继续下一步操作，或查看最新结果。",
    icon: CheckCircle2,
    iconWrap: "bg-ok/10",
    accent: "text-ok",
    badge: "border-ok/30 bg-ok/5 text-ok",
  },
};

export interface StatusNoticeProps {
  state: NoticeState;
  /** 覆盖默认标题 (默认按 state 提供中文标题) */
  title?: string;
  /** 覆盖默认"下一步"提示文案 */
  hint?: string;
  /** 密集模式: 只渲染图标 + 标题徽标 (适合窄栏 / 行内) */
  compact?: boolean;
  className?: string;
}

export function StatusNotice({ state, title, hint, compact = false, className }: StatusNoticeProps) {
  const tone = NOTICE_TONES[state];
  const label = title ?? tone.title;
  // 错误类状态用 alert 让读屏立即播报; 其余用 status (非打断式)
  const role =
    state === "failed" || state === "unauthorized" || state === "disconnected" ? "alert" : "status";

  if (compact) {
    return (
      <span
        data-testid={`status-notice-${state}`}
        role={role}
        className={clsx(
          "inline-flex items-center gap-1 rounded border px-1.5 py-0.5 text-[10px] font-medium",
          tone.badge,
          className,
        )}
      >
        <tone.icon size={11} className={state === "loading" ? "animate-spin" : undefined} />
        {label}
      </span>
    );
  }

  return (
    <div
      data-testid={`status-notice-${state}`}
      role={role}
      className={clsx("flex items-start gap-2 rounded-md border border-line bg-bg-card p-2.5", className)}
    >
      <span className={clsx("mt-0.5 grid size-5 shrink-0 place-items-center rounded", tone.iconWrap)}>
        <tone.icon size={12} className={clsx(tone.accent, state === "loading" && "animate-spin")} />
      </span>
      <div className="min-w-0 flex-1">
        <p className={clsx("text-[11px] font-semibold", tone.accent)}>{label}</p>
        <p className="mt-0.5 text-[10px] leading-relaxed text-ink-mute">{hint ?? tone.hint}</p>
      </div>
    </div>
  );
}

// ── 状态推断 ────────────────────────────────────────────────────────

const UNAUTHORIZED_STATUS_RE = /\b40[13]\b/;
const SERVICE_UNAVAILABLE_RE = /\b50[234]\b/;
const DISCONNECTED_RE =
  /ECONNREFUSED|ENOTFOUND|ETIMEDOUT|EAI_AGAIN|EPIPE|ERR_NETWORK|network|offline|timed out|time out|timeout|超时|网络|无法连接|连接失败|unreachable|refused|fetch failed|failed to fetch/i;
const UNAUTHORIZED_RE =
  /unauthori[sz]ed|forbidden|token expired|invalid token|bad credentials|未授权|无权限|权限不足|请重新登录|登录/i;

function numericStatus(v: unknown): number | undefined {
  if (typeof v === "number") return v;
  if (v && typeof v === "object") {
    const s = (v as Record<string, unknown>).status;
    if (typeof s === "number") return s;
  }
  return undefined;
}

function normalizeError(err: unknown): string {
  if (err == null) return "";
  if (typeof err === "string") return err;
  if (typeof err !== "object") return String(err);
  const e = err as Record<string, unknown>;
  const status = numericStatus(e.status) ?? numericStatus(e.response);
  const code = typeof e.code === "string" || typeof e.code === "number" ? String(e.code) : "";
  const message = typeof e.message === "string" ? e.message : "";
  return [status != null ? `HTTP ${status}` : "", message, code].filter(Boolean).join(" | ");
}

/**
 * 由错误与数据状态推断七态之一:
 * - 有错误: 401/403 → unauthorized; 网络/超时/连接类错误或 502/503/504 → disconnected; 其他 → failed
 * - 无错误: hasData=true → success; 否则 → empty
 * 注意: loading / recovering 需要显式上下文 (是否有进行中的请求), 本函数不会推断返回这两态。
 */
export function inferNoticeState(err?: unknown, hasData?: boolean): NoticeState {
  const msg = normalizeError(err);
  if (msg) {
    if (UNAUTHORIZED_STATUS_RE.test(msg)) return "unauthorized";
    if (DISCONNECTED_RE.test(msg)) return "disconnected";
    if (SERVICE_UNAVAILABLE_RE.test(msg)) return "disconnected";
    if (UNAUTHORIZED_RE.test(msg)) return "unauthorized";
    return "failed";
  }
  return hasData ? "success" : "empty";
}
