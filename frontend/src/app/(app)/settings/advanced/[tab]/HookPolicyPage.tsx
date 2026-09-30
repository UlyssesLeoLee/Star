/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx",type:"file",language:"tsx"}),
  (page:Function {name:"HookPolicyPage",type:"function",signature:"HookPolicyPage()",visibility:"public",complexity:"complex"}),
  (executionPanel:Function {name:"HookExecutionEventPanel",type:"function",signature:"HookExecutionEventPanel({api,projectId})",visibility:"private",complexity:"moderate"}),
  (eventDecisionLabel:Function {name:"hookEventDecisionLabel",type:"function",signature:"hookEventDecisionLabel(decision)",visibility:"private",complexity:"simple"}),
  (ruleEditor:Function {name:"HookRuleEditor",type:"function",signature:"HookRuleEditor({rule,onChange,onDelete})",visibility:"private",complexity:"complex"}),
  (conditionEditor:Function {name:"HookConditionEditor",type:"function",signature:"HookConditionEditor({condition,onChange,onDelete})",visibility:"private",complexity:"moderate"}),
  (loadPolicy:Function {name:"loadHookPolicy",type:"function",signature:"loadHookPolicy(api,scope,projectId,worktreeId)",visibility:"private",complexity:"simple"}),
  (editorDocument:Function {name:"documentForEditor",type:"function",signature:"documentForEditor(response,scope,worktreeId)",visibility:"private",complexity:"moderate"}),
  (digest:Function {name:"computePolicyDigest",type:"function",signature:"computePolicyDigest(document)",visibility:"private",complexity:"moderate"}),
  (formatError:Function {name:"formatError",type:"function",signature:"formatError(error)",visibility:"private",complexity:"simple"}),
  (reason:Function {name:"reasonForDecision",type:"function",signature:"reasonForDecision(decision)",visibility:"private",complexity:"simple"}),
  (conditionValue:Function {name:"valueForCondition",type:"function",signature:"valueForCondition(field,value)",visibility:"private",complexity:"simple"}),
  (operators:Function {name:"operatorsForField",type:"function",signature:"operatorsForField(field)",visibility:"private",complexity:"simple"}),
  (maxRules:Variable {name:"MAX_RULES_PER_SCOPE",type:"variable",language:"typescript"}),
  (maxVisibleEvents:Variable {name:"MAX_VISIBLE_HOOK_EVENTS",type:"variable",language:"typescript"}),
  (newCondition:Function {name:"defaultCondition",type:"function",signature:"defaultCondition()",visibility:"private",complexity:"simple"}),
  (newRule:Function {name:"newRestrictiveRule",type:"function",signature:"newRestrictiveRule()",visibility:"private",complexity:"simple"}),
  (page)-[:CALLS]->(loadPolicy),(page)-[:CALLS]->(editorDocument),(page)-[:CALLS]->(digest),
  (page)-[:CALLS]->(newRule),(page)-[:CALLS]->(reason),(page)-[:CALLS]->(formatError),(page)-[:CALLS]->(executionPanel),
  (executionPanel)-[:CALLS]->(eventDecisionLabel),
  (ruleEditor)-[:CALLS]->(conditionEditor),(conditionEditor)-[:CALLS]->(valueForCondition),
  (conditionEditor)-[:CALLS]->(operators),(newRule)-[:CALLS]->(defaultCondition),
  (page)-[:USES]->(maxRules),
  (executionPanel)-[:USES]->(maxVisibleEvents),
  (file)-[:CONTAINS]->(page),(file)-[:CONTAINS]->(ruleEditor),(file)-[:CONTAINS]->(conditionEditor),
  (file)-[:CONTAINS]->(loadPolicy),(file)-[:CONTAINS]->(editorDocument),(file)-[:CONTAINS]->(digest),
  (file)-[:CONTAINS]->(formatError),(file)-[:CONTAINS]->(reason),(file)-[:CONTAINS]->(conditionValue),
  (file)-[:CONTAINS]->(operators),(file)-[:CONTAINS]->(newCondition),(file)-[:CONTAINS]->(newRule),
  (file)-[:CONTAINS]->(executionPanel),(file)-[:CONTAINS]->(eventDecisionLabel);
*/

"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AlertTriangle, Check, CircleHelp, Clock3, FileClock, Plus, RotateCcw, Save, ShieldCheck, Trash2 } from "lucide-react";
import { useWorktreeGroupApi } from "@/lib/group/groupProjection";
import {
  type HookCondition,
  type HookDecision,
  type HookExecutionEvent,
  type HookExecutionEventPage,
  type HookFactField,
  type HookOperator,
  type HookPolicyDocument,
  type HookPolicyResponse,
  type HookRetentionLockState,
  type HookRule,
  type HookValue,
  type WorktreeGroupApiClient,
} from "@/lib/group/worktreeGroupApi";

type PolicyScopeKind = "project" | "worktree";
type ProjectRow = { project_id: string; role: string };
type WorktreeRow = { id: string; name?: string; branch?: string };
type ProjectPage = { projects: ProjectRow[]; next_cursor: string | null };
type WorktreePage = { worktrees: WorktreeRow[]; next_cursor: string | null };

