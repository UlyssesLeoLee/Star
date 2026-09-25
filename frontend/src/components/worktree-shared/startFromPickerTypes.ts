// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-shared/startFromPickerTypes.ts
//
// ULYS-228 (ULYS-218.1) — FR-ORCA-009 + FR-ORCA-011 frontend DTO types.
//
// 镜像 backend `crates/api/src/worktree_picker.rs` PickerCandidateDto /
// PickerCandidatesDto + `crates/worktree-shared-dir/src/external_worktree_import.rs`
// ExternalWorktree. 这些是 UI ↔ BFF 的契约, 类型改了必须前后端同步改.
//
// 守门: snake_case 字段名跟 serde rename_all = "snake_case" 对齐 (per §B 守门 #11
// 缺标比错标: backend JSON 字段已经是 snake_case, frontend 直接用).

/** 4 选 1 candidate 类型 (per backend PickerCandidateKind). */
export type StartFromKind =
  | "repo_base"
  | "local_branch"
  | "commit_sha"
  | "remote_branch";

/** 单条 picker candidate (per FR-ORCA-009 §3). */
export interface PickerCandidateDto {
  /** 候选 ID (`<kind>:<value>` 格式, per `parse_candidate_id` in worktree-shared-dir). */
  id: string;
  /** 显示名. */
  label: string;
  /** 描述. */
  description: string;
  /** 候选类型 (snake_case). */
  kind: StartFromKind;
}

/**
 * 4 分类候选集合 (per backend PickerCandidatesDto).
 *
 * 注: backend `crates/api/src/worktree_picker.rs` 用的是 4 分类 (`github_branches` /
 * `existing_worktrees` / `local_paths` / `empty`) 而不是 4 选 1 kinds. 我们这个
 * 单独 contract 是给 UI 直接调的 BFF Stage 2 之后的 endpoint, 跟 backend 字段名
 * 一致以便后端替换 schema 时改一处即可.
 */
export interface PickerCandidatesDto {
  /** kind 1: GitHub remote branches (`remote_branch`). */
  github_branches: PickerCandidateDto[];
  /** kind 2: existing local branches (`local_branch`). */
  existing_worktrees: PickerCandidateDto[];
  /** kind 3: 特定 commit SHA 列表 (`commit_sha`). */
  local_paths: PickerCandidateDto[];
  /** kind 4: 默认 base ref (`repo_base`), 单条占位. */
  empty: PickerCandidateDto | null;
}

/** 用户从 modal 选定后回填的 4 选 1 选择 (per StartFromKind). */
export interface StartFromSelection {
  kind: StartFromKind;
  /** 选中 candidate 的 id, e.g. `"local_branch:feat-x"`. */
  candidate_id: string;
  /** 显示 label (UI echo 用, 不参与后续请求). */
  label: string;
}

/** 一条 external (non-Orca) git worktree (per FR-ORCA-011). */
export interface ExternalWorktreeDto {
  /** `git worktree list --porcelain` 的 `worktree` 行绝对路径. */
  path: string;
  /** HEAD commit SHA (full 40 hex). */
  head_commit: string;
  /** 当前分支 (None = detached HEAD). */
  branch: string | null;
  /** `is_managed = false` 表示 Non-Orca 创建 (per AC-1 hidden default). */
  is_managed: boolean;
}

/** External import POST request body. */
export interface ExternalImportRequest {
  /** 要导入的 worktree 绝对路径. */
  path: string;
}

/** External import POST response. */
export interface ExternalImportResponse {
  /** 导入成功后该 worktree 进入 Provisioning 状态, 返回新生成的 worktree_id. */
  worktree_id: string;
  /** 状态机进入的初始态 (per WorktreeStateMachine: "Provisioning"). */
  state: string;
  /** 时间戳. */
  at: string;
}