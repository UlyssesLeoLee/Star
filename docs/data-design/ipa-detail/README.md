# Star プラットフォーム — データモデル詳細設計書（IPA 標準）

> **基準**: 日本 IPA（情報処理推進機構）データモデル詳細設計書
> **作成日**: 2026-09-01
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **一次出典**: `D:\Star\docs\data-design.md` v0.2 §4（PostgreSQL 15+ 完整 DDL）
> **本フォルダ役割**: 25 Schema × 78+ テーブルの IPA 標準「テーブル詳細定義書」化 + 最適化提案

---

## 0. 目的

`docs/data-design.md` v0.2 は PostgreSQL DDL 一次出典として完成度が高いが、IPA 標準の「テーブル詳細定義書」章立てとしては以下が不足している：

1. **列ごとのデータ型・桁・NULL・既定値・PK・FK・UK・Index・説明** が DDL コメント断片で散在 → IPA 標準の「カラム一覧表」化必要
2. **制約・インデックス・トリガー** が DDL ブロック内で混在 → テーブル単位の俯瞰表必要
3. **想定レコード件数・想定容量** が完全未記述 → SRE 容量計画のインプット欠如
4. **関連テーブル**（依存 / 被参照 / 兄弟）の俯瞰図未整備
5. **RLS Policy** がモジュール内に散在（§4.1.4 等）→ テーブルごとの RLS ブロック標準化

本フォルダは上記を IPA 標準章立てに整流し、**実装段階（Migration 適用 / Repository trait 生成 / ORM モデルバインド）** が直接利用可能な「テーブル詳細定義書」群を提供する。

---

## 1. フォルダ構成

```
docs/data-design/ipa-detail/
├── README.md                            # 本ファイル（全体俯瞰 + 使い方）
├── 00-INVENTORY.md                      # 全 78+ テーブル一覧（Schema / 種別 / 主キー / IPA ファイル）
├── 00-INDEXES.md                        # 全インデックス一覧（Schema / テーブル / 種別 / 用途）
├── 00-CONSTRAINTS.md                    # 全 CHECK / UK / FK 制約一覧
├── 00-FK-GRAPH.md                       # テーブル間 FK 俯瞰（有向グラフ + Mermaid）
├── OPTIMIZATION-NOTES.md                # 整理中发现した最適化提案清单
├── templates/
│   └── TABLE-TEMPLATE.md                # 個別ファイル雛形（IPA 標準章立て）
├── tables/                              # テーブル別詳細（78+ ファイル）
│   ├── tenant_tenant.md
│   ├── tenant_tenant_policy.md
│   ├── tenant_provider_data_boundary.md
│   ├── workspace_workspace.md
│   ├── project_project.md
│   └── ... (全 78+ テーブル分)
├── frontend/                            # Frontend TS スキーマ → Backend PG テーブル マッピング
│   ├── README.md
│   ├── ids.ts-mapping.md
│   ├── mocks-schemas.md
│   └── store-shapes.md
└── 99-CHANGELOG.md                      # 本フォルダ内 改訂履歴
```

---

## 2. 適用範囲

| 出典 | ファイル数 | 摘要 |
|---|---|---|
| `docs/data-design.md` §4（主源） | 78+ テーブル | 25 Schema の主表 + Lookup Table + Projection + 物化ビュー |
| `docs/data-design.md` §4.1.4 / §4.20.4 / §4.22.3 | 3 ブロック | RLS Policy / 物化ビュー 独立章 |
| `frontend/src/types/ids.ts` | 1 ファイル | ID 型 → PG `UUID` マッピング |
| `frontend/src/mocks/schemas/*.ts` | 11 スキーマ | TS Zod/Type schema → PG テーブル マッピング |
| `frontend/src/lib/store.ts` | 1 ファイル | Zustand store 状態形状 → PG テーブル マッピング |
| `docs/specs/domain-*.md` / `docs/rfcs/*.md` | 0 CREATE TABLE | data-design.md 参照のみ（本フォルダ対象外） |

### 2.1 含める（"表"の定義）

