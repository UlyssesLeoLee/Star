# development.risk_signal — テーブル詳細設計書

> **テーブル ID**: T67
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.19.5

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T67 |
| **物理名** | `development.risk_signal` |
| **論理名** | リスクシグナル |
| **スキーマ** | `development` |
| **Module** | `domain-development` |
| **種別** | Entity |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Risk Signal。8 種 kind（LargeChange / GeneratedFile / SchemaChange / DependencyUpgrade / SecurityHint / TestCoverageDrop / ConflictRisk / AISelfClaim）、5 段階 severity（Info / Low / Medium / High / Critical）。`severity IN ('High','Critical')` 部分インデックスで高重大度優先。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `change_set_id` | 変更セット ID | UUID | − | NO | − | − | `development.change_set(id)` ON DELETE CASCADE | − | idx | 親 ChangeSet |
| 4 | `kind` | 種別 | VARCHAR | 32 | NO | − | − | − | − | − | 8 値 |
| 5 | `severity` | 重大度 | VARCHAR | 16 | NO | − | − | − | − | idx (PT) | 5 値 |
| 6 | `source` | ソース | VARCHAR | 32 | NO | − | − | − | − | − | 5 値 |
| 7 | `evidence` | 証拠 | TEXT | − | NO | − | − | − | − | − | 検出根拠 |
| 8 | `suggested_action` | 推奨アクション | TEXT | − | YES | `NULL` | − | − | − | − | 推奨対応 |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `risk_signal_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_risk_signal_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` ON DELETE CASCADE | − |
| `fk_risk_signal_change_set` | FOREIGN KEY | `change_set_id` | `development.change_set(id)` ON DELETE CASCADE | 親 ChangeSet 削除時 Risk 削除 |
| `ck_risk_signal_kind` | CHECK | `kind` | `IN ('LargeChange','GeneratedFile','SchemaChange','DependencyUpgrade','SecurityHint','TestCoverageDrop','ConflictRisk','AISelfClaim')` | 8 値 |
| `ck_risk_signal_severity` | CHECK | `severity` | `IN ('Info','Low','Medium','High','Critical')` | 5 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `risk_signal_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_risk_signal_tenant_changeset` | btree | `(tenant_id, change_set_id)` | − | テナント + ChangeSet |
| `idx_risk_signal_severity` | btree (PT) | `(tenant_id, severity)` | `severity IN ('High','Critical')` | 高重大度 |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 5,000,000 |
| 1 年後 | 50,000,000 |
| 3 年後 | 500,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 500 B | 500,000,000 | 約 250 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `development.change_set` | `change_set_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE development.risk_signal ENABLE ROW LEVEL SECURITY;
ALTER TABLE development.risk_signal FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_risk_signal_tenant_isolation ON development.risk_signal
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
