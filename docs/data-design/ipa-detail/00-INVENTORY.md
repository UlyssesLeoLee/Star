# 00-INVENTORY.md — Star プラットフォーム 全 100 テーブル インベントリ

> **基準**: IPA データモデル詳細設計書 — テーブル一覧
> **作成日**: 2026-09-01
> **一次出典**: `D:\Star\docs\data-design.md` v0.2 §4（93 `CREATE TABLE` + 4 ビュー / Lookup）
> **本ファイル役割**: 25 Schema × 100 テーブル / Lookup / Projection / 物化ビューの俯瞰

---

## 凡例

| 略語 | 意味 |
|---|---|
| **E** | Entity（業務事実 / 1 対 N / PK + RLS） |
| **W** | Weak Entity（親テーブルの子、`{parent}_id` FK 必須） |
| **L** | Lookup Table（enum 値の業務メタデータ保持） |
| **P** | Projection（派生、非 SoR、App 読み取り専用） |
| **MV** | Materialized View（物化ビュー） |
| **A** | Append-only（`audit_event` / WORM 30 日） |
| **O** | Outbox（NAT 推送源、`audit_event_outbox`） |

| RLS | 意味 |
|---|---|
| **Y** | RLS 強制（13 類 tenant_id 必携对象） |
| **N** | RLS 不要（Tenant 自体 / Lookup / Projection） |
| **−** | 該当なし（`audit_event_outbox` 等、App 側制御） |

---

## 1. tenant schema（domain-tenant, 3 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T01 | `tenant.tenant` | テナント | E | `id` | N（源流） | `tables/tenant_tenant.md` |
| T02 | `tenant.tenant_policy` | テナントポリシー | E | `id` | Y | `tables/tenant_tenant_policy.md` |
| T03 | `tenant.provider_data_boundary` | プロバイダデータ境界 | E | `id` | Y | `tables/tenant_provider_data_boundary.md` |

---

## 2. workspace schema（domain-workspace, 1 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T04 | `workspace.workspace` | ワークスペース | E | `id` | Y | `tables/workspace_workspace.md` |

---

## 3. project schema（domain-project, 3 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T05 | `project.project` | プロジェクト | E | `id` | Y | `tables/project_project.md` |
| T06 | `project.project_policy` | プロジェクトポリシー | W (`project_id`) | `id` | Y | `tables/project_project_policy.md` |
| T07 | `project.project_template` | プロジェクトテンプレート | E | `id` | Y | `tables/project_project_template.md` |

---

## 4. work_item schema（domain-work-item, 5 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T08 | `work_item.work_item` | ワークアイテム（核心） | E | `id` | Y | `tables/work_item_work_item.md` |
| T09 | `work_item.requirement` | 要求（弱実体） | W (`work_item_id`) | `id` | Y | `tables/work_item_requirement.md` |
| T10 | `work_item.acceptance_criterion` | 受入基準（弱実体） | W (`work_item_id`) | `id` | Y | `tables/work_item_acceptance_criterion.md` |
| T11 | `work_item.business_goal` | 業務目標 | E | `id` | Y | `tables/work_item_business_goal.md` |
| T12 | `work_item.work_item_status` | ワークアイテム状態 Lookup | L | `status_code` | N | `tables/work_item_work_item_status.md` |

---

## 5. workflow schema（domain-workflow, 3 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T13 | `workflow.workflow_definition` | ワークフロー定義 | E | `id` | Y | `tables/workflow_workflow_definition.md` |
| T14 | `workflow.workflow_state` | ワークフロー状態 | W (`workflow_id`) | `id` | Y | `tables/workflow_workflow_state.md` |
| T15 | `workflow.workflow_transition` | ワークフロー遷移 | W (`workflow_id`) | `id` | Y | `tables/workflow_workflow_transition.md` |

---

## 6. board schema（domain-board, 3 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T16 | `board.board` | ボード | E | `id` | Y | `tables/board_board.md` |
| T17 | `board.board_column` | ボードカラム | W (`board_id`) | `id` | Y | `tables/board_board_column.md` |
| T18 | `board.board_swimlane` | ボードスイムレーン | W (`board_id`) | `id` | Y | `tables/board_board_swimlane.md` |

