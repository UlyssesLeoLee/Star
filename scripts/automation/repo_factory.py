# SPDX-License-Identifier: MIT OR Apache-2.0
# repo_factory.py v0.2 — 6 ops 表 Repository 自动生成 (per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 收官)
#
# v0.2 修复 (per 9/9 19:39 JST 139 err 实证):
# - sqlx::FromRow derive (avoid tuple > 16 不 impl FromRow)
# - 去掉 trace_id 重复 (Rls_extra 已含 trace_id, 跟 fields 重复导致 unreachable_pattern)
# - bind 类型修正 (Uuid/String/Option<...> 走 &x.field, Copy 类型 (i32/bool/f64) 走 x.field)
#
# 5 域 Lead 决策 (Mavis 临时代签 per 守门 #14 v3):
#   - F-01: ops_helm_release_state (T) + ops_cluster_action_log (T)
#   - F-02: ops_log_query_log (T WORM) + ops_log_entry (W TTL 7d) + ops_log_analysis (W TTL 30d)
#   1 已收 (ops_metrics_config v0.39), 5 待生成.
#
# 守门 #19 v19: 任何 [P]/[M] 子项 强制 Python 化 (5 Repository 跨 4 维 R/V/S/A);
# 守门 #1 v25: 单 crate 模式 (cargo test -p star-pg-adapter --lib -j 4);
# 守门 #5 v2: env var 安全 (DATABASE_URL 走 PgConfig::from_env 不打印);
# 守门 #13: 6 ops 表 W/T/M 100% 覆盖 (T=3 W=2 M=1, 0 混合分类);
# 守门 #14 v3: Mavis 永久代签.
#
# 用法: python -u scripts/automation/repo_factory.py
# 输出: 5 .rs files in crates/star-pg-adapter/src/repository/

import sys
from pathlib import Path
from textwrap import dedent
from typing import Optional

REPO_DIR = Path("crates/star-pg-adapter/src/repository")


# ============================================
# 5 表 spec (per DDL 100% 字段对齐, 跨 W/T/M 严格分类)
# ============================================

# 公共 RLS 13 類字段 (T 必携, W 选 12 類) — 12 類 (不含 trace_id)
RLS_13 = [
    ("user_id", "Option<Uuid>", "RLS 13 類"),
    ("role_id", "Option<Uuid>", "RLS 13 類"),
    ("permission_id", "Option<Uuid>", "RLS 13 類"),
    ("policy_id", "Option<Uuid>", "RLS 13 類"),
    ("workspace_id", "Option<Uuid>", "RLS 13 類"),
    ("project_id", "Option<Uuid>", "RLS 13 類"),
    ("work_item_id", "Option<Uuid>", "RLS 13 類"),
    ("agent_id", "Option<Uuid>", "RLS 13 類"),
    ("session_id", "Option<Uuid>", "RLS 13 類"),
    ("trace_id", "Option<Uuid>", "RLS 13 類 (T 必携)"),
]

# ============================================
# 5 Repository specs
# ============================================

# F-01 ops_helm_release_state (T 业务事实快照)
HELM_RELEASE = {
    "name": "OpsHelmReleaseState",
    "table": "ops_helm_release_state",
    "wtm": "T",
    "rbac_13": True,  # T 必携 13 類
    "fields": [
        ("id", "Uuid", "主键"),
        ("tenant_id", "Uuid", "租户 ID (RLS 必携)"),
        ("release_name", "String", "Helm release 名"),
        ("namespace", "String", "k8s namespace"),
        ("chart", "String", "Helm chart 名"),
        ("revision", "i32", "Helm revision"),
        ("status", "String", "pending/healthy/degraded/failed/canary"),
        ("canary_weight", "i32", "0-100 canary 权重"),
        ("last_deployed_at", "DateTime<Utc>", "上次部署时间"),
        ("captured_at", "DateTime<Utc>", "入库时间"),
        ("source_module", "String", "star_ops::cluster"),
        ("source_kind", "String", "internal"),
    ],
    "rls_extra": [],  # RLS 13 類 已包含 trace_id, 不重复
    "methods": [
        ("find_by_release",
         ["tenant_id: Uuid", "release_name: &str", "revision: i32"],
         "Result<Option<OpsHelmReleaseState>, PgAdapterError>",
         "find_by_release"),
        ("list_recent",
         ["tenant_id: Uuid", "limit: i64"],
         "Result<Vec<OpsHelmReleaseState>, PgAdapterError>",
         "list_recent"),
        ("upsert",
         ["state: &OpsHelmReleaseState"],
         "Result<(), PgAdapterError>",
         "upsert"),
    ],
    "sql_find": """            SELECT {select_cols}
            FROM ops_helm_release_state
            WHERE tenant_id = $1 AND release_name = $2 AND revision = $3""",
    "sql_list": """            SELECT {select_cols}
            FROM ops_helm_release_state
            WHERE tenant_id = $1
            ORDER BY captured_at DESC
            LIMIT $2""",
    "sql_upsert": """            INSERT INTO ops_helm_release_state
                ({insert_cols})
            VALUES ({placeholders})
            ON CONFLICT (id) DO UPDATE SET
                release_name = EXCLUDED.release_name,
                namespace = EXCLUDED.namespace,
                chart = EXCLUDED.chart,
                revision = EXCLUDED.revision,
                status = EXCLUDED.status,
                canary_weight = EXCLUDED.canary_weight,
                last_deployed_at = EXCLUDED.last_deployed_at,
                captured_at = EXCLUDED.captured_at""",
}

