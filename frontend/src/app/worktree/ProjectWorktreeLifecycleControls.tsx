/* CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/worktree/ProjectWorktreeLifecycleControls.tsx",type:"file",language:"typescript"}),
  (component:Function {name:"ProjectWorktreeLifecycleControls",type:"function",visibility:"public",complexity:"complex"}),
  (loadRepositories:Function {name:"ProjectWorktreeLifecycleControls repository-loading effect",type:"function",visibility:"private",complexity:"moderate"}),
  (discoverCandidates:Function {name:"discoverCandidates",type:"function",visibility:"private",complexity:"moderate"}),
  (submitOperation:Function {name:"submitOperation",type:"function",visibility:"private",complexity:"complex"}),
  (validateRepositories:Function {name:"validateProjectRepositoryEnvelope",type:"function",visibility:"private",complexity:"moderate"}),
  (validateCandidates:Function {name:"validateCandidateEnvelope",type:"function",visibility:"private",complexity:"moderate"}),
  (validateReceipt:Function {name:"validateLifecycleReceipt",type:"function",visibility:"private",complexity:"moderate"}),
  (safeName:Function {name:"isSafeDisplayName",type:"function",visibility:"private",complexity:"simple"}),
  (validRef:Function {name:"isGitRef",type:"function",visibility:"private",complexity:"moderate"}),
  (exactKeys:Function {name:"hasExactKeys",type:"function",visibility:"private",complexity:"simple"}),
  (errorMessage:Function {name:"errorMessage",type:"function",visibility:"private",complexity:"simple"}),
  (repoEnvelope:Class {name:"ProjectRepositoryEnvelope",type:"interface",language:"typescript"}),
  (candidateEnvelope:Class {name:"ImportCandidateEnvelope",type:"interface",language:"typescript"}),
  (lifecycleState:Class {name:"LifecycleState",type:"interface",language:"typescript"}),
  (props:Class {name:"ProjectWorktreeLifecycleControls Props",type:"interface",language:"typescript"}),
  (uuidPattern:Variable {name:"UUID_PATTERN",type:"variable",language:"typescript"}),
  (candidatePattern:Variable {name:"CANDIDATE_ID_PATTERN",type:"variable",language:"typescript"});
MATCH (file:File {name:"frontend/src/app/worktree/ProjectWorktreeLifecycleControls.tsx"});
CREATE (file)-[:CONTAINS]->(component),(file)-[:CONTAINS]->(loadRepositories),(file)-[:CONTAINS]->(discoverCandidates),(file)-[:CONTAINS]->(submitOperation),(file)-[:CONTAINS]->(validateRepositories),(file)-[:CONTAINS]->(validateCandidates),(file)-[:CONTAINS]->(validateReceipt),(file)-[:CONTAINS]->(safeName),(file)-[:CONTAINS]->(validRef),(file)-[:CONTAINS]->(exactKeys),(file)-[:CONTAINS]->(errorMessage),
       (file)-[:CONTAINS]->(repoEnvelope),(file)-[:CONTAINS]->(candidateEnvelope),(file)-[:CONTAINS]->(lifecycleState),(file)-[:CONTAINS]->(props),(file)-[:CONTAINS]->(uuidPattern),(file)-[:CONTAINS]->(candidatePattern),
       (component)-[:CALLS]->(loadRepositories),(component)-[:CALLS]->(discoverCandidates),(component)-[:CALLS]->(submitOperation),(loadRepositories)-[:CALLS]->(validateRepositories),(loadRepositories)-[:CALLS]->(errorMessage),(discoverCandidates)-[:CALLS]->(validateCandidates),(discoverCandidates)-[:CALLS]->(errorMessage),(submitOperation)-[:CALLS]->(validateReceipt),(submitOperation)-[:CALLS]->(errorMessage),
       (validateRepositories)-[:CALLS]->(exactKeys),(validateRepositories)-[:CALLS]->(validRef),(validateRepositories)-[:CALLS]->(safeName),(validateCandidates)-[:CALLS]->(exactKeys),(validateCandidates)-[:CALLS]->(validRef),(validateCandidates)-[:CALLS]->(safeName),(validateReceipt)-[:CALLS]->(exactKeys),(validateReceipt)-[:CALLS]->(validRef),(component)-[:USES]->(props),(validateRepositories)-[:USES]->(uuidPattern),(validateCandidates)-[:USES]->(candidatePattern);
*/