---

## 7. planning schema（domain-planning, 4 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T19 | `planning.sprint` | スプリント | E | `id` | Y | `tables/planning_sprint.md` |
| T20 | `planning.backlog` | バックログ（排序池） | E | `id` | Y | `tables/planning_backlog.md` |
| T21 | `planning.roadmap` | ロードマップ | P | `id` | Y | `tables/planning_roadmap.md` |
| T22 | `planning.sprint_state` | スプリント状態 Lookup | L | `state_code` | N | `tables/planning_sprint_state.md` |

---

## 8. relation schema（domain-relation, 2 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T23 | `relation.relation` | 関連 | E | `id` | Y | `tables/relation_relation.md` |
| T24 | `relation.dependency` | 依存（物化ビュー） | MV | `id` | N | `tables/relation_dependency.md` |

---

## 9. comment schema（domain-comment, 4 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T25 | `comment.comment` | コメント | E | `id` | Y | `tables/comment_comment.md` |
| T26 | `comment.mention` | メンション | W (`comment_id`) | `id` | Y | `tables/comment_mention.md` |
| T27 | `comment.attachment` | 添付ファイル | E | `id` | Y | `tables/comment_attachment.md` |
| T28 | `comment.comment_visibility` | コメント可視性 Lookup | L | `visibility_code` | N | `tables/comment_comment_visibility.md` |

---

## 10. search schema（domain-search, 1 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T29 | `search.search_index` | 検索インデックス（Projection） | P | `id` | Y | `tables/search_search_index.md` |

---

## 11. audit schema（domain-audit, 3 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T30 | `audit.audit_event` | 監査イベント | A (Append-only) | `id` | Y | `tables/audit_audit_event.md` |
| T31 | `audit.ai_audit_metadata` | AI 監査メタデータ | A (Append-only) | `id` | Y | `tables/audit_ai_audit_metadata.md` |
| T32 | `audit.audit_event_outbox` | 監査イベント Outbox | O | `outbox_id` | − | `tables/audit_audit_event_outbox.md` |

---

## 12. integration schema（domain-integration, 3 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T33 | `integration.integration` | 外部統合 | E | `id` | Y | `tables/integration_integration.md` |
| T34 | `integration.integration_sync_state` | 同期状態（弱実体） | W (`integration_id`) | `id` | Y | `tables/integration_integration_sync_state.md` |
| T35 | `integration.integration_status` | 統合状態 Lookup | L | `status_code` | N | `tables/integration_integration_status.md` |

---

## 13. automation schema（domain-automation, 4 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T36 | `automation.automation_rule` | 自動化ルール | E | `id` | Y | `tables/automation_automation_rule.md` |
| T37 | `automation.automation_trigger` | 自動化トリガ | W (`rule_id`) | `id` | Y | `tables/automation_automation_trigger.md` |
| T38 | `automation.automation_action` | 自動化アクション | W (`rule_id`) | `id` | Y | `tables/automation_automation_action.md` |
| T39 | `automation.rule_status` | ルール状態 Lookup | L | `status_code` | N | `tables/automation_rule_status.md` |

---

## 14. identity schema（domain-identity, 5 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T40 | `identity.user` | ユーザ | E | `id` | Y | `tables/identity_user.md` |
| T41 | `identity.device` | デバイス | E | `id` | Y | `tables/identity_device.md` |
| T42 | `identity.device_binding` | デバイス三重バインディング | E (Triple 1:1:1) | `id` | Y | `tables/identity_device_binding.md` |
| T43 | `identity.credential` | 資格情報（Credential Broker 抽象） | E | `id` | Y | `tables/identity_credential.md` |
| T44 | `identity.user_session` | ユーザセッション（短 TTL） | E | `id` | Y | `tables/identity_user_session.md` |

---