const FACT_FIELDS: Array<{ id: HookFactField; label: string }> = [
  { id: "ActorAuthorized", label: "操作者已授权" },
  { id: "LifecycleVersionMatches", label: "Worktree 生命周期版本匹配" },
  { id: "RuntimeHealthy", label: "Runtime 健康" },
  { id: "RetentionLock", label: "Git 保留锁状态" },
  { id: "ActiveRunCount", label: "活跃 Run 数" },
  { id: "ActiveAgentLeaseCount", label: "活跃 Agent Lease 数" },
  { id: "FileClaimCount", label: "文件 Claim 数" },
  { id: "OwnedProcessCount", label: "未退出进程数" },
];
const DECISIONS: Array<{ id: HookRule["decision"]; label: string }> = [
  { id: "Deny", label: "拒绝操作" },
  { id: "RequireHuman", label: "要求人工审批" },
  { id: "Defer", label: "等待外部条件" },
];
const RETENTION_STATES = ["Fresh", "Missing", "Stale", "Conflict", "Unknown"] as const;
const MAX_RULES_PER_SCOPE = 64;
const MAX_VISIBLE_HOOK_EVENTS = 300;
const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

export default function HookPolicyPage() {
  const api = useWorktreeGroupApi();
  const [projects, setProjects] = useState<ProjectRow[]>([]);
  const [worktrees, setWorktrees] = useState<WorktreeRow[]>([]);
  const [projectId, setProjectId] = useState("");
  const [worktreeId, setWorktreeId] = useState("");
  const [scope, setScope] = useState<PolicyScopeKind>("project");
  const [policy, setPolicy] = useState<HookPolicyResponse | null>(null);
  const [document, setDocument] = useState<HookPolicyDocument | null>(null);
  const [selectedRuleId, setSelectedRuleId] = useState("");
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [refreshKey, setRefreshKey] = useState(0);
  const [staleDraftRebased, setStaleDraftRebased] = useState(false);

  useEffect(() => {
    let active = true;
    setProjects([]);
    setProjectId("");
    setWorktrees([]);
    setWorktreeId("");
    setPolicy(null);
    setDocument(null);
    setStaleDraftRebased(false);
    if (!api) return () => { active = false; };

    api.listAuthorizedProjects<ProjectPage>({ limit: 200 }).then((result) => {
      const rows = Array.isArray(result.projects) ? result.projects.filter(isProjectRow) : [];
      if (!active) return;
      setProjects(rows);
      if (rows[0]) setProjectId(rows[0].project_id);
    }).catch((cause: unknown) => {
      if (active) setError(formatError(cause));
    });
    return () => { active = false; };
  }, [api]);

  useEffect(() => {
    let active = true;
    setWorktrees([]);
    setWorktreeId("");
    if (!api || !projectId) return () => { active = false; };
    api.listProjectWorktrees<WorktreePage>(projectId, { limit: 200, include_archived: false }).then((result) => {
      const rows = Array.isArray(result.worktrees) ? result.worktrees.filter(isWorktreeRow) : [];
      if (!active) return;
      setWorktrees(rows);
      if (rows[0]) setWorktreeId(rows[0].id);
    }).catch((cause: unknown) => {
      if (active) setError(formatError(cause));
    });
    return () => { active = false; };
  }, [api, projectId]);

  useEffect(() => {
    let active = true;
    setPolicy(null);
    setDocument(null);
    setStaleDraftRebased(false);
    setSelectedRuleId("");
    if (!api || !projectId || (scope === "worktree" && !worktreeId)) return () => { active = false; };
    setLoading(true);
    setError("");
    loadHookPolicy(api, scope, projectId, worktreeId).then((result) => {
      if (!active) return;
      const editorDocument = documentForEditor(result, scope, worktreeId);
      setPolicy(result);
      setDocument(editorDocument);
      setStaleDraftRebased(false);
      const rules = scope === "project" ? editorDocument?.project_rules : editorDocument?.worktree_rules;
      setSelectedRuleId(rules?.[0] ? ruleKey(rules[0]) : "");
    }).catch((cause: unknown) => {
      if (active) setError(formatError(cause));
    }).finally(() => {
      if (active) setLoading(false);
    });
    return () => { active = false; };
  }, [api, projectId, scope, worktreeId, refreshKey]);

  const currentRules = useMemo(() => {
    if (!document) return [];
    return scope === "project" ? document.project_rules : document.worktree_rules;
  }, [document, scope]);
  const selectedRule = currentRules.find((rule) => ruleKey(rule) === selectedRuleId) ?? null;
  const activeProject = projects.find((project) => project.project_id === projectId) ?? null;
  const canPublish = activeProject?.role === "tenant_admin" || activeProject?.role === "project_admin";
  const canEdit = Boolean(api && policy && document && (!policy.draft?.is_stale || staleDraftRebased) && !busy);

  const updateRules = useCallback((next: HookRule[]) => {
    setDocument((current) => {
      if (!current) return current;
      return scope === "project"
        ? { ...current, project_rules: next }
        : { ...current, worktree_rules: next };
    });
  }, [scope]);

  const addRule = () => {
    const rule = newRestrictiveRule();
    updateRules([...currentRules, rule]);
    setSelectedRuleId(ruleKey(rule));
    setNotice("");
  };

  const saveDraft = async () => {
    if (!api || !policy || !document || !projectId || (scope === "worktree" && !worktreeId)) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const nextDocument = { ...document };
      if (scope === "project") {
        nextDocument.worktree_id = null;
        nextDocument.worktree_version = null;
        nextDocument.project_version = (policy.policy_document?.project_version ?? 0) + 1;
        nextDocument.worktree_rules = [];
      } else {
        nextDocument.worktree_id = uuidToBytes(worktreeId);
        nextDocument.worktree_version = (policy.policy_document?.worktree_version ?? 0) + 1;
        const baseline = policy.effective_policy_document ?? policy.policy_document;
        if (!baseline) throw new Error("Project Hook baseline is not provisioned; Worktree overlay cannot be created.");
        nextDocument.project_version = baseline.project_version;
        nextDocument.project_rules = baseline.project_rules;
      }
      nextDocument.digest = await computePolicyDigest(nextDocument);
      const body = {
        expected_draft_version: policy.draft?.draft_version ?? 0,
        expected_current_policy_set_id: policy.policy_set_id,
        correlation_id: crypto.randomUUID(),
        policy_document: nextDocument,
      };
      if (scope === "project") await api.saveProjectHookDraft(projectId, body);
      else await api.saveWorktreeHookDraft(worktreeId, body);
      setStaleDraftRebased(false);
      setNotice("草稿已保存；当前生效策略未改变。发布前请检查差异并由管理员确认。");
      setRefreshKey((value) => value + 1);
    } catch (cause) {
      setError(formatError(cause));
    } finally {
      setBusy(false);
    }
  };

  const publishDraft = async () => {
    if (!api || !policy?.draft || !canPublish || policy.draft.is_stale) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const body = { expected_draft_version: policy.draft.draft_version, correlation_id: crypto.randomUUID() };
      if (scope === "project") await api.publishProjectHookDraft(projectId, body);
      else await api.publishWorktreeHookDraft(worktreeId, body);
      setNotice("新策略版本已发布。");
      setRefreshKey((value) => value + 1);
    } catch (cause) {
      setError(formatError(cause));
    } finally {
      setBusy(false);
    }
  };

  const rollback = async (targetPolicySetId: string) => {
    if (!api || !policy?.policy_set_id || !canPublish || !targetPolicySetId) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const body = {
        target_policy_set_id: targetPolicySetId,
        expected_current_policy_set_id: policy.policy_set_id,
        correlation_id: crypto.randomUUID(),
      };
      if (scope === "project") await api.rollbackProjectHookPolicy(projectId, body);
      else await api.rollbackWorktreeHookPolicy(worktreeId, body);
      setNotice("回滚已记录为新的不可变策略版本。");
      setRefreshKey((value) => value + 1);
    } catch (cause) {
      setError(formatError(cause));
    } finally {
      setBusy(false);
    }
  };

  if (!api) {
    return (
      <div className="card border border-amber-500/40 p-5" data-testid="hook-auth-required">
        <div className="flex items-start gap-3"><AlertTriangle className="mt-0.5 text-amber-500" size={18} />
          <div><h2 className="font-semibold text-ink">需要宿主认证会话</h2>
            <p className="mt-1 text-sm text-ink-dim">Hooks 页面已接入受控 Group API 客户端，但应用根 Providers 当前没有提供用户 token/session generation。为避免本地模拟策略被误认为生产数据，当前不显示策略样例，也不允许保存或发布。</p>
            <p className="mt-2 text-xs text-ink-mute">接入 WorktreeGroupApiProvider 后，本页面会使用服务端授权 Project/Worktree 目录与 hook:read / hook:write / hook:publish 权限。</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4" data-testid="hook-policy-page">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div><h2 className="text-lg font-semibold text-ink">Worktree 生命周期 Hook 策略</h2><p className="text-xs text-ink-dim">当前实现覆盖 archive/cleanup 安全门；可视化配置限制性规则，内置安全基线不可关闭、删除或放宽。</p></div>
        <div className="flex flex-wrap items-center gap-2">
          <label className="text-xs text-ink-mute">策略范围
            <select value={scope} onChange={(event) => setScope(event.target.value as PolicyScopeKind)} className="ml-2 rounded border border-line bg-bg-soft px-2 py-1.5 text-ink">
              <option value="project">Project 基线</option><option value="worktree">Worktree 限制覆盖</option>
            </select>
          </label>
          <label className="text-xs text-ink-mute">Project
            <select value={projectId} onChange={(event) => { setWorktreeId(""); setPolicy(null); setDocument(null); setStaleDraftRebased(false); setProjectId(event.target.value); }} className="ml-2 max-w-64 rounded border border-line bg-bg-soft px-2 py-1.5 font-mono text-ink" disabled={!projects.length}>
              {projects.map((project) => <option key={project.project_id} value={project.project_id}>{project.project_id} · {project.role}</option>)}
            </select>
          </label>
          {scope === "worktree" && <label className="text-xs text-ink-mute">Worktree
            <select value={worktreeId} onChange={(event) => setWorktreeId(event.target.value)} className="ml-2 max-w-64 rounded border border-line bg-bg-soft px-2 py-1.5 text-ink" disabled={!worktrees.length}>
              {worktrees.map((worktree) => <option key={worktree.id} value={worktree.id}>{worktree.name ?? worktree.branch ?? worktree.id}</option>)}
            </select>
          </label>}
        </div>
      </header>

      {error && <div role="alert" className="rounded border border-red-500/40 bg-red-500/5 px-3 py-2 text-sm text-red-600">{error}</div>}
      {notice && <div role="status" className="rounded border border-emerald-500/40 bg-emerald-500/5 px-3 py-2 text-sm text-emerald-700">{notice}</div>}
      <HookExecutionEventPanel api={api} projectId={projectId} />
      {loading && <div className="card p-4 text-sm text-ink-mute">正在加载服务端策略…</div>}
      {!loading && !projects.length && <div className="card p-4 text-sm text-ink-dim">当前账号没有可读取的 Project，或授权目录尚不可用。</div>}
      {!loading && scope === "worktree" && projectId && !worktrees.length && <div className="card p-4 text-sm text-ink-dim">所选 Project 暂无可管理 Worktree；Worktree 策略仅能增加 Project 基线之上的限制。</div>}
      {!loading && policy && !document && <div className="card p-4 text-sm text-ink-dim">Project Hook 基线尚未 provision。创建第一条基线需要服务端初始化权限和 tenant scope，当前 UI 不会伪造租户身份或策略文档。</div>}

      {policy && document && (
        <>
          <div className="flex flex-wrap items-center gap-2 text-xs">
            <span className="inline-flex items-center gap-1 rounded border border-line px-2 py-1 text-ink-dim"><ShieldCheck size={13} />Builtin baseline: enforced</span>
            <span className="rounded border border-line px-2 py-1 font-mono text-ink-mute">policy set {policy.policy_set_id ?? "none"}</span>
            <span className="rounded border border-line px-2 py-1 font-mono text-ink-mute">project v{document.project_version} · worktree {document.worktree_version ?? "inherited"}</span>
            {policy.draft && <span className={`rounded border px-2 py-1 ${policy.draft.is_stale ? "border-amber-500/50 text-amber-600" : "border-blue-500/40 text-blue-600"}`}>Draft v{policy.draft.draft_version}{policy.draft.is_stale ? " · stale" : " · pending"}</span>}
          </div>
          {policy.draft?.is_stale && <div className="flex flex-wrap items-center justify-between gap-2 rounded border border-amber-500/40 bg-amber-500/5 p-3 text-xs text-amber-700"><span>草稿基于旧策略版本，不能直接发布。</span><button type="button" disabled={busy} onClick={() => { const current = documentForEditor(policy, scope, worktreeId, false); if (current) { setDocument(current); setStaleDraftRebased(true); } }} className="underline">以当前生效版本为基线重新编辑</button></div>}
          <div className="grid min-h-[500px] gap-3 xl:grid-cols-[250px_minmax(440px,1fr)_300px]">
            <aside className="card p-3" aria-label="Hook 规则列表">
              <div className="mb-3 flex items-center justify-between"><h3 className="text-sm font-semibold text-ink">{scope === "project" ? "Project 规则" : "Worktree 规则"} <span className="font-mono text-[10px] text-ink-mute">{currentRules.length}/{MAX_RULES_PER_SCOPE}</span></h3><button type="button" onClick={addRule} disabled={!canEdit || currentRules.length >= MAX_RULES_PER_SCOPE} className="btn-secondary inline-flex items-center gap-1 text-xs"><Plus size={13} />新增</button></div>
              <div className="space-y-1">
                {currentRules.map((rule) => <button key={ruleKey(rule)} type="button" onClick={() => setSelectedRuleId(ruleKey(rule))} className={`w-full rounded border px-2.5 py-2 text-left ${ruleKey(rule) === selectedRuleId ? "border-accent bg-accent/10" : "border-line hover:bg-bg-soft"}`}>
                  <span className="flex items-center justify-between gap-2 text-xs font-medium text-ink"><span>{rule.decision === "Deny" ? "拒绝" : rule.decision === "RequireHuman" ? "人工审批" : "等待条件"} · P{rule.priority}</span><span className={rule.enabled ? "text-emerald-600" : "text-ink-mute"}>{rule.enabled ? "启用" : "停用"}</span></span>
                  <span className="mt-1 block truncate font-mono text-[10px] text-ink-mute">{bytesToUuid(rule.rule_id)}</span>
                </button>)}
                {!currentRules.length && <p className="rounded border border-dashed border-line p-3 text-xs text-ink-mute">当前范围没有自定义规则。服务端 builtin baseline 始终执行。</p>}
              </div>
            </aside>

            <section className="card p-4" aria-label="规则编辑器">
              {selectedRule ? <HookRuleEditor
                rule={selectedRule}
                disabled={!canEdit}
                onChange={(nextRule) => updateRules(currentRules.map((rule) => ruleKey(rule) === selectedRuleId ? nextRule : rule))}
                onDelete={() => { updateRules(currentRules.filter((rule) => ruleKey(rule) !== selectedRuleId)); setSelectedRuleId(""); }}
              /> : <div className="grid h-full min-h-64 place-items-center text-sm text-ink-mute">选择一条规则，或新增限制性规则。</div>}
              <div className="mt-5 flex flex-wrap gap-2 border-t border-line pt-4">
                <button type="button" onClick={saveDraft} disabled={!canEdit} className="btn-primary inline-flex items-center gap-1.5 text-xs"><Save size={13} />保存草稿</button>
                <button type="button" onClick={publishDraft} disabled={!canPublish || !policy.draft || policy.draft.is_stale || busy} title={!canPublish ? "需要 tenant_admin 或 project_admin" : undefined} className="btn-secondary inline-flex items-center gap-1.5 text-xs"><Check size={13} />发布版本</button>
                <button type="button" onClick={() => setRefreshKey((value) => value + 1)} disabled={busy} className="btn-secondary inline-flex items-center gap-1.5 text-xs">刷新</button>
                {!canPublish && <span className="self-center text-[10px] text-ink-mute">当前 Project 角色不能发布（需 Project/Tenant Admin）。</span>}
              </div>
            </section>

            <aside className="space-y-3">
              <section className="card p-3" aria-label="不可覆盖安全基线">
                <h3 className="flex items-center gap-1.5 text-xs font-semibold text-ink"><ShieldCheck size={14} className="text-emerald-600" />不可覆盖的 Rust 基线</h3>
                <ul className="mt-2 space-y-1.5 text-[11px] text-ink-dim">
                  <li>操作者授权与生命周期版本必须有效</li><li>Runtime 健康、Git lock 新鲜且无冲突</li><li>活跃 Run、Agent lease、文件 claim 与进程必须 drain</li><li>策略/审计不可用时关键操作 fail closed</li>
                </ul>
              </section>
              <section className="card p-3" aria-label="策略审计">
                <h3 className="mb-2 flex items-center gap-1.5 text-xs font-semibold text-ink"><FileClock size={14} />策略审计</h3>
                <div className="max-h-56 space-y-2 overflow-y-auto">
                  {policy.audit.map((entry) => <div key={entry.event_id} className="border-l-2 border-line pl-2 text-[10px]">
                    <div className="flex items-center justify-between gap-2"><span className="font-mono text-ink">{entry.event_type}</span><time className="text-ink-mute">{new Date(entry.occurred_at).toLocaleString()}</time></div>
                    {entry.policy_set_id && entry.policy_set_id !== policy.policy_set_id && canPublish && <button type="button" disabled={busy} onClick={() => rollback(entry.policy_set_id!)} className="mt-1 inline-flex items-center gap-1 text-amber-700 underline"><RotateCcw size={10} />回滚到此版本</button>}
                  </div>)}
                  {!policy.audit.length && <p className="text-[10px] text-ink-mute">暂无策略变更记录。</p>}
                </div>
                <p className="mt-3 flex items-start gap-1 text-[10px] text-ink-mute"><CircleHelp size={12} className="mt-px shrink-0" />这里仅显示策略配置审计。实际执行事件单独列在本 Hooks 选项卡下方，并与配置审计分开。</p>
              </section>
              {policy.draft && <section className="card p-3 text-[11px] text-ink-dim"><h3 className="flex items-center gap-1.5 font-semibold text-ink"><Clock3 size={13} />草稿保留期</h3><p className="mt-1">过期时间：{new Date(policy.draft.expires_at).toLocaleString()}</p></section>}
            </aside>
          </div>
        </>
      )}
    </div>
  );
}

