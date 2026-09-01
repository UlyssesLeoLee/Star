"""
Insert new §7 シナリオ + テストデータ into docs/test-design.md
and renumber existing §7-§16 to §8-§17.

Trigger: 2026-09-01 16:13 JST Ulysses 拍板 "加 §X 场景/数据章" (test-design 应用范围 = 主 test-design.md).
Per AGENTS.md §1.0 19:39 JST 用户授权, Mavis 接手代签.
Per 守门 #11 缺标比错标, 缺口列 Test-J.14 ~ J.17.
Per 守门 #12 commit-time 同步, docs-only 改动.
Per 守门 #1 实证要求 git log -p --follow.

v0.6 重写要点 (相对于 v0.5 实证失败):
- 修订历史行 (| v0.X | 2026-...) 整行跳过, 保留历史叙事不沿用 (per 守门 #12)
- 每个 §N.M 单独判断前置文本 (~200 字符) 是否含外部文档名, 不再"整行一刀切"
- EXTERNAL_DOC_NAMES 列表考虑 .md 后缀, 修复 v0.5 实证中 `STAR-P3-WBS-001.md §13` 被错改 bug

不变量保留:
- §6 验收测试现行文本(§6.1 Gherkin + §6.2 AC 矩阵 + §6.3 验收门禁)全部不动
- §10(原 §9) 测试数据管理详细策略不重复
- §14(原 §13) Open Issues Test-J.1 ~ J.13 全部不动, 仅追加 J.14 ~ J.17
- 外部文档对 test-design 的引用都不带具体 §N 编号, 顺位不破坏跨文档链接
- 修订历史 v0.1 ~ v0.5 不动, 仅追加 v0.6
"""

import re
from pathlib import Path

TEST_DESIGN = Path("D:/Star/docs/test-design.md")
BACKUP = Path("D:/Star/docs/test-design.md.v0.5.bak")

# Renumbering map: old -> new
RENUMBER = {
    7: 8, 8: 9, 9: 10, 10: 11,
    11: 12, 12: 13, 13: 14, 14: 15, 15: 16, 16: 17
}

# External doc names (substring match in preceding text)
EXTERNAL_DOC_NAMES = [
    "Requirements", "requirements", "REQUIREMENTS",
    "Basic Design", "basic-design",
    "API Design", "api-design",
    "Security Design", "security-design",
    "Runtime Design", "runtime-design",
    "Data Design", "data-design",
    "AI/Agent Design", "AI Agent Design", "ai-agent-design",
    "Integration Design", "integration-design",
    "External Design", "external-design",
    "Operation Design", "operation-design",
    "frontend-design", "frontend design",
    "STAR-P3-WBS-001",
    "AGENTS.md",
    "QA-DRIFT-001",
    "feedback.md",
    "handoff.md",
    "PHASE-D", "PHASE-E", "PHASE-F", "PHASE-H", "PHASE-MOBILE",
    "IN-001", "INC-001",
]


def is_external_preceding(line_text: str, match_start: int) -> bool:
    """Check if a §N match at match_start is an external doc reference.

    Looks at text BEFORE the match (200 chars) PLUS the § symbol and 5 chars after.
    A doc reference is "external" if a doc name (with optional .md) is followed by
    optional whitespace + § somewhere in that extended window.
    Just mentioning the doc name (without §) is not external.
    """
    # Build extended context: 200 chars before + § + 5 chars after
    context_start = max(0, match_start - 200)
    context_end = min(len(line_text), match_start + 6)  # include § and a few chars
    context = line_text[context_start:context_end]

    # Check for 《》 pair
    if "《" in context and "》" in context:
        return True

    # Check for any doc_name (with optional .md) followed by § in context
    for doc_name in EXTERNAL_DOC_NAMES:
        pattern = re.escape(doc_name) + r"(?:\.md)?\s*§"
        if re.search(pattern, context, re.IGNORECASE):
            return True

    return False


def is_revision_history_line(line: str) -> bool:
    """Detect修订历史 行: `> | v0.X | 2026-...` or `| v0.X | 2026-...`"""
    return bool(re.match(r"^\s*>?\s*\|\s*v\d+\.\d+\s*\|\s*\d{4}-\d{2}-\d{2}", line))


def renumber_text_refs(content: str, debug: bool = False) -> str:
    """Renumber §N references in content, skipping修订历史 and external doc refs.

    Per-match context check: for each §N.M or §N match, check the preceding
    text (~200 chars) for external doc names. If found, skip renumbering.
    """
    lines = content.split("\n")
    new_lines = []
    renumbered_count = 0
    skipped_count = 0

    for line in lines:
        # Skip修订历史 行 (per 守门 #12 不沿用 v0.x 旧叙事)
        if is_revision_history_line(line):
            new_lines.append(line)
            continue

        # Process each §N.M and §N match
        new_line = ""
        last_end = 0

        # Combined regex: match §N.M or §N
        for m in re.finditer(r"§(\d+)(?:\.(\d+)(?:\.(\d+))?)?", line):
            n = int(m.group(1))
            if n not in RENUMBER:
                continue

            # Get preceding text (200 chars before the match in current line)
            preceding_start = max(0, m.start() - 200)
            preceding = line[preceding_start:m.start()]

            if is_external_preceding(line, m.start()):
                # External reference, keep as is
                if debug:
                    skipped_count += 1
                new_line += line[last_end:m.end()]
            else:
                # Internal reference, renumber
                old_section = m.group(1)
                new_section = str(RENUMBER[n])
                if m.group(2):
                    new_section += f".{m.group(2)}"
                if m.group(3):
                    new_section += f".{m.group(3)}"
                new_text = f"§{new_section}"
                if debug:
                    renumbered_count += 1
                new_line += line[last_end:m.start()] + new_text

            last_end = m.end()

        new_line += line[last_end:]
        new_lines.append(new_line)

    if debug:
        print(f"[renumber] renumbered: {renumbered_count}, skipped (external): {skipped_count}")

    return "\n".join(new_lines)


