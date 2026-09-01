# 00-CLASSIFICATION-RULES.md — DB 三類横展開（Work / Transaction / Master）ルール手册 & 跨プロジェクト テンプレート

> **基準**: ユーザー指定 DB 三類横展開原則（2026-09-01 18:30 JST）
> **適用範囲**: 全プロジェクト（RGS / Star / Physis / GVPE / その他新規）の基本設計 DB 章
> **一次出典**: `D:\Star\docs\data-design\ipa-detail\00-CLASSIFICATION-W-T-M.md` v0.1（Star 100 テーブル実例）
> **本ファイル役割**: ルール本体 + 横展開派生規則 + 新規プロジェクト適用のチェックリスト
> **派生元**: 日本 IPA「データモデル詳細設計書」標準章立て + 「SEC ソフトウエア開発管理 ガイド」(IPA SEC)

---

## 0. 目的

ユーザー指示（2026-09-01 18:30 JST）:

> 「数据库表设计应该包含 Work、Transaction、master，分门别类管理，类似问题都要横展开细化，以上应该在基本设计里面包括进去，其他横展开内容可以根据日本 ipa 规则处理。」

を **跨プロジェクト永続ルール** として明文化する。本ファイルは以下を提供する:

1. 三類分類の **定義 / 判定規準**（§1）
2. **基本設計 DB 章** への組み込み方（§2）
3. **類似多分類の横展開派生規則**（§3 — IPA SEC 規則準拠）
4. **新規プロジェクト適用チェックリスト**（§4 — Physis / GVPE 等の新 DB 設計で即利用可）
5. **Star プロジェクト実例参照**（§5 — `00-CLASSIFICATION-W-T-M.md` への クロスリファレンス）
6. **守門派生 10 条**（§6 — DDD Review 段階 / 基本設計 レビュー段階 で適用）

---

## 1. 三類定義 & 判定規準

### 1.1 三類定義

| 業務分類 | 略 | 定義 | ライフサイクル | 削除方針 | 典型例 |
|---|---|---|---|---|---|
| **Work** | **W** | 作業中 / プロセス中の中間データ、session-bound、完了後クリーンアップ | 短 TTL（数分〜数時間）/ 短命 | 物理削除 / タイマー失効 / 完了時 clear | セッション, 観測ログ, Webhook 受信, リアルタイム在席, 排他制御 |
| **Transaction** | **T** | 業務事実 / イベント / 状態変更の記録, append-only または高頻度 R/W SoR | 中〜長期（数ヶ月〜永久） | 論理削除 + 監査保持 / Append-only 不変 | WorkItem, Project, PR, Comment, 監査ログ, 業務イベント, 意思決定 |
| **Master** | **M** | 参考データ / 設定 / テンプレート / 慢変参照データ | 永続 / SCD（Slowly Changing Dimension）戦略適用 | 論理削除 / 物理削除禁止（業務 FK 整合性） | Tenant, User, Role, Permission, Lookup enum, 構成情報, テンプレート |

### 1.2 判定フロー（Decision Tree）

```
新規テーブルを設計中...

Q1: 多数テーブルの FK 参照先?  構成情報?  ゆっくり変化?  物理削除で FK 連鎖 violate?
   → 2/4 YES → Master (M)

Q1 NO →
Q2: 業務事実の記録?  状態変更 / ライフサイクル遷移?  Append-only / 監査要件?
   → 2/4 YES → Transaction (T)

Q2 NO →
Q3: 短 TTL?  session-bound?  完了後クリーンアップ?  非業務事実の観測値?
   → 2/5 YES → Work (W)

Q3 NO →
   → 判定保留: 設計レビューで Lead 確認
```

### 1.3 判定マトリクス（複合指標）

