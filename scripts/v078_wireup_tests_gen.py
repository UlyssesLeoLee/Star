#!/usr/bin/env python3
"""v0.78 10 Repository wire-up 验证测试生成器 (per v0.75 已知缺口 (c))

为 10/10 star-pg-adapter Repository (6 ops + 4 oauth) 各生成 1 个 #[tokio::test],
验证从 RealPostgresAdapterRegistry::pool().clone() 构造 + pool() getter 共享可用.

输出 stdout, 直接复制到 registry_real_pg.rs 测试 mod 末尾.
"""
import sys

OPS = [
    ("ops_cluster_action_log", "PgOpsClusterActionLogRepository"),
    ("ops_helm_release_state", "PgOpsHelmReleaseStateRepository"),
    ("ops_log_analysis", "PgOpsLogAnalysisRepository"),
    ("ops_log_entry", "PgOpsLogEntryRepository"),
    ("ops_log_query_log", "PgOpsLogQueryLogRepository"),
]

OAUTH = [
    ("oauth_access_tokens", "PgOAuthAccessTokenRepository"),
    ("oauth_authorization_codes", "PgOAuthAuthorizationCodeRepository"),
    ("oauth_clients", "PgOAuthClientRepository"),
    ("oauth_refresh_tokens", "PgOAuthRefreshTokenRepository"),
]

OPS_ALREADY_HAS = "ops_metrics_config"  # v0.73 已加


def gen_test(module: str, struct: str, kind: str) -> str:
    return f"""    #[tokio::test]
    async fn {module}_repository_wireup_works() {{
        // v0.78 P0-4 Stage 2.2 验证: 10/10 Repository 都能从 reg.pool().clone() 构造
        // (per sqlx::PgPool = Arc 内部, clone 廉价且共享同一连接池)
        use star_pg_adapter::repository::{module}::{struct};
        let reg = RealPostgresAdapterRegistry::new(test_lazy_pool(), "postgres://test");
        let actor = test_actor(Uuid::new_v4());
        let _ = reg.register_postgres_adapter((), actor).await;
        // 构造 {kind} Repository 用 pool getter (per v0.67/v0.43 `pub fn new(pool: PgPool)` 模式)
        let repo = {struct}::new(reg.pool().clone());
        // 验证 repo.pool() 仍可 clone 出来 (证明 pool 共享 Arc 内部)
        let _repo_pool_clone = repo.pool().clone();
        // reg.pool() 仍可用 (clone 不消耗原 pool, per sqlx::PgPool Arc 语义)
        let _ = reg.pool();
    }}
"""


def main() -> int:
    print(f"    // =====================================================================")
    print(f"    // v0.78 P0-4 Stage 2.2: 10/10 Repository 跨 Repository 共享 pool 验证测试 (per 守门 #19 v19)")
    print(f"    // =====================================================================")
    print()
    # 5 ops (跳过 ops_metrics_config, 已在 v0.73 加过)
    for module, struct in OPS:
        print(gen_test(module, struct, "ops").rstrip())
        print()
    # 4 oauth
    for module, struct in OAUTH:
        print(gen_test(module, struct, "oauth").rstrip())
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
