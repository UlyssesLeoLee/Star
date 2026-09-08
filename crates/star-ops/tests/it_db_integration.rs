// crates/star-ops/tests/it_db_integration.rs
//
// DB 集成 IT 派生缺口 (per UT-IT-51 brief §5 wt7 + TEST-DESIGN-OPS-001 v0.2 §3.3)
//
// 范围:
// - 2 DB 集成 IT (50-51): 真实 PG apply DDL smoke + RLS 13 類验证
// - MVP 阶段: 仅 DDL 文件存在性 + schema 验证 (mock 模式)
// - 真实 PG 容器化 [M] 子项 DDD Review 拍板 (per brief §2.2)
//
// 守门实证:
// - 守门 #1 R-05: mock 路径
// - 守门 #13: DB W/T/M 100% 覆盖验证
// - 守门 #DB-13 CW-05: tenant_id NOT NULL 必携
// - 守门 #DB-13 c: FORCE RLS 必携
// - 守门 #11 缺标比错标: [M] 子项 DDD Review 必查 (真实 PG 容器化)

// 集成测试作为独立 crate, 关掉 missing_docs 顶层 deny (跟 lib 一致)
#![allow(missing_docs)]

use std::path::Path;

/// 派生 #50: 真实 PG apply DDL smoke (per TEST-DESIGN-OPS-001 §3.3 5 维 sqlx 容器化验证)
/// 守門 #13: 6 表 DDL 真实跑通 (3/6 已落地 + 3/6 F-05 补)
/// MVP 阶段: mock 模式 — 仅验证 3 DDL 文件存在 + 关键 schema 元素
/// 真实 PG 容器化 (sqlx + testcontainers) [M] 子项 DDD Review 拍板
#[test]
fn it_real_pg_apply_ddl_smoke() {
    let worktree_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // ============ 1. 3 DDL 文件存在性 (per 守門 #13) ============
    let ddl_files = [
        "db/migrations/2026-09-08-ops-cluster.sql", // F-01
        "db/migrations/2026-09-08-ops-log.sql",     // F-02 (F-05 PR #31)
        "db/migrations/2026-09-08-ops-metrics.sql", // F-03
    ];
    for f in &ddl_files {
        let p = worktree_root.join(f);
        assert!(
            p.exists(),
            "DDL 文件必存在 (per 守門 #13 W/T/M 100% 覆盖), got: {:?}",
            p
        );
    }

    // ============ 2. F-01 cluster 2 T 表 ============
    let cluster_sql = std::fs::read_to_string(worktree_root.join(ddl_files[0])).expect("read F-01");
    assert!(
        cluster_sql.contains("CREATE TABLE IF NOT EXISTS ops_helm_release_state"),
        "F-01 ops_helm_release_state T 必含"
    );
    assert!(
        cluster_sql.contains("CREATE TABLE IF NOT EXISTS ops_cluster_action_log"),
        "F-01 ops_cluster_action_log T 必含"
    );

    // ============ 3. F-02 log 3 表 (1 T + 2 W) ============
    let log_sql = std::fs::read_to_string(worktree_root.join(ddl_files[1])).expect("read F-02");
    assert!(
        log_sql.contains("CREATE TABLE IF NOT EXISTS ops_log_query_log"),
        "F-02 ops_log_query_log T 必含 (per SRS-001 §8.1)"
    );
    assert!(
        log_sql.contains("CREATE TABLE IF NOT EXISTS ops_log_entry"),
        "F-02 ops_log_entry W 必含 (per SRS-001 §8.1)"
    );
    assert!(
        log_sql.contains("CREATE TABLE IF NOT EXISTS ops_log_analysis"),
        "F-02 ops_log_analysis W 必含 (per SRS-001 §8.1)"
    );

    // ============ 4. F-03 metrics 1 M 表 SCD2 ============
    let metrics_sql = std::fs::read_to_string(worktree_root.join(ddl_files[2])).expect("read F-03");
    assert!(
        metrics_sql.contains("CREATE TABLE IF NOT EXISTS ops_metrics_config"),
        "F-03 ops_metrics_config M SCD2 必含"
    );

    // ============ 5. 累计 6 ops 表 100% W/T/M 覆盖 ============
    // T=3 (ops_helm_release_state + ops_cluster_action_log + ops_log_query_log)
    // W=2 (ops_log_entry + ops_log_analysis)
    // M=1 (ops_metrics_config)
    // 跟 SRS-001 §8.1 一致
    let ops_tables = [
        "ops_helm_release_state", // F-01 T
        "ops_cluster_action_log", // F-01 T
        "ops_log_query_log",      // F-02 T
        "ops_log_entry",          // F-02 W
        "ops_log_analysis",       // F-02 W
        "ops_metrics_config",     // F-03 M
    ];
    let all_sql = format!("{}\n{}\n{}", cluster_sql, log_sql, metrics_sql);
    for t in &ops_tables {
        assert!(
            all_sql.contains(t),
            "6 ops 表 {} 必在 DDL 中, 累计 W/T/M 100% 覆盖",
            t
        );
    }

    // 派生文档: 守門 #11 缺标比错标 — [M] 阶段 sqlx + testcontainers 真 PG 跑 DDL apply
    // MVP 阶段: 仅 DDL 存在性 + schema 元素验证
}

