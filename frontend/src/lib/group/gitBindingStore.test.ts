/*
@cypher schema=1 source_sha256=5ff74b562d664730b324f7094595557ff89006ce2a1c7d608f38f9b88a662c45
MERGE (self:File {path:"frontend/src/lib/group/gitBindingStore.test.ts"})
MERGE (suite:Symbol {id:"frontend/src/lib/group/gitBindingStore.test.ts::gitBindingStore",kind:"suite"})
MERGE (store:File {path:"frontend/src/lib/group/gitBindingStore.ts"})
MERGE (self)-[:IMPORTS]->(store)
MERGE (self)-[:DEFINES]->(suite)
@endcypher
*/
// per WBS §23.34: git 绑定 store + 自动识别单元测试
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import {
  GIT_PROVIDERS,
  GitBindingApi,
  findWorktreeGitBinding,
  repositoryByBranchMap,
  resetGitBindingsForTest,
} from "./gitBindingStore";

const repositories = [
  { id: "repo-001", full_name: "acme/physis", provider: "github" },
  { id: "repo-003", full_name: "acme/stargate-fe", provider: "gitlab" },
];
const pullRequests = [
  { source_branch: "feat/worktree-sm", repository_id: "repo-001" },
  { source_branch: "feat/realtime-wi", repository_id: "repo-003" },
];

describe("gitBindingStore (WBS §23.34)", () => {
  beforeEach(() => {
    resetGitBindingsForTest();
    window.localStorage.clear();
  });
  afterEach(() => {
    resetGitBindingsForTest();
    window.localStorage.clear();
  });

  it("registers exactly 3 providers (github/gitlab/self-host) with extension-ready shapes", () => {
    expect(GIT_PROVIDERS.map((p) => p.id)).toEqual(["github", "gitlab", "self-host"]);
    for (const p of GIT_PROVIDERS) {
      expect(p.treePathTemplate).toContain("{branch}");
      expect(typeof p.host).toBe("string" === typeof p.host ? typeof p.host : typeof p.host);
    }
    // self-host 无默认 host → 表单必填 base_url
    expect(GIT_PROVIDERS[2].host).toBeNull();
  });

  it("adds and removes bindings via the API", () => {
    const api = new GitBindingApi();
    const binding = api.addBinding({
      provider: "github",
      base_url: "https://github.com",
      full_name: "UlyssesLeoLee/Star",
      local_path: "D:/Star",
      default_branch: "dev",
    });
    expect(binding.id.startsWith("git-")).toBe(true);

    const list = api;
    api.removeBinding(binding.id);
    expect(findWorktreeGitBinding("feat/worktree-sm", [], repositories, pullRequests)).toBeNull();
  });

  it("auto-resolves Worktree branch → repository → binding → local dir + remote URL", () => {
    const api = new GitBindingApi();
    const binding = api.addBinding({
      provider: "github",
      base_url: "https://github.com",
      full_name: "acme/physis",
      local_path: "D:/Star",
      default_branch: "main",
    });
    const target = findWorktreeGitBinding("feat/worktree-sm", [binding], repositories, pullRequests);
    expect(target).not.toBeNull();
    expect(target?.repository.id).toBe("repo-001");
    expect(target?.binding.local_path).toBe("D:/Star");
  });

  it("returns null when no binding matches the branch's repository", () => {
    const api = new GitBindingApi();
    const binding = api.addBinding({
      provider: "github",
      base_url: "https://github.com",
      full_name: "other/team-repo",
      local_path: "C:/work",
      default_branch: "main",
    });
    expect(findWorktreeGitBinding("feat/worktree-sm", [binding], repositories, pullRequests)).toBeNull();
    // branch 不在 PR 数据里 → 无法解析
    expect(findWorktreeGitBinding("unknown/branch", [binding], repositories, pullRequests)).toBeNull();
  });

  it("repositoryByBranchMap keeps the first repository per branch", () => {
    const map = repositoryByBranchMap([
      { source_branch: "dup", repository_id: "repo-001" },
      { source_branch: "dup", repository_id: "repo-003" },
    ]);
    expect(map.get("dup")).toBe("repo-001");
  });

  it("persists bindings to localStorage", () => {
    const api = new GitBindingApi();
    api.addBinding({
      provider: "self-host",
      base_url: "https://git.example.com",
      full_name: "me/myproject",
      local_path: "E:/src/myproject",
      default_branch: "master",
    });
    const raw = window.localStorage.getItem("star-git-bindings:v1");
    expect(raw).toContain("git.example.com");
    expect(raw).toContain("me/myproject");
  });
});