function HookExecutionEventPanel({ api, projectId }: { api: WorktreeGroupApiClient; projectId: string }) {
  const [page, setPage] = useState<HookExecutionEventPage | null>(null);
  const [pageProjectId, setPageProjectId] = useState("");
  const [loading, setLoading] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState("");
  const [refreshKey, setRefreshKey] = useState(0);
  const requestGeneration = useRef(0);

  useEffect(() => {
    const generation = ++requestGeneration.current;
    setPage(null);
    setPageProjectId("");
    setError("");
    if (!projectId) {
      setLoading(false);
      return () => { requestGeneration.current += 1; };
    }

    setLoading(true);
    api.listHookEvents(projectId, { limit: 30 }).then((result) => {
      if (requestGeneration.current !== generation) return;
      setPage({ ...result, events: result.events.slice(0, 30) });
      setPageProjectId(projectId);
    }).catch((cause: unknown) => {
      if (requestGeneration.current === generation) setError(formatError(cause));
    }).finally(() => {
      if (requestGeneration.current === generation) setLoading(false);
    });

    return () => { requestGeneration.current += 1; };
  }, [api, projectId, refreshKey]);

  const loadMore = async () => {
    const cursor = pageProjectId === projectId ? page?.next_cursor : null;
    if (!projectId || !cursor || loadingMore || (page?.events.length ?? 0) >= MAX_VISIBLE_HOOK_EVENTS) return;
    const generation = requestGeneration.current;
    setLoadingMore(true);
    setError("");
    try {
      const nextPage = await api.listHookEvents(projectId, { limit: 30, cursor });
      if (requestGeneration.current !== generation) return;
      setPage((current) => current && pageProjectId === projectId
        ? { ...nextPage, events: [...current.events, ...nextPage.events].slice(0, MAX_VISIBLE_HOOK_EVENTS) }
        : current);
    } catch (cause) {
      if (requestGeneration.current === generation) setError(formatError(cause));
    } finally {
      if (requestGeneration.current === generation) setLoadingMore(false);
    }
  };

  const visiblePage = pageProjectId === projectId ? page : null;
  const coverage = visiblePage?.coverage;
  const events: HookExecutionEvent[] = visiblePage?.events ?? [];

  return (
    <section className="card space-y-3 p-4" aria-label="Hook 执行事件" data-testid="hook-execution-events">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h3 className="flex items-center gap-1.5 text-sm font-semibold text-ink"><FileClock size={14} />Hook 执行事件</h3>
          <p className="mt-1 text-[11px] text-ink-dim">按 Project 授权读取追加式事件账本；此处与策略配置审计分开。每页 30 条，最多在页面保留 300 条。</p>
        </div>
        <button type="button" onClick={() => setRefreshKey((value) => value + 1)} disabled={!projectId || loading} className="btn-secondary text-xs">刷新事件</button>
      </header>

      {coverage && <div className={`rounded border p-3 text-xs ${coverage.status === "complete" ? "border-emerald-500/40 bg-emerald-500/5 text-emerald-800" : "border-amber-500/40 bg-amber-500/5 text-amber-800"}`} role="status" data-testid="hook-event-coverage">
        <div className="flex flex-wrap items-center gap-2 font-medium"><span>覆盖状态：{coverage.status === "partial" ? "部分接入" : coverage.status === "complete" ? "完整" : "未知"}</span><span className="rounded border border-amber-700/20 px-1.5 py-0.5">{coverage.reported_percentage === null ? "覆盖比例未知" : `${coverage.reported_percentage}%`}</span></div>
        <p className="mt-1">当前接入：{coverage.instrumented_phases.length ? coverage.instrumented_phases.join("、") : "无"}。未接入范围不按 0 次处理。</p>
        {coverage.not_yet_instrumented_phases.length > 0 && <p className="mt-1 text-amber-700">尚未接入：{coverage.not_yet_instrumented_phases.join("、")}</p>}
      </div>}

      {loading && <p className="text-xs text-ink-mute">正在读取执行事件…</p>}
      {!projectId && <p className="text-xs text-ink-mute">选择 Project 后读取其授权事件。</p>}
      {error && <p role="alert" className="rounded border border-red-500/40 bg-red-500/5 px-3 py-2 text-xs text-red-600">事件读取失败：{error}</p>}
      {!loading && projectId && visiblePage && events.length === 0 && <p className="rounded border border-dashed border-line p-3 text-xs text-ink-mute">该 Project 当前没有可显示的 Hook 执行事件；覆盖状态请以上方说明为准。</p>}

      {events.length > 0 && <ol className="max-h-80 space-y-2 overflow-y-auto" aria-label="Hook 执行事件列表">
        {events.map((event) => <li key={event.event_id} className="rounded border border-line px-3 py-2 text-xs" data-testid="hook-execution-event">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <span className="font-medium text-ink">{hookEventDecisionLabel(event.hook_decision)} · {event.hook_phase}</span>
            <time className="text-ink-mute" dateTime={event.occurred_at}>{new Date(event.occurred_at).toLocaleString()}</time>
          </div>
          <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-[10px] text-ink-dim">
            <span>原因：{event.hook_reason_code}</span><span>评估：{event.duration_ms} ms</span>
            <span>Project v{event.project_policy_version ?? "未知"} · Worktree v{event.worktree_policy_version ?? "继承/未知"}</span>
            {event.timed_out && <span className="font-medium text-amber-700">超时</span>}
          </div>
          <p className="mt-1 break-all font-mono text-[10px] text-ink-mute">event {event.event_id} · correlation {event.correlation_id}</p>
        </li>)}
      </ol>}

      {visiblePage?.next_cursor && events.length < MAX_VISIBLE_HOOK_EVENTS && <button type="button" onClick={loadMore} disabled={loadingMore || loading} className="btn-secondary text-xs">{loadingMore ? "正在加载…" : "加载更多"}</button>}
      {events.length >= MAX_VISIBLE_HOOK_EVENTS && <p className="text-[10px] text-ink-mute">已达到本页 300 条内存上限；事件记录仍保留在服务端账本中。</p>}
    </section>
  );
}

