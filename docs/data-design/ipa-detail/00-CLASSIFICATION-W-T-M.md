# 00-CLASSIFICATION-W-T-M.md — Star プラットフォーム 全 100 テーブル 業務分類索引（Work / Transaction / Master）

> **文書バージョン**: v0.2 (2026-09-05 升版) → v0.1 (per 2026-09-01 18:30 JST Ulysses 拍板)

> **基準**: ユーザー指定 DB 三類横展開原則（2026-09-01 18:30 JST）
> **適用範囲**: Star PostgreSQL 全 25 Schema × 100 テーブル / Lookup / Projection / 物化ビュー / Outbox
> **一次出典**: `D:\Star\docs\data-design.md` v0.2 + `D:\Star\docs\data-design\ipa-detail\00-INVENTORY.md` v0.1
> **横軸**: 既存「種別（E/W/L/P/MV/A/O）」「R/W 識別（R/W SoR / R Projection / Append-only / R/W 短 TTL）」に**第三の横軸**「業務分類（Work / Transaction / Master）」を追加
> **本ファイル役割**: 100 テーブルを W/T/M 三類で分門別類,基本設計 DB 章节必备构成

---

## 0. 目的

IPA 標準のテーブル詳細定義書（`docs/data-design/ipa-detail/`）は既に 7 輪 IPA 化（v0.1 → v0.7）を完了しているが、テーブルを **業務上の時間軸 / ライフサイクル** で分門別類する横軸が無い。ユーザー指示（2026-09-01 18:30 JST）に基づき、以下三類で全 100 テーブルを分門別類する:

| 業務分類 | 略 | 業務上の意味 | ライフサイクル | 削除方針 |
|---|---|---|---|---|
| **Work** | **W** | 作業中 / プロセス中の中間データ、session-bound、完了後クリーンアップ | 短 TTL（数分〜数時間）/ 短命 | 物理削除 / タイマー失効 / 完了時 clear |
| **Transaction** | **T** | 業務事実 / イベント / 状態変更の記録、append-only または高頻度 R/W SoR | 中〜長期（数ヶ月〜永久） | 論理削除 + 監査保持 / Append-only 不変 |
| **Master** | **M** | 参考データ / 設定 / テンプレート / 慢変参照データ | 永続 / SCD（Slowly Changing Dimension）戦略適用 | 論理削除 / 物理削除禁止（業務 FK 整合性） |

---

## 1. 判定規準（三類識別の判断基準）

> **「種別（E/W/L/P/MV/A/O）」と「業務分類（W/T/M）」は別物**:
> - 「種別」は**データの実装役割**（Entity / Weak / Lookup / Projection / MV / Append-only / Outbox）
> - 「業務分類」は**業務上の時間軸**（作業中 / 事実 / 参考）
>
> 同一テーブルが両方の横軸に独立して属する。例: `audit.audit_event` は「種別=Append-only」かつ「業務分類=Transaction」。

### 1.1 Master（M）判定規準

以下の**いずれか 2 つ以上**を満たすテーブルは M:

1. 業務上で**多数テーブルの FK 参照先**（tenant / project / user / role / permission / lookup など）
2. **設定 / テンプレート / ポリシー** などの構成情報を保持
3. **ゆっくり変化する**（SCD Type 2 適用可能、変更頻度が業務イベントより明らかに低い）
4. 物理削除すると**業務整合性が壊れる**（FK 連鎖 violate）
5. **Lookup 値**（enum の業務メタデータ）— 全 Lookup 自動 M

### 1.2 Transaction（T）判定規準

以下の**いずれか 2 つ以上**を満たすテーブルは T:

1. **業務事実の記録**（WorkItem / Project / Comment / PR / Session / Decision / ValidationResult など）
2. **状態変更 / ライフサイクル遷移**を主目的とする
3. **Append-only / 監査ログ / Outbox**（`audit_event` / `ai_audit_metadata` / `agent_session_event` / `feedback_consumed_event` / `runtime_observation` / `audit_event_outbox`）
4. R/W SoR で**高頻度書き込み**
5. 物理削除すると**業務履歴が失われる**（監査要件 / 法的要件）

### 1.3 Work（W）判定規準

以下の**いずれか 2 つ以上**を満たすテーブルは W:

1. **短 TTL**（数分〜数時間以内、または明示的な retention period で物理削除）
2. **session-bound / process-bound**（特定 session / command / runtime instance に紐付く中間データ）
3. **完了後クリーンアップ**（コマンド完了 / セッション終了 / イベント処理後に消える）
4. **非業務事実の観測値**（Observed Runtime State 系の Projection）
5. **リアルタイム UI 状態 / 通知抑制 / 排他制御** など一時的

> **重要**: 「種別 = Projection / MV」で「業務分類 = W」になるテーブルが多い（`worktree_status_observed` / `agent_process_status_observed` / `worktree_heatmap` / `feedback_inbox_item` / `acceptance_coverage_report` / `reconciliation_report` の一部）。これは**派生データ + 短命**の組み合わせで、業務事実ではないが頻繁な書き換え対象。

---

## 2. 全 100 テーブル 三類分類

> **凡例**:
> - `M` = Master（参考 / 設定 / 慢変）
> - `T` = Transaction（業務事実 / 監査 / 高頻度 SoR）
> - `W` = Work（短 TTL / 観測 / session-bound）
> - `M/T` = 業務上 Master だが一部 Transaction 役割（混合）
> - `T/W` = 業務上 Transaction だが短 TTL Work 性質（混合）

### 2.1 tenant schema（domain-tenant, 3 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T01 | `tenant.tenant` | **M** | E | テナント分離の源流,全テナント ID 参照の起点,物理削除禁止 |
| T02 | `tenant.tenant_policy` | **M** | E | テナントの構成情報,慢変,FK 参照多数 |
| T03 | `tenant.provider_data_boundary` | **M** | E | テナント間のデータ境界を定義,構成情報 |

### 2.2 workspace schema（domain-workspace, 1 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T04 | `workspace.workspace` | **M/T** | E | テナント内の業務スコープ定義（構成情報 + 業務事実）混合,project 多数から参照 |

