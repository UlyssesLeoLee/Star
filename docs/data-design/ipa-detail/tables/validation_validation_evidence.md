# validation.validation_evidence — テーブル詳細設計書

> **テーブル ID**: T91
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.24.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T91 |
| **物理名** | `validation.validation_evidence` |
| **論理名** | 検証証拠 |
| **スキーマ** | `validation` |
| **Module** | `domain-validation` |
| **種別** | Weak Entity（`validation_result_id` 必須） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Validation 証拠。Object Storage Key 保持。6 種別（test_report / build_log / coverage_report / static_analysis / screenshot / log_excerpt）。`url_expires_at` で 預署名 URL 期限管理。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | idx | RLS 必須 |
| 3 | `validation_result_id` | 検証結果 ID | UUID | − | NO | − | − | `validation.validation_result(id)` ON DELETE CASCADE | idx | 親 Result |
| 4 | `evidence_type` | 証拠種別 | VARCHAR | 32 | NO | − | − | − | − | 6 値 |
| 5 | `storage_ref` | ストレージ参照 | VARCHAR | 2048 | NO | − | − | − | idx | Object Storage Key |
| 6 | `size_bytes` | サイズ | BIGINT | 8 | YES | `NULL` | − | − | − | バイト |
| 7 | `mime_type` | MIME 種別 | VARCHAR | 128 | YES | `NULL` | − | − | − | MIME 形式 |
| 8 | `url_expires_at` | URL 有効期限 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | 預署名 URL 期限 |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `validation_evidence_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `fk_validation_evidence_*` | FOREIGN KEY (2) | `tenant_id` / `validation_result_id` | 各親テーブル | CASCADE | − |
| `ck_evidence_type` | CHECK | `evidence_type` | `IN (6 値, §4.24.2)` | 6 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 説明 |
|---|---|---|---|
| `validation_evidence_pkey` | btree (PK) | `id` | 主キー |
| `idx_validation_evidence_tenant_validation` | btree | `(tenant_id, validation_result_id)` | テナント + Result |
| `idx_validation_evidence_storage_ref` | btree | `storage_ref` | OS キー検索 |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 3,000,000 |（1 Validation 平均 3 Evidence） |
| 1 年後 | 30,000,000 |
| 3 年後 | 300,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 500 B | 300,000,000 | 約 150 GB |

> 実体（Object Storage）は別管理：MVP 3 TB / 3 年後 300 TB

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `validation.validation_result` | `validation_result_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE validation.validation_evidence ENABLE ROW LEVEL SECURITY;
ALTER TABLE validation.validation_evidence FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_validation_evidence_tenant_isolation ON validation.validation_evidence
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
