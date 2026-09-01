# OPTIMIZATION-NOTES.md — Star プラットフォーム データ設計 最適化提案

> **基準**: IPA データモデル詳細設計書 — 整合性 / 性能 / 命名 / 制約 改善提案
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **本ファイル役割**: `docs/data-design.md` v0.2 を IPA 標準章立てに整流する過程で発見した最適化提案清单
> **適用方針**: 提案は「即時適用」「次升版適用」「保留（DDD Review 待ち）」の 3 区分

---

## 0. 凡例

| 優先度 | 説明 | 影響範囲 |
|---|---|---|
| **P0** | 即時修正（誤動作 / 整合性破壊） | Migration 適用前に必修正 |
| **P1** | 次升版（v0.3）で適用 | 機能影響あり、テスト必要 |
| **P2** | V1/V2 適用（V2 候補） | 性能 / UX 改善 |
| **P3** | 保留（要 DDD Review 5 域 Lead 拍板） | 設計議論必要 |

| 状態 | 説明 |
|---|---|
| 🟢 適用済み | data-design.md v0.2 に commit 済 |
| 🟡 提案中 | 本ファイル提案（次升版で取込判断） |
| 🔴 警告 | 整合性 / 性能 / 命名 重大問題、優先対処 |

---

## 1. 命名 / 整合性 即時修正候補

### ON-001 (P0) 🟡 — `worktree.worktree_status` Lookup Table の状態名誤字

| 項目 | 内容 |
|---|---|
| **発見箇所** | `docs/data-design.md` §3.3.1.1 (`worktree.worktree_status` INSERT) |
| **問題** | `'WAITING_FEADBACK'` 表記（誤字: `FEADBACK` → 正しくは `FEEDBACK`） |
| **影響** | §3.3.1.1 INSERT ステートメントに誤字。§3.3.1.2 CHECK 制約リストは正しい `WAITING_FEEDBACK` で書かれており、INSERT が CHECK 制約に違反して実行失敗する |
| **推奨修正** | §3.3.1.1 の INSERT を以下に修正: `'WAITING_FEEDBACK', 'Waiting Feedback', FALSE, 50,` |
| **適用方針** | 即時 P0 修正コミット。修正後 v0.2.1 とし、§10 改訂履歴に記録 |
| **関連 IPA ファイル** | `tables/worktree_worktree_status.md` |

### ON-002 (P1) 🟡 — `worktree.worktree_status_observed` の CHECK 制約名が業務名と乖離

| 項目 | 内容 |
|---|---|
| **発見箇所** | `docs/data-design.md` §4.20.2 |
| **問題** | `ck_worktree_status_observed_validity` 命名が冗長（"observed" + "validity"）。`ck_worktree_observed_time` 等に短縮可能 |
| **推奨** | `ck_worktree_observed_recent` 等。性能影響なし、命名のみ |
| **適用方針** | P1 次升版で統一 |

### ON-003 (P1) 🟡 — 13 個 Lookup Table 命名の不統一

| 項目 | 内容 |
|---|---|
| **発見箇所** | 13 個 Lookup Table 全体（§3.3.2 表） |
| **問題** | 命名規則ブレあり: <br>• `worktree.worktree_status` ← 親名重複（OK）<br>• `agent.agent_session_status` ← 親名重複（OK）<br>• `permission.permission` ← 親名完全重複（曖昧）<br>• `integration.integration_status` ← 親名重複（OK）<br>• 一方 `automation.rule_status` ← **parent 略称 `rule`**、`integration.integration_status` は **parent 完全名 `integration`** |
| **推奨** | 全 Lookup Table を「`{parent_table}_{purpose}`」または「`{parent_table_short}_{purpose}`」に統一。例: `permission.permission` → `permission.permission_code`、 `automation.rule_status` → `automation.automation_rule_status` |
| **影響** | IPA 命名規約準拠、grep / 検索容易化 |
| **適用方針** | P1 次升版で全面リネーム（Migration 影響大、別 PR） |

### ON-004 (P1) 🟡 — `permission.permission` 主キー列名の不統一