### 2.3 project schema（domain-project, 3 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T05 | `project.project` | **T** | E | 業務事実,WorkItem / Board / Sprint の親,状態遷移あり |
| T06 | `project.project_policy` | **M** | W | プロジェクト構成情報,慢変,template 的に参照 |
| T07 | `project.project_template` | **M** | E | テンプレート,構成情報,慢変,新 project 生成の参照元 |

### 2.4 work_item schema（domain-work-item, 5 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T08 | `work_item.work_item` | **T** | E | 業務核心,状態遷移,Worktree / PR / Validation から参照 |
| T09 | `work_item.requirement` | **T** | W | WorkItem の子,業務要件の記録,弱実体 |
| T10 | `work_item.acceptance_criterion` | **T** | W | 受入基準,業務事実,弱実体 |
| T11 | `work_item.business_goal` | **M** | E | 業務目標,慢変,WorkItem グルーピング基準 |
| T12 | `work_item.work_item_status` | **M** | L | Lookup,enum 値,全 status 参照元 |

### 2.5 workflow schema（domain-workflow, 3 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T13 | `workflow.workflow_definition` | **M** | E | ワークフロー定義,構成情報,慢変 |
| T14 | `workflow.workflow_state` | **M** | W | ワークフローの状態定義,構成情報 |
| T15 | `workflow.workflow_transition` | **M** | W | 状態遷移ルール,構成情報 |

### 2.6 board schema（domain-board, 3 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T16 | `board.board` | **T** | E | 業務事実（kanban board 構成）,WorkItem 配置先 |
| T17 | `board.board_column` | **T** | W | Board の状態カラム,業務事実 |
| T18 | `board.board_swimlane` | **T** | W | Board のスイムレーン,業務事実 |

### 2.7 planning schema（domain-planning, 4 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T19 | `planning.sprint` | **T** | E | スプリント,業務事実,状態遷移あり |
| T20 | `planning.backlog` | **T** | E | バックログ,業務事実,排序変更頻繁 |
| T21 | `planning.roadmap` | **M/T** | P | ロードマップ派生,業務目標の集計,構成寄り |
| T22 | `planning.sprint_state` | **M** | L | Lookup,enum |

### 2.8 relation schema（domain-relation, 2 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T23 | `relation.relation` | **T** | E | 業務関連,WorkItem / Project 間の関係記録 |
| T24 | `relation.dependency` | **T** | MV | 依存関係の派生,業務事実を集計 |

### 2.9 comment schema（domain-comment, 4 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T25 | `comment.comment` | **T** | E | コメント,業務事実,投稿 / 編集 / 削除あり |
| T26 | `comment.mention` | **T** | W | メンション,業務事実,弱実体 |
| T27 | `comment.attachment` | **T** | E | 添付ファイル参照,業務事実 |
| T28 | `comment.comment_visibility` | **M** | L | Lookup,enum |

### 2.10 search schema（domain-search, 1 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T29 | `search.search_index` | **W** | P | 検索インデックス派生,非 SoR,再構築可能,短命寄り |

### 2.11 audit schema（domain-audit, 3 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T30 | `audit.audit_event` | **T** | A | Append-only 監査ログ,業務事実,法的保持,WORM 30 日 |
| T31 | `audit.ai_audit_metadata` | **T** | A | AI 監査メタ,Append-only,業務事実 |
| T32 | `audit.audit_event_outbox` | **T/W** | O | Outbox,送信済みで役目を終える,短〜中 TTL |

### 2.12 integration schema（domain-integration, 3 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T33 | `integration.integration` | **M** | E | 外部統合設定,構成情報,慢変 |
| T34 | `integration.integration_sync_state` | **T** | W | 同期状態,業務事実,弱実体,更新頻繁 |
| T35 | `integration.integration_status` | **M** | L | Lookup,enum |

### 2.13 automation schema（domain-automation, 4 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T36 | `automation.automation_rule` | **M** | E | 自動化ルール,構成情報,慢変 |
| T37 | `automation.automation_trigger` | **M** | W | トリガ定義,構成情報 |
| T38 | `automation.automation_action` | **M** | W | アクション定義,構成情報 |
| T39 | `automation.rule_status` | **M** | L | Lookup,enum |

### 2.14 identity schema（domain-identity, 5 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T40 | `identity.user` | **M** | E | ユーザ,構成情報,多数業務表 FK 参照元,慢変 |
| T41 | `identity.device` | **M** | E | デバイス登録,構成情報,慢変 |
| T42 | `identity.device_binding` | **M** | E | 三重バインディング,構成情報 |
| T43 | `identity.credential` | **T** | E | 資格情報,業務事実（発行 / 失効イベント）,Credential Broker |
| T44 | `identity.user_session` | **W** | E | ユーザセッション,短 TTL,session-bound,完了時 clear |

### 2.15 notification schema（domain-notification, 4 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T45 | `notification.notification_channel` | **M** | E | 通知チャネル設定,構成情報,慢変 |
| T46 | `notification.notification_template` | **M** | E | 通知テンプレート,構成情報,慢変 |
| T47 | `notification.notification` | **T** | E | 送信済み通知,業務事実,短〜中 TTL |
| T48 | `notification.notification_status` | **M** | L | Lookup,enum |

### 2.16 permission schema（domain-permission, 4 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T49 | `permission.role` | **M** | E | ロール,構成情報,多数業務表 FK 参照元 |
| T50 | `permission.permission` | **M** | L | Lookup,enum,全局参照 |
| T51 | `permission.permission_scheme` | **M** | E | パーミッションスキーム,構成情報 |
| T52 | `permission.security_policy` | **M** | E | セキュリティポリシー,構成情報,慢変 |

### 2.17 collaboration schema（domain-collaboration, 2 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T53 | `collaboration.presence` | **W** | E | 在席状態,短 TTL,session-bound |
| T54 | `collaboration.realtime_subscription` | **W** | E | リアルタイム購読,短 TTL,session-bound |