function hookEventDecisionLabel(decision: string): string {
  if (decision === "allow") return "允许";
  if (decision === "deny") return "拒绝";
  if (decision === "require_human") return "需要人工审批";
  if (decision === "defer") return "等待条件";
  return `未知决策 (${decision})`;
}

function HookRuleEditor({ rule, disabled, onChange, onDelete }: {
  rule: HookRule;
  disabled: boolean;
  onChange: (rule: HookRule) => void;
  onDelete: () => void;
}) {
  const update = (patch: Partial<HookRule>) => onChange({ ...rule, ...patch });
  return (
    <div data-testid="hook-rule-editor">
      <div className="mb-4 flex items-start justify-between gap-3">
        <div><h3 className="text-sm font-semibold text-ink">可视化规则</h3><p className="mt-1 font-mono text-[10px] text-ink-mute">{bytesToUuid(rule.rule_id)}</p></div>
        <button type="button" onClick={onDelete} disabled={disabled} className="inline-flex items-center gap-1 text-xs text-red-600 disabled:opacity-40"><Trash2 size={13} />从草稿移除</button>
      </div>
      <div className="grid gap-3 sm:grid-cols-3">
        <label className="text-xs text-ink-dim">决策<select value={rule.decision} disabled={disabled} onChange={(event) => { const decision = event.target.value as HookRule["decision"]; update({ decision, reason_code: reasonForDecision(decision) }); }} className="mt-1 w-full rounded border border-line bg-bg-soft px-2 py-2 text-ink">{DECISIONS.map((option) => <option key={option.id} value={option.id}>{option.label}</option>)}</select></label>
        <label className="text-xs text-ink-dim">优先级<input type="number" min={-32768} max={32767} value={rule.priority} disabled={disabled} onChange={(event) => update({ priority: Math.max(-32768, Math.min(32767, Number(event.target.value) || 0)) })} className="mt-1 w-full rounded border border-line bg-bg-soft px-2 py-2 text-ink" /></label>
        <label className="flex items-center gap-2 self-end rounded border border-line px-2 py-2 text-xs text-ink"><input type="checkbox" checked={rule.enabled} disabled={disabled} onChange={(event) => update({ enabled: event.target.checked })} />启用规则</label>
      </div>
      <div className="mt-5 flex items-center justify-between"><div><h4 className="text-xs font-semibold text-ink">条件（全部满足时触发）</h4><p className="mt-1 text-[10px] text-ink-mute">条件只读取已授权的结构化事实，不接触命令正文、Secret 或任意脚本。</p></div><button type="button" disabled={disabled || rule.conditions.length >= 8} onClick={() => update({ conditions: [...rule.conditions, defaultCondition()] })} className="btn-secondary inline-flex items-center gap-1 text-[11px]"><Plus size={12} />添加条件</button></div>
      <div className="mt-3 space-y-2">{rule.conditions.map((condition, index) => <HookConditionEditor key={`${ruleKey(rule)}-${index}`} condition={condition} disabled={disabled} onChange={(next) => update({ conditions: rule.conditions.map((current, itemIndex) => itemIndex === index ? next : current) })} onDelete={() => update({ conditions: rule.conditions.filter((_, itemIndex) => itemIndex !== index) })} />)}</div>
    </div>
  );
}