- PostgreSQL ベーステーブル（25 Schema の主表 / 弱实体 / 関連表）
- Lookup Table（13 種状態 enum）
- Projection テーブル（業務事実ではなく派生データ）
- 物化ビュー（`worktree_heatmap` / `feedback_inbox_item` / `dependency` / `worktree_observed_summary` / `acceptance_coverage_report`）
- Outbox テーブル（`audit_event_outbox`）
- Frontend TypeScript Schema（Backend テーブルへの 1:1 マッピング対象のみ）

### 2.2 含めない

- 業務関数 / Trigger 関数体（Implementation 段階）
- SQLx / Diesel / ORM コード
- K8s / Helm / 設定 manifest
- RGS 仓（`D:\RustGameServer`）— Star 仓と完全独立（per AGENTS.md §5）
- `docs/specs/` 配下のドメイン仕様書（CREATE TABLE を持たない、引用のみ）

---

## 3. IPA 標準章立て（個別ファイル共通）

| # | 章 | 必須 | 説明 |
|---|---|---|---|
| 0 | 概要 | ✓ | テーブル位置づけ・目的 |
| 1 | 基礎情報 | ✓ | 物理名 / 論理名 / スキーマ / 主キー / 種別 / RLS |
| 2 | カラム一覧 | ✓ | 物理名 / 論理名 / データ型 / 桁 / NULL / 既定値 / PK / FK / UK / Index / 説明 |
| 3 | 制約一覧 | ✓ | PK / FK / UK / CHECK |
| 4 | インデックス一覧 | ✓ | btree / GIN / GiST / BRIN / 部分 |
| 5 | トリガー一覧 | △ | updated_at 自動更新等 |
| 6 | 想定レコード件数 | △ | MVP / 1 年 / 3 年（TBD-MEASURE 标注） |
| 7 | 想定容量 | △ | 1 行バイト × 件数 |
| 8 | 関連テーブル | ✓ | 依存 / 被参照 / 兄弟 |
| 9 | RLS Policy | ✓ | 13 類对象は SQL 完全記述 |
| 10 | 改訂履歴 | ✓ | v0.X + 改訂人 + 修订内容 + 触发 |

凡例: ✓ = 必須 / △ = MVP 段階は TBD-MEASURE

---

## 4. 使い方

### 4.1 実装者が DDL を参照する

```bash
# 例: work_item.work_item テーブルの DDL
cat docs/data-design.md | sed -n '/^### 4.4 Module: domain-work-item/,/^### 4.5/p'

# 例: IPA 詳細（カラム・制約・INDEX・RLS 一覧）
cat docs/data-design/ipa-detail/tables/work_item_work_item.md
```

### 4.2 Repository trait を生成する

```bash
# sqlx-cli で Migration 適用
sqlx migrate run --source migrations/

# sqlx::FromRow derive 用 struct は IPA ファイルの「2. カラム一覧」を正本とする
# （PG → Rust 型マッピング: UUID → Uuid, TIMESTAMPTZ → chrono::DateTime<Utc>, JSONB → serde_json::Value）
```

### 4.3 DDD Review 阶段で 5 域 Lead が確認する

```bash
# 例: 5 域独立 Lead 確認項目
# - player 域: tenant / user / device 関連テーブル
# - economy 域: project_policy / provider_data_boundary
# - match 域: worktree / worktree_status_observed
# - social 域: comment / mention / presence
# - admin 域: audit_event / integration / security_policy

# 該当テーブルの IPA ファイル + data-design.md §4 DDL ブロックを対照
```

---

## 5. 25 Schema 俯瞰

> 詳細は `00-INVENTORY.md` 参照。