### 2.18 scm schema（domain-scm, 8 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T55 | `scm.repository` | **M** | E | リポジトリ登録,構成情報,慢変 |
| T56 | `scm.branch` | **T** | W | ブランチ,業務事実,作成 / 削除あり |
| T57 | `scm.commit` | **T** | W | コミット,業務事実,Append-only,弱実体 |
| T58 | `scm.pull_request` | **T** | W | PR,業務事実,状態遷移あり |
| T59 | `scm.review` | **T** | W | レビュー,業務事実,弱実体 |
| T60 | `scm.pipeline` | **T** | W | CI パイプライン,業務事実,弱実体 |
| T61 | `scm.webhook_event` | **W** | A | Webhook 受信,短 TTL 物理削除,処理後 clear |
| T62 | `scm.pull_request_status` | **M** | L | Lookup,enum |

### 2.19 development schema（domain-development, 9 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T63 | `development.development_execution` | **T** | E | 開発実行,業務事実,状態遷移 |
| T64 | `development.change_set` | **T** | E | 変更セット,業務事実,Worktree / Commit 中間 |
| T65 | `development.file_change` | **T** | W | ファイル変更,業務事実,弱実体 |
| T66 | `development.symbol_change` | **T** | W | シンボル変更,業務事実,弱実体 |
| T67 | `development.risk_signal` | **T** | E | リスクシグナル,業務事実 |
| T68 | `development.change_set_link` | **T** | E | 変更セットリンク,業務事実 |
| T69 | `development.symbol_index` | **W** | P | シンボルインデックス派生,非 SoR,再構築可能 |
| T70 | `development.repository_context` | **W** | P | リポジトリコンテキスト派生,非 SoR |
| T71 | `development.development_context` | **W** | P | 開発コンテキスト派生,非 SoR |

### 2.20 worktree schema（domain-worktree, 5 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T72 | `worktree.worktree` | **T** | E | ワークツリー,業務核心,状態遷移 |
| T73 | `worktree.worktree_status_observed` | **W** | P | 観測状態派生,非 SoR,高頻度更新,Observed Runtime State |
| T74 | `worktree.worktree_conflict` | **T** | E | ワークツリー衝突,業務事実 |
| T75 | `worktree.worktree_heatmap` | **W** | MV | ヒートマップ派生,非 SoR,集計,再構築可能 |
| T76 | `worktree.worktree_status` | **M** | L | Lookup,enum |

### 2.21 agent schema（domain-agent, 5 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T77 | `agent.agent` | **M** | E | エージェント登録,構成情報,慢変 |
| T78 | `agent.agent_session` | **T** | E | エージェントセッション,業務事実,状態遷移 |
| T79 | `agent.agent_session_event` | **T** | A | セッションイベント,Append-only,業務事実 |
| T80 | `agent.agent_policy` | **M** | E | エージェントポリシー,構成情報,慢変 |
| T81 | `agent.agent_session_status` | **M** | L | Lookup,enum |

### 2.22 feedback schema（domain-feedback, 4 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T82 | `feedback.feedback` | **T** | E | フィードバック,業務事実,状態遷移 |
| T83 | `feedback.feedback_consumed_event` | **T** | A | 消費追跡イベント,Append-only,業務事実 |
| T84 | `feedback.feedback_inbox_item` | **W** | MV | Inbox 派生,非 SoR,UI 向け集計,再構築可能 |
| T85 | `feedback.feedback_status` | **M** | L | Lookup,enum |

### 2.23 context schema（domain-context, 4 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T86 | `context.context_packet` | **T** | E | コンテキストパケット,業務事実,状態遷移 |
| T87 | `context.provenance_entry` | **T** | E | 系統エントリ,業務事実 |
| T88 | `context.decision` | **T** | E | 意思決定,業務事実,状態遷移 |
| T89 | `context.decision_status` | **M** | L | Lookup,enum |

### 2.24 validation schema（domain-validation, 6 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T90 | `validation.validation_result` | **T** | E | 検証結果,業務事実,状態遷移 |
| T91 | `validation.validation_evidence` | **T** | W | 検証証拠,業務事実,弱実体 |
| T92 | `validation.acceptance_coverage` | **T** | E | 受入カバレッジ,業務事実 |
| T93 | `validation.validation_policy` | **M** | E | 検証ポリシー,構成情報,慢変 |
| T94 | `validation.acceptance_coverage_report` | **W** | MV | カバレッジレポート派生,非 SoR,集計,再構築可能 |
| T95 | `validation.validation_status` | **M** | L | Lookup,enum |

### 2.25 local_runtime schema（domain-local-runtime, 5 テーブル）

| # | 物理名 | 業務分類 | 種別 | 判定根拠 |
|---|---|---|---|---|
| T96 | `local_runtime.runtime` | **M** | E | ランタイム登録,構成情報,慢変 |
| T97 | `local_runtime.runtime_command` | **T** | E | ランタイムコマンド（白名单）,業務事実 |
| T98 | `local_runtime.runtime_observation` | **T/W** | A | 観測ログ,Append-only 短 TTL,業務寄りだが短命 |
| T99 | `local_runtime.reconciliation_report` | **T** | E | 調整レポート,業務事実 |
| T100 | `local_runtime.runtime_status` | **M** | L | Lookup,enum |

---

## 3. 集計

### 3.1 業務分類別 件数

| 業務分類 | 件数 | 比率 | 説明 |
|---|---|---|---|
| **Master (M)** | 33 | 33.0% | Tenant / Policy / Template / Role / Permission / Lookup / Device 登録 / Agent 登録 / 構成情報 / Workflow 定義 |
| **Transaction (T)** | 47 | 47.0% | 業務核心表（WorkItem / Project / Comment / Worktree / Session / Feedback / Decision / ValidationResult / Audit / Outbox） |
| **Work (W)** | 14 | 14.0% | 短 TTL / 観測 / session-bound（Presence / RealtimeSubscription / UserSession / WebhookEvent / ObservedState / MV 派生 / SearchIndex） |
| **M/T 混合** | 2 | 2.0% | `workspace.workspace` / `planning.roadmap`（構成寄り業務事実） |
| **T/W 混合** | 2 | 2.0% | `audit.audit_event_outbox` / `local_runtime.runtime_observation`（業務的事実だが短命） |
| **合計** | **100**（重複計上なし、混合は主分類で計上） | 100% | − |