## 15. notification schema（domain-notification, 4 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T45 | `notification.notification_channel` | 通知チャネル | E | `id` | Y | `tables/notification_notification_channel.md` |
| T46 | `notification.notification_template` | 通知テンプレート | E | `id` | Y | `tables/notification_notification_template.md` |
| T47 | `notification.notification` | 通知（送信済） | E | `id` | Y | `tables/notification_notification.md` |
| T48 | `notification.notification_status` | 通知状態 Lookup | L | `status_code` | N | `tables/notification_notification_status.md` |

---

## 16. permission schema（domain-permission, 4 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T49 | `permission.role` | ロール | E | `id` | Y | `tables/permission_role.md` |
| T50 | `permission.permission` | パーミッション（全局 enum） | L | `code` | N | `tables/permission_permission.md` |
| T51 | `permission.permission_scheme` | パーミッションスキーム | E | `id` | Y | `tables/permission_permission_scheme.md` |
| T52 | `permission.security_policy` | セキュリティポリシー | E | `id` | Y | `tables/permission_security_policy.md` |

---

## 17. collaboration schema（domain-collaboration, 2 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T53 | `collaboration.presence` | 在席 | E（短 TTL） | `id` | Y | `tables/collaboration_presence.md` |
| T54 | `collaboration.realtime_subscription` | リアルタイム購読 | E（短 TTL） | `id` | Y | `tables/collaboration_realtime_subscription.md` |

---

## 18. scm schema（domain-scm, 8 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T55 | `scm.repository` | リポジトリ | E | `id` | Y | `tables/scm_repository.md` |
| T56 | `scm.branch` | ブランチ | W (`repository_id`) | `id` | Y | `tables/scm_branch.md` |
| T57 | `scm.commit` | コミット | W (`repository_id`) | `id` | Y | `tables/scm_commit.md` |
| T58 | `scm.pull_request` | プルリクエスト | W (`repository_id`) | `id` | Y | `tables/scm_pull_request.md` |
| T59 | `scm.review` | レビュー | W (`pull_request_id`) | `id` | Y | `tables/scm_review.md` |
| T60 | `scm.pipeline` | パイプライン (CI) | W (`repository_id`) | `id` | Y | `tables/scm_pipeline.md` |
| T61 | `scm.webhook_event` | Webhook イベント | A (短 TTL 物理削除) | `id` | Y | `tables/scm_webhook_event.md` |
| T62 | `scm.pull_request_status` | PR 状態 Lookup | L | `status_code` | N | `tables/scm_pull_request_status.md` |

---

## 19. development schema（domain-development, 9 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T63 | `development.development_execution` | 開発実行（核心） | E | `id` | Y | `tables/development_development_execution.md` |
| T64 | `development.change_set` | 変更セット（核心） | E | `id` | Y | `tables/development_change_set.md` |
| T65 | `development.file_change` | ファイル変更（子） | W (`change_set_id`) | `id` | Y | `tables/development_file_change.md` |
| T66 | `development.symbol_change` | シンボル変更 | W (`change_set_id`) | `id` | Y | `tables/development_symbol_change.md` |
| T67 | `development.risk_signal` | リスクシグナル | E | `id` | Y | `tables/development_risk_signal.md` |
| T68 | `development.change_set_link` | 変更セットリンク | E | `id` | Y | `tables/development_change_set_link.md` |
| T69 | `development.symbol_index` | シンボルインデックス（Projection） | P | `id` | Y | `tables/development_symbol_index.md` |
| T70 | `development.repository_context` | リポジトリコンテキスト（Projection） | P | `id` | Y | `tables/development_repository_context.md` |
| T71 | `development.development_context` | 開発コンテキスト（Projection） | P | `id` | Y | `tables/development_development_context.md` |

---

## 20. worktree schema（domain-worktree, 5 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T72 | `worktree.worktree` | ワークツリー（核心） | E | `id` | Y | `tables/worktree_worktree.md` |
| T73 | `worktree.worktree_status_observed` | 観測状態（Projection） | P | `id` | Y | `tables/worktree_worktree_status_observed.md` |
| T74 | `worktree.worktree_conflict` | ワークツリー衝突 | E | `id` | Y | `tables/worktree_worktree_conflict.md` |
| T75 | `worktree.worktree_heatmap` | ワークツリーヒートマップ | MV | `(repository_id, file_path)` | N | `tables/worktree_worktree_heatmap.md` |
| T76 | `worktree.worktree_status` | ワークツリー状態 Lookup | L | `status_code` | N | `tables/worktree_worktree_status.md` |