function HookConditionEditor({ condition, disabled, onChange, onDelete }: {
  condition: HookCondition;
  disabled: boolean;
  onChange: (condition: HookCondition) => void;
  onDelete: () => void;
}) {
  const field = condition.field;
  const expectedText = "Boolean" in condition.expected ? String(condition.expected.Boolean)
    : "Count" in condition.expected ? String(condition.expected.Count)
    : condition.expected.RetentionLock;
  const operators = operatorsForField(field);
  return (
    <div className="grid gap-2 rounded border border-line bg-bg-soft/40 p-2 sm:grid-cols-[1fr_1fr_1fr_auto]" data-testid="hook-condition-editor">
      <select aria-label="事实字段" value={field} disabled={disabled} onChange={(event) => { const nextField = event.target.value as HookFactField; onChange(defaultCondition(nextField)); }} className="rounded border border-line bg-bg-soft px-2 py-1.5 text-xs text-ink">{FACT_FIELDS.map((item) => <option key={item.id} value={item.id}>{item.label}</option>)}</select>
      <select aria-label="比较方式" value={condition.operator} disabled={disabled} onChange={(event) => onChange({ ...condition, operator: event.target.value as HookOperator })} className="rounded border border-line bg-bg-soft px-2 py-1.5 text-xs text-ink">{operators.map((operator) => <option key={operator} value={operator}>{operator}</option>)}</select>
      {field === "RetentionLock" ? <select aria-label="期望保留锁状态" value={expectedText} disabled={disabled} onChange={(event) => onChange({ ...condition, expected: { RetentionLock: event.target.value as HookRetentionLockState } })} className="rounded border border-line bg-bg-soft px-2 py-1.5 text-xs text-ink">{RETENTION_STATES.map((value) => <option key={value}>{value}</option>)}</select>
        : field === "ActiveRunCount" || field === "ActiveAgentLeaseCount" || field === "FileClaimCount" || field === "OwnedProcessCount" ? <input aria-label="期望数量" type="number" min={0} max={1_000_000} value={expectedText} disabled={disabled} onChange={(event) => onChange({ ...condition, expected: valueForCondition(field, event.target.value) })} className="rounded border border-line bg-bg-soft px-2 py-1.5 text-xs text-ink" />
          : <select aria-label="期望布尔值" value={expectedText} disabled={disabled} onChange={(event) => onChange({ ...condition, expected: valueForCondition(field, event.target.value) })} className="rounded border border-line bg-bg-soft px-2 py-1.5 text-xs text-ink"><option value="true">true</option><option value="false">false</option></select>}
      <button type="button" aria-label="移除条件" disabled={disabled} onClick={onDelete} className="grid size-8 place-items-center rounded text-ink-mute hover:bg-red-500/10 hover:text-red-600 disabled:opacity-40"><Trash2 size={13} /></button>
    </div>
  );
}

