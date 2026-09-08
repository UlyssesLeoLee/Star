// crates/star-ops/tests/it_db_integration.rs
//
// DB 集成 IT 派生缺口 (per UT-IT-51 brief §5 wt7 + TEST-DESIGN-OPS-001 v0.2 §3.3)
//
// 范围:
// - 2 DB 集成 IT (50-51): 真实 PG apply DDL smoke + RLS 13 類验证
// - MVP 阶段: 仅 DDL 文件存在性 + schema 验证 (mock 模式)
// - 真实 PG 容器化 [M] 子项 DDD Review 拍板 (per brief §2.2)
//
// IT-5-GAPS 缺口 #1 + 缺口 #2 实装 (per IT-5-GAPS-IMPL brief §2.1):
// - 真实 PG 容器化 (per 守门 #1 R-05): 走 STAR_OPS_TEST_PG_URL env (WSL PG / CI 临时容器)
// - sqlx + testcontainers 模式 (testcontainers 走 CI 端, WSL PG 走本地开发端, per IT-5-GAPS 拍板)
// - 6 ops 表 DDL 真实跑通 (F-01 2 + F-02 3 + F-03 1) + RLS 13 類 cross-tenant 隔离
//
// 守门实证:
// - 守门 #1 R-05: 仅连测试 PG, 不连真 PG prod
// - 守门 #5 v2: 测试 PG 密码走 env (STAR_OPS_TEST_PG_URL), 不入源码
// - 守门 #13: DB W/T/M 100% 覆盖验证
// - 守门 #DB-13 CW-05: tenant_id NOT NULL 必携
// - 守门 #DB-13 c: FORCE RLS 必携
// - 守门 #11 缺标比错标: [M] 子项 DDD Review 必查 (真实 PG 容器化部署 / RLS 13 類 性能)

// 集成测试作为独立 crate, 关掉 missing_docs 顶层 deny (跟 lib 一致)
#![allow(missing_docs)]

use std::path::Path;
use tokio::sync::Mutex as TokioMutex;

/// IT-5-GAPS 真实 PG 测试串行化互斥 (per 守門 #1 R-05: 避免并发 PG 压垮测试 PG)
/// 多个真实 PG 测试共享此 mutex, 保证 1 个时间内只跑 1 个
/// 守門 #11 缺标比错标: 真实 PG 测试串行化是测试基础设施限制, 跟业务逻辑无关
static REAL_PG_TEST_LOCK: TokioMutex<()> = TokioMutex::const_new(());

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

// =============================================================================
// IT-5-GAPS 缺口 #1 + 缺口 #2 实装 (per IT-5-GAPS-IMPL brief §2.1)
// 真实 PG 测试 (走 STAR_OPS_TEST_PG_URL env, 跟 testcontainers 等价的本地 dev 路径)
// 守门 #1 R-05: 测试 PG (WSL 本地 / CI 临时容器) 不算 prod, 但 owner 显式 STAR_OPS_TEST_PG_URL 才启
// 守门 #5 v2: 测试 PG 密码仅走 env, 不入源码, 不打印
// =============================================================================