| 指標 | Master (M) | Transaction (T) | Work (W) |
|---|---|---|---|
| **FK 参照元数** | 多い（≥ 3 業務表） | 中（1-3 業務表） | 少（≤ 1） |
| **書き込み頻度** | 低（構成変更時のみ） | 中〜高（業務イベントごと） | 高（観測 / 制御ごと） |
| **保持期間** | 永続（SCD 適用） | 数ヶ月〜永久 | 数分〜数時間 |
| **削除時の影響** | 業務整合性破壊 | 監査 / 履歴損失 | 影響なし（再生成可） |
| **物理削除** | 禁止 | 禁止（論理削除 / Append-only） | 推奨（タイマー / retention job） |
| **RLS 必須** | Yes（13 類 tenant_id 必携对象） | Yes | Yes または基表 RLS 伝播 |
| **監査ログ** | 構成変更時のみ | 必須（CRUD 全記録） | 任意（開始 / 終了のみ） |
| **典型実装** | Entity + Lookup | Entity + Weak + Append-only | Entity(短 TTL) + Projection + MV |

---

## 2. 基本設計 DB 章への組み込み

### 2.1 章立て構成（IPA 標準「データモデル詳細設計書」準拠）

```
第N章 データモデル詳細設計
  N.1 データモデル概要
    N.1.1 業務分類三類（Work / Transaction / Master）の定義
    N.1.2 業務分類 × R/W 識別 × パーティション × RLS の対応マトリクス
  N.2 データ分類別設計標準
    N.2.1 Master 設計標準
    N.2.2 Transaction 設計標準
    N.2.3 Work 設計標準
  N.3 業務分類別のテーブル一覧
    N.3.1 Master 一覧（M 件 / 各表の基本情報 + 設計根拠）
    N.3.2 Transaction 一覧（T 件 / 同上）
    N.3.3 Work 一覧（W 件 / 同上）
  N.4 業務分類集計
    N.4.1 業務分類別 件数 / 比率
    N.4.2 Schema / Module 別 業務分類件数
    N.4.3 「種別」×「業務分類」クロステーブル
  N.5 派生守門（CW-01 〜 CW-10）
  N.6 既知の缺口 / 制約
```

### 2.2 Star プロジェクトでの実装例

`D:\Star\docs\data-design\ipa-detail\00-CLASSIFICATION-W-T-M.md` v0.1 を参照:

- §1: 判定規準
- §2: 全 100 テーブル 三類分類（25 Schema × 4 tables 単位）
- §3: 集計（33 M / 47 T / 14 W + 6 混合 = 100 件）
- §4: 三類ごとの DB 設計標準パターン
- §5: 業務分類と R/W 識別 / パーティション / RLS の対応マトリクス
- §6: 派生守門 10 条

---

## 3. 横展開派生規則（IPA SEC 規則準拠）

ユーザー指示「类似问题都要横展开细化, 其他横展开内容可以根据日本 ipa 规则处理」に基づき、**多分類要素は W/T/M と同様の横展開原則**を適用する。

### 3.1 横展開原則

> **「合一禁止 / 分門別類 / 類似構造の独立列举」**

