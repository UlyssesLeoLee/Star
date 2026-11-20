/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/(app)/lane/[laneId]/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/(app)/lane/[laneId]/page.tsx::LaneDetailPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
// LaneDetailPage — /lane/[laneId] (P0-lane 2026-11-20)
//
// Lane 管理页: Lane 详情 + Worktree 列表 + 挂 Worktree 入口。
// 数据通道与 ProjectRunTree 一致:
//   - 真实模式 (宿主会话): RunDirectoryApiClient.listLaneWorktrees / attachWorktreeToLane
//   - 演示模式 (web 无宿主): 共享 store 注入的「演示目录」数据, 明确 badge 标注
// 选择 Worktree (?worktree_id=) 后展示 Lane App 页签占位 — 真实 App 页签与 CLI 焦点
// 需服务端 Lane workspace 接线 (父代理待办), 本页只做选择语义与诚实空态。

"use client";

import { useEffect, useMemo, useState, type FormEvent } from "react";
import { useParams, useSearchParams } from "next/navigation";
import { CheckCircle, Layers, Link2, Monitor, Plus, RefreshCw } from "lucide-react";
import { useStore } from "@/lib/store";
import { useRunDirectory } from "@/lib/run/runDirectorySession";
import {
  directoryErrorMessage,
} from "@/lib/run/runDirectoryTree";
import {
  canonicalLaneWorktreeHref,
  type AuthorizedRunLane,
  type LaneWorktree,
} from "@/lib/run/runDirectoryApi";
import { useTranslation } from "@/lib/i18n";

// =====================================================================
// 局部三语文案 (共享 dictionary 不碰; 新增 i18n key 清单见报告父代理待办)
// =====================================================================
const LANE_PAGE_COPY = {
  en: {
    title: "Lane",
    demoBadge: "Demo directory",
    demoHint: "Showing demo directory data; real mode never mixes demo rows.",
    realModeHint: "Lane data is served by the authenticated directory API.",
    noSession: "No host session is assembled; Lane workspace requires the authenticated directory session.",
    worktrees: "Worktrees",
    emptyWorktrees: "This Lane has no bound Worktree yet.",
    loadingWorktrees: "Loading authorized Worktrees…",
    worktreeError: "Worktree directory unavailable",
    selectedWorktree: "Selected Worktree",
    appTabsPending: "Lane App tabs and CLI focus open only after a Worktree is selected. App tabs require server-side Lane workspace wiring (pending).",
    createWorktreeHint: "Worktree creation goes through the existing Worktree Index (real Lane-scoped create endpoint pending).",
    createWorktree: "Create Worktree",
    attachTitle: "Attach Worktree",
    attachHint: "Attach an existing Worktree of this project to the Lane (requires project manager role).",
    attachPlaceholder: "worktree UUID",
    attachButton: "Attach",
    attaching: "Attaching…",
    attachOk: "Attached. Refresh to see the Worktree list.",
    attachError: "Attach failed",
    refresh: "Refresh",
    unboundRemote: "no remote bound",
    laneMeta: "Lane metadata",
    version: "version",
  },
  "zh-CN": {
    title: "Lane",
    demoBadge: "演示目录",
    demoHint: "当前展示演示目录数据; 真实模式绝不混入演示行。",
    realModeHint: "Lane 数据由已认证目录 API 提供。",
    noSession: "宿主会话尚未装配; Lane 工作区需要已认证目录会话。",
    worktrees: "Worktree",
    emptyWorktrees: "该 Lane 尚未挂任何 Worktree。",
    loadingWorktrees: "正在读取已授权 Worktree…",
    worktreeError: "Worktree 目录暂不可用",
    selectedWorktree: "已选择 Worktree",
    appTabsPending: "选择 Worktree 后才打开该 Lane 的 App 页签与 CLI 焦点; App 页签需服务端 Lane workspace 接线 (待办)。",
    createWorktreeHint: "Worktree 创建走现有 Worktree Index (Lane 级创建端点待后端接线)。",
    createWorktree: "创建 Worktree",
    attachTitle: "挂 Worktree",
    attachHint: "把该项目已有的 Worktree 挂到本 Lane (需项目管理员角色)。",
    attachPlaceholder: "Worktree UUID",
    attachButton: "挂载",
    attaching: "挂载中…",
    attachOk: "已挂载, 刷新查看 Worktree 列表。",
    attachError: "挂载失败",
    refresh: "刷新",
    unboundRemote: "未绑定远端",
    laneMeta: "Lane 元数据",
    version: "版本",
  },
  ja: {
    title: "Lane",
    demoBadge: "デモディレクトリ",
    demoHint: "デモディレクトリのデータを表示中; リアルモードでは混入しません。",
    realModeHint: "Lane データは認証済みディレクトリ API から提供されます。",
    noSession: "ホストセッション未組み立て; Lane ワークスペースには認証済みセッションが必要です。",
    worktrees: "Worktree",
    emptyWorktrees: "この Lane にはまだ Worktree がありません。",
    loadingWorktrees: "承認済み Worktree を読み込み中…",
    worktreeError: "Worktree ディレクトリが利用できません",
    selectedWorktree: "選択中 Worktree",
    appTabsPending: "Worktree を選択した後にのみ Lane App タブと CLI フォーカスが開きます; App タブはサーバー側 Lane workspace 配線 (保留) が必要です。",
    createWorktreeHint: "Worktree 作成は既存の Worktree Index 経由 (Lane スコープ作成エンドポイントは保留)。",
    createWorktree: "Worktree を作成",
    attachTitle: "Worktree を紐付け",
    attachHint: "このプロジェクトの既存 Worktree を Lane に紐付けます (プロジェクト管理者ロールが必要)。",
    attachPlaceholder: "Worktree UUID",
    attachButton: "紐付け",
    attaching: "紐付け中…",
    attachOk: "紐付けました。更新して確認してください。",
    attachError: "紐付けに失敗",
    refresh: "更新",
    unboundRemote: "リモート未バインド",
    laneMeta: "Lane メタデータ",
    version: "バージョン",
  },
} as const;
type LanePageCopy = (typeof LANE_PAGE_COPY)["zh-CN"];