| 項目 | 内容 |
|---|---|
| **発見箇所** | `docs/data-design.md` §4.16.2 |
| **問題** | 主キーが `code` で他 Lookup Table の `status_code` / `state_code` / `visibility_code` と不統一 |
| **推奨** | `code` → `permission_code` にリネーム |
| **適用方針** | P1 次升版で ON-003 と一括リネーム |

---

## 2. 制約強化候補

### ON-101 (P1) 🟡 — `relation.relation` の自己参照禁止 CHECK 強化

| 項目 | 内容 |
|---|---|
| **発見箇所** | `docs/data-design.md` §4.8.1 `ck_relation_no_self` |
| **問題** | `source_id <> target_id` のみ。`(source_type, source_id) <> (target_type, target_id)` の複合 CHECK が無い |
| **推奨** | CHECK 制約を `(source_type, source_id) <> (target_type, target_id)` に強化（業務的に異なるリソース間の "self-loop" 防止） |
| **適用方針** | P1 次升版で Migration ALTER |

### ON-102 (P1) 🟡 — `work_item.work_item` の自己参照深さ制限

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.4.1 |
| **問題** | `parent_id` で再帰階層可能だが、CHECK で深さ制限が無い（無限ループ / 過深階層リスク） |
| **推奨** | 業務上は 5-7 階層が現実的。App 層で `WITH RECURSIVE` 階層カウント + N 階層で CHECK 違反。DB 層では Z 値（envelope path）または `path` 列追加 |
| **適用方針** | P1 次升版で検討（V1 候補） |

### ON-103 (P2) 🟡 — `comment.attachment.size_bytes` 上限の妥当性

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.9.3 `ck_attachment_size`: `size_bytes <= 104857600` (100MB) |
| **問題** | Object Storage 境界しきい値（§1.5: 1MB）と乖離。100MB 添付は §5.1 推奨と整合性取れていない |
| **推奨** | 上限を 50MB に下げる（業務ユースケース: 動画 / アーカイブ zip 用途で 50MB が現実的上限）または §1.5 と整合する 1MB 推奨 + 100MB 警告 |
| **適用方針** | P2 DDD Review 待ち |

### ON-104 (P1) 🟡 — `validation.validation_result.severity` 必須化

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.24.1 |
| **問題** | `severity` 列が NULL 可だが、業務上は必須。Validation 結果のトリアージに使われる |
| **推奨** | `severity VARCHAR(16) NOT NULL DEFAULT 'INFO'` |
| **適用方針** | P1 次升版で NOT NULL 化（既存データ確認後） |

### ON-105 (P2) 🟡 — `audit.audit_event.action` 値域 CHECK 拡張

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.11.1 `ck_audit_event_action`: `length(action) >= 1 AND length(action) <= 64` |
| **問題** | 文字列長のみ。値域なし。業務上一貫性なし（`user.login` / `workitem.create` / `agent_session.start` 等） |
| **推奨** | 業務イベント分類 enum 化: `('{resource}.{verb}' で管理、`resource IN ('user','workitem','worktree','agent_session','feedback','comment','project','tenant','integration','security')`, `verb IN ('create','update','delete','login','logout','assign','resolve','escalate','merge','commit')`)` |
| **適用方針** | P2 DDD Review 待ち（V1 候補） |

### ON-106 (P1) 🟡 — `worktree.worktree` 状態遷移 CHECK 制約不在

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.20.1 |
| **問題** | 17 状態間の遷移制約が DB 層で強制されない（App 層制御） |
| **推奨** | §7.1 状態機械図に従い、`CREATE TABLE worktree_status_transition (from_status, to_status, PRIMARY KEY (from_status, to_status))` を作成し、`worktree.status` UPDATE 時に TRIGGER で検証 |
| **適用方針** | P1 次升版で追加（性能影響軽微） |

---

## 3. インデックス性能改善候補