async function loadHookPolicy(api: WorktreeGroupApiClient, scope: PolicyScopeKind, projectId: string, worktreeId: string): Promise<HookPolicyResponse> {
  return scope === "project" ? api.getProjectHookPolicy(projectId) : api.getWorktreeHookPolicy(worktreeId);
}

function documentForEditor(response: HookPolicyResponse, scope: PolicyScopeKind, worktreeId: string, preferDraft = true): HookPolicyDocument | null {
  const source = (preferDraft ? response.draft?.policy_document : null) ?? response.policy_document ?? response.effective_policy_document;
  if (!source) return null;
  if (scope === "project") return { ...source, worktree_id: null, worktree_version: null, worktree_rules: [] };
  if (!UUID_RE.test(worktreeId)) return null;
  return {
    ...source,
    worktree_id: source.worktree_id ?? uuidToBytes(worktreeId),
    worktree_version: source.worktree_version ?? 1,
    worktree_rules: source.worktree_id ? source.worktree_rules : [],
  };
}

async function computePolicyDigest(document: HookPolicyDocument): Promise<number[]> {
  const payload = {
    schema_version: document.schema_version,
    evaluator_api_version: document.evaluator_api_version,
    tenant_id: document.tenant_id,
    project_id: document.project_id,
    worktree_id: document.worktree_id,
    project_version: document.project_version,
    worktree_version: document.worktree_version,
    project_rules: document.project_rules.map((rule) => ({
      rule_id: rule.rule_id,
      priority: rule.priority,
      enabled: rule.enabled,
      decision: rule.decision,
      reason_code: rule.reason_code,
      conditions: rule.conditions.map((condition) => ({ field: condition.field, operator: condition.operator, expected: condition.expected })),
    })),
    worktree_rules: document.worktree_rules.map((rule) => ({
      rule_id: rule.rule_id,
      priority: rule.priority,
      enabled: rule.enabled,
      decision: rule.decision,
      reason_code: rule.reason_code,
      conditions: rule.conditions.map((condition) => ({ field: condition.field, operator: condition.operator, expected: condition.expected })),
    })),
  };
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(JSON.stringify(payload)));
  return Array.from(new Uint8Array(digest));
}

