# context.provenance_entry — テーブル詳細設計書

> **テーブル ID**: T87
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.23.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T87 |
| **物理名** | `context.provenance_entry` |
| **論理名** | 系統エントリ（Provenance） |
| **スキーマ** | `context` |
| **Module** | `domain-context` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | ContextPacket Provenance。`source_type` 11 値（Requirement / AcceptanceCriterion / Decision / Feedback / File / Symbol / Test / ADR / FailedValidation / OpenFeedback / Skill）。Priority Layer 6 段階（P0-P5、P5 = Untrusted）。Skill は P5 必須（S5 落点）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | idx | RLS 必須 |
| 3 | `context_packet_id` | コンテキストパケット ID | UUID | − | NO | − | − | `context.context_packet(id)` ON DELETE CASCADE | idx | 親 Packet |
| 4 | `source_type` | ソース種別 | VARCHAR | 32 | NO | − | − | − | idx | 11 値 |
| 5 | `source_id` | ソース ID | UUID | YES | `NULL` | − | − | (App) | idx | 主源 ID |
| 6 | `source_sub_id` | サブソース ID | VARCHAR | 512 | YES | `NULL` | − | − | − | 子源 ID（Symbol パス等） |
| 7 | `version` | バージョン | BIGINT | 8 | NO | − | − | − | − | 置換追跡用 |
| 8 | `included_at_layer` | 組み込みレイヤ | VARCHAR | 8 | NO | − | − | − | − | 6 値（P0-P5） |
| 9 | `snippet` | スニペット | TEXT | − | YES | `NULL` | − | − | − | 引用部抜粋 |
| 10 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `provenance_entry_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_provenance_*` | FOREIGN KEY (2) | `tenant_id` / `context_packet_id` | 各親テーブル | CASCADE | − |
| `ck_provenance_source_type` | CHECK | `source_type` | `IN (11 値, §4.23.2)` | 11 値、`'Skill'` V2 候補 |
| `ck_provenance_layer` | CHECK | `included_at_layer` | `IN ('P0','P1','P2','P3','P4','P5')` | 6 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 説明 |
|---|---|---|---|
| `provenance_entry_pkey` | btree (PK) | `id` | 主キー |
| `idx_provenance_tenant_packet` | btree | `(tenant_id, context_packet_id)` | テナント + Packet |
| `idx_provenance_tenant_source` | btree | `(tenant_id, source_type, source_id)` | ソース別 |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 1,000,000 |
| 1 年後 | 10,000,000 |
| 3 年後 | 100,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 600 B | 100,000,000 | 約 60 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `context.context_packet` | `context_packet_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE context.provenance_entry ENABLE ROW LEVEL SECURITY;
ALTER TABLE context.provenance_entry FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_provenance_entry_tenant_isolation ON context.provenance_entry
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