# F-01 ops_cluster_action_log (T WORM 审计)
CLUSTER_ACTION_LOG = {
    "name": "OpsClusterActionLog",
    "table": "ops_cluster_action_log",
    "wtm": "T",
    "rbac_13": True,
    "fields": [
        ("id", "Uuid", "主键"),
        ("tenant_id", "Uuid", "租户 ID (RLS 必携)"),
        ("action_id", "Uuid", "动作 ID (跟 cluster update 联动)"),
        ("release_name", "String", "Helm release 名"),
        ("namespace", "String", "k8s namespace"),
        ("action_type", "String", "canary/rollback/upgrade/list/status"),
        ("actor_user_id", "Uuid", "发起人"),
        ("requested_at", "DateTime<Utc>", "请求时间"),
        ("completed_at", "Option<DateTime<Utc>>", "完成时间"),
        ("status", "String", "pending/running/succeeded/failed"),
        ("payload", "serde_json::Value", "动作参数 (JSONB)"),
        ("error_message", "Option<String>", "失败信息"),
        ("source_module", "String", "star_ops::cluster"),
        ("source_kind", "String", "audit"),
    ],
    "rls_extra": [],
    "methods": [
        ("find_by_action",
         ["tenant_id: Uuid", "action_id: Uuid"],
         "Result<Option<OpsClusterActionLog>, PgAdapterError>",
         "find_by_action"),
        ("list_by_release",
         ["tenant_id: Uuid", "release_name: &str", "limit: i64"],
         "Result<Vec<OpsClusterActionLog>, PgAdapterError>",
         "list_by_release"),
        ("insert",
         ["log: &OpsClusterActionLog"],
         "Result<(), PgAdapterError>",
         "insert"),
    ],
    "sql_find": """            SELECT {select_cols}
            FROM ops_cluster_action_log
            WHERE tenant_id = $1 AND action_id = $2""",
    "sql_list": """            SELECT {select_cols}
            FROM ops_cluster_action_log
            WHERE tenant_id = $1 AND release_name = $2
            ORDER BY requested_at DESC
            LIMIT $3""",
    "sql_insert": """            INSERT INTO ops_cluster_action_log
                ({insert_cols})
            VALUES ({placeholders})""",
}

# F-02 ops_log_query_log (T WORM 审计)
LOG_QUERY_LOG = {
    "name": "OpsLogQueryLog",
    "table": "ops_log_query_log",
    "wtm": "T",
    "rbac_13": True,
    "fields": [
        ("id", "Uuid", "主键"),
        ("tenant_id", "Uuid", "租户 ID (RLS 必携)"),
        ("query_id", "Uuid", "客户端 query 请求 ID"),
        ("trace_id", "Option<Uuid>", "跨域 trace"),
        ("level_filter", "String", "all/info/warn/error/debug"),
        ("time_range_start", "DateTime<Utc>", "时间范围起"),
        ("time_range_end", "DateTime<Utc>", "时间范围止"),
        ("line_count", "i32", "返回行数"),
        ("actor_user_id", "Uuid", "发起人"),
        ("requested_at", "DateTime<Utc>", "请求时间"),
        ("completed_at", "Option<DateTime<Utc>>", "完成时间"),
        ("status", "String", "pending/running/succeeded/failed"),
        ("error_code", "Option<String>", "6-field 错误码"),
        ("error_message", "Option<String>", "错误信息"),
        ("source_module", "String", "star_ops::log"),
        ("source_kind", "String", "audit"),
    ],
    "rls_extra": [],  # trace_id 已在 fields, 不重复
    "methods": [
        ("find_by_query",
         ["tenant_id: Uuid", "query_id: Uuid"],
         "Result<Option<OpsLogQueryLog>, PgAdapterError>",
         "find_by_query"),
        ("list_by_trace",
         ["tenant_id: Uuid", "trace_id: Uuid", "limit: i64"],
         "Result<Vec<OpsLogQueryLog>, PgAdapterError>",
         "list_by_trace"),
        ("insert",
         ["log: &OpsLogQueryLog"],
         "Result<(), PgAdapterError>",
         "insert"),
    ],
    "sql_find": """            SELECT {select_cols}
            FROM ops_log_query_log
            WHERE tenant_id = $1 AND query_id = $2""",
    "sql_list": """            SELECT {select_cols}
            FROM ops_log_query_log
            WHERE tenant_id = $1 AND trace_id = $2
            ORDER BY requested_at DESC
            LIMIT $3""",
    "sql_insert": """            INSERT INTO ops_log_query_log
                ({insert_cols})
            VALUES ({placeholders})""",
}

