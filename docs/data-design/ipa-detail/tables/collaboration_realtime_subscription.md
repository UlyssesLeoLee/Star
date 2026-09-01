# collaboration.realtime_subscription — テーブル詳細設計書

> **テーブル ID**: T54
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.17.2

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T54 |
| **物理名** | `collaboration.realtime_subscription` |
| **論理名** | リアルタイム購読 |
| **スキーマ** | `collaboration` |
| **Module** | `domain-collaboration` |
| **種別** | Entity（短 TTL） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | WebSocket Subscription 登録。7 日 TTL。`filter JSONB` で 購読フィルタ（resource_types / project_id / event_types）。`last_event_id` で resume。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `user_id` | ユーザ ID | UUID | − | NO | − | − | `identity.user(id)` (App) | − | idx | 購読者 |
| 4 | `filter` | 購読フィルタ | JSONB | − | NO | `'{}'::jsonb` | − | − | − | − | フィルタ式 |
| 5 | `last_event_id` | 最終イベント ID | UUID | − | YES | `NULL` | − | − | − | − | resume 用 |
| 6 | `is_active` | アクティブ | BOOLEAN | 1 | NO | `TRUE` | − | − | − | idx | 有効フラグ |
| 7 | `expires_at` | 有効期限 | TIMESTAMPTZ | 8 | NO | `NOW() + INTERVAL '7 days'` | − | − | − | idx | 7 日 TTL |
| 8 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 9 | `last_active_at` | 最終アクティブ | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | ハートビート |
| 10 | `closed_at` | 終了日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | − | 終了時刻 |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `realtime_subscription_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_realtime_subscription_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `realtime_subscription_pkey` | btree (PK) | `id` | − | 主キー |
| `idx_realtime_subscription_tenant_user` | btree (PT) | `(tenant_id, user_id)` | `is_active = TRUE` | テナント + アクティブ |
| `idx_realtime_subscription_expires` | btree (PT) | `(expires_at)` | `is_active = TRUE` | 期限監視 |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | アクティブ件数 |
|---|---|
| MVP | 5,000 |
| 1 年後 | 50,000 |
| 3 年後 | 500,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 400 B | 500,000 | 約 200 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `identity.user` (App 検証) | `user_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE collaboration.realtime_subscription ENABLE ROW LEVEL SECURITY;
ALTER TABLE collaboration.realtime_subscription FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_realtime_subscription_tenant_isolation ON collaboration.realtime_subscription
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
