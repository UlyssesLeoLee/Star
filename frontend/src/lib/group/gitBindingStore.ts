/*
@cypher schema=1 source_sha256=3d5d27a8cc874d5baf6150355b19bbebad3d01cad0396f080963a0ceb15636cc
MERGE (self:File {path:"frontend/src/lib/group/gitBindingStore.ts"})
MERGE (store:Symbol {id:"frontend/src/lib/group/gitBindingStore.ts::useGitBindings",kind:"hook"})
MERGE (api:Symbol {id:"frontend/src/lib/group/gitBindingStore.ts::GitBindingApi",kind:"class"})
MERGE (providers:Symbol {id:"frontend/src/lib/group/gitBindingStore.ts::GIT_PROVIDERS",kind:"constant"})
MERGE (resolve:Symbol {id:"frontend/src/lib/group/gitBindingStore.ts::resolveWorktreeGitTarget",kind:"function"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(api)
MERGE (self)-[:DEFINES]->(providers)
MERGE (self)-[:DEFINES]->(resolve)
@endcypher
*/
"use client";

// =====================================================================
// gitBindingStore — Git 仓库绑定 (per WBS §23.34, 2026-11-13 用户发令)
//
// 用户发令:
//   "没绑定 git 的时候, 点击 ProjectWorktree Index 应该显示绑定各种 git,
//    包括不限于 gitlab github 和我自己的 git, 前期可以支持这 3 个,
//    预留后期支持其他"
//   "绑定好 git 之后从 Worktree 到本地目录到远端 branch 一系列内容
//    都要依照既存功能自动识别好"
//
// 模型:
//   - GIT_PROVIDERS 注册表 (数组), 前期 github/gitlab/self-host 3 条;
//     后期新增 provider 只需向注册表加条目 (预留在 baseUrl/treePath 规则上)。
//   - GitBinding: 绑定记录 (provider + 远端 URL + 本地目录 + 凭据提示)。
//   - resolveWorktreeGitTarget: 依据既存 Repository/PullRequest 数据
//     (source_branch ↔ Worktree.branch), 自动给出本地目录路径与远端 branch URL。
// =====================================================================

import { useSyncExternalStore } from "react";

export type GitProviderId = "github" | "gitlab" | "self-host";

export interface GitProviderDefinition {
  id: GitProviderId;
  label: string;
  /** 默认远端 host; self-host 为 null → 表单必填 base_url */
  host: string | null;
  /** 远端 branch 页面路径模板 ({host}/{owner}/{repo}/tree/{branch}) */
  treePathTemplate: string;
  /** 是否必须填写本地目录 */
  localPathRequired: boolean;
  /** 是否必须填写访问令牌 (前期仅提示, 不校验) */
  tokenOptional: boolean;
}

/**
 * 注册表 — 前期 3 个 provider; 后期新增 (e.g. bitbucket/gitee) 只加条目。
 * treePathTemplate 的 {host} 对 self-host 由用户填写的 base_url 提供。
 */
export const GIT_PROVIDERS: readonly GitProviderDefinition[] = [
  {
    id: "github",
    label: "GitHub",
    host: "github.com",
    treePathTemplate: "https://github.com/{owner}/{repo}/tree/{branch}",
    localPathRequired: true,
    tokenOptional: true,
  },
  {
    id: "gitlab",
    label: "GitLab",
    host: "gitlab.com",
    treePathTemplate: "https://gitlab.com/{owner}/{repo}/-/tree/{branch}",
    localPathRequired: true,
    tokenOptional: true,
  },
  {
    id: "self-host",
    label: "自托管 Git",
    host: null,
    treePathTemplate: "{host}/{owner}/{repo}/src/branch/{branch}",
    localPathRequired: true,
    tokenOptional: true,
  },
];

export interface GitBinding {
  id: string;
  provider: GitProviderId;
  /** 远端 base URL (e.g. https://github.com 或 https://git.mydomain.com) */
  base_url: string;
  /** owner/repo (e.g. UlyssesLeoLee/Star) */
  full_name: string;
  /** 本地 clone 根目录 (e.g. D:/Star) — Worktree 子目录按 name 拼接 */
  local_path: string;
  default_branch: string;
  /** 绑定时间 */
  bound_at_ms: number;
}

export interface WorktreeGitTarget {
  /** 本地目录路径 (local_path + / + worktree.name) */
  local_dir: string;
  /** 远端 branch 页面 URL */
  remote_branch_url: string | null;
  /** 匹配到的仓库 (既存 Repository 数据) */
  matched_repository: string | null;
}

export interface GitBindingCandidate {
  id: string;
  provider: GitProviderId;
  full_name: string;
  branch: string;
}

const STORAGE_KEY = "star-git-bindings:v1";