# New §7 content (post-renumber numbering, i.e. §10 not §9 for testing data management)
NEW_S7 = '''## 7. シナリオ + テストデータ (Test Scenarios & Test Data Catalogue)

> **本节目的**: 把 §6 验收测试的 AC↔Test 映射展开为可执行的「シナリオ + テストデータ」表, 覆盖 25 domain-* Module + 5 域业务 (per 守门 #3 历史治理命名) + 6 E2E 关键流程 (per §5.1), 以及 §6.2.1 / §6.3.3 / §6.3.4 三项 V1 Should-Have REQ。
>
> **继承**: §6 验收测试 + §10 测试数据管理 (本次升版顺位) + §1.3 数据原则 + §2 测试层级 + §5 E2E 策略。
>
> **不重复**: §6 已有 VAL-001/DSG-001/OPS-001 场景表, 此处只补充其他 REQ 的场景; §10 测试数据管理详细策略不重复, 此处聚焦"场景 ↔ 数据"映射。
>
> **场景表 Schema**: §7.1.1; **测试数据目录**: §7.3; **可追溯矩阵**: §7.4; **已知缺口**: §7.5。

### 7.1 シナリオ表 Schema

#### 7.1.1 通用 Schema

| 列 | 含义 | 必填 |
|---|---|---|
| # | 场景编号 (REQ-ID + 序号) | ✅ |
| シナリオ (Scenario) | 步骤或条件描述 (Gherkin Given/When/Then 风格) | ✅ |
| 入力 (Input) | 触发条件 / 请求参数 | ✅ |
| 期待結果 (Expected) | 响应 / 状态 / 副作用 | ✅ |
| テストデータ (Test Data) | 关联 fixture ID / seed / 数据来源 | ✅ |
| レベル (Level) | UnitTestLevel / IntegrationTestLevel / AcceptanceTestLevel / SystemTestLevel (per §6.2.1) | ✅ |
| 自動化 (Automation) | Auto / Semi-Auto / Manual | ✅ |
| 5 域 (Domain) | player / economy / match / social / admin (per 守门 #3 真人 Lead 结构, 非 DDD 映射) | optional |

#### 7.1.2 レベル ↔ 场景类型 默认映射

| Level | 适用场景 | 默认 Automation |
|---|---|---|
| UnitTestLevel | §7.2.1 25 domain 纯函数 / 状态机单测 | Auto |
| IntegrationTestLevel | §7.2.2 5 域业务 + §7.2.3 跨域 Saga | Auto |
| AcceptanceTestLevel | §7.2.4 6 E2E 关键流程 | Auto (Playwright) |
| SystemTestLevel | §7.2.5 NATS+PostgreSQL+Valkey 全容器集成 (per §2.2.4 顺延) | Semi-Auto |

### 7.2 シナリオ (Scenarios)

#### 7.2.1 25 Domain Module シナリオ (per §2.1.1)

> **范围**: 25 domain-* crate 的纯函数 / 状态机 / Guard 单测, 每个 domain 列 3-5 个核心场景。
> **Level**: UnitTestLevel (Rust `cargo test`)
> **Test Data**: `crates/domain-X/tests/fixtures/` 各自维护
> **5 域 disclaimer**: 25 domain 是 DDD bounded context (per §5 仓库拓扑), 不是 5 域业务命名, 不做强制映射

| Domain | # | シナリオ | 期待結果 | テストデータ |
|---|---|---|---|---|
| **domain-tenant** | T-TN1 | 创建 tenant + 默认 workspace + 默认 RBAC scheme | 200 + tenant_id 返回 + 3 子对象级联创建 | `tenant_factory.acme()` |
|  | T-TN2 | 跨 tenant RLS 隔离 (A 创建, B 查询) | 403 RLS_BYPASS_DENIED | `tenant_a / tenant_b` |
|  | T-TN3 | 软删除 + 回收站 | query 含 `include_deleted=true` 返 deleted | `tenant_factory.soft_deleted()` |
|  | T-TN4 | 4 域 Lead 越权 (per 守门 #3 独立) | 跨域操作 拒绝 | `actor_factory(Lead, other_domain)` |
| **domain-workspace** | T-WS1 | CRUD + 关联 tenant | 200 + tenant_id 校验 | `workspace_factory.default()` |
|  | T-WS2 | 工作区权限继承 | 子对象继承父 workspace 策略 | `workspace_factory.with_rbac()` |
|  | T-WS3 | 多 workspace 切换 | session 切换后 RLS 立即生效 | `workspace_factory.multi()` |
| **domain-project** | T-PJ1 | 模板实例化 + policy 注入 | 200 + policy 列表 | `project_factory.from_template()` |
|  | T-PJ2 | 敏捷 vs 瀑布模式切换 | mode 字段决定 Guard 集 | `project_factory.waterfall()` |
|  | T-PJ3 | Project 删除级联 | 子对象 soft delete 不级联物理删 | `project_factory.with_children()` |
| **domain-work-item** | T-WI1 | 3 态状态机: Created → InProgress → Done | happy path | `work_item_factory.created()` |
|  | T-WI2 | 非法转换 (try_transition invalid) | Err + reason | `work_item_factory.done()` + 目标非法状态 |
|  | T-WI3 | 扩展 4 态 (per basic-design §2.5) | 状态机含 Blocked / Cancelled | `work_item_factory.blocked()` |
|  | T-WI4 | Guard 未配置 (非瀑布) | 透明放行 | `project_factory.agile()` + WorkItem with Guard |
| **domain-workflow** | T-WF1 | Workflow 定义 + 状态迁移 | 200 + DAG 校验 | `workflow_factory.3step()` |
|  | T-WF2 | 多分支 + Join | 全部 branch 完成后流转 | `workflow_factory.diamond()` |
|  | T-WF3 | 状态迁移 + 事件发布 | event bus 收到 event | `workflow_factory + event subscriber` |
| **domain-board** | T-BD1 | Board 视图 + Column CRUD | 200 + column 顺序保持 | `board_factory.kanban()` |
|  | T-BD2 | 列重排 | order 字段更新 | `board_factory.with_5_columns()` |
|  | T-BD3 | WorkItem 拖拽跨列 | status 同步更新 | `board_factory + work_item_factory` |
| **domain-planning** | T-PL1 | Sprint 创建 + 容量计算 | 200 + capacity vs committed | `sprint_factory.2week()` |
|  | T-PL2 | Backlog 优先级排序 | rank 字段反映拖拽顺序 | `backlog_factory.with_20_items()` |
|  | T-PL3 | Sprint 结束未完成项回退 | status → backlog | `sprint_factory.ended()` |
| **domain-permission** | T-PM1 | RBAC scheme 评估 | allow / deny 矩阵 | `rbac_factory.scheme()` |
|  | T-PM2 | 5 域 Lead 互不兼任 (per 守门 #3) | 跨域操作 拒绝 | `actor_factory(Lead, player) + admin domain op` |
|  | T-PM3 | Agent 操作受限 | Agent 仅 Own 域 | `actor_factory(AgentSession)` |
|  | T-PM4 | 资源策略 + ABAC | attribute 评估 | `policy_factory.with_conditions()` |
| **domain-comment** | T-CM1 | @ 提及 + 通知 | notification 创建 | `comment_factory.with_mention()` |
|  | T-CM2 | 附件上传 + PII 检测 | PII 字段脱敏 | `comment_factory.with_attachment()` |
|  | T-CM3 | 编辑历史 | edit history 保留 | `comment_factory.edited()` |
| **domain-relation** | T-RL1 | 阻塞 + 关联 | 双向 link 创建 | `relation_factory.blocked_by()` |
|  | T-RL2 | 循环依赖检测 | 拒绝 + 报错 | `relation_factory.cycle_attempt()` |
|  | T-RL3 | 跨 WorkItem 关联 | cross-link 写入 | `relation_factory.cross_workitem()` |
| **domain-development** | T-DV1 | ChangeSet 创建 + 关联 WorkItem | 200 + parent_id 校验 | `change_set_factory` |
|  | T-DV2 | Symbol Index 同步 | index 写入 + 检索命中 | `change_set_factory + symbol_index_query` |
|  | T-DV3 | 文件级 diff 追踪 | diff 字段记录 | `change_set_factory.with_diff()` |
| **domain-worktree** | T-WT1 | 17 状态机 happy path | 状态逐步迁移 | `worktree_factory.creating()` |
|  | T-WT2 | Conflict 检测 + 隔离 | conflict field 写入 + 隔离分支 | `worktree_factory.conflict()` |
|  | T-WT3 | Merge 后清理 | 子对象清理 | `worktree_factory.merged()` |
|  | T-WT4 | Isolation 字段强制 (per 守门 #1 派生) | 越权访问隔离区 拒绝 | `worktree_factory.isolated + other_tenant` |
| **domain-agent** | T-AG1 | 14 状态机 | 状态逐步迁移 | `agent_factory.idle()` |
|  | T-AG2 | AgentPolicy 强制 | 违规操作 拒绝 | `agent_factory.with_policy_violation()` |
|  | T-AG3 | Lease + Heartbeat 过期 | session 自动回收 | `agent_factory.lease_expired()` |
|  | T-AG4 | 5 域 Lead 真人指派 (per 守门 #3) | actor 必须含 Lead role | `actor_factory.sre_lead_player` |
| **domain-feedback** | T-FB1 | 6 状态 happy path | Open → Ack → Resolving → Resolved → Closed | `feedback_factory.open()` |
|  | T-FB2 | 5 段式 Instruction 校验 | 缺段 拒绝 | `feedback_factory.missing_provenance` |
|  | T-FB3 | Supersede 旧版本 | 旧版只读可追溯 | `feedback_factory.supersede()` |
| **domain-context** | T-CX1 | Context Compiler 编译 | Provenance.source 完整 | `context_factory.untrusted` |
|  | T-CX2 | Decision 3 态: Proposed/Approved/Rejected | 状态机转换 | `decision_factory.proposed()` |
|  | T-CX3 | P5 隔离层 | Skill/Playbook 走隔离通道 | `context_factory.skill_source` |
|  | T-CX4 | Untrusted 隔离 | README 注入不影响 P0 | `context_factory.readme_injection` |
| **domain-validation** | T-VL1 | Acceptance Coverage 报告 | coverage% + 缺项清单 | `validation_factory.with_5_ac` |
|  | T-VL2 | Evidence 权重评估 | weighted score 正确 | `validation_factory.weighted` |
|  | T-VL3 | Level 维度 (per §6.2.1) | 4 Level 字段 | `validation_factory.level_split` |
|  | T-VL4 | VAL-001 四重门 (per §6.3.2) | 4 门同时通过 | `validation_factory.all_gates_pass` |
| **domain-scm** | T-SC1 | ACL 翻译 (GitHub / GitLab / Gitea) | 规则映射正确 | `scm_factory.github_acl` |
|  | T-SC2 | Event 映射 (push / PR / comment) | domain event 触发 | `scm_factory.push_event` |
|  | T-SC3 | Webhook HMAC 验证 (per §1 S3) | 签名错误 拒绝 | `scm_factory.bad_signature` |
| **domain-identity** | T-ID1 | Device 注册 + DeviceId | 200 + device_id 强类型 | `device_factory.new()` |
|  | T-ID2 | User + Credential 绑定 | 凭据校验通过 | `user_factory.with_credential` |
|  | T-ID3 | Token 轮换 | 旧 token 失效 | `identity_factory.token_rotation` |
|  | T-ID4 | 5 域 Lead 真人凭据 (WebAuthn 强制, per 守门 #3 派生) | 缺 WebAuthn 拒绝 | `actor_factory(Lead) no_webauthn` |
| **domain-audit** | T-AU1 | 9 问必答 (per §2.5 顺延) | 缺问 拒绝 | `audit_factory.9q_incomplete` |
|  | T-AU2 | 7 级 Retention 策略 | 旧记录按级别归档 | `audit_factory.retention_l3` |
|  | T-AU3 | 不可篡改 (append-only) | 修改尝试 拒绝 | `audit_factory.attempt_modify` |
|  | T-AU4 | 强 MFA 强制 (per 8/21 JST 拒绝兼任) | SRE 角色缺 MFA 拒绝 | `actor_factory.sre no_mfa` |
| **domain-search** | T-SR1 | Query 语法解析 | AST 正确 | `search_factory.complex_query` |
|  | T-SR2 | Index 同步延迟 | lag < 5s | `search_factory.indexer` |
|  | T-SR3 | 跨 domain 联合搜索 | 多 domain 命中 | `search_factory.cross_domain` |
| **domain-notification** | T-NT1 | 模板渲染 | placeholder 替换 | `notification_factory.template` |
|  | T-NT2 | 渠道 (邮件 / 站内 / IM) | 全部发送 | `notification_factory.multi_channel` |
|  | T-NT3 | 退避策略 | 失败重试 N 次后放弃 | `notification_factory.retry_exhausted` |
|  | T-NT4 | Inbox 噪声抑制 (per §1 S2) | Agent 中间步骤被抑制 | `notification_factory.agent_intermediate` |
| **domain-integration** | T-IG1 | 双向同步 (Star ↔ SCM) | 两侧状态一致 | `integration_factory.bidirectional` |
|  | T-IG2 | Conflict 解决 | last-write-wins 或 manual | `integration_factory.conflict` |
|  | T-IG3 | Webhook 接收 + 签名 | 合法 webhook 处理 | `integration_factory.webhook_valid` |
| **domain-automation** | T-AM1 | Trigger 规则: Schedule/Cron (per §1 S1) | Event 路径不被触发 | `automation_factory.schedule` |
|  | T-AM2 | 条件 + 动作 | 条件满足执行 | `automation_factory.condition_action` |
|  | T-AM3 | 子队列隔离 (Event vs Schedule) | 互不干扰 | `automation_factory.isolation` |
| **domain-collaboration** | T-CB1 | Realtime Subscription | 多客户端同步 | `collab_factory.ws` |
|  | T-CB2 | 离线消息 | 重连后补发 | `collab_factory.offline` |
|  | T-CB3 | 协作冲突 (同字段并发) | 提示冲突 | `collab_factory.concurrent_edit` |
| **domain-local-runtime** | T-LR1 | 8 种白名单命令 (per §9.4 顺延) | 白名单内执行 | `local_runtime_factory.whitelisted` |
|  | T-LR2 | 禁 Shell 注入 | shell 字符 拒绝 | `local_runtime_factory.shell_inject` |
|  | T-LR3 | Device Identity 强制 | 缺 device_id 拒绝 | `local_runtime_factory.no_device` |
|  | T-LR4 | ExecuteArbitraryShell 拒绝 | 任意 shell 拒绝 | `local_runtime_factory.arbitrary_shell` |

**已知缺口** (per 守门 #11 缺标比错标):
- 25 Module 各 3-5 场景为最小覆盖, **真实数据接入完整场景**待 STAR-P3-WBS-001 §1 P0-1 (25 domain 真实数据接入 ~6M token 预算, W1-W5 软参考周)
- T-WT1/T-AG1/T-FB1 状态机场景假设状态名/转换规则已 frozen, 实际 spec 落地 per `docs/specs/domain-X-spec.md` (TBD per §0.1 §X 上游回填)

#### 7.2.2 5 域业务 シナリオ (per 守门 #3 真人 Lead 问责结构)

> **命名 disclaimer** (per §5 仓库拓扑 + 守门 #3): 5 域 (player/economy/match/social/admin) 是**历史治理命名**, 指 5 位真人 Lead 问责结构, **不等于** Star 仓 22 domain-* DDD bounded context, **不建立业务子域↔DDD 映射**。文档提到 "5 域" 时默认指真人 Lead 结构, 提到 "DDD bounded context" 时指 22 domain-* crate。

##### 7.2.2.1 player 域 (player Lead)

| # | シナリオ | 入力 | 期待結果 | テストデータ |
|---|---|---|---|---|
| T-PL-1 | 新玩家注册 + 默认 profile | tenant + email | 200 + profile_id | `player_factory.newbie()` |
| T-PL-2 | 跨 tenant 玩家数据隔离 (A tenant 玩家在 B tenant 不可见) | tenant_a.player_id | 404 NOT_FOUND | `player_factory.tenant_a + actor.tenant_b` |
| T-PL-3 | Player 软删除 + GDPR 删除请求 | delete request | soft delete + 30 天后 hard delete | `player_factory.gdpr` |
| T-PL-4 | player Lead 真人操作 (per 守门 #3) | actor=player Lead | 操作可执行 | `actor_factory(Lead, player)` |
| T-PL-5 | 非 player 域 Lead 越权 (per 8/21 JST 拒绝兼任) | actor=economy Lead | 拒绝 + audit | `actor_factory(Lead, economy) + player op` |

##### 7.2.2.2 economy 域 (economy Lead)

| # | シナリオ | 入力 | 期待結果 | テストデータ |
|---|---|---|---|---|
| T-EC-1 | 货币交易 (per Q-003 Saga) | debit + credit | 双方 ledger 更新 | `economy_factory.transaction` |
| T-EC-2 | 跨域 Saga 失败回滚 | step 3 失败 | 全部回滚 | `economy_factory.saga_rollback` |
| T-EC-3 | 货币上限检查 | amount > cap | 拒绝 + 错误码 | `economy_factory.over_cap` |
| T-EC-4 | economy Lead 真人 (per 守门 #3) | actor=economy Lead | 操作可执行 | `actor_factory(Lead, economy)` |
| T-EC-5 | Q-003 决策: economy Lead 独立拍板 (per §0.1 Q-003 跨域核心) | 跨域冲突 | economy 拍板 | `actor_factory(Lead, economy) + cross_domain` |

##### 7.2.2.3 match 域 (match Lead)

| # | シナリオ | 入力 | 期待結果 | テストデータ |
|---|---|---|---|---|
| T-MT-1 | 匹配请求 + 房间分配 | player_id | room_id 返回 | `match_factory.queue` |
| T-MT-2 | 匹配超时回退 | timeout | 状态回退 | `match_factory.timeout` |
| T-MT-3 | 跨域数据访问 (match → player) | match 域读 player | 仅可见必要字段 | `match_factory + player_factory` |
| T-MT-4 | match Lead 真人 (per 守门 #3) | actor=match Lead | 操作可执行 | `actor_factory(Lead, match)` |
| T-MT-5 | 5 域 Lead 互不兼任 (per 8/21 JST 拒绝兼任硬约束) | actor=player Lead 做 match 操作 | 拒绝 | `actor_factory(Lead, player) + match op` |

##### 7.2.2.4 social 域 (social Lead)

| # | シナリオ | 入力 | 期待結果 | テストデータ |
|---|---|---|---|---|
| T-SC-1 | 好友请求 + 双向确认 | requester + target | 双向 friend 关系 | `social_factory.friend_request` |
| T-SC-2 | 私聊消息 | sender + content | 消息持久化 | `social_factory.message` |
| T-SC-3 | 聊天 PII 脱敏 (per §7.3.4) | 含 email / phone | 脱敏后展示 | `social_factory.message_pii` |
| T-SC-4 | social Lead 真人 (per 守门 #3) | actor=social Lead | 操作可执行 | `actor_factory(Lead, social)` |
| T-SC-5 | 跨域消息审计 (social → audit) | 敏感操作 | audit 记录 | `social_factory + audit_factory` |

##### 7.2.2.5 admin 域 (admin Lead, 含 COC 属 admin 域独立控制面)

| # | シナリオ | 入力 | 期待結果 | テストデータ |
|---|---|---|---|---|
| T-AD-1 | COC 配置 (per 8/21 JST admin 域独立控制面) | config key-value | 200 + 持久化 | `admin_factory.coc_config` |
| T-AD-2 | 系统级操作 (全局查 / 强制下线) | admin actor | 200 + audit | `actor_factory(Lead, admin)` |
| T-AD-3 | SRE 兼任 admin 越权 (per 8/21 JST 拒绝 SRE 兼任 admin) | actor=SRE + admin op | 拒绝 | `actor_factory(SRE) + admin op` |
| T-AD-4 | admin Lead 真人 (per 守门 #3) | actor=admin Lead | 操作可执行 | `actor_factory(Lead, admin)` |
| T-AD-5 | admin 操作全部走强 MFA (per 8/21 JST 强约束) | admin op no MFA | 拒绝 | `actor_factory(Lead, admin) no_mfa` |

**已知缺口**:
- 5 域 Lead 真人到位后, 上表 5 域 Lead 场景的 actor 字段需真实身份 (per 守门 #3 DDD Review 阶段补)
- 5 域业务 mock 数据完整化已落地 commit `3dde2b4` + `b424611` (per §17 顺延 v0.5 实证)
- 5 域 disclaimer 必须在所有报表 label 显示 (per REQ-OPS-012, 见 04-ops-test-design-spec.md T-601 ~ T-605)

#### 7.2.3 跨域 Saga シナリオ (per Q-003 / §2.2.4)

> 跨域场景 (per 守门 #3 5 域独立 + 守门 #9 子代理授权):
> - **T-CROSS-1**: economy 域 + match 域 联合作战 (Q-003 跨域核心问题)
> - **T-CROSS-2**: player 域 + social 域 好友 + 私聊联调
> - **T-CROSS-3**: admin 域 + 全部域 全局配置下发
> - **T-CROSS-4**: NATS + PostgreSQL + Valkey 全容器集成 (per §2.2.4 SystemTestLevel)
>
> **已知缺口**: 跨域 Saga 场景的 actor 字段需 5 域 Lead 真人到位 (per 守门 #3 派生), 当前是 mock actor。
> 详见 Test-J.15 (§7.5)。

#### 7.2.4 6 E2E 关键流程 シナリオ (per §5.1)

> **Level**: AcceptanceTestLevel (Playwright)
> **Test Data**: `tests/e2e/fixtures/<flow>.spec.ts` 各自维护
> **5 域 disclaimer**: 6 流程跨 5 域业务, 流程级别不绑定单域

##### 7.2.4.1 流程 1: 从 WorkItem 创建 Worktree

| # | シナリオ | 入力 | 期待結果 | テストデータ |
|---|---|---|---|---|
| T-E2E-1-1 | User 打开 WorkItem 详情 | work_item_id | UI 渲染 + "Create Worktree" 按钮可见 | `e2e_fixtures.work_item_detail` |
| T-E2E-1-2 | 点击 "Create Worktree" + 选择 base branch | work_item_id + base | 跳转到 Worktree 详情页 + 状态 Creating | `e2e_fixtures.create_worktree_dialog` |
| T-E2E-1-3 | Worktree 创建完成 (SCM clone 完成) | poll status | 状态 → Ready | `e2e_fixtures.scm_clone_complete` |
| T-E2E-1-4 | 关联 ChangeSet 创建 | worktree_id | 0 个 ChangeSet (新建空) | `e2e_fixtures.empty_worktree` |
| T-E2E-1-5 | 跨租户越权创建 Worktree | actor.tenant_b + work_item.tenant_a | 403 + UI 报错 | `e2e_fixtures.cross_tenant` |

##### 7.2.4.2 流程 2: 分配 Worktree 给 Agent

| # | シナリオ | 入力 | 期待結果 | テストデータ |
|---|---|---|---|---|
| T-E2E-2-1 | User 选择 Agent 分配 | worktree_id + agent_id | worktree.assigned_agent_id 更新 | `e2e_fixtures.assign_agent` |
| T-E2E-2-2 | Agent 启动 + Lease 申请 | worktree_id | agent.status → AgentRunning + lease 写入 | `e2e_fixtures.agent_lease` |
| T-E2E-2-3 | Agent 5 域 Lead 越权 (per 守门 #3) | agent + other domain | 拒绝 + audit | `e2e_fixtures.cross_domain_agent` |
| T-E2E-2-4 | Heartbeat 超时回收 | no heartbeat 30s | agent.status → Idle | `e2e_fixtures.heartbeat_timeout` |
| T-E2E-2-5 | 5 域 Lead 真人指派 (per 守门 #3) | actor=Lead | 操作可执行 | `e2e_fixtures.lead_assign` |

##### 7.2.4.3 流程 3: Agent 修改后 Review + 提交 Feedback

| # | シナリオ | 入力 | 期待結果 | テストデータ |
|---|---|---|---|---|
| T-E2E-3-1 | Agent 修改文件 + 提交 ChangeSet | worktree_id + diff | changeset 写入 + symbol index 同步 | `e2e_fixtures.agent_change` |
| T-E2E-3-2 | Agent 提交 Feedback (5 段式) | feedback payload | feedback.status = Open + Provenance 完整 | `e2e_fixtures.feedback_5segment` |
| T-E2E-3-3 | 5 段式缺段拒绝 (per §7.2.1 T-FB2) | missing provenance | 拒绝 + reason 列出 | `e2e_fixtures.feedback_missing_provenance` |
| T-E2E-3-4 | Review 页面渲染 | feedback_id | UI 渲染 + 5 段展示 | `e2e_fixtures.review_page` |
| T-E2E-3-5 | Review 拒绝 + Agent 重新提交 | decision=reject | feedback → Rejected + Agent 重新 Open | `e2e_fixtures.reject_retry` |

##### 7.2.4.4 流程 4: 处理 Feedback Inbox (Resolve / Supersede)

| # | シナリオ | 入力 | 期待結果 | テストデータ |
|---|---|---|---|---|
| T-E2E-4-1 | Inbox 列表 + 过滤 | user_id + filter | 列表 + 计数 | `e2e_fixtures.inbox_list` |
| T-E2E-4-2 | Inbox 噪声抑制 (per §1 S2) | agent intermediate notification | 列表中**不**显示 | `e2e_fixtures.noise_suppress` |
| T-E2E-4-3 | Resolve Feedback | feedback_id + resolution | status → Resolving → Resolved | `e2e_fixtures.resolve` |
| T-E2E-4-4 | Supersede 旧版本 (per §7.2.1 T-FB3) | old + new | old 只读, new 独立生命周期 | `e2e_fixtures.supersede` |
| T-E2E-4-5 | 关键事件突破上限 (per §1 S2) | critical event | 即使超上限也展示 | `e2e_fixtures.critical_event` |

##### 7.2.4.5 流程 5: 处理 Conflict (Rebase / Merge)

| # | シナリオ | 入力 | 期待結果 | テストデータ |
|---|---|---|---|---|
| T-E2E-5-1 | Conflict 检测 | worktree_id + base | conflict field 写入 | `e2e_fixtures.conflict_detect` |
| T-E2E-5-2 | Rebase 操作 | worktree_id + target | rebase 完成 + conflict 列表 | `e2e_fixtures.rebase` |
| T-E2E-5-3 | Merge 操作 | worktree_id + target | merge commit + worktree 状态变化 | `e2e_fixtures.merge` |
| T-E2E-5-4 | 冲突未解决强行 merge | unresolved conflict | 拒绝 + UI 引导 | `e2e_fixtures.force_merge_blocked` |
| T-E2E-5-5 | Conflict 隔离分支 (per §7.2.1 T-WT2) | worktree + isolation | 隔离分支独立 | `e2e_fixtures.isolation_branch` |

##### 7.2.4.6 流程 6: Merge PR

| # | シナリオ | 入力 | 期待結果 | テストデータ |
|---|---|---|---|---|
| T-E2E-6-1 | PR 创建 | worktree + base | pr_url 返回 | `e2e_fixtures.pr_create` |
| T-E2E-6-2 | CI 触发 + 状态同步 | pr_id | ci_status 同步 | `e2e_fixtures.ci_trigger` |
| T-E2E-6-3 | VAL-001 四重门 (per §6.3.2) | is_ai_complete_claim | 4 门全部通过才接受 | `e2e_fixtures.val001_all_gates` |
| T-E2E-6-4 | AI 抢跑 (per §6.3.2 TC-VAL001-N8) | claim done + no approval | 拒绝 + 审计 | `e2e_fixtures.ai_self_complete` |
| T-E2E-6-5 | Merge 完成 + Worktree 清理 | pr merge | worktree 状态 → Merged + 资源清理 | `e2e_fixtures.pr_merged` |

#### 7.2.5 AC 跨引用 (per §6.2.1 / §6.3.3 / §6.3.4)

> §6 已包含 3 个 V1 Should-Have REQ 的场景表:
> - **VAL-001 四重门** (§6.3.2 TC-VAL001-P1 + N1~N8): 9 个场景
> - **DSG-001/002 Design Artifact Guard** (§6.3.3 5 行场景表): 5 个场景
> - **OPS-001/002/003 IncidentRecord** (§6.3.4 5 行场景表): 5 个场景
>
> **不重复列出**, 详见 §6 原文。

### 7.3 テストデータ 目录 (Test Data Catalogue)

> **本节目的**: 把 §10 测试数据管理详细策略聚焦到"场景 ↔ 数据"映射层。
> 详细策略 (Factory 模式 / Test DB 沙箱 / 脱敏 / Snapshot) 见 §10。

#### 7.3.1 Master Fixture 清单

| Fixture 文件 / 位置 | 用途 | 数据量 | 维护者 |
|---|---|---|---|
| `crates/domain-X/tests/fixtures/<name>.rs` | 25 domain Module 各自 fixture | < 1000 行 / file | domain owner |
| `frontend/src/mocks/schemas/<name>.ts` | 前端 mock schema (Zod) | 1 file / domain | frontend |
| `frontend/src/mocks/data/<name>.ts` | 前端 mock data | 10-100 rows | frontend |
| `frontend/src/mocks/handlers/<name>.ts` | MSW handler | 1 file / endpoint | frontend |
| `frontend/src/mocks/schemas/five-domain.ts` | 5 域业务 mock schema (per §7.2.2) | 243 行 / 6 type guard (per §17 顺延 v0.5 实证) | frontend |
| `frontend/src/mocks/data/five-domain.ts` | 5 域业务 mock data | 338 行 / 6 dataset (per §17 顺延 v0.5 实证) | frontend |
| `frontend/src/mocks/__tests__/uat/uat-test-data.ts` | UAT 测试数据 (已存在) | ~500 行 | frontend |
| `tests/e2e/fixtures/<flow>.spec.ts` | E2E 流程 fixture (per §7.2.4 6 流程) | < 1000 行 / file | e2e owner |
| `docs/ac-test-matrix.csv` | AC ↔ Test 矩阵 (auto-generated) | 35 行 = 1 header + 34 REQ (per §6.2 + §17 v0.5 §16.1) | auto |

#### 7.3.2 Seed 规则 (per §1.3 不污染生产 + §10 数据脱敏)

- **种子来源**: `factory_bot` / `faker` (Rust) + `@faker-js/faker` (TS) 随机生成
- **种子固定**: CI 环境用 `SEED=42` 固定, 本地用 `SEED=now()` 重生
- **PII 脱敏**: per §10 + Security Design §7.3
  - Email → `user_{N}@example.com`
  - 真实姓名 → `User {N}`
  - Token / Secret → 随机生成
  - Code Symbol 路径 → `src/file_{N}.rs`
  - 5 域业务数据: player_id / economy_id / match_id / social_id / admin_id 用 `domain_{N}` 前缀避免撞真实

#### 7.3.3 5 域 disclaimer 数据隔离 (per REQ-OPS-012 + 守门 #3)

- 5 域业务 mock 数据 (five-domain.ts) 与 22 DDD domain-* crate 数据**不混用**
- 报表 label / 测试报告 / UI banner 必显示 "5 域是历史治理命名" (per 04-ops-test-design-spec.md T-601~T-605)
- 测试报告**不**做业务子域↔DDD 映射 (per 守门 #3 派生)

#### 7.3.4 PII 脱敏映射 (per §10 + Security Design §7.3)

| PII 类型 | 脱敏规则 | 5 域场景 |
|---|---|---|
| Email | `user_{N}@example.com` | player 域 (`player_factory`) |
| 真实姓名 | `User {N}` | player / social 域 |
| Token / Secret | `secret_{uuid}` | 全部 5 域 |
| Code Symbol 路径 | `src/file_{N}.rs` | development 域 |
| IP / 设备指纹 | `10.0.0.{N}` / `device_{uuid}` | identity 域 |
| 经济数据 (per economy 域) | `amount_{random_usd}` | economy 域 |
| 聊天内容 (per social 域) | `message_{N}` | social 域 |

### 7.4 シナリオ ↔ テストデータ 可追溯矩阵

#### 7.4.1 主矩阵结构 (Schema)

| シナリオ # | 所属 REQ/Domain | テストデータ # | Fixture 文件 | Level |
|---|---|---|---|---|
| T-TN1 | domain-tenant | fixture: `tenant_factory.acme()` | `crates/domain-tenant/tests/fixtures/` | UnitTestLevel |
| T-WT1 | domain-worktree | fixture: `worktree_factory.creating()` | `crates/domain-worktree/tests/fixtures/` | UnitTestLevel |
| T-PL-1 | player 域 | fixture: `player_factory.newbie()` | `frontend/src/mocks/data/five-domain.ts` | IntegrationTestLevel |
| T-E2E-1-1 | E2E 流程 1 | fixture: `e2e_fixtures.work_item_detail` | `tests/e2e/fixtures/create-worktree.spec.ts` | AcceptanceTestLevel |
| T-VL4 | VAL-001 (per §6.3.2) | fixture: `validation_factory.all_gates_pass` | `crates/domain-validation/tests/fixtures/` | UnitTestLevel |
| ... (完整 ~200 行主矩阵由 §7.4.2 自动生成器产出) | | | | |

#### 7.4.2 矩阵自动生成 (per §6.2 `scripts/generate_ac_matrix.py`)

- 现有 `generate_ac_matrix.py` 已生成 `docs/ac-test-matrix.csv` (per §17 顺延 v0.5 §16.1 T1)
- **本节扩展**: matrix 增加 シナリオ + テストデータ 列 (per §7.1.1 Schema)
- **不变量保留**: §6.2 现行列结构 + AC 覆盖率公式 + E2E 路径全部不动, 矩阵按 RFC 增列
- **生成产物**: `docs/scenario-data-matrix.md` (TBD, 待 generate_scenario_matrix.py 落地, ~0.2M token)

### 7.5 已知缺口 + Test-J.14 ~ J.17 (per 守门 #11 缺标比错标)

- **Test-J.14**: 25 domain 真实数据接入 (per §7.2.1, 25 Module 3-5 场景为最小覆盖, **真实数据接入完整场景**待 STAR-P3-WBS-001 §1 P0-1 25 domain 真实数据接入, ~6M token 预算, W1-W5 软参考周)
- **Test-J.15**: 跨域 Saga 场景 (per §7.2.3, actor 字段需 5 域 Lead 真人到位 per 守门 #3 派生)
- **Test-J.16**: Level 维度落地后 §7.4 矩阵按 Level 判定 (per §6.2.1 + Test-J.11 关联, 等 RFC)
- **Test-J.17**: Prompts 实际模板 / Resources 独立资源类型 (per STAR-P3-WBS-001 §1 #4, 1.8M token 预算, W9-W10.5 软参考周)

**v0.6 已知缺口** (本节):
- §7.2.1 25 Module 场景为最小覆盖, 不含完整 17/14 状态机全路径 (per 守门 #11 缺标比错标, 全路径测试由各自 domain spec 负责, 不在 test-design 重复)
- §7.2.2 5 域业务场景的 actor 字段当前是 mock, 5 域 Lead 真人到位后需真实身份 (per 守门 #3 派生)
- §7.3.1 Master Fixture 清单为代表性样本, 不是穷举
- §7.4.1 主矩阵仅列 5 行示例, 完整 ~200 行的"全 scenario ↔ 全 fixture"映射由 `generate_ac_matrix.py` 自动生成 (per §7.4.2)

**不变量保留** (per 守门 #12 + 守门 #1):
- §6 验收测试现行文本 (§6.1 Gherkin + §6.2 AC 矩阵 + §6.3 验收门禁) 全部不动
- §10 测试数据管理详细策略不重复
- §14 (原 §13) Open Issues Test-J.1 ~ J.13 全部不动, 仅追加 J.14 ~ J.17
- §17 (原 §16) 代码跟进实证不重复新增, 5 域业务 mock 完整化 commit 引用为 `3dde2b4` + `b424611`
'''