"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { FormEvent } from "react";
import type {
  ProjectWorktreeLifecycleReceipt,
  ProjectWorktreeRepository,
  WorktreeGroupApiClient,
  WorktreeImportCandidate,
} from "@/lib/group/worktreeGroupApi";

interface ProjectRepositoryEnvelope {
  project_id: string;
  repositories: ProjectWorktreeRepository[];
}

interface ImportCandidateEnvelope {
  project_id: string;
  repository_id: string;
  candidates: WorktreeImportCandidate[];
}

interface LifecycleState {
  mode: "loading" | "ready" | "error";
  repositories: ProjectWorktreeRepository[];
  error?: string;
}

interface Props {
  api: WorktreeGroupApiClient | null;
  projectId: string;
  role?: string;
  onAccepted: () => Promise<void> | void;
}

const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const CANDIDATE_ID_PATTERN = /^[A-Za-z0-9_-]{1,128}$/;

export function ProjectWorktreeLifecycleControls({ api, projectId, role, onAccepted }: Props) {
  const [state, setState] = useState<LifecycleState>({ mode: "loading", repositories: [] });
  const [repositoryId, setRepositoryId] = useState("");
  const [mode, setMode] = useState<"create" | "import">("create");
  const [branch, setBranch] = useState("");
  const [baseRef, setBaseRef] = useState("");
  const [candidates, setCandidates] = useState<WorktreeImportCandidate[]>([]);
  const [candidateId, setCandidateId] = useState("");
  const [candidateState, setCandidateState] = useState<"idle" | "loading" | "ready" | "error">("idle");
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const requestSequence = useRef(0);
  const candidateSequence = useRef(0);

  const selectedRepository = useMemo(
    () => state.repositories.find((repository) => repository.repository_id === repositoryId) ?? null,
    [repositoryId, state.repositories],
  );
  const canWrite = role === "tenant_admin" || role === "project_admin" || role === "developer";

  useEffect(() => {
    const sequence = ++requestSequence.current;
    candidateSequence.current += 1;
    setCandidates([]);
    setCandidateId("");
    setCandidateState("idle");
    setMessage("");
    setError("");
    setRepositoryId("");
    setBranch("");
    setBaseRef("");
    if (!api) {
      setState({ mode: "error", repositories: [], error: "宿主认证 API 尚未连接。" });
      return () => { requestSequence.current += 1; };
    }
    setState({ mode: "loading", repositories: [] });
    void api.listProjectWorktreeRepositories<ProjectRepositoryEnvelope>(projectId)
      .then((response) => {
        const repositories = validateProjectRepositoryEnvelope(response, projectId);
        if (sequence !== requestSequence.current) return;
        setState({ mode: "ready", repositories });
        setRepositoryId((current) => repositories.some((repository) => repository.repository_id === current)
          ? current
          : repositories[0]?.repository_id ?? "");
        setBranch(repositories[0]?.default_branch ?? "");
        setBaseRef(repositories[0]?.default_branch ?? "");
      })
      .catch((reason: unknown) => {
        if (sequence !== requestSequence.current) return;
        setState({ mode: "error", repositories: [], error: errorMessage(reason) });
      });
    return () => { requestSequence.current += 1; };
  }, [api, projectId]);

  const discoverCandidates = useCallback(async () => {
    if (!api || !selectedRepository || !canWrite) return;
    const sequence = ++candidateSequence.current;
    setCandidateState("loading");
    setError("");
    setMessage("");
    setCandidates([]);
    setCandidateId("");
    try {
      const response = await api.listWorktreeImportCandidates<ImportCandidateEnvelope>(
        projectId,
        selectedRepository.repository_id,
        25,
      );
      const nextCandidates = validateCandidateEnvelope(response, projectId, selectedRepository.repository_id);
      if (sequence !== candidateSequence.current) return;
      setCandidates(nextCandidates);
      setCandidateId(nextCandidates[0]?.candidate_id ?? "");
      setCandidateState("ready");
      if (nextCandidates.length === 0) setMessage("当前 Project Repository 没有可导入的 Worktree。");
    } catch (reason) {
      if (sequence !== candidateSequence.current) return;
      setCandidateState("error");
      setError(errorMessage(reason));
    }
  }, [api, canWrite, projectId, selectedRepository]);

  const submitOperation = useCallback(async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!api || !selectedRepository || !canWrite || busy) return;
    const selectedCandidate = candidates.find((candidate) => candidate.candidate_id === candidateId);
    if (mode === "import" && !selectedCandidate) {
      setError("请先从当前 Repository 的候选列表中选择一个 Worktree。");
      return;
    }
    setBusy(true);
    setError("");
    setMessage("");
    const correlationId = crypto.randomUUID();
    const idempotencyKey = `worktree-${crypto.randomUUID()}`;
    try {
      const body = mode === "create"
        ? { repository_id: selectedRepository.repository_id, branch: branch.trim(), base_ref: baseRef.trim(), correlation_id: correlationId }
        : { repository_id: selectedRepository.repository_id, candidate_id: selectedCandidate!.candidate_id, correlation_id: correlationId };
      const receipt = mode === "create"
        ? await api.createProjectWorktree<ProjectWorktreeLifecycleReceipt>(projectId, body, idempotencyKey)
        : await api.importProjectWorktree<ProjectWorktreeLifecycleReceipt>(projectId, body, idempotencyKey);
      validateLifecycleReceipt(receipt, projectId, selectedRepository.repository_id, correlationId,
        mode === "create" ? branch.trim() : selectedCandidate!.branch);
      setMessage(`已受理 Worktree 操作：${receipt.worktree_id}（${receipt.state}）。Index 正在刷新。`);
      await onAccepted();
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setBusy(false);
    }
  }, [api, baseRef, branch, busy, canWrite, candidateId, candidates, mode, onAccepted, projectId, selectedRepository]);

  return (
    <section className="card mb-4 grid gap-3" aria-label="Worktree 创建与导入" data-testid="worktree-lifecycle-controls">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h2 className="text-sm font-semibold text-ink">Worktree 创建 / 导入</h2>
          <p className="mt-1 text-xs text-ink-mute">仓库与导入候选来自当前 Project 的授权服务端投影，不读取本地 seed 或展示主机路径。</p>
        </div>
        {state.mode === "loading" && <span role="status" className="text-xs text-ink-mute">正在读取 Project Repository…</span>}
      </div>

      {state.mode === "error" && (
        <div role="alert" className="text-xs text-danger">
          Repository 列表不可用：{state.error} 创建和导入保持关闭。
        </div>
      )}
      {state.mode === "ready" && state.repositories.length === 0 && (
        <div role="status" className="text-xs text-ink-mute">当前 Project 尚未绑定可用 Repository。</div>
      )}
      {state.mode === "ready" && state.repositories.length > 0 && (
        <>
          <div className="flex flex-wrap items-end gap-3">
            <label className="grid gap-1 text-xs text-ink-dim">
              Project Repository
              <select
                aria-label="Project Repository"
                value={repositoryId}
                disabled={busy || !canWrite}
                onChange={(event) => {
                  candidateSequence.current += 1;
                  const next = state.repositories.find((repository) => repository.repository_id === event.target.value);
                  setRepositoryId(event.target.value);
                  setCandidates([]);
                  setCandidateId("");
                  setCandidateState("idle");
                  if (next) {
                    setBranch(next.default_branch);
                    setBaseRef(next.default_branch);
                  }
                  setMessage("");
                }}
                className="min-w-56 rounded-md border border-line bg-bg-soft px-3 py-2 text-sm"
              >
                {state.repositories.map((repository) => (
                  <option key={repository.repository_id} value={repository.repository_id}>
                    {repository.name} · {repository.default_branch}
                  </option>
                ))}
              </select>
            </label>
            <label className="grid gap-1 text-xs text-ink-dim">
              操作
              <select
                aria-label="Worktree 操作"
                value={mode}
                disabled={busy || !canWrite}
                onChange={(event) => setMode(event.target.value as "create" | "import")}
                className="rounded-md border border-line bg-bg-soft px-3 py-2 text-sm"
              >
                <option value="create">创建新 Worktree</option>
                <option value="import">导入已有 Worktree</option>
              </select>
            </label>
            {!canWrite && <span className="text-xs text-warning">当前 Project role 无 Worktree 创建权限。</span>}
          </div>

          {mode === "create" ? (
            <form className="flex flex-wrap items-end gap-3" onSubmit={submitOperation}>
              <label className="grid gap-1 text-xs text-ink-dim">
                新分支
                <input aria-label="新分支" required maxLength={255} value={branch} onChange={(event) => setBranch(event.target.value)}
                  disabled={busy || !canWrite} className="rounded-md border border-line bg-bg-soft px-3 py-2 text-sm" />
              </label>
              <label className="grid gap-1 text-xs text-ink-dim">
                基准 Ref
                <input aria-label="基准 Ref" required maxLength={255} value={baseRef} onChange={(event) => setBaseRef(event.target.value)}
                  disabled={busy || !canWrite} className="rounded-md border border-line bg-bg-soft px-3 py-2 text-sm" />
              </label>
              <button className="btn" type="submit" disabled={busy || !canWrite || !selectedRepository}>
                {busy ? "正在受理…" : "创建 Worktree"}
              </button>
            </form>
          ) : (
            <div className="grid gap-3">
              <button className="btn w-fit" type="button" disabled={busy || !canWrite || candidateState === "loading"}
                onClick={() => void discoverCandidates()}>
                {candidateState === "loading" ? "正在发现…" : "发现可导入 Worktree"}
              </button>
              {candidateState === "ready" && candidates.length > 0 && (
                <form className="flex flex-wrap items-end gap-3" onSubmit={submitOperation}>
                  <label className="grid gap-1 text-xs text-ink-dim">
                    候选 Worktree
                    <select aria-label="候选 Worktree" value={candidateId} onChange={(event) => setCandidateId(event.target.value)}
                      disabled={busy || !canWrite} className="min-w-72 rounded-md border border-line bg-bg-soft px-3 py-2 text-sm">
                      {candidates.map((candidate) => (
                        <option key={candidate.candidate_id} value={candidate.candidate_id}>
                          {candidate.name} · {candidate.branch} · {candidate.dirty ? "有未提交改动" : "干净"}
                        </option>
                      ))}
                    </select>
                  </label>
                  <button className="btn" type="submit" disabled={busy || !canWrite || !candidateId}>
                    {busy ? "正在受理…" : "导入到 Project Index"}
                  </button>
                </form>
              )}
            </div>
          )}
        </>
      )}

      {error && <div role="alert" className="text-xs text-danger">操作失败：{error}</div>}
      {message && <div role="status" className="text-xs text-ink-dim">{message}</div>}
    </section>
  );
}