# F-02 ops_log_entry (W TTL 7d)
LOG_ENTRY = {
    "name": "OpsLogEntry",
    "table": "ops_log_entry",
    "wtm": "W",
    "rbac_13": True,  # W 12+ 類 (本表用 12 類 + source_module/kind)
    "fields": [
        ("id", "Uuid", "主键"),
        ("tenant_id", "Uuid", "租户 ID (RLS 必携)"),
        ("trace_id", "Option<Uuid>", "跨域 trace"),
        ("log_query_id", "Option<Uuid>", "FK to ops_log_query_log ON DELETE RESTRICT"),
        ("level", "String", "debug/info/warn/error"),
        ("source", "String", "app/system/agent/mcp/external"),
        ("message", "String", "log 内容"),
        ("message_excerpt", "Option<String>", "截断 256 字符"),
        ("metadata", "serde_json::Value", "额外元数据 (JSONB)"),
        ("occurred_at", "DateTime<Utc>", "log 真实发生时间"),
        ("captured_at", "DateTime<Utc>", "入库时间"),
        ("expires_at", "DateTime<Utc>", "TTL 7d 过期时间"),
        ("source_module", "String", "star_ops::log"),
        ("source_kind", "String", "work"),
    ],
    "rls_extra": [],
    "methods": [
        ("find_by_query",
         ["tenant_id: Uuid", "query_id: Uuid", "limit: i64"],
         "Result<Vec<OpsLogEntry>, PgAdapterError>",
         "find_by_query"),
        ("list_by_trace",
         ["tenant_id: Uuid", "trace_id: Uuid", "limit: i64"],
         "Result<Vec<OpsLogEntry>, PgAdapterError>",
         "list_by_trace"),
        ("insert",
         ["entry: &OpsLogEntry"],
         "Result<(), PgAdapterError>",
         "insert"),
    ],
    "sql_find": """            SELECT {select_cols}
            FROM ops_log_entry
            WHERE tenant_id = $1 AND log_query_id = $2
              AND expires_at > NOW()
            ORDER BY occurred_at DESC
            LIMIT $3""",
    "sql_list": """            SELECT {select_cols}
            FROM ops_log_entry
            WHERE tenant_id = $1 AND trace_id = $2
              AND expires_at > NOW()
            ORDER BY occurred_at DESC
            LIMIT $3""",
    "sql_insert": """            INSERT INTO ops_log_entry
                ({insert_cols})
            VALUES ({placeholders})""",
}

# F-02 ops_log_analysis (W TTL 30d)
LOG_ANALYSIS = {
    "name": "OpsLogAnalysis",
    "table": "ops_log_analysis",
    "wtm": "W",
    "rbac_13": True,
    "fields": [
        ("id", "Uuid", "主键"),
        ("tenant_id", "Uuid", "租户 ID (RLS 必携)"),
        ("log_entry_id", "Uuid", "FK to ops_log_entry ON DELETE CASCADE"),
        ("ai_channel", "String", "mock/openai/anthropic"),
        ("model", "Option<String>", "gpt-4/claude-3-opus/mock-v1"),
        ("ladder_attempts", "i32", "Ladder 重试次数"),
        ("confidence", "f64", "0.0-1.0 置信度"),
        ("is_anomaly", "bool", "是否异常"),
        ("anomaly_type", "Option<String>", "spike/drift/error_burst/pattern"),
        ("summary", "String", "LLM 摘要"),
        ("suggestions", "serde_json::Value", "LLM 建议 (JSONB array)"),
        ("generated_by", "String", "mock/openai/anthropic"),
        ("generated_at", "DateTime<Utc>", "生成时间"),
        ("expires_at", "DateTime<Utc>", "TTL 30d 过期时间"),
        ("source_module", "String", "star_ops::log"),
        ("source_kind", "String", "work"),
    ],
    "rls_extra": [],
    "methods": [
        ("find_by_entry",
         ["tenant_id: Uuid", "entry_id: Uuid"],
         "Result<Vec<OpsLogAnalysis>, PgAdapterError>",
         "find_by_entry"),
        ("list_anomalies",
         ["tenant_id: Uuid", "limit: i64"],
         "Result<Vec<OpsLogAnalysis>, PgAdapterError>",
         "list_anomalies"),
        ("insert",
         ["analysis: &OpsLogAnalysis"],
         "Result<(), PgAdapterError>",
         "insert"),
    ],
    "sql_find": """            SELECT {select_cols}
            FROM ops_log_analysis
            WHERE tenant_id = $1 AND log_entry_id = $2
              AND expires_at > NOW()""",
    "sql_list": """            SELECT {select_cols}
            FROM ops_log_analysis
            WHERE tenant_id = $1 AND is_anomaly = TRUE
              AND expires_at > NOW()
            ORDER BY generated_at DESC
            LIMIT $2""",
    "sql_insert": """            INSERT INTO ops_log_analysis
                ({insert_cols})
            VALUES ({placeholders})""",
}