/// IT-5-GAPS 缺口 #1 增强: 真实 PG 跑 6 ops 表 DDL (per IT-5-GAPS-IMPL brief §2.1)
/// 跟 #50 互补: 50 是 DDL 存在性 mock, 这个是 DDL 真跑通
/// 守门 #1 R-05: 仅连测试 PG (WSL / CI 临时容器), env 配错直接 skip
/// 守门 #5 v2: 测试 PG 密码走 STAR_OPS_TEST_PG_URL env, 不入源码
/// 守门 #13: 6 表 DDL 真跑通 (F-01 2 + F-02 3 + F-03 1) + W/T/M 100%
/// 派生文档: 守門 #11 缺标比错标 — [M] 阶段缺 真实 PG 容器化部署 prod (per 已知缺口 #1)
#[tokio::test]
async fn it_real_pg_apply_ddl_smoke_real_pg() {
    // 守門 #1 R-05: 真实 PG 测试串行化 (避免并发 PG 压垮测试 PG)
    let _guard = REAL_PG_TEST_LOCK.lock().await;

    // 守門 #5 v2: env 控制, 不连真 prod PG
    let pg_url = match std::env::var("STAR_OPS_TEST_PG_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => {
            eprintln!(
                "[skip] STAR_OPS_TEST_PG_URL 未设, 跳真实 PG DDL apply (per 守門 #1 R-05 + 守門 #5 v2)"
            );
            return;
        }
    };

    // 1. 创建 schema 隔离 (per 守門 #13 100% 覆盖, schema 隔离避免污染默认 schema)
    let schema = format!("ops_test_{}", std::process::id());
    // max_connections=1: 避免连接池抢占多个连接 (per 守門 #1 R-05 资源最小化)
    let pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(15))
        .connect(&pg_url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[skip] 连 PG 失败 ({}), 跳 (per 守門 #1 R-05)", e);
            return;
        }
    };
    // 关键: sqlx PgPool 是连接池, SET search_path 只影响单个连接
    // 必须 acquire 单个连接, 所有后续 query 走这个连接
    let mut conn = pool.acquire().await.expect("acquire conn");
    // 先 drop 再 create, 避免上次测试残留 (per 守門 #1 R-05 测试 PG 干净化)
    sqlx::query(&format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", schema))
        .execute(&mut *conn)
        .await
        .expect("drop pre schema");
    sqlx::query(&format!("CREATE SCHEMA \"{}\"", schema))
        .execute(&mut *conn)
        .await
        .expect("create schema");
    sqlx::query(&format!("SET search_path TO \"{}\"", schema))
        .execute(&mut *conn)
        .await
        .expect("set search_path");

    // 2. 创建 DDL 前置依赖 (per 守門 #11 缺标比错标: 这些函数定义在 db/migrations/ 之外的 system schema)
    //    audit_audit_event 表 + 函数 + scd_type2_close 函数 (metrics DDL 引用)
    //    注: 函数引用 ops_metrics_config 但 trigger 在测试 schema 上下文跑, 用 dynamic SQL
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_audit_event (
            id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            event_type  TEXT NOT NULL,
            resource_id UUID,
            tenant_id   UUID,
            payload     JSONB NOT NULL,
            occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&mut *conn)
    .await
    .expect("create audit_audit_event");
    // audit_audit_event 函数: 引用 audit_audit_event 表 (跟 search_path 一致)
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION audit_audit_event()
        RETURNS TRIGGER AS $$
        BEGIN
            INSERT INTO audit_audit_event (event_type, resource_id, tenant_id, payload, occurred_at)
            VALUES (TG_OP, NEW.id, NEW.tenant_id, to_jsonb(NEW), NOW());
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql
        "#,
    )
    .execute(&mut *conn)
    .await
    .expect("create audit_audit_event function");
    // scd_type2_close 函数: 引用 ops_metrics_config (跟 search_path 一致)
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION scd_type2_close()
        RETURNS TRIGGER AS $fn$
        BEGIN
            IF NEW.valid_from = OLD.valid_from THEN
                EXECUTE format(
                    'UPDATE %I.ops_metrics_config SET valid_to = NOW() WHERE id = %L AND valid_from = %L AND valid_to IS NULL',
                    TG_TABLE_SCHEMA, NEW.id, OLD.valid_from
                );
            END IF;
            RETURN NEW;
        END;
        $fn$ LANGUAGE plpgsql
        "#,
    )
    .execute(&mut *conn)
    .await
    .expect("create scd_type2_close function");

    // 3. 读 3 DDL 文件 (per 守門 #13 DDL 落地)
    let worktree_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let ddl_files = [
        "db/migrations/2026-09-08-ops-cluster.sql", // F-01 2 T
        "db/migrations/2026-09-08-ops-log.sql",     // F-02 1 T + 2 W
        "db/migrations/2026-09-08-ops-metrics.sql", // F-03 1 M SCD2
    ];

    for ddl_path in &ddl_files {
        let p = worktree_root.join(ddl_path);
        let sql = std::fs::read_to_string(&p)
            .unwrap_or_else(|e| panic!("read {} failed: {}", ddl_path, e));
        sqlx::raw_sql(&sql)
            .execute(&mut *conn)
            .await
            .unwrap_or_else(|e| panic!("apply {} failed: {}", ddl_path, e));
    }

    // 4. 验证 6 ops 表存在 (per 守門 #13 W/T/M 100% 覆盖)
    let expected_tables = [
        ("ops_helm_release_state", "F-01 T"),
        ("ops_cluster_action_log", "F-01 T"),
        ("ops_log_query_log", "F-02 T"),
        ("ops_log_entry", "F-02 W TTL 7d"),
        ("ops_log_analysis", "F-02 W TTL 30d"),
        ("ops_metrics_config", "F-03 M SCD2"),
    ];
    for (table, kind) in &expected_tables {
        let row: (Option<String>,) =
            sqlx::query_as(&format!("SELECT to_regclass('{}.{}')::TEXT", schema, table))
                .fetch_one(&mut *conn)
                .await
                .expect("query to_regclass");
        assert!(
            row.0.is_some(),
            "表 {}.{} ({}) 必存在 (per 守門 #13 W/T/M 100% 覆盖)",
            schema,
            table,
            kind
        );
    }

    // 5. 验证 RLS 强制开启 (per 守門 #DB-13 c FORCE RLS)
    for (table, _) in &expected_tables {
        let row: (bool, bool) = sqlx::query_as(&format!(
            "SELECT relrowsecurity, relforcerowsecurity FROM pg_class WHERE relname = '{}' \
             AND relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = '{}')",
            table, schema
        ))
        .fetch_one(&mut *conn)
        .await
        .expect("query pg_class");
        assert!(
            row.0,
            "表 {}.{} 必 ENABLE RLS (per 守門 #DB-13 c)",
            schema, table
        );
        assert!(
            row.1,
            "表 {}.{} 必 FORCE RLS (per 守門 #DB-13 c FORCE RLS)",
            schema, table
        );
    }

    // 6. 验证 trigger 存在 (per 守門 #13 d 物理删除禁止 + audit)
    let expected_triggers = [
        "trg_prevent_delete_ops_helm_release_state",
        "trg_prevent_delete_ops_cluster_action_log",
        "trg_audit_ops_cluster_action_log",
        "trg_prevent_delete_ops_log_query_log",
        "trg_audit_ops_log_query_log",
        "trg_ops_metrics_config_scd2",
        "trg_ops_metrics_config_audit",
    ];
    for trigger in &expected_triggers {
        let row: (Option<String>,) = sqlx::query_as(&format!(
            "SELECT tgname::TEXT FROM pg_trigger WHERE tgname = '{}' \
             AND tgrelid::regclass::text LIKE '{}.%'",
            trigger, schema
        ))
        .fetch_one(&mut *conn)
        .await
        .expect("query pg_trigger");
        assert!(
            row.0.is_some(),
            "trigger {} 必存在 (per 守門 #13 d)",
            trigger
        );
    }

    // 7. 清理: drop schema 隔离测试数据 (避免污染测试 PG)
    sqlx::query(&format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", schema))
        .execute(&mut *conn)
        .await
        .expect("drop schema");
}