function newRestrictiveRule(): HookRule {
  const id = crypto.randomUUID();
  return {
    rule_id: uuidToBytes(id),
    priority: 100,
    enabled: true,
    decision: "Deny",
    reason_code: "RuleDenied",
    conditions: [defaultCondition("ActiveRunCount")],
  };
}

function defaultCondition(field: HookFactField = "ActiveRunCount"): HookCondition {
  if (field === "RetentionLock") return { field, operator: "Equal", expected: { RetentionLock: "Fresh" } };
  if (field === "ActiveRunCount" || field === "ActiveAgentLeaseCount" || field === "FileClaimCount" || field === "OwnedProcessCount") {
    return { field, operator: "GreaterThan", expected: { Count: 0 } };
  }
  return { field, operator: "Equal", expected: { Boolean: false } };
}

function valueForCondition(field: HookFactField, value: string): HookValue {
  if (field === "RetentionLock") return { RetentionLock: value as HookRetentionLockState };
  if (field === "ActiveRunCount" || field === "ActiveAgentLeaseCount" || field === "FileClaimCount" || field === "OwnedProcessCount") {
    const count = Number(value);
    return { Count: Number.isSafeInteger(count) ? Math.max(0, Math.min(1_000_000, count)) : 0 };
  }
  return { Boolean: value === "true" };
}

