#!/usr/bin/env python3
"""
gen_supporting_specs.py
========================
为 7 个 supporting crate 生成 docs/specs/domain-*-spec.md (GAP-01)

模板: 7 章简化版
  §1 职责与边界
  §2 关键实体
  §3 关键不变量
  §4 接口契约
  §5 跨 domain 接触面 (v0.16 协作细化新增)
  §6 风险与缓解
  §7 修订历史

数据: 内嵌 7 个 crate 的职责描述 (从 Cargo.toml + lib.rs 头几行提取)

Per: AGENTS.md §5 v0.6 + 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec
守门 #9: 模板确定性 + 可 git log -p --follow 实证
"""
from pathlib import Path

REPO_ROOT = Path("D:/Star")
SPECS_DIR = REPO_ROOT / "docs/specs"

# 7 个 supporting crate 的元数据 (从 Cargo.toml + lib.rs 头几行提取)
CRATES = [
    {
        "name": "cli",
        "display": "domain-cli",
        "responsibility": "Star CLI Agent Registry (per crates/domain-cli/src/lib.rs v0.1 实施实装)",
        "boundary_owned": [
            "6 种内置 CLI Agent 注册 (claude / codex / openclaw / hermes / gemini / aider) + 自定义 schema",
            "**双模式 API Key 存储** (per 2026-08-29 09:07 JST 用户拍板):",
            "  - `EncryptedRust`: AES-256-GCM 加密存储于 domain-cli",
            "  - `EnvironmentVar`: 运行时读 process env 即可,不存储",
            "API Agent 适配 (OpenClaw / Hermes HTTP API, 替代 CLI spawn)",
            "Agent Adapter 模式 (per ADR-0025 vendor adapter anti-contamination)",
        ],
        "boundary_not_owned": [
            "Agent Process 生命周期 (spawn/kill/lease) (属 `domain-agent`)",
            "Worktree 实装 (属 `domain-worktree`)",
            "Context Packet 编译 (属 `domain-context`)",
        ],
        "entities": [
            "`CliAgent` (聚合根): agent_id / name / kind (内置 6 种 / Custom) / schema / api_key_ref (EncryptedRust | EnvironmentVar) / created_at",
            "`ApiKeyRef`: ref_type (EncryptedRust | EnvironmentVar) + ciphertext (EncryptedRust) | env_var_name (EnvironmentVar)",
            "`AgentAdapter`: provider / endpoint / auth_pattern / rate_limit",
        ],
        "invariants": [
            "INV-CLI-01: 6 种内置 CLI Agent schema 不得修改 (向后兼容)",
            "INV-CLI-02: API Key 不得 plaintext 存储 (per REQ-SEC-002)",
            "INV-CLI-03: API Key EnvVar 模式不写入 domain-cli (运行时读取)",
            "INV-CLI-04: agent_id 全局唯一,跨 tenant 隔离",
        ],
        "interfaces": [
            "`CliAgentCommandPort`: register / deregister / list / get",
            "`CliAgentQueryPort`: lookup by name / by kind / list-enabled",
            "`ApiKeyResolver`: resolve(ref) -> SecretString (注入式,调用方不接触)",
        ],
        "risks": [
            "RISK-CLI-01: API Key 泄漏 → AES-256-GCM 加密 + 注入式 (per ADR-0025)",
            "RISK-CLI-02: 自定义 schema 兼容性 → schema version 字段 + 渐进式升级",
            "RISK-CLI-03: EnvVar 模式 secret 误打印 → 引用 $env:VAR 不打印 (per AGENTS.md §4 #5 hard ban)",
        ],
    },
    {
        "name": "form",
        "display": "domain-form",
        "responsibility": "Star Form Engine (per crates/domain-form/src/lib.rs v0.1 实施实装)",
        "boundary_owned": [
            "12 字段类型 (text / number / date / select / multi-select / file / user / work-item / ...)",
            "条件逻辑 (show_if / require_if / hide_if, JSON Logic 风格)",
            "提交触发 (工单创建 / 字段更新 / 发邮件 / 调 Webhook)",
            "公开 URL 表单 (匿名提交)",
            "表单版本管理",
        ],
        "boundary_not_owned": [
            "WorkItem 聚合根 (属 `domain-work-item`,form 触发工单创建后由 work-item 接管)",
            "Notification 投递 (属 `domain-notification`,form 触发后调用 notification Port)",
        ],
        "entities": [
            "`FormDefinition` (聚合根): form_id / tenant_id / project_id / title / version / fields[] / triggers[] / public_url_slug?",
            "`FormField`: field_id / type (12 种) / label / required / default_value? / options? (select) / validation?",
            "`FormCondition`: rule (show_if / require_if / hide_if) + expression (JSON Logic)",
            "`FormSubmission`: submission_id / form_id / submitted_by / submitted_at / field_values{}",
        ],
        "invariants": [
            "INV-FORM-01: 表单 schema 不可变,变更需新建 version (per 缺标比错标)",
            "INV-FORM-02: 公开 URL 表单不可要求登录 (匿名提交场景)",
            "INV-FORM-03: 条件逻辑不得循环引用 (静态分析检测)",
            "INV-FORM-04: 提交触发必走 Workflow Guard 校验,不可绕过 (per REQ-WF-003)",
        ],
        "interfaces": [
            "`FormDefinitionCommandPort`: create / update / publish (锁定 schema) / archive",
            "`FormDefinitionQueryPort`: get / list-by-project / get-by-public-slug",
            "`FormSubmissionPort`: submit (含条件逻辑执行 + 触发动作) / list-by-form",
        ],
        "risks": [
            "RISK-FORM-01: 公开 URL 表单被滥用 → 限流 + Captcha + 提交审核 (per integration-design §8 Rate Limit)",
            "RISK-FORM-02: 条件逻辑 bug 导致字段绕过 → 单元测试覆盖 show_if/require_if",
            "RISK-FORM-03: 提交触发失败导致工单半成品 → Saga 编排 (per spec/saga/01 v0.2 §4)",
        ],
    },
    {
        "name": "report",
        "display": "domain-report",
        "responsibility": "Star Report Engine (per crates/domain-report/src/lib.rs v0.1 实施实装)",
        "boundary_owned": [
            "10 种报表类型:",
            "  1. Burndown (Sprint 燃尽图, per REQ-PLAN-005)",
            "  2. Burnup (Sprint 燃起图)",
            "  3. Velocity (跨 Sprint 速度)",
            "  4. CFD (Cumulative Flow Diagram)",
            "  5. Control Chart (周期时间 + 异常检测)",
            "  + 5 种 V1 报表 (Cycle Time / Throughput / Workload / SLA / Forecast)",
        ],
        "boundary_not_owned": [
            "WorkItem 数据源 (从 `domain-work-item` Projection 读取, 不持有事实)",
            "Sprint 数据源 (从 `domain-planning` 读取)",
        ],
        "entities": [
            "`ReportDefinition` (聚合根): report_id / tenant_id / project_id / type (10 种) / config{} / schedule? (cron) / recipient_ids[]",
            "`ReportSnapshot` (Projection): report_id / generated_at / data{} / data_source_refs[]",
            "`ReportSchedule`: cron_expression / next_run_at / last_run_at? / enabled",
        ],
        "invariants": [
            "INV-REPORT-01: Report 是 Projection, 不得持有 SoR 业务事实 (per requirements §12 REQ-SEARCH-001)",
            "INV-REPORT-02: Report 数据走 cache 5min TTL, 不实时拉源",
            "INV-REPORT-03: Report 订阅触发走 worker projection role (per basic-design v0.16 §4.12.2)",
        ],
        "interfaces": [
            "`ReportDefinitionCommandPort`: create / update / enable / disable / delete",
            "`ReportQueryPort`: get / list-by-project / generate (即时生成, 不缓存) / latest-snapshot",
            "`ReportSchedulePort`: schedule / unschedule / list-pending-runs",
        ],
        "risks": [
            "RISK-REPORT-01: 大报表性能 → 异步生成 + 缓存 + 分页 (per ADR-0026 §3 Fallback Ladder)",
            "RISK-REPORT-02: 数据不一致 (源数据更新中) → snapshot 时间戳 + 增量更新",
            "RISK-REPORT-03: 订阅触发噪音 → REQ-NOTIF-002 降噪策略",
        ],
    },
    {
        "name": "theme",
        "display": "domain-theme",
        "responsibility": "Star 主题系统 (per crates/domain-theme/src/lib.rs 实施实装, per 2026-08-29 04:09 JST 用户拍板)",
        "boundary_owned": [
            "前端: next-themes (React 主题切换)",
            "结构: 三元组 enum (Light + Dark + 扩展位)",
            "优先级: 3 层覆盖 (Personal > Tenant > Global)",
            "拓展: 主题 + 主题色系, 后续可扩展 (动画 / 字体)",
        ],
        "boundary_not_owned": [
            "用户身份 (从 `domain-identity` 读 user_id)",
            "Tenant 隔离 (从 `domain-tenant` 读 tenant_id)",
        ],
        "entities": [
            "`ThemePreference` (值对象): user_id? (Personal) / tenant_id? (Tenant) / global (bool) / theme (light | dark) / accent_color?",
            "`ThemeRegistry`: theme_id / name / accent_colors[] / preview_image_url",
        ],
        "invariants": [
            "INV-THEME-01: Personal > Tenant > Global 优先级严格生效",
            "INV-THEME-02: 主题切换不刷新页面 (前端 only)",
            "INV-THEME-03: 跨设备同步 Personal 主题 (经 user 身份)",
        ],
        "interfaces": [
            "`ThemePreferencePort`: get (resolve 3 层) / set (per layer) / reset",
            "`ThemeRegistryPort`: list-available / get-default",
        ],
        "risks": [
            "RISK-THEME-01: 主题不兼容 (色盲 / 高对比) → WCAG 2.1 AA 合规检测",
            "RISK-THEME-02: 跨设备同步冲突 → last-write-wins + Audit",
        ],
    },
    {
        "name": "dashboard",
        "display": "domain-dashboard",
        "responsibility": "Star Dashboard Engine (per crates/domain-dashboard/src/lib.rs v0.1 实施实装)",
        "boundary_owned": [
            "12-grid 布局 (Tailwind 标准)",
            "10 Gadget 类型 (WorkItem 列表 / Sprint 燃尽 / 报表快照 / 自定义查询 / ...)",
            "Wallboard 全屏模式",
            "共享 / 权限",
            "订阅 + 邮件",
        ],
        "boundary_not_owned": [
            "WorkItem 数据 (从 `domain-work-item` Projection 读)",
            "Report 数据 (从 `domain-report` Projection 读)",
            "User / 权限 (从 `domain-identity` / `domain-permission` 读)",
        ],
        "entities": [
            "`Dashboard` (聚合根): dashboard_id / tenant_id / owner_id / title / layout (12-grid) / gadgets[] / shared_with[]",
            "`DashboardGadget`: gadget_id / type (10 种) / position (x, y, w, h) / config{}",
            "`DashboardSubscription`: dashboard_id / subscriber_id / cadence (realtime / hourly / daily)",
        ],
        "invariants": [
            "INV-DASH-01: 12-grid 布局严格 12 列 (Tailwind 标准)",
            "INV-DASH-02: Gadget 不重叠 (静态分析检测)",
            "INV-DASH-03: Wallboard 模式无编辑权限 (read-only)",
        ],
        "interfaces": [
            "`DashboardCommandPort`: create / update / add-gadget / remove-gadget / reorder / share / delete",
            "`DashboardQueryPort`: get / list-by-owner / list-shared-with-me / get-wallboard",
            "`DashboardSubscriptionPort`: subscribe / unsubscribe / notify-update",
        ],
        "risks": [
            "RISK-DASH-01: 大 Dashboard 性能 → 懒加载 + 虚拟滚动",
            "RISK-DASH-02: Wallboard 模式被滥用 → 只读强制 + Audit",
        ],
    },
    {
        "name": "ai",
        "display": "domain-ai",
        "responsibility": "Star AI Engine (per crates/domain-ai/src/lib.rs v0.1 实施实装)",
        "boundary_owned": [
            "3 类 Rovo-like Agent:",
            "  1. Workflow Builder (自然语言 → Workflow JSON)",
            "  2. Work Readiness Checker (开工前 AI 自检: AC 覆盖 / 依赖 / Conflict)",
            "  3. Report Insight (报表 → 自然语言洞察)",
            "+ JQL AI (自然语言 → JQL, per requirements §12 REQ-SEARCH-002 V1 候选)",
        ],
        "boundary_not_owned": [
            "Coding Agent 进程 (属 `domain-agent`,本 crate 只生成 Workflow JSON 等元数据,不 spawn agent)",
            "AI Provider 凭据 (属 `domain-kms`,本 crate 通过 Adapter 抽象调用)",
        ],
        "entities": [
            "`AiAgent` (聚合根): agent_id / kind (4 类) / input_schema / output_schema / model_preference / prompt_template",
            "`AiInvocation` (Projection): invocation_id / agent_id / input{} / output{} / latency_ms / token_used / created_at",
            "`AiFeedback` (Feedback): invocation_id / user_id / rating (1-5) / comment?",
        ],
        "invariants": [
            "INV-AI-01: AI 输出必须可解释 (per requirements §28 AI Extension)",
            "INV-AI-02: AI 操作必须可审计 (per REQ-AUDIT-002)",
            "INV-AI-03: AI Token 预算受控 (per basic-design §4.4.4 Token Budget P0/P1/P2/P3/P4)",
            "INV-AI-04: Workflow Builder 输出必须 JSON Schema 验证 (拒绝幻觉结构)",
        ],
        "interfaces": [
            "`AiAgentCommandPort`: register / deregister / list / get",
            "`AiInvocationPort`: invoke (sync) / invoke-async (fire-and-forget) / get-result / list-by-user",
            "`AiFeedbackPort`: submit / list-by-agent / aggregate-rating",
        ],
        "risks": [
            "RISK-AI-01: AI 幻觉 → 强制 JSON Schema 验证 + 用户 confirm gate",
            "RISK-AI-02: Token 超支 → Token Budget P0 不可裁剪 (per basic-design §4.4.4)",
            "RISK-AI-03: Workflow Builder 生成危险操作 → dry-run 强制 + 显式 confirm",
            "RISK-AI-04: AI Provider 数据泄漏 → ProviderDataBoundary (per domain-tenant SecurityPolicy)",
        ],
    },
    {
        "name": "kms",
        "display": "domain-kms",
        "responsibility": "KMS 抽象 (Vault / AWS KMS 凭据) — E.4 mock 备选 (per 29692a7 mock 备选路径, GAP-01 提前补 spec)",
        "boundary_owned": [
            "KMS Provider 抽象 (Vault / AWS KMS / Mock / Future)",
            "凭据 CRUD: encrypt / decrypt / sign / verify (per use case)",
            "凭据轮转: schedule / force / grace-period",
            "凭据审计: 谁在何时访问哪个凭据",
        ],
        "boundary_not_owned": [
            "具体 Provider 实现 (Mock / Vault / AWS KMS, 由 adapter 实现)",
            "凭据内容 (调用方通过 `SecretString` 引用,不接触 plaintext)",
        ],
        "entities": [
            "`KmsKey` (聚合根): key_id / tenant_id / alias / kind (Encryption | Signing) / algorithm / rotation_policy / status (Active | PendingDeletion | Disabled)",
            "`KmsAuditEvent`: event_id / key_id / actor_id / operation / timestamp / success",
            "`SecretString` (值对象): 持有 plaintext, 0 处打印 (per AGENTS.md §4 #5 hard ban)",
        ],
        "invariants": [
            "INV-KMS-01: SecretString 不得离开 domain-kms 边界 (调用方通过 API 注入, 不接触 plaintext)",
            "INV-KMS-02: 凭据轮转期间旧凭据 grace-period 仍可用 (per use case)",
            "INV-KMS-03: 凭据删除前必 force-rotate 一次 (防残留)",
            "INV-KMS-04: 凭据审计必 Append-only (per REQ-AUDIT-001)",
        ],
        "interfaces": [
            "`KmsCommandPort`: create-key / rotate / schedule-deletion / force-rotate",
            "`KmsQueryPort`: get-key (不返回 plaintext) / list-by-tenant / get-rotation-status",
            "`SecretResolver`: resolve(ref) -> SecretString (注入式,调用方不接触)",
        ],
        "risks": [
            "RISK-KMS-01: 凭据泄漏 → SecretString 0 打印 + 引用 SecretResolver 注入式",
            "RISK-KMS-02: 凭据轮转失败 → grace-period + 自动 retry + 人工介入",
            "RISK-KMS-03: Provider 不可用 → fallback 到 Mock (per 29692a7 mock 备选路径)",
            "RISK-KMS-04: 删除残留 → force-rotate 前置 + PendingDeletion 状态保留 grace-period",
        ],
    },
]