function validateProjectRepositoryEnvelope(value: ProjectRepositoryEnvelope, projectId: string): ProjectWorktreeRepository[] {
  if (!hasExactKeys(value, ["project_id", "repositories"]) || value.project_id !== projectId
    || !Array.isArray(value.repositories) || value.repositories.length > 100) {
    throw new Error("Project Repository 投影与当前 Project 不匹配");
  }
  const seen = new Set<string>();
  for (const repository of value.repositories) {
    if (!hasExactKeys(repository, ["repository_id", "name", "default_branch"])
      || !UUID_PATTERN.test(repository.repository_id) || seen.has(repository.repository_id)
      || !isSafeDisplayName(repository.name) || !isGitRef(repository.default_branch)) {
      throw new Error("Project Repository 投影包含无效字段");
    }
    seen.add(repository.repository_id);
  }
  return value.repositories;
}

function validateCandidateEnvelope(value: ImportCandidateEnvelope, projectId: string, repositoryId: string): WorktreeImportCandidate[] {
  if (!hasExactKeys(value, ["project_id", "repository_id", "candidates"])
    || value.project_id !== projectId || value.repository_id !== repositoryId
    || !Array.isArray(value.candidates) || value.candidates.length > 50) {
    throw new Error("Worktree 导入候选与当前 Project Repository 不匹配");
  }
  const seen = new Set<string>();
  for (const candidate of value.candidates) {
    if (!hasExactKeys(candidate, ["candidate_id", "repository_id", "name", "branch", "head_commit", "dirty", "observed_at"])
      || !CANDIDATE_ID_PATTERN.test(candidate.candidate_id) || seen.has(candidate.candidate_id)
      || candidate.repository_id !== repositoryId || typeof candidate.name !== "string"
      || !isSafeDisplayName(candidate.name) || !isGitRef(candidate.branch)
      || !/^(?:[0-9a-f]{40}|[0-9a-f]{64})$/i.test(candidate.head_commit)
      || typeof candidate.dirty !== "boolean" || Number.isNaN(Date.parse(candidate.observed_at))) {
      throw new Error("Worktree 导入候选包含无效字段");
    }
    seen.add(candidate.candidate_id);
  }
  return value.candidates;
}