### ON-201 (P1) 🟡 — `work_item.work_item` 期限ソート用複合 INDEX 不足

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.4.1 |
| **問題** | ボード UI の「期限順」表示で `(tenant_id, project_id, due_date ASC)` が必要だが、現状 `(tenant_id, project_id, status)` のみ |
| **推奨** | `CREATE INDEX idx_work_item_tenant_project_due ON work_item.work_item (tenant_id, project_id, due_date) WHERE deleted_at IS NULL AND due_date IS NOT NULL;` |
| **適用方針** | P1 次升版で追加（容量影響 +1-2%） |

### ON-202 (P1) 🟡 — `agent.agent_session` Agent 別履歴 INDEX 不足

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.21.2 |
| **問題** | `(tenant_id, worktree_id, status)` はあるが、`(tenant_id, agent_id, started_at DESC)` が無い |
| **推奨** | `CREATE INDEX idx_agent_session_tenant_agent_started ON agent.agent_session (tenant_id, agent_id, started_at DESC) WHERE deleted_at IS NULL;` |
| **適用方針** | P1 次升版で追加 |

### ON-203 (P2) 🟡 — `feedback.feedback` プロジェクト横断 INDEX 追加

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.22.1 |
| **問題** | `(tenant_id, worktree_id, status)` はあるが、プロジェクト管理画面での横断表示に `(tenant_id, project_id, status)` が必要 |
| **推奨** | `CREATE INDEX idx_feedback_tenant_project_status ON feedback.feedback (tenant_id, project_id, status) WHERE deleted_at IS NULL;`（project_id 列追加が前提、ON-XXX と一括） |
| **適用方針** | P2 V1 候補 |

### ON-204 (P2) 🟡 — `scm.commit` 作成者 INDEX 不足

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.18.3 |
| **問題** | `(repository_id, sha)` はあるが、`(author_user_id, committed_at DESC)` 無い。ユーザプロフィール画面で「自分のコミット一覧」表示で必要 |
| **推奨** | `CREATE INDEX idx_commit_tenant_author ON scm.commit (tenant_id, author_user_id, committed_at DESC) WHERE deleted_at IS NULL;` |
| **適用方針** | P2 V1 候補 |

### ON-205 (P1) 🟡 — `audit.audit_event` tenant_id 起首 BRIN 適用範囲

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.11.1 |
| **問題** | `idx_audit_event_tenant_created_brin` は `(created_at)` のみで、`tenant_id` フィルタ併用時に性能劣化する可能性 |
| **推奨** | BRIN の特性上、`(tenant_id, created_at)` 複合より `created_at` 単一 + 別途 tenant_id btree 推奨（PostgreSQL BitmapAnd で結合される） |
| **適用方針** | P1 SRE 監視で実測後判断 |

---

## 4. RLS / セキュリティ 強化候補

### ON-301 (P1) 🟡 — RLS Policy 命名規約の不統一

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.1.4 (Tenant RLS 戦略), §7 全体 |
| **問題** | RLS Policy 命名が明示されていない箇所多数。実装時に `{table}_tenant_isolation` / `{table}_tenant_select` 等に統一必要 |
| **推奨** | 命名規約: `policy_{table}_{action}` (例: `policy_work_item_select` / `policy_work_item_insert` / `policy_work_item_update` / `policy_work_item_delete`) |
| **適用方針** | P1 次升版で明示 |

### ON-302 (P1) 🟡 — RLS session GUC 設定の Application 責務明示

| 項目 | 内容 |
|---|---|
| **発見箇所** | §7 全体 |
| **問題** | `SET LOCAL app.current_tenant_id = ...` の実行責務が Application 層（`docs/runtime-design.md`）に分散。明示的な checklist がない |
| **推奨** | 13 類 RLS 必須对象に対し、各リクエスト前に必ず SET LOCAL する checklist を `docs/security-design.md` に追加。Application ミドルウェア層で全リクエストに強制 |
| **適用方針** | P1 次升版で `docs/security-design.md` 参照追加 |

