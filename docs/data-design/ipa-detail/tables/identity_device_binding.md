# identity.device_binding — テーブル詳細設計書

> **テーブル ID**: T42
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.14.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T42 |
| **物理名** | `identity.device_binding` |
| **論理名** | デバイス三重バインディング |
| **スキーマ** | `identity` |
| **Module** | `domain-identity` |
| **種別** | Entity（Device ↔ Project 多対多） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Device ↔ Project バインド。`allowed_repositories UUID[]` で SCM 範囲限定。`unbound_at IS NULL` でアクティブなバインディング 1 件保証。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `device_id` | デバイス ID | UUID | − | NO | − | − | `identity.device(id)` ON DELETE CASCADE | − | idx | 親 Device |
| 4 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | バインド先 PJ |
| 5 | `allowed_repositories` | 許可リポジトリ ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | GIN | SCM 範囲限定 |
| 6 | `bound_at` | バインド日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | バインド時刻 |
| 7 | `unbound_at` | アンバインド日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | ✓ | `uq_device_binding_active` | NULL = アクティブ |
| 8 | `bound_by_user_id` | バインド実行者 ID | UUID | − | NO | − | − | `identity.user(id)` ON DELETE RESTRICT | − | − | 通常 Tenant Admin |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `device_binding_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_device_binding_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_device_binding_device` | FOREIGN KEY | `device_id` | `identity.device(id)` | CASCADE | − |
| `fk_device_binding_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |
| `fk_device_binding_user` | FOREIGN KEY | `bound_by_user_id` | `identity.user(id)` | RESTRICT | − |
| `uq_device_binding_active` | UNIQUE (PT) | `(device_id, project_id)` | `WHERE unbound_at IS NULL` | − | アクティブ 1 件保証 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `device_binding_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_device_binding_active` | btree (UK/PT) | `(device_id, project_id)` | `unbound_at IS NULL` | アクティブ 1 件 |
| `idx_device_binding_tenant_device` | btree | `(tenant_id, device_id)` | − | デバイス別 |
| `idx_device_binding_tenant_project` | btree | `(tenant_id, project_id)` | − | PJ 別 |

---

## 5. トリガー一覧

なし

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 50,000 |
| 1 年後 | 500,000 |
| 3 年後 | 5,000,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 500 B | 5,000,000 | 約 2.5 GB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `identity.device` | `device_id` |
| `project.project` | `project_id` |
| `identity.user` | `bound_by_user_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE identity.device_binding ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity.device_binding FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_device_binding_tenant_isolation ON identity.device_binding
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