let bindings: GitBinding[] = loadBindings();
const listeners = new Set<() => void>();

function loadBindings(): GitBinding[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function emitBindingsChange() {
  for (const listener of listeners) listener();
}

function persistBindings() {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(bindings));
  } catch {
    // 隐私模式/配额 → 仅内存态
  }
}

function subscribeBindings(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function getGitBindings(): GitBinding[] {
  return bindings;
}

/** 测试辅助: 清空绑定 */
export function resetGitBindingsForTest() {
  bindings = [];
  emitBindingsChange();
}

export function useGitBindings(): GitBinding[] {
  return useSyncExternalStore(subscribeBindings, getGitBindings, getGitBindings);
}

export class GitBindingApi {
  addBinding(input: Omit<GitBinding, "id" | "bound_at_ms">): GitBinding {
    const binding: GitBinding = {
      ...input,
      id: `git-${globalThis.crypto.randomUUID()}`,
      bound_at_ms: Date.now(),
    };
    bindings = [...bindings, binding];
    persistBindings();
    emitBindingsChange();
    return binding;
  }

  removeBinding(id: string) {
    bindings = bindings.filter((b) => b.id !== id);
    persistBindings();
    emitBindingsChange();
  }
}

export const gitBindingApi = new GitBindingApi();

// =====================================================================
// 自动识别 — 依照既存 Repository/PullRequest 数据解析 Worktree 目标
// =====================================================================

/**
 * 给定候选 (binding + branch) 与既存仓库数据, 产出:
 *   - local_dir: binding.local_path + "/" + worktree.name (既存约定: worktree.name
 *     即本地 checkout 目录名, seed 里 wt-worktree-sm 等)
 *   - remote_branch_url: 按 provider treePathTemplate 拼 (需 host 或 base_url)
 *   - matched_repository: 仓库 full_name 与候选一致的 Repository.id
 */
export function resolveWorktreeGitTarget(
  candidate: GitBindingCandidate,
  binding: GitBinding,
  repositories: Array<{ id: string; full_name: string; provider: string }>,
): WorktreeGitTarget {
  const provider = GIT_PROVIDERS.find((p) => p.id === binding.provider);
  const matched = repositories.find((r) => r.full_name === binding.full_name) ?? null;

  let remote_branch_url: string | null = null;
  if (provider) {
    const [owner, ...rest] = binding.full_name.split("/");
    const repo = rest.join("/");
    if (owner && repo) {
      const host = provider.host ?? binding.base_url.replace(/^https?:\/\//, "").replace(/\/$/, "");
      if (host) {
        remote_branch_url = provider.treePathTemplate
          .replace("{host}", provider.host ?? binding.base_url.replace(/\/$/, ""))
          .replace("{owner}", owner)
          .replace("{repo}", repo)
          .replace("{branch}", encodeURIComponent(candidate.branch));
      }
    }
  }

  const cleanLocal = binding.local_path.replace(/[\\/]+$/, "");
  return {
    local_dir: `${cleanLocal}/${candidate.branch.replace(/[^a-zA-Z0-9._-]/g, "-")}`,
    remote_branch_url,
    matched_repository: matched?.id ?? null,
  };
}

/**
 * 依据既存 PullRequest 数据 (source_branch) 反查 Worktree 关联的仓库:
 * 返回 branch → repository_id 映射 (auto 识别远端 branch 的来源)。
 */
export function repositoryByBranchMap(
  pullRequests: Array<{ source_branch: string; repository_id: string }>,
): Map<string, string> {
  const map = new Map<string, string>();
  for (const pr of pullRequests) {
    if (!map.has(pr.source_branch)) map.set(pr.source_branch, pr.repository_id);
  }
  return map;
}

/**
 * 为一条 Worktree (branch) 自动解析匹配的 Git 绑定与仓库 —
 * 链路: Worktree.branch → PullRequest.source_branch → Repository.full_name → GitBinding.full_name。
 * 返回 null = 无绑定或既存数据未覆盖该 branch (此时 UI 只显示绑定面板提示)。
 */
export function findWorktreeGitBinding(
  branch: string,
  bindings: GitBinding[],
  repositories: Array<{ id: string; full_name: string; provider: string; project_id?: string }>,
  pullRequests: Array<{ source_branch: string; repository_id: string }>,
): { binding: GitBinding; repository: { id: string; full_name: string; provider: string } } | null {
  const repoIdByBranch = repositoryByBranchMap(pullRequests);
  const repositoryId = repoIdByBranch.get(branch);
  const repository = repositoryId
    ? repositories.find((r) => r.id === repositoryId) ?? null
    : null;
  if (!repository) return null;
  const binding = bindings.find((b) => b.full_name === repository.full_name) ?? null;
  if (!binding) return null;
  return { binding, repository };
}
