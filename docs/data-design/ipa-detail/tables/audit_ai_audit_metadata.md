# audit.ai_audit_metadata — テーブル詳細設計書

> **テーブル ID**: T31
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.11.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T31 |
| **物理名** | `audit.ai_audit_metadata` |
| **論理名** | AI 監査メタデータ（Append-only） |
| **スキーマ** | `audit` |
| **Module** | `domain-audit` |
| **種別** | **Append-only（A）**（§R-17 / §R-AUDIT-002） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **パーティション** | **RANGE (occurred_at) 月次** |
| **soft delete** | **No** |
| **概要** | AI Audit 9 問必答元データ。Full Prompt / Response は Object Storage 参照。`retention_until` で 90 日後自動削除（§6.8）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | − | − | idx | RLS 必須 |
| 3 | `audit_event_id` | 監査イベント ID | UUID | − | NO | − | − | `audit.audit_event(id)` ON DELETE RESTRICT | − | idx | 親 audit_event |
| 4 | `agent_session_id` | エージェントセッション ID | UUID | − | YES | `NULL` | − | `agent.agent_session(id)` (App) | − | idx | AI セッション |
| 5 | `context_packet_id` | コンテキストパケット ID | UUID | − | YES | `NULL` | − | `context.context_packet(id)` (App) | − | idx | 入力コンテキスト |
| 6 | `change_set_id` | 変更セット ID | UUID | − | YES | `NULL` | − | `development.change_set(id)` (App) | − | idx | 出力変更 |
| 7 | `validation_result_ids` | 検証結果 ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | 検証連動 |
| 8 | `feedback_consumed_ids` | 消費 FB ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | FB 消費追跡 |
| 9 | `approver_user_id` | 承認者 ID | UUID | − | YES | `NULL` | − | `identity.user(id)` (App) | − | − | Commit / PR / Merge 時 |
| 10 | `worktree_id` | ワークツリー ID | UUID | − | YES | `NULL` | − | `worktree.worktree(id)` (App) | − | − | AI 作業対象 |
| 11 | `agent_type` | エージェント種別 | VARCHAR | 64 | YES | `NULL` | − | − | − | − | `'AI_CODING'` 等 |
| 12 | `agent_provider` | エージェント Provider | VARCHAR | 64 | YES | `NULL` | − | − | − | − | `'openai'` / `'anthropic'` 等 |
| 13 | `agent_version` | エージェントバージョン | VARCHAR | 32 | YES | `NULL` | − | − | − | − | Agent バージョン |
| 14 | `full_prompt_ref` | Full Prompt 参照 | VARCHAR | 2048 | YES | `NULL` | − | − | − | − | Object Storage Key |
| 15 | `full_response_ref` | Full Response 参照 | VARCHAR | 2048 | YES | `NULL` | − | − | − | − | Object Storage Key |
| 16 | `retention_until` | 保持期限 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 既定 90 日、PJ Policy で調整可 |
| 17 | `is_redacted` | Redact 済 | BOOLEAN | 1 | NO | `FALSE` | − | − | − | − | 敏感コード Redact フラグ |
| 18 | `occurred_at` | 発生日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | BRIN | パーティションキー |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `ai_audit_metadata_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_ai_audit_event` | FOREIGN KEY | `audit_event_id` | `audit.audit_event(id)` | RESTRICT | 親 audit 削除禁止 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `ai_audit_metadata_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_ai_audit_tenant_occurred` | btree | `(tenant_id, occurred_at DESC)` | − | テナント + 時系列 |
| `idx_ai_audit_agent_session` | btree | `agent_session_id` | − | セッション別 |
| `idx_ai_audit_context_packet` | btree | `context_packet_id` | − | コンテキスト別 |
| `idx_ai_audit_change_set` | btree | `change_set_id` | − | 変更セット別 |
| `idx_ai_audit_validation_ids_gin` | GIN | `validation_result_ids` | − | 検証配列 |
| `idx_ai_audit_feedback_ids_gin` | GIN | `feedback_consumed_ids` | − | FB 配列 |
| `idx_ai_audit_occurred_brin` | BRIN | `occurred_at` | − | 大規模時系列 |

---

## 5. トリガー / 権限

| 種別 | 名前 | 説明 |
|---|---|---|
| REVOKE | `UPDATE, DELETE ON audit.ai_audit_metadata FROM PUBLIC` | WORM 強制 |

---

## 6. 想定レコード件数

| フェーズ | 件数 / 月 |
|---|---|
| MVP | 100,000 |
| 1 年後 | 1,000,000 |
| 3 年後 | 10,000,000 |

---

## 7. 想定容量

| 1 行バイト | 月次件数 | 月次容量 |
|---|---|---|
| 約 800 B | 1,000,000 | 約 800 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `audit.audit_event` | `audit_event_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE audit.ai_audit_metadata ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit.ai_audit_metadata FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_ai_audit_metadata_tenant_isolation ON audit.ai_audit_metadata
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