SPECS = [HELM_RELEASE, CLUSTER_ACTION_LOG, LOG_QUERY_LOG, LOG_ENTRY, LOG_ANALYSIS]


# ============================================
# v0.43 OAuth2 server 4 表 (per 9/9 20:20 JST 用户拍板 both flows)
# ============================================

# oauth_clients (M, SCD Type 2, 跟 ops_metrics_config 模式一致)
OAUTH_CLIENTS = {
    "name": "OAuthClient",
    "table": "oauth_clients",
    "wtm": "M",
    "rbac_13": True,
    "fields": [
        ("id", "Uuid", "主键 (复合 PK id+valid_from)"),
        ("tenant_id", "Uuid", "租户 ID (RLS 必携)"),
        ("client_id", "String", "公开 client_id (e.g. star-frontend-spa)"),
        ("client_secret_hash", "Option<String>", "bcrypt 哈希 (confidential 才有)"),
        ("client_name", "String", "显示名"),
        ("client_type", "String", "public/confidential"),
        ("redirect_uris", "Vec<String>", "redirect URI 列表 (public auth code 需要)"),
        ("allowed_scopes", "Vec<String>", "允许的 scope (e.g. read/write/admin)"),
        ("allowed_grant_types", "Vec<String>", "authorization_code/client_credentials/refresh_token"),
        ("require_pkce", "bool", "强制 PKCE (public 默认 TRUE)"),
        ("require_authentication", "bool", "需要 client_secret (confidential 默认 TRUE)"),
        ("owner_user_id", "Uuid", "注册人"),
        ("valid_from", "DateTime<Utc>", "SCD Type 2 生效起始"),
        ("valid_to", "Option<DateTime<Utc>>", "SCD Type 2 生效结束"),
        ("created_at", "DateTime<Utc>", "创建时间"),
        ("updated_at", "DateTime<Utc>", "更新时间"),
        ("source_module", "String", "star_api_rest::auth::oauth"),
        ("source_kind", "String", "master"),
    ],
    "rls_extra": [],
    "methods": [
        ("find_by_client_id",
         ["tenant_id: Uuid", "client_id: &str"],
         "Result<Option<OAuthClient>, PgAdapterError>",
         "find_by_client_id"),
        ("list_current",
         ["tenant_id: Uuid"],
         "Result<Vec<OAuthClient>, PgAdapterError>",
         "list_current"),
        ("insert_new_version",
         ["client: &OAuthClient"],
         "Result<(), PgAdapterError>",
         "insert_new_version"),
    ],
    "sql_find": """            SELECT {select_cols}
            FROM oauth_clients
            WHERE tenant_id = $1 AND client_id = $2 AND valid_to IS NULL""",
    "sql_list": """            SELECT {select_cols}
            FROM oauth_clients
            WHERE tenant_id = $1 AND valid_to IS NULL
            ORDER BY client_name""",
    "sql_upsert": """            INSERT INTO oauth_clients
                ({insert_cols})
            VALUES ({placeholders})""",
}