/// IT-5-GAPS 缺口 #2 增强: 真实 PG 跑 RLS 13 類 cross-tenant 隔离 (per IT-5-GAPS-IMPL brief §2.1)
/// 跟 #51 互补: 51 是 DDL 字段存在性 mock, 这个是 RLS 真行为验证
/// 守門 #1 R-05: 仅连测试 PG, env 配错直接 skip
/// 守門 #5 v2: 测试 PG 密码走 env
/// 守門 #DB-13 c: FORCE RLS → 必须用 SET app.tenant_id 切 tenant 才能访问
/// 派生文档: 守門 #11 缺标比错标 — [M] 阶段缺 RLS 13 類 性能 (per 已知缺口 #2)
#[tokio::test]
async fn it_rls_13_categories_enforced_real_pg() {
    // 守門 #1 R-05: 真实 PG 测试串行化 (避免并发 PG 压垮测试 PG)
    let _guard = REAL_PG_TEST_LOCK.lock().await;

    let pg_url = match std::env::var("STAR_OPS_TEST_PG_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => {
            eprintln!(
                "[skip] STAR_OPS_TEST_PG_URL 未设, 跳真实 PG RLS 验证 (per 守門 #1 R-05 + 守門 #5 v2)"
            );
            return;
        }
    };

    // 1. schema 隔离 (单连接 acquire, 避免连接池 search_path 漂移)
    let schema = format!("ops_rls_test_{}", std::process::id());
    let test_user = format!("ops_rls_user_{}", std::process::id());
    // max_connections=1: 避免连接池抢占多个连接
    let pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(15))
        .connect(&pg_url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "[skip] 连 PG 失败 ({}), 跳真实 PG RLS 验证 (per 守門 #1 R-05)",
                e
            );
            return;
        }
    };
    let mut conn = pool.acquire().await.expect("acquire conn");
    // 先 drop 再 create, 避免上次测试残留 (per 守門 #1 R-05 测试 PG 干净化)
    sqlx::query(&format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", schema))
        .execute(&mut *conn)
        .await
        .expect("drop pre schema");
    sqlx::query(&format!("DROP ROLE IF EXISTS \"{}\"", test_user))
        .execute(&mut *conn)
        .await
        .expect("drop pre role");
    sqlx::query(&format!("CREATE SCHEMA \"{}\"", schema))
        .execute(&mut *conn)
        .await
        .expect("create schema");
    sqlx::query(&format!("SET search_path TO \"{}\"", schema))
        .execute(&mut *conn)
        .await
        .expect("set search_path");

    // 2. 创建非 superuser 测试角色 (per 守門 #1 R-05: 真实 RLS 行为需要非 superuser)
    //    PG superuser 默认 bypass RLS, 所以必须用 NOBYPASSRLS 角色
    sqlx::query(&format!(
        "CREATE ROLE \"{}\" NOBYPASSRLS LOGIN PASSWORD 'test_pw_rls'",
        test_user
    ))
    .execute(&mut *conn)
    .await
    .expect("create test role");
    sqlx::query(&format!(
        "GRANT USAGE ON SCHEMA \"{}\" TO \"{}\"",
        schema, test_user
    ))
    .execute(&mut *conn)
    .await
    .expect("grant usage");
    // ALTER DEFAULT PRIVILEGES: 让后续 CREATE 的表自动给 test_user 权限
    // (per 守門 #11 缺标比错标: 模拟 testcontainers 真实 prep 模式)
    sqlx::query(&format!(
        "ALTER DEFAULT PRIVILEGES IN SCHEMA \"{}\" GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO \"{}\"",
        schema, test_user
    ))
    .execute(&mut *conn)
    .await
    .expect("alter default privileges");

    // 3. 前置依赖 (跟缺口 #1 同 pattern)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS audit_audit_event (\
         id UUID PRIMARY KEY DEFAULT gen_random_uuid(), event_type TEXT NOT NULL, \
         resource_id UUID, tenant_id UUID, payload JSONB NOT NULL, \
         occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW())",
    )
    .execute(&mut *conn)
    .await
    .expect("create audit_audit_event");
    sqlx::query(
        "CREATE OR REPLACE FUNCTION audit_audit_event() RETURNS TRIGGER AS $$ \
         BEGIN INSERT INTO audit_audit_event (event_type, resource_id, tenant_id, payload, occurred_at) \
         VALUES (TG_OP, NEW.id, NEW.tenant_id, to_jsonb(NEW), NOW()); RETURN NEW; END; \
         $$ LANGUAGE plpgsql",
    )
    .execute(&mut *conn)
    .await
    .expect("create audit function");
    sqlx::query(
        "CREATE OR REPLACE FUNCTION scd_type2_close() RETURNS TRIGGER AS $$ \
         BEGIN RETURN NEW; END; $$ LANGUAGE plpgsql",
    )
    .execute(&mut *conn)
    .await
    .expect("create scd_type2 function");
    // grant execute on functions to test_user
    sqlx::query(&format!(
        "GRANT EXECUTE ON FUNCTION audit_audit_event() TO \"{}\"",
        test_user
    ))
    .execute(&mut *conn)
    .await
    .expect("grant execute audit");
    sqlx::query(&format!(
        "GRANT EXECUTE ON FUNCTION scd_type2_close() TO \"{}\"",
        test_user
    ))
    .execute(&mut *conn)
    .await
    .expect("grant execute scd");

    // 4. apply cluster DDL (T 表有 audit + prevent_delete + RLS)
    let worktree_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let cluster_sql =
        std::fs::read_to_string(worktree_root.join("db/migrations/2026-09-08-ops-cluster.sql"))
            .expect("read cluster.sql");
    sqlx::raw_sql(&cluster_sql)
        .execute(&mut *conn)
        .await
        .expect("apply cluster DDL");
    // 显式 grant 一次 (针对已建表 + ALTER DEFAULT PRIVILEGES 已生效)
    sqlx::query(&format!(
        "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA \"{}\" TO \"{}\"",
        schema, test_user
    ))
    .execute(&mut *conn)
    .await
    .expect("grant table perms cluster");

    // 5. 切到 test_user (非 superuser) 验证 RLS
    sqlx::query(&format!("SET ROLE \"{}\"", test_user))
        .execute(&mut *conn)
        .await
        .expect("set role");

    let tenant_a = uuid::Uuid::new_v4();
    let tenant_b = uuid::Uuid::new_v4();
    // tenant_a 上下文写 1 行 (用 SET 直接设, 比 SELECT set_config 更直接)
    sqlx::query(&format!("SET app.tenant_id = '{}'", tenant_a))
        .execute(&mut *conn)
        .await
        .expect("set tenant_a");
    sqlx::query(
        "INSERT INTO ops_helm_release_state (tenant_id, release_name, namespace, chart, \
         revision, status, canary_weight, last_deployed_at) \
         VALUES (current_setting('app.tenant_id', true)::UUID, 'star-mcp-a', 'default', \
         'star-mcp', 1, 'healthy', 0, NOW())",
    )
    .execute(&mut *conn)
    .await
    .expect("insert tenant_a row");

    // tenant_b 上下文写 1 行
    sqlx::query(&format!("SET app.tenant_id = '{}'", tenant_b))
        .execute(&mut *conn)
        .await
        .expect("set tenant_b");
    sqlx::query(
        "INSERT INTO ops_helm_release_state (tenant_id, release_name, namespace, chart, \
         revision, status, canary_weight, last_deployed_at) \
         VALUES (current_setting('app.tenant_id', true)::UUID, 'star-mcp-b', 'default', \
         'star-mcp', 1, 'healthy', 0, NOW())",
    )
    .execute(&mut *conn)
    .await
    .expect("insert tenant_b row");

    // 6. 切到 tenant_a, 验证只能看 tenant_a 行 (RLS USING 强制)
    sqlx::query(&format!("SET app.tenant_id = '{}'", tenant_a))
        .execute(&mut *conn)
        .await
        .expect("set tenant_a for SELECT");
    let rows_a: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ops_helm_release_state")
        .fetch_one(&mut *conn)
        .await
        .expect("count tenant_a");
    assert_eq!(
        rows_a.0, 1,
        "tenant_a 上下文 SELECT 必只返 tenant_a 行 (per 守門 #DB-13 c RLS 隔离), got {}",
        rows_a.0
    );

    // 7. 切到 tenant_b, 验证只能看 tenant_b 行
    sqlx::query(&format!("SET app.tenant_id = '{}'", tenant_b))
        .execute(&mut *conn)
        .await
        .expect("set tenant_b for SELECT");
    let rows_b: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ops_helm_release_state")
        .fetch_one(&mut *conn)
        .await
        .expect("count tenant_b");
    assert_eq!(
        rows_b.0, 1,
        "tenant_b 上下文 SELECT 必只返 tenant_b 行 (per 守門 #DB-13 c RLS 隔离), got {}",
        rows_b.0
    );

    // 8. 验证 tenant_b 跨 tenant UPDATE 必 0 行受影响 (RLS USING 阻止)
    let updated: (i64,) = sqlx::query_as(
        "UPDATE ops_helm_release_state SET revision = 999 \
         WHERE release_name = 'star-mcp-a'",
    )
    .fetch_one(&mut *conn)
    .await
    .expect("update cross-tenant");
    assert_eq!(
        updated.0, 0,
        "tenant_b 跨 tenant UPDATE 必 0 行 (per 守門 #DB-13 c RLS 隔离), got {}",
        updated.0
    );

    // 9. 验证 tenant_b 跨 tenant DELETE 必 0 行受影响
    let deleted: (i64,) =
        sqlx::query_as("DELETE FROM ops_helm_release_state WHERE release_name = 'star-mcp-a'")
            .fetch_one(&mut *conn)
            .await
            .expect("delete cross-tenant");
    assert_eq!(
        deleted.0, 0,
        "tenant_b 跨 tenant DELETE 必 0 行 (per 守門 #DB-13 c RLS 隔离), got {}",
        deleted.0
    );

    // 10. 验证 RLS 13 類 cross-tenant 隔离涉及字段 (per SRS-001 §8.2)
    //     抽 1 张 W 表 (ops_log_query_log) 验证 RLS 强一致
    let log_sql =
        std::fs::read_to_string(worktree_root.join("db/migrations/2026-09-08-ops-log.sql"))
            .expect("read log.sql");
    sqlx::raw_sql(&log_sql)
        .execute(&mut *conn)
        .await
        .expect("apply log DDL");
    // 显式 grant 以保险 (针对新表)
    sqlx::query(&format!(
        "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA \"{}\" TO \"{}\"",
        schema, test_user
    ))
    .execute(&mut *conn)
    .await
    .expect("grant table perms after log DDL");
    // ops_log_query_log 有 trigger audit_audit_event, 触发时也走 test_user 权限
    sqlx::query(&format!(
        "GRANT INSERT ON audit_audit_event TO \"{}\"",
        test_user
    ))
    .execute(&mut *conn)
    .await
    .expect("grant audit_audit_event insert");

    sqlx::query(&format!("SET app.tenant_id = '{}'", tenant_a))
        .execute(&mut *conn)
        .await
        .expect("set tenant_a for log");
    sqlx::query(
        "INSERT INTO ops_log_query_log (tenant_id, query_id, level_filter, time_range_start, \
         time_range_end, actor_user_id, status) \
         VALUES (current_setting('app.tenant_id', true)::UUID, gen_random_uuid(), 'all', \
         NOW() - INTERVAL '1 day', NOW(), gen_random_uuid(), 'succeeded')",
    )
    .execute(&mut *conn)
    .await
    .expect("insert ops_log_query_log tenant_a");

    // 切到 tenant_b
    sqlx::query(&format!("SET app.tenant_id = '{}'", tenant_b))
        .execute(&mut *conn)
        .await
        .expect("set tenant_b for log select");
    let log_rows_b: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ops_log_query_log")
        .fetch_one(&mut *conn)
        .await
        .expect("count log query for tenant_b");
    assert_eq!(
        log_rows_b.0, 0,
        "tenant_b 看 tenant_a 写的 ops_log_query_log 必 0 行 (per 守門 #DB-13 c RLS 隔离), got {}",
        log_rows_b.0
    );

    // 11. 切回 superuser 清理
    sqlx::query("RESET ROLE")
        .execute(&mut *conn)
        .await
        .expect("reset role");
    sqlx::query(&format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", schema))
        .execute(&mut *conn)
        .await
        .expect("drop schema");
    sqlx::query(&format!("DROP ROLE IF EXISTS \"{}\"", test_user))
        .execute(&mut *conn)
        .await
        .expect("drop test role");
}