### ON-303 (P2) 🟡 — Append-only テーブルの UPDATE/DELETE 防止

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.11.1 `audit_event`, §4.11.2 `ai_audit_metadata`, §4.21.3 `agent_session_event`, §4.22.2 `feedback_consumed_event` |
| **問題** | 論理上 Append-only だが、DB 層で UPDATE/DELETE を防ぐ TRIGGER がない |
| **推奨** | `CREATE TRIGGER trg_audit_event_immutable BEFORE UPDATE OR DELETE ON audit.audit_event FOR EACH ROW EXECUTE FUNCTION fn_reject_mutation();` |
| **適用方針** | P2 V1 候補（WORM 要件 §R-17） |

### ON-304 (P2) 🟡 — `audit_event_outbox` 物理削除戦略

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.11.3 |
| **問題** | 物理削除が想定されている（§3.1.5 MVP 例外）が、保持期間・削除ジョブが未定義 |
| **推奨** | `published_at` から 30 日後 physical delete を App 側 cron job で実装。`document` で明示 |
| **適用方針** | P2 DDD Review 待ち |

---

## 5. データ型 / 容量 改善候補

### ON-401 (P2) 🟡 — `TEXT` vs `VARCHAR(n)` 戦略の明示

| 項目 | 内容 |
|---|---|
| **発見箇所** | §3.1.6 文字 / テキスト長さ |
| **問題** | 業務上 200 文字制限の名前列が `VARCHAR(200)` で定義されているが、本文は `TEXT` 無制限。Attachment 本文 / Comment 本文で容量爆発リスク |
| **推奨** | §5.1 Object Storage 境界（> 1MB）と整合し、本文は `TEXT` だが App 層で 1MB 超なら Object Storage へ |
| **適用方針** | P2 V1 候補 |

### ON-402 (P1) 🟡 — `JSONB` 列のデフォルト値整合性

| 項目 | 内容 |
|---|---|
| **発見箇所** | §3.6 `payload_json JSONB NOT NULL`、 §4.1.2 `specific_provider_allowed JSONB NOT NULL DEFAULT '[]'::jsonb` |
| **問題** | 一部テーブルは `'[]'::jsonb` / `'{}'::jsonb` / `'null'::jsonb` のデフォルト揺れ |
| **推奨** | 全 JSONB 列: <br>- 配列系 → `DEFAULT '[]'::jsonb` <br>- オブジェクト系 → `DEFAULT '{}'::jsonb` <br>- Optional 許容 → `NULL` 許可（NOT NULL 制約しない）<br>規約を §3.1.x に追記 |
| **適用方針** | P1 次升版で全面整合 |

### ON-403 (P2) 🟡 — `UUID v7` 適用範囲の明示

| 項目 | 内容 |
|---|---|
| **発見箇所** | §2.3 UUID v7 選型理由 |
| **問題** | 「Application 側で v7 生成」と書かれているが、実装時の Rust crate（`uuid` v1.5+ の `Uuid::now_v7()`）が固定されていない |
| **推奨** | §2.3 に実装 crate を明示: `uuid = { version = "1.10", features = ["v7"] }`、App 層で `Uuid::now_v7()` を `id` デフォルト値に適用。DB 側 `gen_random_uuid()` は v4 降格用 |
| **適用方針** | P2 V1 候補 |

### ON-404 (P1) 🟡 — `TIMESTAMPTZ` デフォルト値の整合性

| 項目 | 内容 |
|---|---|
| **発見箇所** | §3.5 |
| **問題** | `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` だが、Lookup Table には `created_at` 列が無いものあり（`worktree_status` / `work_item_status` 等） |
| **推奨** | Lookup Table にも `created_at` / `updated_at` 追加して管理画面から編集日時トラッキング可能化 |
| **適用方針** | P1 次升版で追加 |

### ON-405 (P2) 🟡 — `version` 楽観ロック列の NotNull 強制

| 項目 | 内容 |
|---|---|
| **発見箇所** | §3.1.2 |
| **問題** | `version INT NOT NULL DEFAULT 1` だが、一部に `version` 列が無いテーブルあり（Lookup Table, Projection 等） |
| **推奨** | 全 Entity / Weak Entity テーブルに `version INT NOT NULL DEFAULT 1` 追加。Lookup / Projection / MV は不要 |
| **適用方針** | P2 V1 候補 |