| 多分類要素 | 横展開細目 | 禁止パターン |
|---|---|---|
| **業務分類** | W / T / M | 「混在」一括列举禁止,必ず三類で分門別類 |
| **R/W 識別** | R/W(SoR) / R(Projection) / Append-only / R/W(短 TTL) | 1 つの R/W カテゴリに統合禁止, 4 種独立 |
| **データ種別** | Entity / Weak Entity / Lookup / Projection / MV / Append-only / Outbox | 1 つの「テーブル種別」に統合禁止, 7 種独立 |
| **削除方針** | 物理削除 / 論理削除 / Append-only 不変 / SCD Type 2 | 「削除」一括禁止, 4 方針独立 |
| **パーティション戦略** | なし / RANGE(`created_at`) 月次 / LIST(`tenant_id`) | 1 戦略に統合禁止, 用途別 3 戦略独立 |
| **RLS** | Yes(13 類) / No(Tenant 自体) / No(基表伝播) / −(Outbox 別管理) | 1 つの RLS フラグに統合禁止, 4 値独立 |
| **status 列挙** | 全 Lookup を **`lookup_{table}_{column}` 独立表** で実装 | PostgreSQL `ENUM` 型で硬直化禁止, 業務値変更容易性優先 |
| **policy 列挙** | `project_policy` / `tenant_policy` / `validation_policy` / `agent_policy` のように **Module 別独立** | 単一 `policy` 表に統合禁止, Module 自治 |
| **role / permission** | `role` + `permission_scheme`（多:多 関連） で実装 | `role.permission_codes TEXT[]` で配列突っ込み禁止, 関連表必須 |
| **event 記録** | `audit_event` / `agent_session_event` / `feedback_consumed_event` のように **Module 別独立 Append-only** | 単一 `event_log` に統合禁止, Module 自治 + 監査独立性 |
| **派生データ** | 業務事実は Entity, 派生は Projection / MV, 中間計算結果はキャッシュ層 (Valkey 等) | 業務表に JSONB で派生全部押し込み禁止 |
| **添付 / Blob** | 1MB 超は Object Storage, メタのみ PostgreSQL | PostgreSQL `bytea` で全部突っ込み禁止 |
| **tag / category** | 多:多 関連表 (`{table}_tag`) 実装 | 単一 `tags TEXT[]` 配列禁止, 関連独立 |

### 3.2 IPA SEC 規則（ソフトウェア開発管理ガイド）からの派生

| 規則 | 適用 |
|---|---|
| **トレーサビリティ** | 業務分類 M / T / W は要件（REQ-*）に紐付け可能でなければならない |
| **影響分析** | テーブル削除時、業務分類に基づく影響範囲を即座に判定できること |
| **変更管理** | M → T 昇格 / T → M 降格 は破壊的変更扱い、Migration 履歴保持必須 |
| **監査対応** | T 系は全 CRUD 監査、M 系は構成変更時のみ監査、W 系は任意 |
| **可用性目標** | M / T 系は HA 必須、W 系は best-effort で可 |
| **バックアップ戦略** | M / T 系は PITR 必須、W 系はバックアップ不要 / 派生再構築可 |
| **データ保持期間** | M = 永続、T = 業務要件依存、W = 明示的 retention 設定 |

### 3.3 禁止パターン 早見表

| 禁止 | 理由 | 推奨 |
|---|---|---|
| 「全部 Transaction」一括 | 分門別類漏れ | W / T / M 三類分離 |
| 「全部 Master」一括 | 業務事実の損失リスク | 業務事実は T 必須 |
| 「全部 Work」一括 | 永続データ損失 | 永続データは M / T 必須 |
| PostgreSQL `ENUM` で硬直化 | 値変更が高コスト | Lookup Table 実装 |
| `policy` 単一表に統合 | Module 自治喪失 | Module 別 `*_policy` 独立 |
| `event_log` 単一表に統合 | 監査独立性喪失 | Module 別 `*_event` 独立 |
| `tags TEXT[]` 配列 | クエリ性能 / 整合性崩壊 | 多:多 関連表 |
| `bytea` での Blob 突っ込み | DB 肥大 / 性能劣化 | Object Storage + Key 参照 |
| 業務表に JSONB 派生全押し | 業務事実の SoR 崩壊 | 派生は Projection / MV へ |
| `soft delete` 統一（Yes/Yes/Yes） | W 系は短命で不要 | 業務分類別の既定値 |

---

## 4. 新規プロジェクト適用チェックリスト

> Physis / GVPE / その他新規プロジェクトの基本設計 DB 章で使う即利用可チェックリスト。

### 4.1 設計着手時

- [ ] 全テーブル洗い出し後、**W / T / M 三類全て**を割り当てる
- [ ] **3 類とも 1 件以上**存在することを確認（欠落は「分門別類漏れ」）
- [ ] 各 Module 内で W / T / M の **混在状況** を確認、運用設計で TTL 差異を明示

### 4.2 Master 設計時