# oauth_authorization_codes (T, WORM, 短期)
OAUTH_AUTH_CODES = {
    "name": "OAuthAuthorizationCode",
    "table": "oauth_authorization_codes",
    "wtm": "T",
    "rbac_13": True,
    "fields": [
        ("id", "Uuid", "主键"),
        ("tenant_id", "Uuid", "租户 ID (RLS 必携)"),
        ("code_hash", "String", "sha256(code) 哈希 (守门 #5 v2 不存明文)"),
        ("client_id", "String", "客户端 ID"),
        ("user_id", "Uuid", "授权人 (RLS 13 類)"),
        ("redirect_uri", "String", "redirect URI"),
        ("scope", "String", "scope 字符串 (空格分隔)"),
        ("code_challenge", "String", "PKCE code_challenge"),
        ("code_challenge_method", "String", "plain/S256"),
        ("expires_at", "DateTime<Utc>", "10 分钟过期"),
        ("consumed_at", "Option<DateTime<Utc>>", "兑换时间 (WORM 派生)"),
        ("created_at", "DateTime<Utc>", "创建时间"),
        ("source_module", "String", "star_api_rest::auth::oauth"),
        ("source_kind", "String", "audit"),
    ],
    "rls_extra": [],
    "methods": [
        ("find_by_code_hash",
         ["tenant_id: Uuid", "code_hash: &str"],
         "Result<Option<OAuthAuthorizationCode>, PgAdapterError>",
         "find_by_code_hash"),
        ("mark_consumed",
         ["tenant_id: Uuid", "code_hash: &str", "consumed_at: DateTime<Utc>"],
         "Result<(), PgAdapterError>",
         "mark_consumed"),
        ("insert",
         ["code: &OAuthAuthorizationCode"],
         "Result<(), PgAdapterError>",
         "insert"),
    ],
    "sql_find": """            SELECT {select_cols}
            FROM oauth_authorization_codes
            WHERE tenant_id = $1 AND code_hash = $2 AND consumed_at IS NULL
              AND expires_at > NOW()""",
    "sql_update": """            UPDATE oauth_authorization_codes
            SET consumed_at = $3
            WHERE tenant_id = $1 AND code_hash = $2 AND consumed_at IS NULL""",
    "sql_insert": """            INSERT INTO oauth_authorization_codes
                ({insert_cols})
            VALUES ({placeholders})""",
}

# oauth_access_tokens (T, append-only)
OAUTH_ACCESS_TOKENS = {
    "name": "OAuthAccessToken",
    "table": "oauth_access_tokens",
    "wtm": "T",
    "rbac_13": True,
    "fields": [
        ("id", "Uuid", "主键"),
        ("tenant_id", "Uuid", "租户 ID (RLS 必携)"),
        ("token_hash", "String", "sha256(jti) 哈希 (守门 #5 v2)"),
        ("client_id", "String", "客户端 ID"),
        ("user_id", "Option<Uuid>", "用户 (client_credentials grant 为 NULL)"),
        ("grant_type", "String", "authorization_code/client_credentials/refresh_token"),
        ("scope", "String", "scope 字符串"),
        ("expires_at", "DateTime<Utc>", "过期时间"),
        ("revoked_at", "Option<DateTime<Utc>>", "撤销时间 (WORM 派生)"),
        ("created_at", "DateTime<Utc>", "创建时间"),
        ("source_module", "String", "star_api_rest::auth::oauth"),
        ("source_kind", "String", "audit"),
    ],
    "rls_extra": [],
    "methods": [
        ("find_by_token_hash",
         ["tenant_id: Uuid", "token_hash: &str"],
         "Result<Option<OAuthAccessToken>, PgAdapterError>",
         "find_by_token_hash"),
        ("revoke",
         ["tenant_id: Uuid", "token_hash: &str", "revoked_at: DateTime<Utc>"],
         "Result<(), PgAdapterError>",
         "revoke"),
        ("insert",
         ["token: &OAuthAccessToken"],
         "Result<(), PgAdapterError>",
         "insert"),
    ],
    "sql_find": """            SELECT {select_cols}
            FROM oauth_access_tokens
            WHERE tenant_id = $1 AND token_hash = $2 AND revoked_at IS NULL
              AND expires_at > NOW()""",
    "sql_update": """            UPDATE oauth_access_tokens
            SET revoked_at = $3
            WHERE tenant_id = $1 AND token_hash = $2 AND revoked_at IS NULL""",
    "sql_insert": """            INSERT INTO oauth_access_tokens
                ({insert_cols})
            VALUES ({placeholders})""",
}