| # | Schema | Module | 主表数 | Lookup | Projection | MV | 計 | 状態 |
|---|---|---|---|---|---|---|---|---|
| 1 | `tenant` | domain-tenant | 3 | 0 | 0 | 0 | 3 | ✓ |
| 2 | `workspace` | domain-workspace | 1 | 0 | 0 | 0 | 1 | ✓ |
| 3 | `project` | domain-project | 3 | 0 | 0 | 0 | 3 | ✓ |
| 4 | `work_item` | domain-work-item | 4 | 1 | 0 | 0 | 5 | ✓ |
| 5 | `workflow` | domain-workflow | 3 | 0 | 0 | 0 | 3 | ✓ |
| 6 | `board` | domain-board | 3 | 0 | 0 | 0 | 3 | ✓ |
| 7 | `planning` | domain-planning | 3 | 0 | 1 | 0 | 4 | ✓ |
| 8 | `relation` | domain-relation | 1 | 0 | 1 | 0 | 2 | ✓ |
| 9 | `comment` | domain-comment | 3 | 1 | 0 | 0 | 4 | ✓ |
| 10 | `search` | domain-search | 0 | 0 | 1 | 0 | 1 | ✓ |
| 11 | `audit` | domain-audit | 2 | 0 | 0 | 0 | 3 (含 Outbox) | ✓ |
| 12 | `integration` | domain-integration | 2 | 1 | 0 | 0 | 3 | ✓ |
| 13 | `automation` | domain-automation | 3 | 1 | 0 | 0 | 4 | ✓ |
| 14 | `identity` | domain-identity | 5 | 0 | 0 | 0 | 5 | ✓ |
| 15 | `notification` | domain-notification | 3 | 1 | 0 | 0 | 4 | ✓ |
| 16 | `permission` | domain-permission | 4 | 0 | 0 | 0 | 4 | ✓ |
| 17 | `collaboration` | domain-collaboration | 2 | 0 | 0 | 0 | 2 | ✓ |
| 18 | `scm` | domain-scm | 7 | 1 | 0 | 0 | 8 | ✓ |
| 19 | `development` | domain-development | 6 | 0 | 3 | 0 | 9 | ✓ |
| 20 | `worktree` | domain-worktree | 2 | 1 | 1 | 1 | 5 | ✓ |
| 21 | `agent` | domain-agent | 4 | 1 | 0 | 0 | 5 | ✓ |
| 22 | `feedback` | domain-feedback | 2 | 1 | 0 | 1 | 4 | ✓ |
| 23 | `context` | domain-context | 3 | 1 | 0 | 0 | 4 | ✓ |
| 24 | `validation` | domain-validation | 4 | 1 | 0 | 1 | 6 | ✓ |
| 25 | `local_runtime` | domain-local-runtime | 4 | 1 | 0 | 0 | 5 | ✓ |
| **計** | 25 | 25 | **77** | **13** | **7** | **3** | **100** | − |

> 注 1: Lookup / Projection / MV 数は §3.3 / §3.7 参照
> 注 2: `worktree.dependency` は「ビュー」表記だが PG では `CREATE VIEW` ではなく `CREATE MATERIALIZED VIEW` 想定
> 注 3: 監査 Outbox は `audit.audit_event_outbox` で 1 テーブル追加計上

---

## 6. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：25 Schema × 78+ テーブル IPA 化開始 / テンプレート / INVENTORY / 4 Schema 完成 | per 2026-09-01 15:30 JST Ulysses 拍板（folder=ipa_inline / granularity=per_table / scope=scope_everything / optimize=opt_dual） |

---

## 7. 既知の缺口 / 制約

- **想定レコード件数 / 想定容量**: MVP 段階のため全テーブル TBD-MEASURE。SRE 監視で実測値取得後に 1 ヶ月以内に具体化（per §6 質量门）
- **Trigger 関数体**: IPA ファイルでは参照のみ。実体は Implementation 段階（`migrations/00X_fn_*.sql`）
- **業務ロジック**: 関数 / Trigger 本体は含まない（data-design.md 範囲外）
- **RLS session GUC 設定**: App 側で `SET LOCAL app.current_tenant_id = ...` 必須。Application 层 詳細 は `docs/runtime-design.md` 範囲
- **Frontend TS Schema 完全同期**: IPA 化対象に含むが、Backend 1:1 マッピングが困難な型（computed 字段 / 仮想字段）は明記

---