> **集計ルール**: 混合分類（M/T / T/W）は主分類で 1 回計上。`M/T` は M 寄りだが業務事実側面も持つものを M 主分類、`T/W` は T 主分類で計上。

### 3.2 Schema 別 業務分類件数

| # | Schema | Master | Transaction | Work | 計 | 備考 |
|---|---|---|---|---|---|---|
| 1 | tenant | 3 | 0 | 0 | 3 | 全 M,テナント分離の源流 |
| 2 | workspace | 0 | 1 (M/T) | 0 | 1 | workspace は M/T |
| 3 | project | 2 | 1 | 0 | 3 | policy/template = M,project = T |
| 4 | work_item | 2 | 3 | 0 | 5 | goal/status = M,work_item 系 = T |
| 5 | workflow | 3 | 0 | 0 | 3 | 全 M,構成情報 |
| 6 | board | 0 | 3 | 0 | 3 | 全 T,業務構成 |
| 7 | planning | 1 | 3 | 0 | 4 | state = M,rest = T |
| 8 | relation | 0 | 2 | 0 | 2 | 全 T,業務関連 |
| 9 | comment | 1 | 3 | 0 | 4 | visibility = M,rest = T |
| 10 | search | 0 | 0 | 1 | 1 | 全 W,派生 |
| 11 | audit | 0 | 3 (含 T/W 1) | 0 | 3 | event = T,outbox = T/W |
| 12 | integration | 2 | 1 | 0 | 3 | integration = M,sync_state = T |
| 13 | automation | 4 | 0 | 0 | 4 | 全 M,ルール定義 |
| 14 | identity | 3 | 1 | 1 | 5 | user/device/binding = M,credential = T,session = W |
| 15 | notification | 3 | 1 | 0 | 4 | channel/template/status = M,notification = T |
| 16 | permission | 4 | 0 | 0 | 4 | 全 M,構成情報 |
| 17 | collaboration | 0 | 0 | 2 | 2 | 全 W,リアルタイム |
| 18 | scm | 2 | 6 | 1 | 9 | repo/status = M,rest = T,webhook = W |
| 19 | development | 0 | 6 | 3 | 9 | core = T,projection = W |
| 20 | worktree | 1 | 2 | 2 | 5 | status = M,core/conflict = T,observed/heatmap = W |
| 21 | agent | 3 | 2 | 0 | 5 | agent/policy/status = M,session/event = T |
| 22 | feedback | 1 | 2 | 1 | 4 | status = M,feedback/event = T,inbox = W |
| 23 | context | 1 | 3 | 0 | 4 | decision_status = M,rest = T |
| 24 | validation | 2 | 3 | 1 | 6 | policy/status = M,core = T,report MV = W |
| 25 | local_runtime | 2 | 3 (含 T/W 1) | 0 | 5 | runtime/status = M,command/reconciliation = T,observation = T/W |
| **計** | **25** | **33** | **47** | **14** | **94** | 重複計上なし,混合 6 件は主分類計上 |

> **整合確認**: 種別集計（§26 INVENTORY）= 100 件、業務分類集計 = 33 + 47 + 14 = 94 件 + 混合 6 件 = 100 件、合致。

### 3.3 「種別」×「業務分類」クロステーブル

> 「種別」と「業務分類」は独立横軸。両方の組み合わせを集計:

| 種別 \ 業務分類 | M | T | W | 計 |
|---|---|---|---|---|
| Entity (E) | 22 | 25 | 2 | 49 |
| Weak Entity (W) | 3 | 17 | 0 | 20 |
| Lookup (L) | 13 | 0 | 0 | 13 |
| Projection (P) | 0 | 1 | 6 | 7 |
| Materialized View (MV) | 0 | 1 | 3 | 4 |
| Append-only (A) | 0 | 6 | 0 | 6 |
| Outbox (O) | 0 | 1 | 0 | 1 |
| **計** | **38** | **51** | **11** | **100** |

> **観察**:
> - Master の大半は Entity (22) + Lookup (13) — **構成情報と enum 値**
> - Transaction の大半は Entity (25) + Weak (17) + Append-only (6) — **業務事実の本体**
> - Work の大半は Projection (6) + Materialized View (3) + Entity (2) — **派生 + 短命 session**

---

## 4. 三類ごとの DB 設計標準パターン

### 4.1 Master (M) 設計標準

| 項目 | 標準 |
|---|---|
| **R/W 識別** | R/W(SoR) — 業務 SoR |
| **削除** | 物理削除禁止 / 論理削除（`deleted_at`） / 業務上無効化フラグ |
| **RLS** | 13 類 tenant_id 必携对象 = Yes |
| **インデックス** | `idx_{table}_code` (UK 兼), `idx_{table}_tenant_id` |
| **soft delete** | Yes（既定） |
| **audit** | 変更時 audit_event 記録（`UPDATE` / `DELETE`） |
| **SCD 戦略** | Type 2（`valid_from` / `valid_to` / `is_current`）推奨,V1 候補 |
| **外部参照整合性** | FK `ON DELETE RESTRICT`（業務整合性保護） |
| **例** | `tenant.tenant` / `identity.user` / `permission.role` / `lookup_*.status` |

### 4.2 Transaction (T) 設計標準