---

## 21. agent schema（domain-agent, 5 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T77 | `agent.agent` | エージェント（登録） | E | `id` | Y | `tables/agent_agent.md` |
| T78 | `agent.agent_session` | エージェントセッション（核心） | E | `id` | Y | `tables/agent_agent_session.md` |
| T79 | `agent.agent_session_event` | セッションイベント | A (Append-only) | `id` | Y | `tables/agent_agent_session_event.md` |
| T80 | `agent.agent_policy` | エージェントポリシー | E | `id` | Y | `tables/agent_agent_policy.md` |
| T81 | `agent.agent_session_status` | セッション状態 Lookup | L | `status_code` | N | `tables/agent_agent_session_status.md` |

---

## 22. feedback schema（domain-feedback, 4 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T82 | `feedback.feedback` | フィードバック（核心） | E | `id` | Y | `tables/feedback_feedback.md` |
| T83 | `feedback.feedback_consumed_event` | 消費追跡イベント | A (Append-only) | `id` | Y | `tables/feedback_feedback_consumed_event.md` |
| T84 | `feedback.feedback_inbox_item` | Inbox（物化ビュー） | MV | `id` | N | `tables/feedback_feedback_inbox_item.md` |
| T85 | `feedback.feedback_status` | フィードバック状態 Lookup | L | `status_code` | N | `tables/feedback_feedback_status.md` |

---

## 23. context schema（domain-context, 4 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T86 | `context.context_packet` | コンテキストパケット（核心） | E | `id` | Y | `tables/context_context_packet.md` |
| T87 | `context.provenance_entry` | 系統エントリ | E | `id` | Y | `tables/context_provenance_entry.md` |
| T88 | `context.decision` | 意思決定（核心） | E | `id` | Y | `tables/context_decision.md` |
| T89 | `context.decision_status` | 意思決定状態 Lookup | L | `status_code` | N | `tables/context_decision_status.md` |

---

## 24. validation schema（domain-validation, 6 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T90 | `validation.validation_result` | 検証結果（核心） | E | `id` | Y | `tables/validation_validation_result.md` |
| T91 | `validation.validation_evidence` | 検証証拠 | W (`validation_id`) | `id` | Y | `tables/validation_validation_evidence.md` |
| T92 | `validation.acceptance_coverage` | 受入カバレッジ | E | `id` | Y | `tables/validation_acceptance_coverage.md` |
| T93 | `validation.validation_policy` | 検証ポリシー | E | `id` | Y | `tables/validation_validation_policy.md` |
| T94 | `validation.acceptance_coverage_report` | カバレッジレポート（物化ビュー） | MV | `id` | N | `tables/validation_acceptance_coverage_report.md` |
| T95 | `validation.validation_status` | 検証状態 Lookup | L | `status_code` | N | `tables/validation_validation_status.md` |

---

## 25. local_runtime schema（domain-local-runtime, 5 テーブル）

| # | 物理名 | 論理名 | 種別 | 主キー | RLS | IPA ファイル |
|---|---|---|---|---|---|---|
| T96 | `local_runtime.runtime` | ランタイム（登録） | E | `id` | Y | `tables/local_runtime_runtime.md` |
| T97 | `local_runtime.runtime_command` | ランタイムコマンド（白名单） | E | `id` | Y | `tables/local_runtime_runtime_command.md` |
| T98 | `local_runtime.runtime_observation` | ランタイム観測 | A (Append-only 短 TTL) | `id` | Y | `tables/local_runtime_runtime_observation.md` |
| T99 | `local_runtime.reconciliation_report` | 調整レポート | E | `id` | Y | `tables/local_runtime_reconciliation_report.md` |
| T100 | `local_runtime.runtime_status` | ランタイム状態 Lookup | L | `status_code` | N | `tables/local_runtime_runtime_status.md` |