export default function LaneDetailPage() {
  const params = useParams<{ laneId: string }>();
  const search = useSearchParams();
  const laneId = typeof params?.laneId === "string" ? params.laneId : "";
  const projectIdParam = search?.get("project_id") ?? "";
  const worktreeIdParam = search?.get("worktree_id") ?? "";
  const { t, language } = useTranslation();
  const copy = LANE_PAGE_COPY[language];

  const { client, status } = useRunDirectory();
  const storeLanes = useStore((s) => s.lanes);
  const storeWorktrees = useStore((s) => s.worktrees);
  const storeProjects = useStore((s) => s.projects);

  const [worktrees, setWorktrees] = useState<LaneWorktree[] | null>(null);
  const [worktreeError, setWorktreeError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [attachValue, setAttachValue] = useState("");
  const [attachBusy, setAttachBusy] = useState(false);
  const [attachMessage, setAttachMessage] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  // 演示数据: store 注入的 Lane (badge 标注); 真实模式: store 为空
  const demo = status !== "ready" || !client;
  const demoLane = useMemo(
    () => storeLanes.find((lane) => lane.id === laneId) ?? null,
    [storeLanes, laneId],
  );
  const demoProject = useMemo(
    () => storeProjects.find((project) => project.id === demoLane?.project_id) ?? null,
    [storeProjects, demoLane],
  );
  const demoWorktrees = useMemo(
    () => storeWorktrees.filter((worktree) => worktree.lane_id === laneId),
    [storeWorktrees, laneId],
  );

  // 真实模式: 经服务端 Lane API 读该 Lane 的 Worktree (需 project_id 上下文)
  const realProjectId = projectIdParam || demoLane?.project_id || "";
  useEffect(() => {
    if (demo || !client || !realProjectId) {
      setWorktrees(null);
      return;
    }
    setLoading(true);
    setWorktreeError(null);
    const controller = new AbortController();
    const lane: AuthorizedRunLane = {
      lane_id: laneId,
      project_id: realProjectId,
      name: "",
      description: null,
      created_by: "",
      lock_version: 0,
      created_at: "",
      updated_at: "",
      repository_binding: null,
      branch_binding: null,
    };
    void client
      .listLaneWorktrees(lane, undefined, controller.signal)
      .then((page) => {
        if (controller.signal.aborted) return;
        setWorktrees(page.rows);
      })
      .catch((error: unknown) => {
        if (!controller.signal.aborted) setWorktreeError(directoryErrorMessage(error));
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => {
      controller.abort();
    };
  }, [client, demo, laneId, realProjectId, refreshKey]);

  async function attachWorktree(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!client || demo || !realProjectId || attachBusy) return;
    const worktreeId = attachValue.trim();
    if (!worktreeId) return;
    setAttachBusy(true);
    setAttachMessage(null);
    try {
      await client.attachWorktreeToLane(
        realProjectId,
        laneId,
        worktreeId,
        globalThis.crypto.randomUUID(),
      );
      setAttachMessage(copy.attachOk);
      setAttachValue("");
      setRefreshKey((key) => key + 1);
    } catch (error: unknown) {
      setAttachMessage(`${copy.attachError}: ${directoryErrorMessage(error)}`);
    } finally {
      setAttachBusy(false);
    }
  }

  if (!laneId) {
    return <section className="p-6" role="alert"><h1 className="text-lg font-semibold">{copy.title}</h1><p className="mt-2 text-sm text-ink-mute">Lane 路径无效。</p></section>;
  }

  const visibleWorktrees: LaneWorktree[] | null = demo ? null : worktrees;
  const selectedWorktree = visibleWorktrees?.find((worktree) => worktree.worktree_id === worktreeIdParam) ?? null;

  return <section data-testid="lane-detail-page" className="min-h-[calc(100vh-4rem)] p-4 md:p-6">
    <header className="border-b border-line pb-4">
      <p className="break-all text-xs text-ink-mute">
        Lane {laneId}{realProjectId ? ` · Project ${realProjectId}` : ""}
      </p>
      <h1 className="mt-2 flex items-center gap-2 break-words text-xl font-semibold">
        <Layers size={20} />
        {demoLane?.name ?? laneId}
        {demo && <span data-testid="demo-directory-badge" title={copy.demoHint}
          className="rounded border border-warn/40 bg-warn/10 px-1 text-[10px] font-normal text-warn">{copy.demoBadge}</span>}
      </h1>
      <p className="mt-2 text-sm text-ink-mute">{demo ? copy.demoHint : copy.realModeHint}</p>
    </header>

    <div className="mt-4 grid gap-4 lg:grid-cols-3">
      <aside className="rounded border border-line bg-bg-soft p-4 text-sm lg:col-span-1">
        <h2 className="font-semibold">{copy.laneMeta}</h2>
        <dl className="mt-3 grid gap-2 break-all text-xs">
          <div><dt className="text-ink-mute">{copy.version}</dt><dd>{demoLane ? "—" : copy.version}</dd></div>
          <div><dt className="text-ink-mute">Project</dt><dd>{demoProject?.name ?? realProjectId || "—"}</dd></div>
          <div>
            <dt className="text-ink-mute">Git</dt>
            <dd>{demoLane?.git_remote_url ?? copy.unboundRemote}</dd>
          </div>
        </dl>
        <div className="mt-4 border-t border-line pt-3 text-xs">
          <p className="text-ink-mute">{copy.createWorktreeHint}</p>
          <a data-testid="lane-create-worktree" className="mt-2 inline-flex items-center gap-1 rounded border border-line px-3 py-2 hover:border-accent"
            href={`/worktree?project_id=${encodeURIComponent(realProjectId || "")}&lane_id=${encodeURIComponent(laneId)}`}>
            <Plus size={14} />{copy.createWorktree}
          </a>
        </div>
        <form onSubmit={attachWorktree} className="mt-4 border-t border-line pt-3">
          <h3 className="flex items-center gap-1 text-xs font-semibold"><Link2 size={12} />{copy.attachTitle}</h3>
          <p className="mt-1 text-[10px] text-ink-mute">{copy.attachHint}</p>
          <div className="mt-2 flex gap-2">
            <input data-testid="lane-attach-worktree-input" value={attachValue} disabled={demo || attachBusy}
              onChange={(event) => setAttachValue(event.target.value)} placeholder={copy.attachPlaceholder} maxLength={64}
              className="min-w-0 flex-1 rounded border border-line bg-bg-card px-2 py-1 text-xs disabled:opacity-50" />
            <button data-testid="lane-attach-worktree-submit" type="submit" disabled={demo || attachBusy || !attachValue.trim()}
              className="rounded border border-line px-2 py-1 text-xs disabled:opacity-50">{attachBusy ? copy.attaching : copy.attachButton}</button>
          </div>
          {attachMessage && <p role="status" className="mt-2 text-[10px] text-ink-mute">{attachMessage}</p>}
        </form>
      </aside>

      <main className="lg:col-span-2">
        <div className="flex items-center justify-between">
          <h2 className="flex items-center gap-1 font-semibold"><Monitor size={16} />{copy.worktrees}</h2>
          {!demo && <button type="button" onClick={() => setRefreshKey((key) => key + 1)}
            className="flex items-center gap-1 rounded border border-line px-2 py-1 text-xs hover:border-accent"><RefreshCw size={12} />{copy.refresh}</button>}
        </div>
        {loading && <p role="status" className="mt-3 text-sm text-ink-mute">{copy.loadingWorktrees}</p>}
        {worktreeError && <p role="alert" className="mt-3 text-sm text-danger">{worktreeError}</p>}
        {!loading && !worktreeError && demo && (
          <div data-testid="lane-worktree-list" className="mt-3 grid gap-2">
            {demoWorktrees.length === 0 && <p className="text-sm text-ink-mute">{copy.emptyWorktrees}</p>}
            {demoWorktrees.map((worktree) => (
              <a key={worktree.id} data-testid={`lane-worktree-${worktree.id}`} className="block rounded border border-line bg-bg-card p-3 hover:border-accent"
                href={`/worktree?project_id=${encodeURIComponent(worktree.project_id)}&lane_id=${encodeURIComponent(laneId)}&worktree_id=${encodeURIComponent(worktree.id)}&branch=${encodeURIComponent(worktree.branch)}`}>
                <span className="block truncate text-sm font-medium">{worktree.name}</span>
                <span className="mt-1 block truncate text-xs text-ink-mute">Worktree · {worktree.branch} · {copy.demoBadge}</span>
              </a>
            ))}
          </div>
        )}
        {!loading && !worktreeError && !demo && (
          <div data-testid="lane-worktree-list" className="mt-3 grid gap-2">
            {worktrees && worktrees.length === 0 && <p className="text-sm text-ink-mute">{copy.emptyWorktrees}</p>}
            {worktrees?.map((worktree) => (
              <a key={worktree.worktree_id} data-testid={`lane-worktree-${worktree.worktree_id}`} className="block rounded border border-line bg-bg-card p-3 hover:border-accent"
                href={canonicalLaneWorktreeHref(worktree.project_id, worktree.lane_id, worktree.worktree_id)}>
                <span className="block truncate text-sm font-medium">{worktree.name}</span>
                <span className="mt-1 block truncate text-xs text-ink-mute">
                  Worktree · {worktree.branch} · {worktree.human_state}/{worktree.machine_state}
                  {worktree.engineering_run_id === null ? " · 未挂 Run" : ` · Run ${worktree.engineering_run_id}`}
                </span>
              </a>
            ))}
          </div>
        )}
        {selectedWorktree && <section data-testid="lane-selected-worktree" className="mt-4 rounded border border-line bg-bg-soft p-4">
          <h3 className="flex items-center gap-1 text-sm font-semibold"><CheckCircle size={14} />{copy.selectedWorktree}</h3>
          <p className="mt-1 break-all text-xs text-ink-mute">{selectedWorktree.name} · {selectedWorktree.branch}</p>
          <p className="mt-2 text-xs text-ink-mute">{copy.appTabsPending}</p>
        </section>}
      </main>
    </div>
  </section>;
}