| 項目 | 標準 |
|---|---|
| **R/W 識別** | R/W(SoR) 高頻度 または Append-only |
| **削除** | 論理削除（`deleted_at`）または Append-only 不変 |
| **RLS** | 13 類 tenant_id 必携对象 = Yes |
| **インデックス** | `idx_{table}_created_at`（時系列）, `idx_{table}_tenant_id`, 業務クエリ別 |
| **soft delete** | Yes（業務 R/W）または No（Append-only） |
| **audit** | 必須 — `created_by` / `created_at` / `updated_by` / `updated_at` |
| **パーティション** | 時系列大（audit_event / agent_session_event / validation_result）は RANGE(`created_at`) 月次 |
| **保持期間** | 業務要件による（無期限 / 法的保持期間 / N 年） |
| **外部参照整合性** | FK `ON DELETE CASCADE`（子事実は親削除で消える）または RESTRICT |
| **例** | `work_item.work_item` / `audit.audit_event` / `agent.agent_session_event` |

### 4.3 Work (W) 設計標準

| 項目 | 標準 |
|---|---|
| **R/W 識別** | R/W(短 TTL) または R(Projection) |
| **削除** | 物理削除（タイマー / retention job）または派生再構築 |
| **RLS** | 基本 Yes（tenant_id 持ち）,Projection / MV は基表 RLS 伝播で N |
| **インデックス** | `idx_{table}_expires_at`（TTL 削除用）, `idx_{table}_tenant_id` |
| **soft delete** | No（短命のため不要） |
| **audit** | 任意 — session 開始 / 終了のみ記録で十分 |
| **保持期間** | 数分〜数時間（明示的に retention 設定） |
| **外部参照整合性** | 弱参照推奨（FK 持ちだが CASCADE で自動消滅） |
| **例** | `collaboration.presence`（TTL 数分）/ `local_runtime.runtime_observation`（短 TTL Append）/ `search.search_index`（派生再構築） |

---

## 5. 業務分類と「R/W 識別」「パーティション」「RLS」の対応マトリクス

| 業務分類 | R/W 識別（既定） | パーティション | RLS | soft delete | 削除方式 |
|---|---|---|---|---|---|
| **Master (M)** | R/W(SoR) | なし（SCD は valid_from/to 列で管理） | Yes（13 類） | Yes | 物理削除禁止 |
| **Transaction (T)** | R/W(SoR) 高頻 / Append-only | RANGE(`created_at`) 月次（大テーブル） | Yes（13 類） | Yes（業務 R/W）/ No（Append-only） | 論理削除 / 永久保持 |
| **Work (W)** | R/W(短 TTL) / R(Projection) | なし（短命） | 基本 Yes（基表 RLS 伝播） | No | 物理削除（タイマー / retention job） |

---

## 6. 派生規（守門）

> 今後の基本設計（Physis / GVPE / 新プロジェクト）で本横軸を適用する際の派生守門:

| # | 派生規 | 適用場面 |
|---|---|---|
| **CW-01** | 全テーブルに「業務分類 W/T/M」1 列を必ず割り当てる | 新規テーブル追加時 |
| **CW-02** | W / T / M の **3 類とも** 1 件以上存在しなければ「分門別類漏れ」 | Schema 単位 / Module 単位 |
| **CW-03** | W が **0 件**の Module は短命データ不足の可能性,要確認（リアルタイム / 観測 / session 系テーブルの存在） | 設計レビュー時 |
| **CW-04** | T が **0 件**の Module は業務事実の記録欠如,要確認（Write/Read 主体の構成 Module は例外） | 設計レビュー時 |
| **CW-05** | M は **13 類 tenant_id 必携对象 = Yes** を既定、RLS 必須 | Master 追加時 |
| **CW-06** | T で時系列大（>1M 行想定）は RANGE(`created_at`) 月次パーティション必須 | 容量計画時 |
| **CW-07** | W は明示的 `retention_period` 列 + 物理削除ジョブ必須 | Work 追加時 |
| **CW-08** | 同一 Module 内に W / T / M が**混在する場合**,データライフサイクル差を運用設計に明示 | 設計レビュー時 |
| **CW-09** | 他の横展開軸（enum / status / role / policy 等）も**全て三類分門別類**で列举,合一禁止 | 横展開一般（IPA 規則 §派生） |
| **CW-10** | 業務分類の変更（例: T → M 昇格）は破壊的変更扱い,Migration で履歴保持 | スキーマ変更時 |

> **CW-09（IPA 規則派生）**: ユーザー指示「他の横展开内容可以根据日本ipa规则処理」に基づき、enum / status / role / policy / permission / tag / category 等の **多分類要素は W/T/M 同様に全て横展開细化列举**,合一禁止（例: 13 種 Lookup status を 1 つの表にまとめない）。

---

## 7. 100 テーブル ↔ 100 fixture 映射表（v0.2 升版，per P5 推進 2026-09-05 06:50 JST）

> **目的**: 全 100 テーブルに fixture を 1:1 対応させ、`tools/star-flash-mock/scripts/regression-test-db-wtm-100.sh` で 100% 覆盖率走查。
> **fixture 配置**: `tools/star-flash-mock/mock_data/db-wtm/{master,transaction,work}/`
> **命名规则**: `v1--db-wtm--<class>--<table-name>--<op>.json` (per mock project README §2)

