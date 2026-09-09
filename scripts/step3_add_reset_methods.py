#!/usr/bin/env python3
"""step 3/3 idempotent: add reset() to 6 InMemory*Service + pub(crate) service().

per 2026-09-09 15:21 JST 用户发令"按顺序全推" step 3 拍板"现在落地 (推荐)".
per self-review §1 "OnceLock<...> 全局单例在测试间共享状态".
"""
import sys
from pathlib import Path

ROOT = Path(r"D:\Star")

# ===== 1. Add reset() to 6 InMemory*Service (idempotent) =====

RESET_MARK = "/// 测试隔离: 清空所有内存存储 (per self-review §1, v0.31 step 3/3 OnceLock 共享 state)"

# 1a. InMemoryWorkItemService (std::sync::RwLock, sync reset)
p = ROOT / "crates/domain-work-item/src/lib.rs"
t = p.read_text(encoding="utf-8")
if "v0.31 step 3/3" in t and "InMemoryWorkItemService" in t:
    if "fn reset(&self) {\n        self.items.write().unwrap().clear();" in t:
        print("[skip] domain-work-item: reset() already added")
    else:
        # Reset was added in different format. Skip safely.
        print("[skip] domain-work-item: reset() already added (variant)")
else:
    OLD = '''    /// 创建一个空的内存工作项服务实例
    pub fn new() -> Self {
        Self {
            repo: Arc::new(InMemoryWorkItemRepository::new()),
            items: Arc::new(RwLock::new(HashMap::new())),
            requirements: Arc::new(RwLock::new(HashMap::new())),
            acs: Arc::new(RwLock::new(HashMap::new())),
        }
    }'''
    NEW = OLD + '''
    /// 测试隔离: 清空所有内存存储 (per self-review §1, v0.31 step 3/3 OnceLock 共享 state)
    pub fn reset(&self) {
        self.items.write().unwrap().clear();
        self.requirements.write().unwrap().clear();
        self.acs.write().unwrap().clear();
    }'''
    if OLD not in t:
        print("[FAIL] domain-work-item: OLD pattern not found")
        sys.exit(1)
    p.write_text(t.replace(OLD, NEW, 1), encoding="utf-8")
    print("[ok] domain-work-item: + reset()")

# 1b. InMemoryWorkspaceService (tokio::sync::RwLock, async reset)
p = ROOT / "crates/domain-workspace/src/lib.rs"
t = p.read_text(encoding="utf-8")
if "pub async fn reset(&self) {\n        self.workspaces.write().await.clear();" in t:
    print("[skip] domain-workspace: reset() already added")
else:
    OLD = '''    /// 仅 svc(测试用,事件丢)
    pub fn new_for_test() -> Arc<Self> {
        Self::new().0
    }'''
    NEW = OLD + '''
    /// 测试隔离: 清空所有内存存储 (per self-review §1, v0.31 step 3/3 OnceLock 共享 state)
    pub async fn reset(&self) {
        self.workspaces.write().await.clear();
        self.members.write().await.clear();
    }'''
    if OLD not in t:
        print("[FAIL] domain-workspace: OLD pattern not found")
        sys.exit(1)
    p.write_text(t.replace(OLD, NEW, 1), encoding="utf-8")
    print("[ok] domain-workspace: + reset()")

# 1c. InMemoryWorktreeService (std::sync::RwLock, sync reset)
p = ROOT / "crates/domain-worktree/src/lib.rs"
t = p.read_text(encoding="utf-8")
if "fn reset(&self) {\n        self.store.write().unwrap().clear();" in t:
    print("[skip] domain-worktree: reset() already added")
else:
    OLD = '''    /// 使用指定的 Repository 实现构造服务实例
    pub fn with_repo(repo: Arc<dyn WorktreeRepository>) -> Self {
        Self {
            repo,
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}'''
    NEW = '''    /// 使用指定的 Repository 实现构造服务实例
    pub fn with_repo(repo: Arc<dyn WorktreeRepository>) -> Self {
        Self {
            repo,
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    /// 测试隔离: 清空所有内存存储 (per self-review §1, v0.31 step 3/3 OnceLock 共享 state)
    pub fn reset(&self) {
        self.store.write().unwrap().clear();
    }
}'''
    if OLD not in t:
        print("[FAIL] domain-worktree: OLD pattern not found")
        sys.exit(1)
    p.write_text(t.replace(OLD, NEW, 1), encoding="utf-8")
    print("[ok] domain-worktree: + reset()")

# 1d. InMemorySearchService (std::sync::RwLock, sync reset)
p = ROOT / "crates/domain-search/src/lib.rs"
t = p.read_text(encoding="utf-8")
if "fn reset(&self) {\n        self.index.write().unwrap().clear();" in t:
    print("[skip] domain-search: reset() already added")