function operatorsForField(field: HookFactField): HookOperator[] {
  return field === "ActiveRunCount" || field === "ActiveAgentLeaseCount" || field === "FileClaimCount" || field === "OwnedProcessCount"
    ? ["Equal", "NotEqual", "GreaterThan", "GreaterThanOrEqual", "LessThan", "LessThanOrEqual"]
    : ["Equal", "NotEqual"];
}

function reasonForDecision(decision: HookDecision): HookRule["reason_code"] {
  if (decision === "RequireHuman") return "HumanApprovalRequired";
  if (decision === "Defer") return "ExternalConditionPending";
  return "RuleDenied";
}

function ruleKey(rule: HookRule): string {
  return rule.rule_id.join(",");
}

/* CYPHER STRUCTURE MANIFEST ADDENDUM
MATCH (executionPanel:Function {name:"HookExecutionEventPanel",type:"function"}),
      (listHookEvents:Function {name:"WorktreeGroupApiClient.listHookEvents",type:"function"});
CREATE (executionPanel)-[:CALLS]->(listHookEvents);
*/

function uuidToBytes(uuid: string): number[] {
  if (!UUID_RE.test(uuid)) throw new Error("Group API returned an invalid UUID scope.");
  return uuid.replaceAll("-", "").match(/.{2}/g)?.map((value) => Number.parseInt(value, 16)) ?? [];
}

function bytesToUuid(bytes: number[]): string {
  if (bytes.length !== 16 || bytes.some((byte) => !Number.isInteger(byte) || byte < 0 || byte > 255)) return "invalid rule id";
  const hex = bytes.map((byte) => byte.toString(16).padStart(2, "0")).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

function isProjectRow(value: unknown): value is ProjectRow {
  if (!value || typeof value !== "object") return false;
  const row = value as Record<string, unknown>;
  return typeof row.project_id === "string" && UUID_RE.test(row.project_id) && typeof row.role === "string";
}

function isWorktreeRow(value: unknown): value is WorktreeRow {
  if (!value || typeof value !== "object") return false;
  const row = value as Record<string, unknown>;
  return typeof row.id === "string" && UUID_RE.test(row.id);
}

function formatError(error: unknown): string {
  return error instanceof Error ? error.message : "Group API 请求失败。";
}