- [ ] R/W 識別 = R/W(SoR) を既定
- [ ] 物理削除禁止 / 論理削除（`deleted_at`）既定
- [ ] 13 類 tenant_id 必携对象 = Yes なら RLS 必須
- [ ] SCD Type 2（`valid_from` / `valid_to` / `is_current`）V1 候補
- [ ] FK `ON DELETE RESTRICT`（業務整合性保護）
- [ ] 構成変更時 audit_event 記録

### 4.3 Transaction 設計時

- [ ] R/W 識別 = R/W(SoR) 高頻 / Append-only を用途別に選択
- [ ] Append-only の場合: `created_at` のみ / `updated_at` なし / 物理削除禁止
- [ ] 時系列大（>1M 行想定）は RANGE(`created_at`) 月次パーティション
- [ ] 必須 audit: `created_by` / `created_at` / `updated_by` / `updated_at`
- [ ] 13 類 tenant_id 必携对象 = Yes なら RLS 必須
- [ ] 法的保持期間 / 業務要件保持期間 を明示
- [ ] FK `ON DELETE CASCADE` または `RESTRICT` を子 / 親役割で使い分け

### 4.4 Work 設計時

- [ ] R/W 識別 = R/W(短 TTL) / R(Projection) を用途別に選択
- [ ] 明示的 `retention_period` 列 + 物理削除ジョブ必須
- [ ] Projection / MV の場合: 基表 RLS 伝播, 自身は RLS 不要
- [ ] 短 TTL Entity の場合: `expires_at` 列 + タイマー削除
- [ ] 外部参照整合性: 弱参照推奨（FK 持ちだが CASCADE で自動消滅）
- [ ] 監査: session 開始 / 終了のみ記録で十分

### 4.5 集計 / レビュー時

- [ ] 業務分類別 件数 / 比率 表を必ず作成
- [ ] Schema / Module 別 業務分類件数 表を必ず作成
- [ ] 「種別」×「業務分類」クロステーブルで二重横軸整合確認
- [ ] DDD Review 段階 で 5 域 Lead / Module Lead が業務分類を承認
- [ ] 既知の缺口 を明示（混合分類 / V2 候補 / Frontend 同期 等）

### 4.6 IPA 規則派生チェック

- [ ] 多分類要素（status / role / permission / policy / event / tag / category）が **横展開列举** されている
- [ ] 合一禁止 パターンに抵触していない（§3.3 早見表）
- [ ] トレーサビリティ要件（REQ-* ↔ 業務分類 ↔ テーブル）確立
- [ ] 影響分析 / 変更管理 / 監査対応 / 可用性 / バックアップ / 保持期間 6 観点が §IPA SEC で網羅

---

## 5. クロスリファレンス

| ファイル | 役割 |
|---|---|
| `D:\Star\docs\data-design.md` v0.2 | Star プラットフォーム Data Design 一次出典,25 Schema 划分 + PostgreSQL DDL 完整 |
| `D:\Star\docs\data-design\ipa-detail\00-INVENTORY.md` v0.1 | Star 100 テーブル一覧（Schema / 種別 / 主キー / RLS） |
| `D:\Star\docs\data-design\ipa-detail\00-INDEXES.md` | Star インデックス一覧 |
| `D:\Star\docs\data-design\ipa-detail\00-CONSTRAINTS.md` | Star 制約一覧 |
| `D:\Star\docs\data-design\ipa-detail\00-CLASSIFICATION-W-T-M.md` v0.1 | Star 100 テーブル W/T/M 三類分類実例（§3.1 集計 / §3.2 Schema 別 / §3.3 クロステーブル） |
| `D:\Star\docs\data-design\ipa-detail\README.md` v0.1 | Star IPA 化フォルダ構成 + 使い方 |
| `D:\Star\docs\data-design\ipa-detail\templates\TABLE-TEMPLATE.md` | Star 個別テーブル詳細設計書 IPA 標準章立て雛形 |
| `D:\Star\AGENTS.md` | STAR プロジェクト AI 協作文脈 + 守門 |
| `D:\RustGameServer` (RGS 仓) | RGS プロジェクト,Star 仓 と完全独立（per AGENTS.md §5）— 本ルール適用可,別仓で管理 |