---

## 6. パーティション 改善候補

### ON-501 (P1) 🟡 — `audit.audit_event` RANGE パーティション戦略

| 項目 | 内容 |
|---|---|
| **発見箇所** | §9 (data-design.md 内) / §4.11.1 |
| **問題** | パーティション戦略は言及されているが、実装詳細（PARTITION BY RANGE (created_at) / 月次 or 週次）が未確定 |
| **推奨** | 月次 RANGE パーティション: `PARTITION BY RANGE (created_at)`, 12 ヶ月先まで pre-create。古いパーティションは 90 日後に DROP（§5.1 Hot/WORM 30 日 + Warm 90 日） |
| **適用方針** | P1 SRE Lead 拍板で確定後 Migration 適用 |

### ON-502 (P2) 🟡 — `agent.agent_session_event` パーティション

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.21.3 |
| **問題** | Append-only + 高頻度（Agent 14 状態遷移 × 数千セッション）→ 1 テーブルで 100M+ 行 |
| **推奨** | ON-501 と同様の月次 RANGE パーティション |
| **適用方針** | P2 V1 候補 |

### ON-503 (P2) 🟡 — `validation.validation_evidence` パーティション

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.24.2 |
| **問題** | テスト出力 / スクリーンショット / ログへのポインタ。1 テスト 10+ evidence で 1M+ evidence/月 |
| **推奨** | ON-501 と同様 |
| **適用方針** | P2 V1 候補 |

---

## 7. 命名 / セマンティクス 改善候補

### ON-601 (P2) 🟡 — `scm.pull_request_status` の 7 状態意味統一

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.18.4 + §3.3.2 |
| **問題** | `OPEN` / `MERGED` / `CLOSED` / `DRAFT` / `READY_FOR_REVIEW` / `APPROVED` / `CHANGES_REQUESTED` の 7 状態。`OPEN` と `READY_FOR_REVIEW` / `DRAFT` の関係が直感的でない |
| **推奨** | 状態図: `DRAFT → READY_FOR_REVIEW → (APPROVED | CHANGES_REQUESTED) → MERGED` または `CLOSED`。`OPEN` は deprecate し `READY_FOR_REVIEW` に統合 |
| **適用方針** | P2 DDD Review 5 域 Lead 拍板（player 域外部依存のため） |

### ON-602 (P2) 🟡 — `worktree.worktree_status` 17 状態の略語化

| 項目 | 内容 |
|---|---|
| **発見箇所** | §3.3.1.1 / §4.20.1 |
| **問題** | 17 状態すべて正式名称（CREATED, READY, ASSIGNED 等）。業務上で「Block された Worktree」等カジュアル表記したい場面 |
| **推奨** | Lookup Table に `short_code` 列追加: `CR / RD / AS / AR / WF / FR / VL / BL / CF / RR / RV / RC / CM / PO / MG / AB / AR` |
| **適用方針** | P2 V1 候補 |

### ON-603 (P3) 🟡 — 5 域独立 Lead 名の Schema 命名への反映

| 項目 | 内容 |
|---|---|
| **発見箇所** | §5 命名 disclaimer (AGENTS.md) |
| **問題** | 5 域 (player/economy/match/social/admin) は歴史治理命名、Star 仓 22 DDD bounded context は別分類。両者のマッピング禁止 |
| **推奨** | Schema 名に 5 域要素を混ぜない（現状 OK）。DDD Review 5 域 Lead による schema 別 ownership 確認 |
| **適用方針** | P3 DDD Review 待ち |

---

## 8. 統合 / API 改善候補

### ON-701 (P1) 🟡 — `audit.audit_event` payload スキーマバリデーション

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.11.1 `payload_json JSONB NOT NULL` |
| **問題** | スキーマ強制なし。`action` ごとの payload shape が Application 層任せ |
| **推奨** | §1.7 / §API-3.x で `action` ごとの JSON Schema 定義、`CHECK (jsonb_matches_schema(payload_json, '...'))` で PG 側強制 |
| **適用方針** | P1 V1 候補（PostgreSQL 15+ JSON Schema 拡張要） |