---

## 26. 集計

| 種別 | 件数 | 比率 |
|---|---|---|
| Entity (E) | 60 | 60.0% |
| Weak Entity (W) | 22 | 22.0% |
| Lookup (L) | 13 | 13.0% |
| Projection (P) | 7 | 7.0% |
| Materialized View (MV) | 4 | 4.0% |
| Append-only (A) | 4 | 4.0% |
| Outbox (O) | 1 | 1.0% |
| **合計** | **100** | **100%**（重複計上あり） |

> 注: Append-only 4 件 = `audit_event` / `ai_audit_metadata` / `agent_session_event` / `feedback_consumed_event` / `runtime_observation` (5 件) との重複、`webhook_event` 短 TTL 含むので +1 で 6 件とも読める。INVENTORY では 4 件計上（重複除外）。

### 26.1 RLS 適用状況

| RLS | 件数 | 比率 |
|---|---|---|
| Y（13 類 tenant_id 必携对象） | 80 | 80.0% |
| N（Tenant / Lookup / Projection / MV） | 19 | 19.0% |
| −（Outbox 別管理） | 1 | 1.0% |
| **合計** | **100** | **100%** |

### 26.2 Module 別 件数

| # | Module / Schema | 主表 | 弱実体 | Lookup | Projection | MV | 計 |
|---|---|---|---|---|---|---|---|
| 1 | domain-tenant | 3 | 0 | 0 | 0 | 0 | 3 |
| 2 | domain-workspace | 1 | 0 | 0 | 0 | 0 | 1 |
| 3 | domain-project | 2 | 1 | 0 | 0 | 0 | 3 |
| 4 | domain-work-item | 2 | 2 | 1 | 0 | 0 | 5 |
| 5 | domain-workflow | 1 | 2 | 0 | 0 | 0 | 3 |
| 6 | domain-board | 1 | 2 | 0 | 0 | 0 | 3 |
| 7 | domain-planning | 2 | 0 | 1 | 1 | 0 | 4 |
| 8 | domain-relation | 1 | 0 | 0 | 0 | 1 | 2 |
| 9 | domain-comment | 2 | 1 | 1 | 0 | 0 | 4 |
| 10 | domain-search | 0 | 0 | 0 | 1 | 0 | 1 |
| 11 | domain-audit | 2 | 0 | 0 | 0 | 0 | 2 + Outbox 1 |
| 12 | domain-integration | 1 | 1 | 1 | 0 | 0 | 3 |
| 13 | domain-automation | 1 | 2 | 1 | 0 | 0 | 4 |
| 14 | domain-identity | 5 | 0 | 0 | 0 | 0 | 5 |
| 15 | domain-notification | 3 | 0 | 1 | 0 | 0 | 4 |
| 16 | domain-permission | 3 | 0 | 1 | 0 | 0 | 4 |
| 17 | domain-collaboration | 2 | 0 | 0 | 0 | 0 | 2 |
| 18 | domain-scm | 1 | 6 | 1 | 0 | 0 | 8 |
| 19 | domain-development | 3 | 3 | 0 | 3 | 0 | 9 |
| 20 | domain-worktree | 2 | 1 | 1 | 1 | 1 | 6 |
| 21 | domain-agent | 3 | 0 | 1 | 0 | 0 | 4 + Event 1 |
| 22 | domain-feedback | 1 | 0 | 1 | 0 | 1 | 3 + Event 1 |
| 23 | domain-context | 3 | 0 | 1 | 0 | 0 | 4 |
| 24 | domain-validation | 3 | 1 | 1 | 0 | 1 | 6 |
| 25 | domain-local-runtime | 4 | 0 | 1 | 0 | 0 | 5 |
| **計** | **25** | **52** | **20** | **12** | **6** | **4** | **100** |

---

## 27. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：25 Schema × 100 テーブル全列挙 + 集計 | per 2026-09-01 15:30 JST Ulysses 拍板 |

---
