# テーブル詳細設計書 — テンプレート

> **基準**: 日本 IPA データモデル詳細設計書 標準構成
> **適用対象**: Star プラットフォーム 全 PostgreSQL テーブル（25 Schema / 78+ テーブル / Lookup / Projection / 物化ビュー）
> **参照元**: `D:\Star\docs\data-design.md` v0.2 §4 完整 DDL（一次出典 SoR）
> **本テンプレ役割**: IPA 標準の「テーブル詳細定義書」章立てへ整流する雛形

---

## 0. 概要

本テンプレートは、IPA（情報処理推進機構）が推奨する「データモデル詳細設計書」章立てに従い、Star プラットフォームの PostgreSQL テーブルを統一フォーマットで記述するための雛形である。実テーブルは本テンプレートを clone して `docs/data-design/ipa-detail/tables/` 配下へ `{schema}_{table}.md` のファイル名で配置する。

### 0.1 IPA 標準章立て対応

| IPA 章 | 本テンプレ章 | 必須度 | 説明 |
|---|---|---|---|
| 1. 概要 | 0 / 1 | 必須 | テーブル位置づけ・目的・種別 |
| 2. 基礎情報 | 1 | 必須 | 物理名 / 論理名 / スキーマ / 主キー / 概要 |
| 3. カラム一覧 | 2 | 必須 | 全列の属性・型・桁・NULL・PK・FK・UK・既定値・説明 |
| 4. 制約一覧 | 3 | 必須 | PK / FK / UK / CHECK |
| 5. インデックス一覧 | 4 | 必須 | ツリー / GIN / GiST / BRIN / 部分インデックス |
| 6. トリガー一覧 | 5 | 任意 | 自動採番 / updated_at / 整合性 |
| 7. 想定レコード件数 | 6 | 任意 | MVP / 1 年後 / 3 年後の想定件数 |
| 8. 想定容量 | 7 | 任意 | 1 行バイトサイズ × 想定件数 |
| 9. 関連テーブル | 8 | 必須 | 依存・被参照・兄弟 |
| 10. RLS Policy | 9 | 必須 | 13 類 tenant_id 必携对象は RLS 必須 |
| 11. 改訂履歴 | 10 | 必須 | v0.X + 改訂人 + 改訂内容 + 触发 |

### 0.2 命名規約

- **ファイル名**: `tables/{schema}_{table}.md`（snake_case）
- **例**: `tenant.tenant` → `tenant_tenant.md`、`work_item.work_item` → `work_item_work_item.md`
- **ディレクトリ**: 25 Schema 横断検索のため、Schema 別サブディレクトリは作らず `tables/` フラット配置

---

## 1. 基礎情報

| 項目 | 値 | 備考 |
|---|---|---|
| **テーブル ID** | TBD | 連番、INDEX と連動 |
| **物理名** | `{schema}.{table}` | PostgreSQL 修飾子付き |
| **論理名** | TBD | 日本語 / 業務名 |
| **スキーマ** | `{schema}` | 25 Schema のいずれか |
| **Module** | `domain-{module}` | 1:1 対応 |
| **種別** | Entity / Weak Entity / Projection / Materialized View / Lookup Table / Outbox | 継承 §3.7 / §3.6 |
| **主キー** | `id UUID`（既定） / 複合 PK | §3.1.2 |
| **R/W 識別** | R/W(SoR) / R(Projection) / Append-only / R/W(短TTL) | 継承 §1.3 |
| **RLS 必須** | Yes / No | 13 類 tenant_id 必携对象 = Yes |
| **パーティション** | None / RANGE(created_at) / LIST(tenant_id) | 継承 §9 |
| **soft delete** | Yes / No | デフォルト Yes（§3.1.5） |
| **概要** | TBD | 1〜2 行、業務上の役割 |

---

## 2. カラム一覧

> **桁数表示規約**:
> - `VARCHAR(n)`: 最大 n 文字（PostgreSQL は文字数ベース、n は文字数）
> - `NUMERIC(p,s)`: p = 精度（全桁）、s = スケール（小数桁）
> - `TEXT`: 制限なし（IPA 標準では「−」表記）
> - `UUID`: 128 bit 固定（IPA 標準では桁数「−」、バイトサイズ 16）
> - `JSONB`: 可変長（IPA 標準では「−」、上限なし）
> - `BOOLEAN`: 1 byte（IPA 標準では「1」）
> - `TIMESTAMPTZ`: 8 byte（IPA 標準では「8」）
> - `DATE`: 4 byte
> - `INTEGER`: 4 byte
> - `BIGINT`: 8 byte

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 | 備考 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | レコード識別子（UUID v7 推奨、App 側生成） | §3.1.2 / §2.3 |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | ✓→`tenant.tenant(id)` | − | idx | マルチテナント分離キー、RLS 必須 | §7 / §3.1.2 |
| 3 | ... | ... | ... | ... | ... | ... | ... | ... | ... | ... | ... | ... |