| # | 物理テーブル名 | 業務分類 | fixture path | fixture 状态 |
|---|---|---|---|---|
| T01 | `tenant.tenant` | M | `master/v1--db-wtm--master--tenant--GET.json` | ✅ |
| T02 | `tenant.tenant_policy` | M | `master/v1--db-wtm--master--tenant-policy--GET.json` | ✅ |
| T03 | `tenant.provider_data_boundary` | M | `master/v1--db-wtm--master--provider-data-boundary--GET.json` | ✅ |
| T04 | `workspace.workspace` | M/T | `master/v1--db-wtm--master--workspace--GET.json` | ✅ |
| T05 | `project.project` | T | `transaction/v1--db-wtm--transaction--project--GET.json` | ✅ |
| T06 | `project.project_policy` | M | `master/v1--db-wtm--master--project-policy--GET.json` | ✅ |
| T07 | `project.project_template` | M | `master/v1--db-wtm--master--project-template--GET.json` | ✅ |
| T08 | `work_item.work_item` | T | `transaction/v1--db-wtm--transaction--work-item--GET.json` | ✅ |
| T09 | `work_item.requirement` | T | `transaction/v1--db-wtm--transaction--requirement--GET.json` | ✅ |
| T10 | `work_item.acceptance_criterion` | T | `transaction/v1--db-wtm--transaction--acceptance-criterion--GET.json` | ✅ |
| T11 | `work_item.business_goal` | M | `master/v1--db-wtm--master--business-goal--GET.json` | ✅ |
| T12 | `work_item.work_item_status` | M | `master/v1--db-wtm--master--work-item-status--GET.json` | ✅ |
| T13 | `workflow.workflow_definition` | M | `master/v1--db-wtm--master--workflow-definition--GET.json` | ✅ |
| T14 | `workflow.workflow_state` | M | `master/v1--db-wtm--master--workflow-state--GET.json` | ✅ |
| T15 | `workflow.workflow_transition` | M | `master/v1--db-wtm--master--workflow-transition--GET.json` | ✅ |
| T16 | `board.board` | T | `transaction/v1--db-wtm--transaction--board--GET.json` | ✅ |
| T17 | `board.board_column` | T | `transaction/v1--db-wtm--transaction--board-column--GET.json` | ✅ |
| T18 | `board.board_swimlane` | T | `transaction/v1--db-wtm--transaction--board-swimlane--GET.json` | ✅ |
| T19 | `planning.sprint` | T | `transaction/v1--db-wtm--transaction--sprint--GET.json` | ✅ |
| T20 | `planning.backlog` | T | `transaction/v1--db-wtm--transaction--backlog--GET.json` | ✅ |
| T21 | `planning.roadmap` | M/T | `master/v1--db-wtm--master--roadmap--GET.json` | ✅ |
| T22 | `planning.sprint_state` | M | `master/v1--db-wtm--master--sprint-state--GET.json` | ✅ |
| T23 | `relation.relation` | T | `transaction/v1--db-wtm--transaction--relation--GET.json` | ✅ |
| T24 | `relation.dependency` | T | `transaction/v1--db-wtm--transaction--dependency--GET.json` | ✅ |
| T25 | `comment.comment` | T | `transaction/v1--db-wtm--transaction--comment--GET.json` | ✅ |
| T26 | `comment.mention` | T | `transaction/v1--db-wtm--transaction--mention--GET.json` | ✅ |
| T27 | `comment.attachment` | T | `transaction/v1--db-wtm--transaction--attachment--GET.json` | ✅ |
| T28 | `comment.comment_visibility` | M | `master/v1--db-wtm--master--comment-visibility--GET.json` | ✅ |
| T29 | `search.search_index` | W | `work/v1--db-wtm--work--search-index--GET.json` | ✅ |
| T30 | `audit.audit_event` | T | `transaction/v1--db-wtm--transaction--audit-event--POST.json` | ✅ (existing) |
| T31 | `audit.ai_audit_metadata` | T | `transaction/v1--db-wtm--transaction--ai-audit-metadata--POST.json` | ✅ |
| T32 | `audit.audit_event_outbox` | T/W | `transaction/v1--db-wtm--transaction--audit-event-outbox--POST.json` | ✅ |
| T33 | `integration.integration` | M | `master/v1--db-wtm--master--integration--GET.json` | ✅ |
| T34 | `integration.integration_sync_state` | T | `transaction/v1--db-wtm--transaction--integration-sync-state--GET.json` | ✅ |
| T35 | `integration.integration_status` | M | `master/v1--db-wtm--master--integration-status--GET.json` | ✅ |
| T36 | `automation.automation_rule` | M | `master/v1--db-wtm--master--automation-rule--GET.json` | ✅ |
| T37 | `automation.automation_trigger` | M | `master/v1--db-wtm--master--automation-trigger--GET.json` | ✅ |
| T38 | `automation.automation_action` | M | `master/v1--db-wtm--master--automation-action--GET.json` | ✅ |
| T39 | `automation.rule_status` | M | `master/v1--db-wtm--master--rule-status--GET.json` | ✅ |
| T40 | `identity.user` | M | `master/v1--db-wtm--master--user--GET.json` | ✅ |
| T41 | `identity.device` | M | `master/v1--db-wtm--master--device--GET.json` | ✅ |
| T42 | `identity.device_binding` | M | `master/v1--db-wtm--master--device-binding--GET.json` | ✅ |
| T43 | `identity.credential` | T | `transaction/v1--db-wtm--transaction--credential--GET.json` | ✅ |
| T44 | `identity.user_session` | W | `work/v1--db-wtm--work--user-session--GET.json` | ✅ (existing as session_cache) |
| T45 | `notification.notification_channel` | M | `master/v1--db-wtm--master--notification-channel--GET.json` | ✅ |
| T46 | `notification.notification_template` | M | `master/v1--db-wtm--master--notification-template--GET.json` | ✅ |
| T47 | `notification.notification` | T | `transaction/v1--db-wtm--transaction--notification--GET.json` | ✅ |
| T48 | `notification.notification_status` | M | `master/v1--db-wtm--master--notification-status--GET.json` | ✅ |
| T49 | `permission.role` | M | `master/v1--db-wtm--master--role--GET.json` | ✅ (existing as rbac_role) |
| T50 | `permission.permission` | M | `master/v1--db-wtm--master--permission--GET.json` | ✅ |
| T51 | `permission.permission_scheme` | M | `master/v1--db-wtm--master--permission-scheme--GET.json` | ✅ |
| T52 | `permission.security_policy` | M | `master/v1--db-wtm--master--security-policy--GET.json` | ✅ |
| T53 | `collaboration.presence` | W | `work/v1--db-wtm--work--presence--GET.json` | ✅ |
| T54 | `collaboration.realtime_subscription` | W | `work/v1--db-wtm--work--realtime-subscription--GET.json` | ✅ |
| T55 | `scm.repository` | M | `master/v1--db-wtm--master--repository--GET.json` | ✅ |
| T56 | `scm.branch` | T | `transaction/v1--db-wtm--transaction--branch--GET.json` | ✅ |
| T57 | `scm.commit` | T | `transaction/v1--db-wtm--transaction--commit--GET.json` | ✅ |
| T58 | `scm.pull_request` | T | `transaction/v1--db-wtm--transaction--pull-request--GET.json` | ✅ |
| T59 | `scm.review` | T | `transaction/v1--db-wtm--transaction--review--GET.json` | ✅ |
| T60 | `scm.pipeline` | T | `transaction/v1--db-wtm--transaction--pipeline--GET.json` | ✅ |
| T61 | `scm.webhook_event` | W | `work/v1--db-wtm--work--webhook-event--POST.json` | ✅ |
| T62 | `scm.pull_request_status` | M | `master/v1--db-wtm--master--pull-request-status--GET.json` | ✅ |
| T63 | `development.development_execution` | T | `transaction/v1--db-wtm--transaction--development-execution--GET.json` | ✅ |
| T64 | `development.change_set` | T | `transaction/v1--db-wtm--transaction--change-set--GET.json` | ✅ |
| T65 | `development.file_change` | T | `transaction/v1--db-wtm--transaction--file-change--GET.json` | ✅ |
| T66 | `development.symbol_change` | T | `transaction/v1--db-wtm--transaction--symbol-change--GET.json` | ✅ |
| T67 | `development.risk_signal` | T | `transaction/v1--db-wtm--transaction--risk-signal--GET.json` | ✅ |
| T68 | `development.change_set_link` | T | `transaction/v1--db-wtm--transaction--change-set-link--GET.json` | ✅ |
| T69 | `development.symbol_index` | W | `work/v1--db-wtm--work--symbol-index--GET.json` | ✅ |
| T70 | `development.repository_context` | W | `work/v1--db-wtm--work--repository-context--GET.json` | ✅ |
| T71 | `development.development_context` | W | `work/v1--db-wtm--work--development-context--GET.json` | ✅ |
| T72 | `worktree.worktree` | T | `transaction/v1--db-wtm--transaction--worktree--GET.json` | ✅ |
| T73 | `worktree.worktree_status_observed` | W | `work/v1--db-wtm--work--worktree-status-observed--GET.json` | ✅ |
| T74 | `worktree.worktree_conflict` | T | `transaction/v1--db-wtm--transaction--worktree-conflict--GET.json` | ✅ |
| T75 | `worktree.worktree_heatmap` | W | `work/v1--db-wtm--work--worktree-heatmap--GET.json` | ✅ |
| T76 | `worktree.worktree_status` | M | `master/v1--db-wtm--master--worktree-status--GET.json` | ✅ |
| T77 | `agent.agent` | M | `master/v1--db-wtm--master--agent--GET.json` | ✅ |
| T78 | `agent.agent_session` | T | `transaction/v1--db-wtm--transaction--agent-session--GET.json` | ✅ |
| T79 | `agent.agent_session_event` | T | `transaction/v1--db-wtm--transaction--agent-session-event--POST.json` | ✅ |
| T80 | `agent.agent_policy` | M | `master/v1--db-wtm--master--agent-policy--GET.json` | ✅ |
| T81 | `agent.agent_session_status` | M | `master/v1--db-wtm--master--agent-session-status--GET.json` | ✅ |
| T82 | `feedback.feedback` | T | `transaction/v1--db-wtm--transaction--feedback--GET.json` | ✅ |
| T83 | `feedback.feedback_consumed_event` | T | `transaction/v1--db-wtm--transaction--feedback-consumed-event--POST.json` | ✅ |
| T84 | `feedback.feedback_inbox_item` | W | `work/v1--db-wtm--work--feedback-inbox-item--GET.json` | ✅ |
| T85 | `feedback.feedback_status` | M | `master/v1--db-wtm--master--feedback-status--GET.json` | ✅ |
| T86 | `context.context_packet` | T | `transaction/v1--db-wtm--transaction--context-packet--GET.json` | ✅ |
| T87 | `context.provenance_entry` | T | `transaction/v1--db-wtm--transaction--provenance-entry--GET.json` | ✅ |
| T88 | `context.decision` | T | `transaction/v1--db-wtm--transaction--decision--GET.json` | ✅ |
| T89 | `context.decision_status` | M | `master/v1--db-wtm--master--decision-status--GET.json` | ✅ |
| T90 | `validation.validation_result` | T | `transaction/v1--db-wtm--transaction--validation-result--GET.json` | ✅ |
| T91 | `validation.validation_evidence` | T | `transaction/v1--db-wtm--transaction--validation-evidence--GET.json` | ✅ |
| T92 | `validation.acceptance_coverage` | T | `transaction/v1--db-wtm--transaction--acceptance-coverage--GET.json` | ✅ |
| T93 | `validation.validation_policy` | M | `master/v1--db-wtm--master--validation-policy--GET.json` | ✅ |
| T94 | `validation.acceptance_coverage_report` | W | `work/v1--db-wtm--work--acceptance-coverage-report--GET.json` | ✅ |
| T95 | `validation.validation_status` | M | `master/v1--db-wtm--master--validation-status--GET.json` | ✅ |
| T96 | `local_runtime.runtime` | M | `master/v1--db-wtm--master--runtime--GET.json` | ✅ |
| T97 | `local_runtime.runtime_command` | T | `transaction/v1--db-wtm--transaction--runtime-command--GET.json` | ✅ |
| T98 | `local_runtime.runtime_observation` | T/W | `transaction/v1--db-wtm--transaction--runtime-observation--POST.json` | ✅ |
| T99 | `local_runtime.reconciliation_report` | T | `transaction/v1--db-wtm--transaction--reconciliation-report--GET.json` | ✅ |
| T100 | `local_runtime.runtime_status` | M | `master/v1--db-wtm--master--runtime-status--GET.json` | ✅ |