def render_spec(crate):
    """生成单份 spec markdown"""
    name = crate["name"]
    display = crate["display"]
    lines = []
    lines.append(f"# {display} 实施 spec")
    lines.append("")
    lines.append("> **状态**: Draft v0.1 (2026-09-01)")
    lines.append("> **触发**: per 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01)")
    lines.append("> **下游交付**: Implementation team — Rust crate 路径 `crates/{display}/`")
    lines.append("")
    lines.append(f"> **dual-use 警告** (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板):")
    lines.append(f"> {display} 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../../basic-design.md))")
    lines.append(f"> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 1. 职责与边界")
    lines.append("")
    lines.append(f"`{display}` 负责 **{crate['responsibility']}**。")
    lines.append("")
    lines.append("**属于本 crate 的**:")
    for item in crate["boundary_owned"]:
        lines.append(f"- {item}")
    lines.append("")
    lines.append("**不属于本 crate 的**:")
    for item in crate["boundary_not_owned"]:
        lines.append(f"- {item}")
    lines.append("")
    lines.append("## 2. 关键实体")
    lines.append("")
    for entity in crate["entities"]:
        lines.append(f"- {entity}")
    lines.append("")
    lines.append("## 3. 关键不变量")
    lines.append("")
    lines.append("| ID | 不变量 |")
    lines.append("|---|---|")
    for inv in crate["invariants"]:
        lines.append(f"| {inv.split(':')[0]} | {inv.split(':', 1)[1].strip() if ':' in inv else inv} |")
    lines.append("")
    lines.append("## 4. 接口契约")
    lines.append("")
    for iface in crate["interfaces"]:
        lines.append(f"- {iface}")
    lines.append("")
    lines.append("## 5. 跨 domain 接触面 (v0.16 协作细化新增)")
    lines.append("")
    lines.append(f"per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md),`{display}` 作为 supporting crate 跨 22 bounded context + 6 supporting crate 的接触面。")
    lines.append("")
    lines.append("**协作模式** (per [basic-design v0.16 §3.1 解耦机制](../../../basic-design.md) 8 种):")
    lines.append("")
    lines.append("| 接触类型 | 目标 domain | 协作方式 | 引用 |")
    lines.append("|---|---|---|---|")
    lines.append(f"| `{name}` 触发工单创建 | work-item | Customer-Supplier (Port) | per `{name}` 提交触发 |")
    lines.append(f"| `{name}` 读取用户身份 | identity | Shared Kernel (UserId) | per User 引用 |")
    lines.append(f"| `{name}` 审计所有操作 | audit | Separate Ways (Append-only) | per AuditRecorder Port |")
    lines.append(f"| `{name}` 触发降噪通知 | notification | Separate Ways (异步) | per REQ-NOTIF-002 |")
    lines.append("")
    lines.append("> 详细接触面待 [basic-design v0.16 §3.2.9](../../../basic-design.md) 后续 sweep 补充 (per GAP-01 后续 P3 阶段)。")
    lines.append("")
    lines.append("## 6. 风险与缓解")
    lines.append("")
    lines.append("| Risk | 影响 | 缓解 | 引用 |")
    lines.append("|---|---|---|---|")
    for risk in crate["risks"]:
        parts = risk.split(" → ", 1)
        if len(parts) == 2:
            rid, rest = parts
            # rest format: "description → mitigation"
            if " → " in rest:
                desc, mit = rest.split(" → ", 1)
                lines.append(f"| {rid} | {desc.strip()} | {mit.strip()} | {display} §6 |")
            else:
                lines.append(f"| {rid} | {rest.strip()} | — | {display} §6 |")
        else:
            lines.append(f"| {risk} | — | — | {display} §6 |")
    lines.append("")
    lines.append("## 7. 修订历史")
    lines.append("")
    lines.append("| 版本 | 日期 | 修订人 | 修订内容 | 触发 |")
    lines.append("|---|---|---|---|---|")
    lines.append("| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版：7 章简化模板 (职责 / 实体 / 不变量 / 接口 / 接触面 / 风险 / 修订历史) | 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (per PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01) |")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01")
    lines.append("> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问")
    lines.append("")
    return "\n".join(lines)


def main():
    print(f"[gen_supporting_specs] 生成 7 份 supporting crate spec ...")
    summary = []
    for crate in CRATES:
        name = crate["name"]
        spec_path = SPECS_DIR / f"domain-{name}-spec.md"
        if spec_path.exists():
            print(f"  [SKIP] {name}: spec 已存在")
            continue
        content = render_spec(crate)
        spec_path.write_text(content, encoding="utf-8")
        lines = len(content.splitlines())
        summary.append((name, lines, spec_path))
        print(f"  [OK]   {name}: 生成 {lines} 行")

    print(f"\n[gen_supporting_specs] 完成: {len(summary)} 份 spec 已生成")
    print("\n=== Summary ===")
    for n, l, p in summary:
        print(f"  {n:15s}  {l:4d} lines  →  {p.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    main()