---

## 6. 派生守門（10 条）

| # | 派生守門 | 適用場面 | 確認者 |
|---|---|---|---|
| **CW-01** | 全テーブルに「業務分類 W/T/M」1 列を必ず割り当てる | 新規テーブル追加時 | 設計者 / Lead |
| **CW-02** | W / T / M の **3 類とも** 1 件以上存在しなければ「分門別類漏れ」 | Schema 単位 / Module 単位 | DDD Review Lead |
| **CW-03** | W が **0 件**の Module は短命データ不足の可能性,要確認（リアルタイム / 観測 / session 系テーブルの存在） | 設計レビュー時 | SRE Lead |
| **CW-04** | T が **0 件**の Module は業務事実の記録欠如,要確認（Write/Read 主体の構成 Module は例外） | 設計レビュー時 | Module Lead |
| **CW-05** | M は **13 類 tenant_id 必携对象 = Yes** を既定、RLS 必須 | Master 追加時 | SRE Lead |
| **CW-06** | T で時系列大（>1M 行想定）は RANGE(`created_at`) 月次パーティション必須 | 容量計画時 | SRE Lead |
| **CW-07** | W は明示的 `retention_period` 列 + 物理削除ジョブ必須 | Work 追加時 | SRE Lead |
| **CW-08** | 同一 Module 内に W / T / M が**混在する場合**,データライフサイクル差を運用設計に明示 | 設計レビュー時 | Module Lead + SRE Lead |
| **CW-09** | 他の横展開軸（enum / status / role / policy / permission / tag / category 等）も**全て三類分門別類**で列举,合一禁止 | 横展開一般（IPA 規則 §派生） | 設計者 / Lead |
| **CW-10** | 業務分類の変更（例: T → M 昇格）は破壊的変更扱い,Migration で履歴保持 | スキーマ変更時 | Module Lead + DDD Review |

---

## 7. 既知の缺口 / 制約

- **混合分類**: Star で 6 件存在（`workspace.workspace` / `planning.roadmap` / `audit.audit_event_outbox` / `local_runtime.runtime_observation` 等）,主分類で計上したが場合により細分化必要,DDD Review 阶段で Lead 確認推奨
- **V2 候補**: V2 化（symbol_index_snapshot / forgejo provider / Squad V2 など）で T → W 降格候補あり,Migration で履歴保持
- **Frontend 同期**: Backend PG のみ W/T/M 適用,Frontend Zustand store / MSW mock の状態分類は未同期,Frontend Design 章で別途横展要
- **新規プロジェクト（Physis / GVPE 等）への展開**: 本ファイル（§4 チェックリスト）は汎用, Physis / GVPE 等の新 DB 設計着手時に本ファイルを **必ず** コピー / 引用して適用
- **IPA SEC 規則**: 「ソフトウェア開発管理ガイド」/「SEC ソフトウエア開発 プロセス self-check」等他 IPA 文書の参照は本ファイルで要約, 詳細リンクは Implementation 段階で補完
- **Physis 等のリアルタイム性能制約**: Physis はゲーム物理エンジン, 1ms 以下の hot path テーブルは W 寄りだが Write 自体が hot path になる可能性, 業務分類と性能特性の二軸評価必要

---

## 8. 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 跨プロジェクト ルール手册（§1-§4）+ IPA 規則派生（§3）+ 新規プロジェクト チェックリスト（§4）+ 派生守門 10 条（§6）+ 既知缺口（§7） | per 2026-09-01 18:30 JST Ulysses 指示「DB 表设计应包含 Work/Transaction/master, 分门别类, 类似问题横展开细化, 其他横展内容按日本 IPA 规则处理」 |
