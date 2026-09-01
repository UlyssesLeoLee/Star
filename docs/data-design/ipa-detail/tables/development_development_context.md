# development.development_context — テーブル詳細設計書

> **テーブル ID**: T71
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.19.9

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T71 |
| **物理名** | `development.development_context` |
| **論理名** | 開発コンテキスト（Projection） |
| **スキーマ** | `development` |
| **Module** | `domain-development` |
| **種別** | **Projection（P）**（派生） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Development Context Projection。`intent` + `files JSONB` + `symbols JSONB`。`context-build` worker 异步生成。Agent への context packet 入力源。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `execution_id` | 実行 ID | UUID | − | NO | − | − | `development.development_execution(id)` ON DELETE CASCADE | − | idx | 親 Execution |
| 4 | `intent` | 意図 | TEXT | − | YES | `NULL` | − | − | − | − | 業務意図 |
| 5 | `files` | ファイル配列 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | ファイル要約配列 |
| 6 | `symbols` | シンボル配列 | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 涉及 Symbol 配列 |
| 7 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | 索引鮮度 |

> **注**: data-design §4.19.9 には `created_at` 列なし、`updated_at` のみ

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `development_context_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_development_context_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_development_context_execution` | FOREIGN KEY | `execution_id` | `development.development_execution(id)` | CASCADE | 親 Execution 削除時 削除 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `development_context_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_development_context_tenant_execution` | btree | `(tenant_id, execution_id)` | − | テナント + Execution |

---

## 5. トリガー / Worker

| 種別 | 名前 / 戦略 | 説明 |
|---|---|---|
| Worker | `context-build` | `intent` + `files` + `symbols` を build |

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 100,000 |
| 1 年後 | 1,000,000 |
| 3 年後 | 10,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 1.5 KB (JSONB ×2 + intent) | 10,000,000 | 約 15 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `development.development_execution` | `execution_id` |

### 8.2 被参照元

| 被参照元 | FK 列 |
|---|---|
| `agent.agent_session` (reverse 配列) | 入力 context として |

---

## 9. RLS Policy

```sql
ALTER TABLE development.development_context ENABLE ROW LEVEL SECURITY;
ALTER TABLE development.development_context FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_development_context_tenant_isolation ON development.development_context
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
-- 注: WITH CHECK なし（Worker INSERT 専用）
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