function validateLifecycleReceipt(
  value: ProjectWorktreeLifecycleReceipt,
  projectId: string,
  repositoryId: string,
  correlationId: string,
  expectedBranch: string,
) {
  if (!hasExactKeys(value, ["operation_id", "worktree_id", "project_id", "repository_id", "branch", "state", "accepted_at", "correlation_id"])
    || !UUID_PATTERN.test(value.operation_id) || !UUID_PATTERN.test(value.worktree_id)
    || value.project_id !== projectId || value.repository_id !== repositoryId
    || value.correlation_id !== correlationId || !isGitRef(value.branch) || value.branch !== expectedBranch
    || (value.state !== "provisioning" && value.state !== "ready") || Number.isNaN(Date.parse(value.accepted_at))) {
    throw new Error("Worktree 服务端受理回执与本次请求不匹配");
  }
}

function hasExactKeys(value: unknown, expected: string[]): value is Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const actual = Object.keys(value).sort();
  return actual.length === expected.length && actual.every((key, index) => key === [...expected].sort()[index]);
}

function isGitRef(value: string): boolean {
  if (typeof value !== "string" || value.length === 0 || value.length > 255 || value.startsWith("-")
    || value.startsWith(".") || value.startsWith("/") || value.endsWith(".") || value.endsWith("/")
    || value.includes("..") || value.includes("//") || value.includes("@{") || /[\\:?*\[\]^~\s\u0000-\u001f]/.test(value)
    || /[^\x00-\x7F]/.test(value)) return false;
  return value.split("/").every((part) => part.length > 0 && !part.startsWith(".") && !part.endsWith(".") && !part.endsWith(".lock"));
}

function isSafeDisplayName(value: string): boolean {
  if (typeof value !== "string") return false;
  const trimmed = value.trim();
  return trimmed.length > 0 && new TextEncoder().encode(trimmed).length <= 128
    && !/[\u0000-\u001f\u007f]/.test(trimmed) && !trimmed.includes("://") && !trimmed.includes("..")
    && !trimmed.startsWith("/") && !trimmed.startsWith("\\") && !/^[A-Za-z]:[\\/]/.test(trimmed);
}

function errorMessage(value: unknown): string {
  return value instanceof Error ? value.message : "服务暂不可用，请稍后重试。";
}