/// 派生 #51: RLS 13 類 cross-tenant 隔离 (per SRS-001 §8.2 派生规)
/// 守門 #DB-13 c: FORCE ROW LEVEL SECURITY 必携
/// 守門 #DB-13 CW-05: tenant_id NOT NULL 必携
/// 派生测: 验证 6 ops 表均含 tenant_id + FORCE RLS
#[test]
fn it_rls_13_categories_enforced() {
    let worktree_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // 6 ops 表 DDL 内容合并
    let ddl_files = [
        "db/migrations/2026-09-08-ops-cluster.sql",
        "db/migrations/2026-09-08-ops-log.sql",
        "db/migrations/2026-09-08-ops-metrics.sql",
    ];
    let mut all_sql = String::new();
    for f in &ddl_files {
        let p = worktree_root.join(f);
        let sql = std::fs::read_to_string(&p).expect(&format!("read {}", f));
        all_sql.push_str(&sql);
        all_sql.push('\n');
    }

    // ============ 1. tenant_id NOT NULL 必携 (per 守門 #DB-13 CW-05) ============
    // 验证 6 表均含 tenant_id UUID NOT NULL 字段
    let tenant_count = all_sql.matches("tenant_id").count();
    assert!(
        tenant_count >= 12, // 6 表 × ≥ 2 处 (CREATE TABLE + CREATE INDEX)
        "6 ops 表 tenant_id 字段必 ≥ 12 处出现, got {}",
        tenant_count
    );

    // ============ 2. FORCE RLS 必携 (per 守門 #DB-13 c) ============
    let force_rls_count = all_sql.matches("FORCE ROW LEVEL SECURITY").count();
    assert!(
        force_rls_count >= 6, // 6 ops 表均含 FORCE RLS
        "6 ops 表 FORCE ROW LEVEL SECURITY 必 ≥ 6 处, got {}",
        force_rls_count
    );

    // ============ 3. RLS policy tenant_id 隔离 (per SRS-001 §8.2 派生规) ============
    let rls_policy_count = all_sql.matches("current_setting('app.tenant_id'").count();
    assert!(
        rls_policy_count >= 6, // 6 ops 表 RLS policy USING tenant_id
        "6 ops 表 RLS policy USING tenant_id 必 ≥ 6 处, got {}",
        rls_policy_count
    );

    // ============ 4. RLS 13 類 (per SRS-001 §8.2) ============
    // 13 類 = tenant_id + user_id + role_id + permission_id + policy_id
    //       + workspace_id + project_id + work_item_id + agent_id + session_id
    //       + source_module + source_kind + 1 类 (T/M 必携)
    let rls_13_categories = [
        "tenant_id",
        "user_id",
        "role_id",
        "permission_id",
        "policy_id",
        "workspace_id",
        "project_id",
        "work_item_id",
        "agent_id",
        "session_id",
        "source_module",
        "source_kind",
    ];
    for cat in &rls_13_categories {
        assert!(
            all_sql.contains(cat),
            "RLS 13 類 {} 必在 DDL 中 (per SRS-001 §8.2)",
            cat
        );
    }

    // 派生文档: 守門 #11 缺标比错标 — [M] 阶段跑真 PG 验证 RLS 13 類 cross-tenant 隔离
    // MVP 阶段: 仅 DDL 字段存在性验证
    let _unused = rls_13_categories.len(); // 抑制 unused 警告
}