else:
    OLD = '''    /// 使用指定的 SearchRepository 创建服务实例
    pub fn with_repo(repo: Arc<dyn SearchRepository>) -> Self {
        Self {
            repo,
            index: Arc::new(RwLock::new(HashMap::new())),
            saved: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}'''
    NEW = '''    /// 使用指定的 SearchRepository 创建服务实例
    pub fn with_repo(repo: Arc<dyn SearchRepository>) -> Self {
        Self {
            repo,
            index: Arc::new(RwLock::new(HashMap::new())),
            saved: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    /// 测试隔离: 清空所有内存存储 (per self-review §1, v0.31 step 3/3 OnceLock 共享 state)
    pub fn reset(&self) {
        self.index.write().unwrap().clear();
        self.saved.write().unwrap().clear();
    }
}'''
    if OLD not in t:
        print("[FAIL] domain-search: OLD pattern not found")
        sys.exit(1)
    p.write_text(t.replace(OLD, NEW, 1), encoding="utf-8")
    print("[ok] domain-search: + reset()")

# 1e. InMemoryScmService (tokio::sync::RwLock, async reset)
p = ROOT / "crates/domain-scm/src/lib.rs"
t = p.read_text(encoding="utf-8")
if "pub async fn reset(&self) {\n        self.repos.write().await.clear();" in t:
    print("[skip] domain-scm: reset() already added")
else:
    OLD = '''        (svc, rx)
    }
    /// 构造仅用于测试的服务实例(丢弃事件接收端)
    pub fn new_for_test() -> Arc<Self> {'''
    NEW = '''        (svc, rx)
    }
    /// 测试隔离: 清空所有内存存储 (per self-review §1, v0.31 step 3/3 OnceLock 共享 state)
    pub async fn reset(&self) {
        self.repos.write().await.clear();
        self.branches.write().await.clear();
        self.prs.write().await.clear();
        self.webhooks.write().await.clear();
        self.idempotency.write().await.clear();
        self.pipelines.write().await.clear();
        self.pipeline_external_index.write().await.clear();
        self.reviews.write().await.clear();
    }
    /// 构造仅用于测试的服务实例(丢弃事件接收端)
    pub fn new_for_test() -> Arc<Self> {'''
    if OLD not in t:
        print("[FAIL] domain-scm: OLD pattern not found")
        sys.exit(1)
    p.write_text(t.replace(OLD, NEW, 1), encoding="utf-8")
    print("[ok] domain-scm: + reset()")

# 1f. InMemoryValidationService (std::sync::RwLock, sync reset)
p = ROOT / "crates/domain-validation/src/service.rs"
t = p.read_text(encoding="utf-8")
if "fn reset(&self) {\n        self.results.write().unwrap().clear();" in t:
    print("[skip] domain-validation: reset() already added")
else:
    OLD = '''    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<ValidationEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let svc = Arc::new(Self {
            results: Arc::new(RwLock::new(HashMap::new())),
            evidences: Arc::new(RwLock::new(HashMap::new())),
            evidence_links: Arc::new(RwLock::new(std::collections::HashSet::new())),
            coverages: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
            overrides: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        });
        (svc, rx)
    }'''
    NEW = OLD + '''
    /// 测试隔离: 清空所有内存存储 (per self-review §1, v0.31 step 3/3 OnceLock 共享 state)
    pub fn reset(&self) {
        self.results.write().unwrap().clear();
        self.evidences.write().unwrap().clear();
        self.evidence_links.write().unwrap().clear();
        self.coverages.write().unwrap().clear();
        self.policies.write().unwrap().clear();
        self.overrides.write().unwrap().clear();
    }'''
    if OLD not in t:
        print("[FAIL] domain-validation: OLD pattern not found")
        sys.exit(1)
    p.write_text(t.replace(OLD, NEW, 1), encoding="utf-8")
    print("[ok] domain-validation: + reset()")

# ===== 2. Change `fn service()` to `pub(crate) fn service()` in 10 route files (idempotent) =====

ROUTE_FILES = [
    ("work_items.rs", "fn service()"),
    ("workspaces.rs", "fn service()"),
    ("worktrees.rs", "fn service()"),
    ("code.rs", "fn service()"),
    ("context.rs", "fn search_service()"),
    ("merge_requests.rs", "fn service()"),
    ("reviews.rs", "fn service()"),
    ("validations.rs", "fn service()"),
    ("submissions.rs", "fn service()"),
    ("pipelines.rs", "fn service()"),
]
for fn_name, pat in ROUTE_FILES:
    p = ROOT / f"crates/star-api-rest/src/routes/{fn_name}"
    t = p.read_text(encoding="utf-8")
    pub_pat = pat.replace("fn ", "pub(crate) fn ", 1)
    if pub_pat in t:
        print(f"[skip] routes/{fn_name}: pub(crate) already")
    elif pat in t:
        p.write_text(t.replace(pat, pub_pat, 1), encoding="utf-8")
        print(f"[ok] routes/{fn_name}: {pat} -> pub(crate) {pat}")
    else:
        print(f"[FAIL] routes/{fn_name}: pattern not found")
        sys.exit(1)

print("\nAll step 3 changes applied.")