V06_ROW = '''| v0.6 | 2026-09-01 | 新增 §7 シナリオ + テストデータ 章节 (per 2026-09-01 16:13 JST Ulysses 拍板 "加 §X 场景/数据章"); 覆盖 25 domain Module + 5 域业务 (per 守门 #3 历史治理命名) + 6 E2E 关键流程 + 3 AC 跨引用 (VAL-001/DSG-001/OPS-001); 原 §7-§16 顺位 §8-§17 同步 (修订历史 v0.1 ~ v0.5 中 §N 引用保留, 仅 body 内容 + 新增 §7 引用新编号); Test-J.14 ~ J.17 追加 (per 守门 #11 缺标比错标); 守门 #1 + #9 + #12 三过 (docs-only 改动, 无代码变更) | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 |
'''


def main():
    # Verify backup exists
    if not BACKUP.exists():
        raise RuntimeError(f"Backup not found: {BACKUP}. Restore manually first.")

    # Always start from backup to ensure idempotency
    content = BACKUP.read_text(encoding="utf-8")
    print(f"[load] loaded v0.5 from {BACKUP} ({len(content)} bytes)")

    # Step 1: Renumber text references (per-match context check)
    content = renumber_text_refs(content, debug=True)
    print("[step 1] text references renumbered (with external doc check)")

    # Step 2: Renumber section headers (top-level, sub-headers)
    for old, new in sorted(RENUMBER.items(), key=lambda x: -x[0]):
        # Top-level headers: ## N. (followed by space)
        content = re.sub(
            rf"^## {old}\. ",
            f"## {new}. ",
            content,
            flags=re.MULTILINE,
        )
        # Sub-headers: ### N.M or ### N.M.K (followed by space, dot, or end)
        content = re.sub(
            rf"^### {old}\.(\d+(?:\.\d+)*)(?=\s|$)",
            lambda m: f"### {new}.{m.group(1)}",
            content,
            flags=re.MULTILINE,
        )
        # Sub-sub headers: #### N.M
        content = re.sub(
            rf"^#### {old}\.(\d+(?:\.\d+)*)(?=\s|$)",
            lambda m: f"#### {new}.{m.group(1)}",
            content,
            flags=re.MULTILINE,
        )
    print("[step 2] section headers renumbered (including sub-headers)")

    # Step 3: Insert new §7 before current §8 (which was originally §7 性能测试)
    insert_anchor = "## 8. 性能测试(详细)"
    if insert_anchor not in content:
        raise RuntimeError(f"Could not find anchor '{insert_anchor}' for new §7 insertion")

    insertion = NEW_S7 + "\n---\n\n"
    content = content.replace(insert_anchor, insertion + insert_anchor, 1)
    print("[step 3] new §7 シナリオ + テストデータ inserted")

    # Step 4: Add v0.6 row to修订历史 (right after v0.5 row)
    v05_marker = "| v0.5 | 2026-08-31 | handoff 兜底分批 2:"
    if v05_marker not in content:
        raise RuntimeError("Could not find v0.5 row in修订历史 for v0.6 insertion")

    v05_end_pattern = r"(\| v0\.5 \| 2026-08-31 \| handoff 兜底分批 2:[^\n]*架构师 \(Mavis 接手 agent per DEC-008\) — Mavis 接手 \|)"
    m = re.search(v05_end_pattern, content, re.DOTALL)
    if not m:
        raise RuntimeError("Could not find v0.5 row end pattern")

    content = content[: m.end()] + "\n" + V06_ROW + content[m.end():]
    print("[step 4] v0.6 row added to修订历史")

    # Write back
    TEST_DESIGN.write_text(content, encoding="utf-8")
    new_size = len(content)
    line_count = content.count("\n") + 1
    print(f"[done] {TEST_DESIGN} updated to v0.6 ({new_size} bytes, {line_count} lines)")
    print(f"[stats] new §7 inserted, §7-§16 renumbered to §8-§17, v0.6 row added")


if __name__ == "__main__":
    main()