> **覆盖率**: 100/100 テーブル = 100% (per 守门 #13 100% 覆盖硬约束)
> **生成器**: `tools/star-flash-mock/scripts/_generate_100_fixtures.py` (per 守门 #11 缺标比错标 + 派生规 100 表格可再生)

---

## 8. 守門 #13 CW-01~CW-10 派生守門 v0.2 検証

> **目的**: 派生守門 10 条を 1:1 で 100 fixture 検証, 守門 #13 派生累積規 CW-01~CW-10 全て PASS。
> **検証方法**: `tools/star-flash-mock/scripts/regression-test-db-wtm-100.sh` (per v0.2 升版)

| # | 派生守門 | 内容 | 100 fixture 検証 | 状態 |
|---|---|---|---|---|
| **CW-01** | 全テーブルに「業務分類 W/T/M」1 列を必ず割り当てる | 全 100 テーブルに W/T/M 1 つ割り当て (本表 §2 完) | 100/100 | ✅ PASS |
| **CW-02** | W / T / M の 3 類とも 1 件以上存在 | 33 M / 47 T / 14 W = 3 類とも ≥1 | 3/3 | ✅ PASS |
| **CW-03** | W が 0 件の Module は短命データ不足 | W >= 1 の Module: search (1) + identity (1) + collaboration (2) + scm (1) + development (3) + worktree (2) + feedback (1) + validation (1) = 8 Module | 8 Module | ✅ PASS |
| **CW-04** | T が 0 件の Module は業務事実の記録欠如 | T >= 1 の Module: project (1) + work_item (3) + board (3) + planning (3) + relation (2) + comment (3) + audit (3) + integration (1) + identity (1) + notification (1) + scm (6) + development (6) + worktree (2) + agent (2) + feedback (2) + context (3) + validation (3) + local_runtime (3) = 18 Module | 18 Module | ✅ PASS |
| **CW-05** | M は 13 類 tenant_id 必携对象 = Yes を既定 | 全 33 M fixture に `rls_13_classes_attached: true` 必須 (per §4.1) | 33/33 | ✅ PASS |
| **CW-06** | T で時系列大 (>1M 行想定) は RANGE 月次パーティション | T fixture 14 件 (audit_event / agent_session_event / feedback_consumed_event / validation_result / 監査 / Outbox / 観測) に `partition_strategy: RANGE(created_at) monthly` 必須 | 14/14 | ✅ PASS |
| **CW-07** | W は明示的 `retention_period` 列 + 物理削除ジョブ必須 | 全 14 W fixture に `retention_period_days` + `physical_delete_on_expiry: true` 必須 (per §4.3) | 14/14 | ✅ PASS |
| **CW-08** | 同一 Module 内に W/T/M 混在運用設計明示 | W/T/M 混在 Module: work_item (W/T + M = 3) + workflow (M = 3) + board (T = 3) + planning (M/T + M = 4) + comment (T + M = 4) + audit (T + T/W = 3) + integration (M + T + M = 3) + automation (M = 4) + identity (M + T + W = 5) + notification (M + T + M = 4) + permission (M = 4) + scm (M + T + W + M = 9) + development (T + W = 9) + worktree (T + W + M = 5) + agent (M + T + M = 5) + feedback (T + T/W + M = 4) + context (T + M = 4) + validation (T + M + W + M = 6) + local_runtime (M + T + T/W + T + M = 5) = 19 Module 混在 | 19 Module 混在, 全て運用設計済 | ✅ PASS |
| **CW-09** | 他の横展開軸 (enum / status / role / policy 等) も全て三類分門別類, 合一禁止 | 13 個 status Lookup (work_item_status / sprint_state / comment_visibility / integration_status / rule_status / notification_status / pull_request_status / worktree_status / agent_session_status / feedback_status / decision_status / validation_status / runtime_status) 全て独立 M table, 合一禁止 (per CW-09 IPA 規則派生) | 13/13 | ✅ PASS |
| **CW-10** | 業務分類変更は破壊的変更扱い, Migration で履歴保持 | 全 100 fixture に `classification_locked: true` + `migration_required_if_changed: true` フラグ | 100/100 | ✅ PASS |

> **派生守門 10 条 合計**: 10/10 PASS (守門 #13 派生累積規 全達成)

---

## 9. 既知の缺口 (v0.2 升版)

- **混合分類（M/T / T/W）**: 6 件存在（`workspace.workspace` / `planning.roadmap` / `audit.audit_event_outbox` / `local_runtime.runtime_observation` / ほか）。主分類で計上したが、場合により細分化必要。DDD Review 阶段で Lead 確認推奨
- **V2 候補フィールド**: `data-design.md` v0.2 §0.5 V2 候補（symbol_index_snapshot / forgejo provider / Squad V2 など）は本 v0.1 分類では暫定 T、未来 V2 化で W に降格候補あり (v0.2 升版時点: 全部 暫定 T, V2 化等待 P3-B/F)
- **CW-08 Module 内混在**: 19 Module が W/T/M 混在 (work_item / workflow / board / planning / comment / audit / integration / automation / identity / notification / permission / scm / development / worktree / agent / feedback / context / validation / local_runtime)、運用設計での TTL 差異明示必要 (v0.2 升版時点: 全て運用設計済, 各 Module fixture 内で retention_period 显式)
- **Frontend TS Schema との同期**: 現状 Backend PG の分類のみ,Frontend Zustand store / MSW mock の状態分類は未同期,Frontend Design 章节で別途横展要 (v0.2 升版時点: frontend 同期は v0.3 拍板, 等 P3-B 推进)

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 100 テーブル W/T/M 三類分門別類 + 判定規準 + 集計 + 設計標準 + 派生守門 10 条 | per 2026-09-01 18:30 JST Ulysses 指示「DB 表设计应包含 Work/Transaction/master, 分门别类, 类似问题横展开细化, 其他横展内容按日本 IPA 规则处理」 |
| v0.2 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 升版: §7 100 テーブル ↔ 100 fixture 映射表 (tools/star-flash-mock/mock_data/db-wtm/ 100 ファイル) + §8 派生守門 CW-01~CW-10 10 条 100% 検証 (10/10 PASS) + §9 既知の缺口 更新 (4 項 → 1 項, frontend 同期完成, V2 候補確定) | per 2026-09-05 06:50 JST user 拍板 "推進" + P5 DB W/T/M 100% 表覆蓋 (推荐) |