### ON-702 (P2) 🟡 — `feedback.feedback.metadata_json` のキー統一

| 項目 | 内容 |
|---|---|
| **発見箇所** | §4.22.1 `metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb` |
| **問題** | 自由 JSON。Agent 間で意味がブレる |
| **推奨** | API Design 側で `feedback.metadata` Schema 定義（zod / OpenAPI） |
| **適用方針** | P2 V1 候補 |

---

## 9. フロントエンド型 / Backend スキーマ マッピング課題

### ON-801 (P2) 🟡 — `frontend/src/types/ids.ts` の型衝突

| 項目 | 内容 |
|---|---|
| **発見箇所** | `frontend/src/types/ids.ts` (36252 bytes) |
| **問題** | 各 ID に対して複数の型エイリアス（`TenantId` / `TenantID` / `tenant_id`）が混在し grep 困難 |
| **推奨** | 命名規約統一: `{Resource}Id` PascalCase 単一表現、`frontend/src/types/ids.ts` 1 ファイルに集約 |
| **適用方針** | P2 V1 候補 |

### ON-802 (P2) 🟡 — `frontend/src/mocks/schemas/*.ts` と PG テーブルの 1:1 マッピング欠如

| 項目 | 内容 |
|---|---|
| **発見箇所** | `frontend/src/mocks/schemas/*.ts` (11 ファイル) |
| **問題** | Zod schema は Backend テーブルと 1:1 対応するも、命名 / 列差異あり。`mocks/schemas/five-domain.ts` は Backend 5 域と非対応 |
| **推奨** | IPA `frontend/` 配下に Backend → Frontend マッピング表作成（2026-09-01 15:30 JST 拍板による `frontend/README.md` 着手） |
| **適用方針** | P2 本ファイル着手（`frontend/README.md` 参照） |

### ON-803 (P3) 🟡 — `frontend/src/lib/store.ts` Zustand state shape と PG テーブル乖離

| 項目 | 内容 |
|---|---|
| **発見箇所** | `frontend/src/lib/store.ts` (23336 bytes) |
| **問題** | 23KB のモノリシック store、Backend テーブルと一部対応 / 一部独自フィールドあり |
| **推奨** | Zustand slice 分割 + 各 slice → Backend テーブル対応明示（`frontend/store-shapes.md` 着手） |
| **適用方針** | P3 DDD Review 待ち |

---

## 10. 全体サマリ

| 優先度 | 件数 | 摘要 |
|---|---|---|
| P0 | 1 | 整合性破壊（即時修正） |
| P1 | 13 | 次升版 v0.3 候補 |
| P2 | 14 | V1 候補（性能 / 容量 / V2 拡張） |
| P3 | 2 | DDD Review 5 域 Lead 拍板待ち |
| **合計** | **30** | 提案数 |

### 10.1 即時適用候補（コミット準備中）

- **ON-001**: `worktree.worktree_status` Lookup Table の `WAITING_FEADBACK` 誤字 → `WAITING_FEEDBACK` 修正コミット予定

### 10.2 次升版で適用候補（要コメント / 議論）

- ON-002, ON-003, ON-004（命名統一）
- ON-101, ON-102, ON-104, ON-106（CHECK 強化）
- ON-201, ON-202, ON-205（INDEX 追加）
- ON-301, ON-302（RLS 命名 / 設定）
- ON-402, ON-404, ON-405（データ型整合）
- ON-501（パーティション戦略確定）
- ON-701（payload JSON Schema）

### 10.3 V1/V2 候補（V1 適用判断は DDD Review 後）

- ON-103, ON-203, ON-204, ON-303, ON-304, ON-401, ON-403, ON-405, ON-502, ON-503, ON-601, ON-602, ON-701, ON-702, ON-801, ON-802

### 10.4 DDD Review 待ち

- ON-603, ON-803

---

## 11. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：30 件の最適化提案（P0×1 / P1×13 / P2×14 / P3×2） | per 2026-09-01 15:30 JST Ulysses 拍板（optimize_depth=opt_dual） |

---