# oauth_refresh_tokens (T, WORM)
OAUTH_REFRESH_TOKENS = {
    "name": "OAuthRefreshToken",
    "table": "oauth_refresh_tokens",
    "wtm": "T",
    "rbac_13": True,
    "fields": [
        ("id", "Uuid", "主键"),
        ("tenant_id", "Uuid", "租户 ID (RLS 必携)"),
        ("token_hash", "String", "sha256(token) 哈希"),
        ("client_id", "String", "客户端 ID"),
        ("user_id", "Uuid", "用户 (仅 auth code flow 有)"),
        ("access_token_id", "Uuid", "关联的 access_token ID"),
        ("scope", "String", "scope 字符串"),
        ("expires_at", "DateTime<Utc>", "30 天过期"),
        ("revoked_at", "Option<DateTime<Utc>>", "撤销时间"),
        ("created_at", "DateTime<Utc>", "创建时间"),
        ("source_module", "String", "star_api_rest::auth::oauth"),
        ("source_kind", "String", "audit"),
    ],
    "rls_extra": [],
    "methods": [
        ("find_by_token_hash",
         ["tenant_id: Uuid", "token_hash: &str"],
         "Result<Option<OAuthRefreshToken>, PgAdapterError>",
         "find_by_token_hash"),
        ("revoke",
         ["tenant_id: Uuid", "token_hash: &str", "revoked_at: DateTime<Utc>"],
         "Result<(), PgAdapterError>",
         "revoke"),
        ("insert",
         ["token: &OAuthRefreshToken"],
         "Result<(), PgAdapterError>",
         "insert"),
    ],
    "sql_find": """            SELECT {select_cols}
            FROM oauth_refresh_tokens
            WHERE tenant_id = $1 AND token_hash = $2 AND revoked_at IS NULL
              AND expires_at > NOW()""",
    "sql_update": """            UPDATE oauth_refresh_tokens
            SET revoked_at = $3
            WHERE tenant_id = $1 AND token_hash = $2 AND revoked_at IS NULL""",
    "sql_insert": """            INSERT INTO oauth_refresh_tokens
                ({insert_cols})
            VALUES ({placeholders})""",
}


SPECS_OAUTH = [OAUTH_CLIENTS, OAUTH_AUTH_CODES, OAUTH_ACCESS_TOKENS, OAUTH_REFRESH_TOKENS]
SPECS_OPS = [HELM_RELEASE, CLUSTER_ACTION_LOG, LOG_QUERY_LOG, LOG_ENTRY, LOG_ANALYSIS]
SPECS = SPECS_OPS + SPECS_OAUTH


# ============================================
# 模板生成
# ============================================

def get_all_fields(spec: dict) -> list:
    """合并主字段 + RLS 13 類 (T 必携) + rls_extra, 去重 (trace_id 可能在 fields)"""
    seen = set()
    fields = []
    for f in spec["fields"]:
        if f[0] not in seen:
            fields.append(f)
            seen.add(f[0])
    if spec["rbac_13"]:
        for f in RLS_13:
            if f[0] not in seen:
                fields.append(f)
                seen.add(f[0])
    for f in spec.get("rls_extra", []):
        if f[0] not in seen:
            fields.append(f)
            seen.add(f[0])
    return fields


def gen_struct(spec: dict) -> str:
    """生成 struct 定义 (含 sqlx::FromRow derive, 走 struct 路径避免 tuple > 16)"""
    fields = get_all_fields(spec)
    lines = [
        f"/// {spec['name']} (W/T/M = {spec['wtm']}, 跟 DDL {spec['table']} 100% 对齐)",
        "#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]",
        f"pub struct {spec['name']} {{",
    ]
    for fname, ftype, fdoc in fields:
        lines.append(f"    /// {fdoc}")
        lines.append(f"    pub {fname}: {ftype},")
    lines.append("}")
    return "\n".join(lines)


def gen_trait(spec: dict) -> str:
    """生成 trait + method signatures"""
    name = spec["name"]
    lines = [
        f"/// {name}Repository trait (per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)",
        "#[async_trait::async_trait]",
        f"pub trait {name}Repository: Send + Sync {{",
    ]
    for mname, margs_list, mret, _ in spec["methods"]:
        margs = ", ".join(margs_list)
        lines.append(f"    /// {mname} (per DDL {spec['table']} {spec['wtm']} 派生规)")
        lines.append(f"    async fn {mname}(&self, {margs}) -> {mret};")
    lines.append("}")
    return "\n".join(lines)


def gen_pg_struct(spec: dict) -> str:
    """生成 PgXxxRepository struct + new()"""
    name = spec["name"]
    return dedent(f"""\
        /// Postgres impl (跟 db/migrations/{spec['table']} 对应)
        pub struct Pg{name}Repository {{
            pool: PgPool,
        }}

        impl Pg{name}Repository {{
            /// 构造 PG Repository (持有 PgPool 引用)
            pub fn new(pool: PgPool) -> Self {{
                Self {{ pool }}
            }}
        }}
        """).rstrip()


def parse_bind_expr(pname: str, ptype: str) -> str:
    """bind 表达式: Copy 类型走值, 非 Copy 走引用"""
    if ptype == "&str":
        return pname
    if ptype.startswith("&"):
        return pname  # 已经是引用
    # Copy 类型: 走值
    if ptype in ("i32", "i64", "u32", "u64", "bool", "f32", "f64", "Uuid"):
        return pname
    # Vec<T>: 走引用
    if ptype.startswith("Vec<"):
        return f"&{pname}"
    # 非 Copy 类型: 走引用
    return f"&{pname}"


