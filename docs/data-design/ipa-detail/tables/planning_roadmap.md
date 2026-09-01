# planning.roadmap — テーブル詳細設計書

> **テーブル ID**: T21
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `docs/data-design.md` v0.2.1 §4.7.3

---

## 1. 基礎情報

| 項目 | 値 |
|---|---|
| **テーブル ID** | T21 |
| **物理名** | `planning.roadmap` |
| **論理名** | ロードマップ（Projection） |
| **スキーマ** | `planning` |
| **Module** | `domain-planning` |
| **種別** | **Projection（P）**（派生、非 SoR） |
| **主キー** | `id UUID` |
| **RLS 必須** | **Yes** |
| **概要** | Roadmap Projection。`milestones JSONB` で四半期マイルストーン、`work_item_ids UUID[]` で関連 WorkItem。Worker projection role が非同期で refresh（§12 / §R-SEARCH-001）。 |

---

## 2. カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁 | NULL | デフォルト | PK | FK | UK | Index | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | ID | UUID | − | NO | `gen_random_uuid()` | ✓ | − | − | PK | − |
| 2 | `tenant_id` | テナント ID | UUID | − | NO | − | − | `tenant.tenant(id)` ON DELETE CASCADE | − | idx | RLS 必須 |
| 3 | `project_id` | プロジェクト ID | UUID | − | NO | − | − | `project.project(id)` ON DELETE CASCADE | − | idx | 所属 PJ（1:1） |
| 4 | `milestones` | マイルストーン | JSONB | − | NO | `'[]'::jsonb` | − | − | − | − | 四半期マイルストーン配列 |
| 5 | `work_item_ids` | 関連 WorkItem ID 配列 | UUID[] | − | NO | `'{}'::uuid[]` | − | − | − | − | Roadmap 関連 WorkItem |
| 6 | `business_goal_id` | 業務目標 ID | UUID | − | YES | `NULL` | − | `work_item.business_goal(id)` ON DELETE SET NULL | − | − | 紐付業務目標 |
| 7 | `fiscal_year` | 会計年度 | INT | 4 | YES | `NULL` | − | − | − | idx | ソート用 |
| 8 | `fiscal_quarter` | 会計四半期 | INT | 4 | YES | `NULL` | − | − | − | idx | ソート用 |
| 9 | `created_at` | 作成日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | − |
| 10 | `updated_at` | 更新日時 | TIMESTAMPTZ | 8 | NO | `NOW()` | − | − | − | − | Projection 更新時刻 |
| 11 | `deleted_at` | 削除日時 | TIMESTAMPTZ | 8 | YES | `NULL` | − | − | − | PT | 論理削除 |
| 12 | `version` | バージョン | INT | 4 | NO | `1` | − | − | − | − | 楽観ロック |

---

## 3. 制約一覧

| 制約名 | 種類 | 対象列 | 条件 / 参照 | ON DELETE | 説明 |
|---|---|---|---|---|---|
| `roadmap_pkey` | PRIMARY KEY | `id` | − | − | 主キー |
| `fk_roadmap_tenant` | FOREIGN KEY | `tenant_id` | `tenant.tenant(id)` | CASCADE | − |
| `fk_roadmap_project` | FOREIGN KEY | `project_id` | `project.project(id)` | CASCADE | − |
| `fk_roadmap_business_goal` | FOREIGN KEY | `business_goal_id` | `work_item.business_goal(id)` | SET NULL | − |
| `uq_roadmap_per_project` | UNIQUE | `(project_id, deleted_at)` | − | − | PJ 1:1 |

---

## 4. インデックス一覧

| インデックス名 | 種別 | キー列 | 条件 | 説明 |
|---|---|---|---|---|
| `roadmap_pkey` | btree (PK) | `id` | − | 主キー |
| `uq_roadmap_per_project` | btree (UK/PT) | `(project_id, deleted_at)` | − | PJ 1:1 |
| `idx_roadmap_tenant_project` | btree (PT) | `(tenant_id, project_id)` | `deleted_at IS NULL` | テナント + PJ |
| `idx_roadmap_tenant_quarters` | btree (PT) | `(tenant_id, fiscal_year, fiscal_quarter)` | `deleted_at IS NULL` | 年度/四半期ソート |

---

## 5. トリガー / Refresh 戦略

| 種別 | 名称 / 戦略 | 説明 |
|---|---|---|
| TRIGGER | `trg_roadmap_updated_at` | `updated_at` 自動更新 |
| Worker | 5 分間隔 refresh | `proj-runner projection roadmap` ジョブが `milestones` / `work_item_ids` を再計算 |

> **Projection 整合性**: `roadmap` の `updated_at` 鮮度で「最後に同期された時刻」を確認できる。SLA は 5 分以内（per §R-SEARCH-001 派生）。

---

## 6. 想定レコード件数

| フェーズ | 件数 |
|---|---|
| MVP | 1,000 |（1 PJ 1 Roadmap） |
| 1 年後 | 10,000 |
| 3 年後 | 100,000 |

---

## 7. 想定容量

| 1 行バイト | 想定件数 | 想定容量 |
|---|---|---|
| 約 2 KB (JSONB + 配列) | 100,000 | 約 200 MB |

---

## 8. 関連テーブル

### 8.1 依存先

| 参照先 | FK 列 |
|---|---|
| `tenant.tenant` | `tenant_id` |
| `project.project` | `project_id` |
| `work_item.business_goal` | `business_goal_id` |

### 8.2 被参照元

なし

---

## 9. RLS Policy

```sql
ALTER TABLE planning.roadmap ENABLE ROW LEVEL SECURITY;
ALTER TABLE planning.roadmap FORCE ROW LEVEL SECURITY;

CREATE POLICY policy_roadmap_tenant_isolation ON planning.roadmap
  USING (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID)
  WITH CHECK (tenant_id = current_setting('app.current_tenant_id', TRUE)::UUID);
```

---

## 10. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | per 2026-09-01 15:30 JST Ulysses 拍板 |
