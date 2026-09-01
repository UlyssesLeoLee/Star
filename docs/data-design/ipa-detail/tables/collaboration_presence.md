# collaboration.presence — テーブル詳細設計書

> **テーブル ID**: T53
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.17.1

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T53 |
| **物理名** | `collaboration.presence` |
| **論理名** | 在席（Presence） |
| **スキーマ** | `collaboration` |
| **Module** | `domain-collaboration` |
| **種別** | Entity（短 TTL） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | リアルタイム在席。TTL 5 分。Valkey キャッシュ（§R-RT-003 / §API-3.18）。3 状態。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `user_id` | ユーザ ID | UUID | − | NO | − | − | `identity.user(id)` (App) | − | idx | 在席ユーザ |
| 4 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` (App) | − | idx | 所属 PJ |
| 5 | `status` | 状態 | VARCHAR | 32 | NO | `'ONLINE'` | − | − | − | idx | 3 値 |
| 6 | `last_seen_at` | 最終確認 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | idx | ハートビート |
| 7 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |

> **注**: data-design §4.17.1 には `user_id` / `project_id` への FK 制約なし（App 層検証）、`uq_presence_tenant_user` 制約も未実装、§00-INVENTORY.md では UK 想定

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 | 説明 |
|---|---|---|---|---|
| `presence_pkey` | PRIMARY KEY | `id` | − | 主キー |
| `ck_presence_status` | CHECK | `status` | `IN ('ONLINE','AWAY','OFFLINE')` | 3 値 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `presence_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_presence_tenant_project_status` | btree | `(tenant_id, project_id, status, last_seen_at DESC)` | − | PJ + 状態 + 最終確認 |
| `idx_presence_user_active` | btree (PT) | `user_id` | `status = 'ONLINE'` | アクティブユーザ |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | 件数（アクティブ） |
|---|---|
| MVP | 5,000 |
| 1 年後 | 50,000 |
| 3 年後 | 500,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 200 B | 500,000 | 約 100 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `identity.user` (App 検証) | `user_id` |
| `project.project` (App 検証) | `project_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE collaboration.presence ENABLE ROW LEVEL SECURITY;
ALTER TABLE collaboration.presence FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_presence_tenant_isolation ON collaboration.presence
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