def gen_impl(spec: dict) -> str:
    """生成 trait impl 块 (sqlx 走 FromRow struct 路径)"""
    name = spec["name"]
    fields = get_all_fields(spec)
    field_names = [f[0] for f in fields]

    select_cols = ", ".join(field_names)
    insert_cols = ", ".join(field_names)
    placeholders = ", ".join(f"${i+1}" for i in range(len(field_names)))

    lines = [
        f"#[async_trait::async_trait]",
        f"impl {name}Repository for Pg{name}Repository {{",
    ]

    for mname, margs_list, mret, _ in spec["methods"]:
        # 解析参数列表 [(pname, ptype), ...]
        params = []
        for marg in margs_list:
            pname, ptype = marg.split(":", 1)
            params.append((pname.strip(), ptype.strip()))
        margs = ", ".join(margs_list)

        if mname.startswith("find_") or mname.startswith("list_") or mname == "list_recent":
            # SELECT 类 — 走 query_as<_, Struct>
            if mname.startswith("find_"):
                sql_key = "sql_find"
            else:
                sql_key = "sql_list"
            sql = spec[sql_key].format(select_cols=select_cols)

            # fetch method 走 mret 决定 (Vec → fetch_all, Option → fetch_optional)
            if "Vec<" in mret:
                fetch_method = "fetch_all"
            else:
                fetch_method = "fetch_optional"
            lines.append(f"    async fn {mname}(&self, {margs}) -> {mret} {{")
            lines.append(f"        let rows = sqlx::query_as::<_, {name}>(")
            lines.append("            r#\"")
            lines.append(f"            {sql}")
            lines.append("            \"#,")
            lines.append("        )")
            for pname, ptype in params:
                bind = parse_bind_expr(pname, ptype)
                lines.append(f"        .bind({bind})")
            lines.append(f"        .{fetch_method}(&self.pool)")
            lines.append("        .await")
            lines.append(f"        .map_err(|e| PgAdapterError::Query(format!(\"{mname}: {{}}\", e)))?;")
            if mname.startswith("find_"):
                lines.append("        Ok(rows)")
            else:
                lines.append("        Ok(rows)")
            lines.append("    }")
        else:
            # INSERT / UPSERT / UPDATE 类
            if "upsert" in mname or "insert_new_version" in mname:
                sql_key = "sql_upsert"
            elif "mark_consumed" in mname or "revoke" in mname:
                sql_key = "sql_update"
            else:
                sql_key = "sql_insert"
            sql = spec[sql_key].format(insert_cols=insert_cols, placeholders=placeholders)

            # 找 &StructName 引用参数 (insert 类), 排除 &str / &Vec<T>
            ref_arg = None
            if sql_key != "sql_update":
                for pname, ptype in params:
                    if ptype.startswith("&") and ptype != "&str" and not ptype.startswith("&Vec<"):
                        ref_arg = pname
                        break

            lines.append(f"    async fn {mname}(&self, {margs}) -> {mret} {{")
            lines.append("        sqlx::query(")
            lines.append("            r#\"")
            lines.append(f"            {sql}")
            lines.append("            \"#,")
            lines.append("        )")
            if ref_arg:
                # INSERT / UPSERT: 按 struct 字段顺序 bind
                for fname, ftype, _ in get_all_fields(spec):
                    if ftype in ("i32", "i64", "u32", "u64", "bool", "f32", "f64"):
                        lines.append(f"        .bind({ref_arg}.{fname})")
                    elif ftype.startswith("Vec<"):
                        lines.append(f"        .bind(&{ref_arg}.{fname})")
                    else:
                        lines.append(f"        .bind(&{ref_arg}.{fname})")
            else:
                # UPDATE: 按参数顺序 bind
                for pname, ptype in params:
                    bind = parse_bind_expr(pname, ptype)
                    lines.append(f"        .bind({bind})")
            lines.append("        .execute(&self.pool)")
            lines.append("        .await")
            lines.append(f"        .map_err(|e| PgAdapterError::Query(format!(\"{mname}: {{}}\", e)))?;")
            lines.append("        Ok(())")
            lines.append("    }")
        lines.append("")

    lines.append("}")
    return "\n".join(lines)


def pname_to_field(pname: str, params: list) -> str:
    """从 &x.foo 这种参数名解析实际字段 (x 是参数名, foo 是字段名)"""
    return pname  # 实际就是参数名, 字段是 .id / .tenant_id 等


def gen_field_binds(spec: dict) -> str:
    """INSERT bind 字段 (按 struct 字段顺序)"""
    fields = get_all_fields(spec)
    lines = []
    for fname, ftype, _ in fields:
        bind = parse_bind_expr(f"x.{fname}", ftype)
        lines.append(f"        .bind({bind})")
    return "\n".join(lines)