> **FK 表記**: `fk_xxx` 形式（`fk_work_item_project` 等）。複合 FK の場合は `(col_a, col_b) → ref_table(a, b)`
> **Index 表記**:
> - `PK` = 主キーインデックス
> - `uq_xxx` = ユニーク制約兼インデックス
> - `idx_xxx` = 業務インデックス
> - `gin` / `gist` / `brin` = 特殊インデックス種別

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 参照 / 条件 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `pk_{table}` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_{table}_{ref}` | FOREIGN KEY | `{fk_col}` | `{schema}.{ref_table}({ref_col})` | RESTRICT / CASCADE / SET NULL | 外部キー |
| `uq_{table}_{col}` | UNIQUE | `{col}` | − | − | 一意制約 |
| `ck_{table}_{col}` | CHECK | `{col}` | `{条件式}` | − | 値域制約 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 包含列 | 条件 (`WHERE`) | 説明 |
|---|---|---|---|---|---|
| `pk_{table}` | btree (PK) | `id` | − | − | 主キー |
| `idx_{table}_{col}` | btree | `(col1, col2, ...)` | − | `deleted_at IS NULL` | 業務検索用 |
| `idx_{table}_{col}_gin` | GIN | `(col)` | − | − | JSONB / tsvector / 配列検索 |
| `idx_{table}_{col}_gist` | GiST | `(col)` | − | − | ltree / 幾何 / 全文曖昧 |
| `idx_{table}_{col}_brin` | BRIN | `(col)` | − | − | 大規模時系列 |

---

## 5. トリガー一覧

| トリガー名 | 発火 | レベル | 関数 | 説明 |
|---|---|---|---|---|
| `trg_{table}_updated_at` | BEFORE UPDATE | ROW | `public.fn_update_updated_at()` | `updated_at = NOW()` 自動更新 |
| ... | ... | ... | ... | ... |

> 関数本体は Implementation 段階。本設計書は参照のみ。

---

## 6. 想定レコード件数

| フェーズ | 件数 | 根拠 |
|---|---|---|
| MVP（ローンチ半年） | TBD | 1 テナント / ○○ 件 |
| 1 年後 | TBD | 1 テナント / ○○ 件 |
| 3 年後（飽和） | TBD | 1 テナント / ○○ 件 × 想定テナント数 |

> MVP 段階は TBD-MEASURE 标注。SRE 監視で実測後 1 ヶ月で具体化。

---

## 7. 想定容量

| 1 行バイト（推定） | 想定件数 | 想定容量 | 備考 |
|---|---|---|---|
| TBD | TBD | TBD | 1 行 = SUM(列バイト + 28 byte PostgreSQL overhead) |

---

## 8. 関連テーブル

### 8.1 依存先（このテーブルが参照）

| 参照先 | FK 列 | 関係 | カーディナリティ |
|---|---|---|---|
| `{schema}.{table}` | `{fk_col}` | N:1 | N |

### 8.2 被参照元（このテーブルを参照）

| 被参照元 | FK 列 | 関係 | カーディナリティ |
|---|---|---|---|
| `{schema}.{table}` | `{fk_col}` | 1:N | 1 |

### 8.3 兄弟・関連（同 Module / 跨 Module）

- `tables/{schema}_{sibling}.md`
- `tables/{schema}_{related}.md`

---

## 9. RLS Policy

> **13 類 tenant_id 必携对象（§7.4）の場合**：

```sql
ALTER TABLE {schema}.{table} ENABLE ROW LEVEL SECURITY;
ALTER TABLE {schema}.{table} FORCE ROW LEVEL SECURITY;

CREATE POLICY {policy_name} ON {schema}.{table}
  USING (tenant_id = current_setting('app.current_tenant_id')::uuid);
```

> **Tenant 自体（`tenant.tenant`）は RLS 不要**（Tenant がテナント分離の源）。
> **Projection / Lookup Table / 物化ビュー** は基本 RLS 不要（基表 RLS が伝播）。
> **session GUC** は App 側で `SET LOCAL app.current_tenant_id = ...` を必ず実行。

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | TBD | TBD | 初版 | TBD |
| ... | ... | ... | ... | ... |

> **改訂人欄**: 自動代理 `Mavis 接手` (per `AGENTS.md` §1.1)、真人 Lead サインオフは DDD Review 段階で補完。
> **触发欄**: 必ず「per {commit hash}」または「per {doc short ref}」を記載し、禁則「per X 历史形态」を回避する。

---

## 附録 A: IPA 標準準拠チェックリスト

| 項目 | チェック | 備考 |
|---|---|---|
| 章立て 0-10 全項目存在 | ☐ | 必須 |
| 全列に物理名・論理名・型・桁・NULL・既定値・PK・FK・UK・Index・説明 | ☐ | 必須 |
| 全制約に制約名・種類・対象列・参照・ON DELETE・説明 | ☐ | 必須 |
| 全インデックスに名前・種別・キー列・包含列・条件・説明 | ☐ | 必須 |
| 想定レコード件数・容量に TBD-MEASURE 标注 | ☐ | MVP 段階 |
| 関連テーブルの依存・被参照・兄弟を全て列挙 | ☐ | 必須 |
| RLS Policy SQL 完全記述（13 類对象） | ☐ | 必須 |
| 改訂履歴に v0.1 + 改訂人 + 修订内容 + 触发 | ☐ | 必須 |
| 触发欄が commit hash / doc short ref で git 实证可能 | ☐ | 禁回溯叙事 |

---
