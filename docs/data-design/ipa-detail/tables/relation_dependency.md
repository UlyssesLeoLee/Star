# relation.dependency — テーブル詳細設計書

> **テーブル ID**: T24
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.8.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T24 |
| **物理名** | `relation.dependency` |
| **論理名** | 依存ビュー（Projection） |
| **スキーマ** | `relation` |
| **Module** | `domain-relation` |
| **種別** | **VIEW（Projection）**（MV） |
| **主キー** | なし（VIEW） |
| **RLS 必須** | **No**（基表 RLS 伝播） |
| **概要** | WorkItem 直接依存ビュー（`relation.relation` の `blocks` / `blocked_by` 派生）。`CREATE OR REPLACE VIEW` による派生 Read-Only ビュー。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `from_work_item_id` | 依存元 WorkItem ID | UUID | − | NO | (派生) | − | − | − | − | source_work_item_id の別名 |
| 2 | `to_work_item_id` | 依存先 WorkItem ID | UUID | − | NO | (派生) | − | − | − | − | target_work_item_id の別名 |
| 3 | `relation_type` | 関連種別 | VARCHAR | 16 | NO | (派生) | − | − | − | − | `'blocks'` / `'blocked_by'` のみ |
| 4 | `tenant_id` | テナント ID | UUID | − | NO | (派生) | − | − | − | − | RLS フィルタ用 |

> VIEW は DDL 上の固定スキーマなし。基表 `relation.relation` の列を参照。

---

## 3. VIEW 定義

```sql
CREATE OR REPLACE VIEW relation.dependency AS
SELECT
  source_work_item_id AS from_work_item_id,
  target_work_item_id AS to_work_item_id,
  relation_type,
  tenant_id
FROM relation.relation
WHERE deleted_at IS NULL
  AND relation_type IN ('blocks', 'blocked_by');

COMMENT ON VIEW relation.dependency IS 'WorkItem 直接依存(Projection);只読派生ビュー';
```

---

## 4. インデックス

> VIEW のため直接の INDEX なし。基表 `relation.relation` の INDEX（`idx_relation_source` / `idx_relation_target` / `idx_relation_tenant_project`）が透過的に利用される。

---

## 5. トリガー一覧

なし（VIEW はトリガー不可）

---

## 6. 想定レコード件数

基表 `relation.relation` のうち `relation_type IN ('blocks', 'blocked_by')` のサブセット。MVP 50,000 / 3 年後 5,000,000。

---

## 7. 想定容量

派生ビュー、容量は基表依存

---

## 8. 関連テーブル

### 8.1 依存先（基表）

| 参照先 | 関係 |
|---|---|
| `relation.relation` | 基表 |

### 8.2 被参照元

なし（末端 Read-Only VIEW）

---

## 9. RLS Policy

VIEW の RLS は基表に依存。`relation.relation` の RLS Policy が透過的に適用される。

```sql
-- 基表側の RLS（relation.relation 参照）
ALTER TABLE relation.relation ENABLE ROW LEVEL SECURITY;
CREATE POLICY policy_relation_tenant_isolation ON relation.relation
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