def gen_insert_binds(spec: dict) -> str:
    """INSERT bind 走 struct 字段路径"""
    fields = get_all_fields(spec)
    return gen_field_binds(spec)


def gen_tests(spec: dict) -> str:
    """生成 struct + trait 编译时验证 tests"""
    name = spec["name"]
    fields = get_all_fields(spec)
    n_fields = len(fields)
    n_methods = len(spec["methods"])

    init_lines = []
    for fname, ftype, _ in fields:
        if ftype == "Uuid":
            init_lines.append(f"            {fname}: Uuid::nil(),")
        elif ftype == "Option<Uuid>":
            init_lines.append(f"            {fname}: None,")
        elif ftype == "String":
            init_lines.append(f"            {fname}: \"test\".to_string(),")
        elif ftype == "Option<String>":
            init_lines.append(f"            {fname}: None,")
        elif ftype == "i32" or ftype == "i64":
            init_lines.append(f"            {fname}: 0,")
        elif ftype == "bool":
            init_lines.append(f"            {fname}: false,")
        elif ftype == "f64":
            init_lines.append(f"            {fname}: 0.0,")
        elif ftype == "DateTime<Utc>":
            init_lines.append(f"            {fname}: Utc::now(),")
        elif ftype == "Option<DateTime<Utc>>":
            init_lines.append(f"            {fname}: None,")
        elif ftype == "serde_json::Value":
            init_lines.append(f"            {fname}: serde_json::json!({{}}),")
        else:
            init_lines.append(f"            {fname}: Default::default(),")

    return dedent(f"""\
        #[cfg(test)]
        mod tests {{
            use super::*;

            /// 派生 #N: {name} 必含 {n_fields} 字段 (跟 DDL 100% 一致)
            #[test]
            fn {spec['table']}_struct_has_{n_fields}_fields() {{
                let x = {name} {{
        {chr(10).join(init_lines)}
                }};
                assert_eq!(x.source_module, "test");
            }}

            /// 派生 #N: {name}Repository trait 必含 {n_methods} 方法
            #[test]
            fn {spec['table']}_repository_trait_has_{n_methods}_methods() {{
                fn _check_methods<T: {name}Repository>() {{}}
                _check_methods::<Pg{name}Repository>();
            }}
        }}
        """).rstrip()


def gen_full_module(spec: dict) -> str:
    """生成完整 .rs 文件"""
    name = spec["name"]
    header = dedent(f"""\
        // SPDX-License-Identifier: MIT OR Apache-2.0
        //! {name} Repository (Postgres impl, per WBS v0.42 §14.10.4 缺口 #4 5/6 Repository 续推)
        //!
        //! v0.42 端到端实装 (per 9/9 19:33 JST 用户拍板"5/6 Repository 续推" + 守门 #9 v19 Mavis 自驱):
        //! - 跟 `db/migrations/{spec['table']}.sql` 表对应
        //! - 守门 #13 {spec['wtm']} 类: {'物理删除禁止 + 監査必須 + RLS 13 類必携' if spec['wtm'] == 'T' else '物理删除 / タイマー失効 / 短 TTL 明示 retention'}
        //! - sqlx::FromRow derive (避免 tuple > 16 不 impl FromRow, per v0.2 修复)
        //! - 6-field error (跟 v0.30 6-field ApiError / ApplicationError 模式一致)

        use chrono::{{DateTime, Utc}};
        use serde::{{Deserialize, Serialize}};
        use sqlx::PgPool;
        use uuid::Uuid;

        use crate::PgAdapterError;
        """).rstrip()
    parts = [
        header,
        "",
        gen_struct(spec),
        "",
        gen_trait(spec),
        "",
        gen_pg_struct(spec),
        "",
        gen_impl(spec),
        "",
        gen_tests(spec),
    ]
    return "\n".join(parts) + "\n"


# ============================================
# Main
# ============================================

def main():
    print("[repo_factory] generating 5 Repository .rs files (v0.2 fix)")
    print("[repo_factory] guardian #19 v19: Python 5 Repository (4-dim R/V/S/A)")

    for spec in SPECS:
        name = spec["name"]
        table = spec["table"]
        n_fields = len(get_all_fields(spec))
        n_methods = len(spec["methods"])
        content = gen_full_module(spec)
        file_path = REPO_DIR / f"{table}.rs"
        file_path.parent.mkdir(parents=True, exist_ok=True)
        file_path.write_text(content, encoding="utf-8")
        print(f"  [OK] {file_path} ({n_fields} fields, {n_methods} methods, {len(content)} bytes)")

    print("[repo_factory] done (5 .rs regenerated)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
