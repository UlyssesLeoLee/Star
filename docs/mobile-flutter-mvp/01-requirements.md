# Star Mobile Flutter MVP — 要件定義書

> **基準**: 日本 IPA（情報処理推進機構）SEC ソフトウェア要件定義書 標準章立て
> **作成日**: 2026-09-02
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **バージョン**: v1.1 (UAT 完全版, 2026-09-02 16:27 JST)
> **前身**: v1.0 (read-only, commit `6bd6aa2`, 2026-09-02 16:14 JST) → UAT 範囲追加により v1.1 へ全面書き換え
> **Pre-IPA 草稿**: `docs/architecture/2026-09-02-upgrade/spec/mobile/01-flutter-mvp-design.md` v0.1 (commit `bd4998e`)
> **上流要件定義書**: `D:\Star\docs\requirements.md` v2.0 (Star プラットフォーム全体)
> **上流基本設計書**: `D:\Star\docs\basic-design.md` v0.1 (Star プラットフォーム全体)

---

## §1 目的

本文書は、Star プラットフォーム（AI Coding Worktree Control Plane + Jira-class Work Management + SCM Integration）における**モバイルクライアント第一版（Android Flutter MVP）**の要件を定義する。**本バージョン v1.1 は UAT（User Acceptance Test）レベル**の完全版であり、v1.0 の read-only MVP（ログイン＋閲覧）から、**核心写操作（状態遷移 / コメント / フィールド編集）**、**オフラインキャッシュ（Drift/SQLite）**、**自前 WebSocket 推送**を含む包括的な UAT 対応版に拡張する。

- 上流：Star プラットフォーム要件定義書 v2.0（プラットフォーム要件の正本）
- 本書：モバイル UAT 要件定義（プラットフォーム要件のうち UAT スコープを取り出し、モバイル固有の追加要件で拡張）
- 下流：モバイル UAT 基本設計書（`02-basic-design.md` v1.1）→ モバイル UAT 詳細設計書（`03-detailed-design.md` v1.1）

---

## §2 適用範囲

### 2.1 In Scope（UAT で実装する）

| 領域 | 範囲 | 出典 |
|---|---|---|
| プラットフォーム | **Android のみ**（minSdk 24 / Android 7.0+） | per 2026-09-02 15:52 JST 発令「安卓版」 |
| 通信プロトコル | REST + WebSocket 両対応 | `api-design.md` §1.1 + §4 |
| 認証 | JWT Bearer + Refresh Token | `api-design.md` §1.12 |
| **核心写操作** | **状態遷移 / コメント投稿 / フィールド編集** (Work Item) | per 2026-09-02 16:27 JST UAT 拍板 |
| **オフラインキャッシュ** | **Drift/SQLite + 同期キュー + 競合解決** | per 2026-09-02 16:27 JST UAT 拍板 |
| **リアルタイム推送** | **自前 WebSocket** (per `api-design.md` §4) | per 2026-09-02 16:27 JST UAT 拍板, FCM 不可 (ADR-0021) |
| Tablet 対応 | **対応しない**（スマホ縦画面のみ） | per 9/2 デフォルト |
| 多言語 | **中国語のみ**（i18n 対応なし） | per 内網利用 |

### 2.2 Out of Scope（UAT でも実装しない、後の版で対応）

| 領域 | 計画 | 参照 |
|---|---|---|
| iOS 対応 | V2 で対応 | `internal-design.md:1600` + V2 モバイル計画 |
| 添付ファイル Upload/Download | V1.2 で対応 | 容量/帯域/MIME 検証の複雑度 |
| Tablet / 横画面 | V1.2 で対応 | §11 既知未解決 |
| 多言語 i18n | V2 で対応 | 内部設計 §10 |
| 生体認証（指紋/顔認証） | V1.2 で対応 | 詳細設計で再評価 |
| Web 統合 SSO (OAuth/OIDC) | V1.2 で対応 | `api-design.md` §6.2 OAuth Phase 2+ |
| 外部 SDK（Crashlytics / Firebase） | 永久禁止 | ADR-0021 |
| Device 三重バインディング 厳密化 | V1.2 で対応 | `internal-design.md:23.2` |

---

## §3 前提条件・制約事項

### 3.1 前提条件

1. Star プラットフォーム本体（Backend / API / DB / WebSocket Service）が既に運用可能
2. WebSocket 推送エンドポイント（`wss://star.internal:8080/api/v1/ws`）が production 利用可能
3. WS サブスクリプション対象リソースに `work_item` と `notification` が含まれている（§11 G-16 で拍板待ち）
4. 対象ユーザーは Star テナントの既存ユーザーである
5. 利用は**企業内ネットワーク**（内網）に限定
6. 利用者は Android 7.0+ のスマートフォンを所有
7. 利用者は Star のメールアドレス + パスワードを既に保有

### 3.2 制約事項

| 制約 | 出典 |
|---|---|
| **零廠商合作**（外部 SDK 不使用） | `docs/architecture/2026-08-26-upgrade/adr/0021-zero-vendor-cooperation.md` |
| **5 域独立 Lead, 兼任禁止** | `AGENTS.md` §4 #3 + 8/21 JST 拍板 |
| **トークン予算制** | `AGENTS.md` §4 #4 + `STAR-OLU-001.md` v0.1 (1 SRE·周 = 1.2M) |
| **環境変数ハードコード禁止** | `AGENTS.md` §4 #5 + 8/27 11:06 JST hard ban |
| **AI ドキュメント治理解禁** | `AGENTS.md` §1.2 + §4 #12 |
| **V1 はモバイル App を範囲外**（本書は V1 範囲の**例外**として新規追加） | `docs/internal-design.md:50` + 2026-09-02 15:52 JST 発令 |
| **Mobile V2 計画は React Native 候補**（本書は Flutter 採用で V2 と並走） | `docs/internal-design.md:1633` |
| **FCM / Firebase 等の外部推送サービス使用禁止** | ADR-0021 + `api-design.md` §4 (自前 WebSocket のみ) |

---

## §4 システムの概要

### 4.1 システム構成図

```
┌────────────────────────────────────────────────────┐         ┌──────────────────────────────────┐
│   Android スマートフォン                            │         │   Star プラットフォーム           │
│  ┌──────────────────────────────────────────────┐  │         │                                  │
│  │ Flutter App (本 UAT MVP)                      │  │         │  ┌────────────────┐              │
│  │ - Riverpod 状態管理                            │  │  HTTP   │  │ API Gateway    │              │
│  │ - Dio HTTP クライアント                        │  │ ───────▶│  │ (Rust axum)    │              │
│  │ - web_socket_channel 推送受信                  │  │  REST   │  └────────────────┘              │
│  │ - Drift/SQLite オフラインキャッシュ            │  │ ◀───────│  ┌────────────────┐              │
│  │ - flutter_secure_storage 認証トークン          │  │         │  │ work-core      │              │
│  │ - 同期キュー + 競合解決                         │  │  WSS    │  │ (Rust Modular  │              │
│  │ - 接続性監視 (connectivity_plus)                │  │ ═══════▶│  │  Monolith)     │              │
│  │ - worktree 状態管理                            │  │ ◀═══════│  └────────────────┘              │
│  └──────────────────────────────────────────────┘  │  WS Push│  ┌────────────────┐              │
│                                 内網限定（HTTP/WS） │         │  │ WS Service     │              │
└────────────────────────────────────────────────────┘         │  │ (Axum WS)     │              │
                                                               │  └────────────────┘              │
                                                               │  ┌────────────────┐              │
                                                               │  │ PostgreSQL     │              │
                                                               │  │ (SoR)          │              │
                                                               │  └────────────────┘              │
                                                               └──────────────────────────────────┘
```

### 4.2 主要ユーザー像（Persona, `requirements.md` §3 から抜粋 + モバイル UAT 特化）

| Persona | モバイル UAT 利用シーン |
|---|---|
| **Developer（人間）** | 通勤中に Work Item の状態遷移実行、コメント投稿、フィールド編集 |
| **Product Owner / PM** | 移動中にボードの進捗確認、Sprint 内の Work Item 優先順位をその場で編集 |
| **Reviewer** | 移動中に PR レビュー依頼通知に気づき、モバイルで状態遷移実行、帰社後 PC で詳細 |
| **Tech Lead** | 会議中に Work Item の状態を確認、必要に応じてその場で状態遷移実行 |
| **オフライン ユーザー** | 電車内/地下/海外出張先などネット断絶環境でも、過去閲覧データを参照 + ローカル操作を後で同期 |

---

## §5 機能要件（Functional Requirements）

### 5.1 FR-AUTH: 認証機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-AUTH-001** | ユーザーはメールアドレス + パスワードでログインできる | P0 | v1.0 継承 |
| **FR-AUTH-002** | ログイン成功時、access_token（15 分有効）と refresh_token（7 日有効）を受け取る | P0 | v1.0 継承 |
| **FR-AUTH-003** | access_token 期限切れ時、refresh_token で自動更新し、元の API 呼び出しを retry する | P0 | v1.0 継承 |
| **FR-AUTH-004** | refresh_token 期限切れ時、ユーザーのローカル資格情報をクリアしログイン画面に遷移する | P0 | v1.0 継承 |
| **FR-AUTH-005** | ログアウト時、ローカルの全資格情報 + オフラインキャッシュを完全削除し、ログアウト API を呼ぶ（best-effort） | P0 | **UAT 拡張**: ローカル削除範囲拡大 |
| **FR-AUTH-006** | ログイン状態は `flutter_secure_storage`（Android Keystore）に暗号化して保存する | P0 | v1.0 継承 |
| **FR-AUTH-007** | ログアウト時に未同期の SyncQueue が残っていた場合、ユーザーに確認ダイアログを表示する | P0 | **UAT 拡張** |
| FR-AUTH-008 | ❌ OAuth 2.0 は実装しない（V1.2 候補） | — | 保留 |
| FR-AUTH-009 | ❌ 生体認証は実装しない（V1.2 候補） | — | 保留 |
| FR-AUTH-010 | ❌ Device 三重バインディング（`internal-design.md:23.2`）は MVP スキップ（V1.2 で対応） | — | 保留 |

### 5.2 FR-BOARD: ボード閲覧機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-BOARD-001** | ユーザーはプロジェクト単位でボードを閲覧できる | P0 | v1.0 継承 |
| **FR-BOARD-002** | ボードはカンバン形式（横スクロール Columns）で表示される | P0 | v1.0 継承 |
| **FR-BOARD-003** | 各 Column には複数の Work Item カードが縦に並ぶ | P0 | v1.0 継承 |
| **FR-BOARD-004** | Work Item カードには title / assignee アバター / priority chip / 状態 chip / **最終更新時刻** が表示される | P0 | **UAT 拡張** |
| **FR-BOARD-005** | ボード画面は Pull-to-Refresh で再取得できる | P0 | v1.0 継承 |
| **FR-BOARD-006** | ボード画面に入った瞬間に最新データを取得する | P0 | v1.0 継承 |
| **FR-BOARD-007** | **カードに未同期の SyncQueue アイテムがある場合にバッジ表示** | P0 | **UAT 新規** |
| **FR-BOARD-008** | **WS 推送でボードが更新された場合、差分のみ部分更新**（全件取得しない） | P1 | **UAT 新規** |
| FR-BOARD-009 | ❌ カードのドラッグ&ドロップによる状態遷移は実装しない（V1.2 候補） | — | 保留 |
| FR-BOARD-010 | ❌ Column のリネーム / 並び替えは実装しない（V1.2 候補） | — | 保留 |

### 5.3 FR-WORK-ITEM: Work Item 詳細 + 編集機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-WORK-001** | ユーザーは Work Item 詳細を閲覧できる | P0 | v1.0 継承 |
| **FR-WORK-002** | 詳細画面は 3 タブ構成: Overview / Comments / Transitions | P0 | v1.0 継承 |
| **FR-WORK-003** | Overview タブには title, description, status, priority, assignee, reporter, due date, repository/worktree へのリンクが表示される | P0 | v1.0 継承 |
| **FR-WORK-004** | Comments タブにはコメント一覧（時系列降順）が表示される | P0 | v1.0 継承 |
| **FR-WORK-005** | Transitions タブには現在の状態から遷移可能な状態一覧が表示される | P0 | v1.0 継承 |
| **FR-WORK-006** | 詳細画面右上に「Web で開く」ボタンを配置し、Star Web の該当 Work Item 詳細ページに遷移する | P0 | v1.0 継承 |
| **FR-WORK-007** | **ユーザーは priority / assignee / due date / description フィールドをインライン編集できる** | P0 | **UAT 新規** |
| **FR-WORK-008** | **ユーザーは Comments タブからコメントを投稿できる** | P0 | **UAT 新規** |
| **FR-WORK-009** | **ユーザーは Transitions タブから状態遷移を実行できる** | P0 | **UAT 新規** |
| **FR-WORK-010** | **編集操作はオフライン時にローカルで記録され、接続回復時に同期される** | P0 | **UAT 新規** |
| **FR-WORK-011** | **編集が成功すると WS 推送で他クライアントにも即時反映** | P1 | **UAT 新規** |
| **FR-WORK-012** | **添付ファイル表示は実装するが Upload/Download は V1.2 まで保留** | P1 | **UAT 部分実装** |
| FR-WORK-013 | ❌ 添付ファイル Upload/Download は V1.2 | — | 保留 |

### 5.4 FR-NOTIF: 通知閲覧 + 推送機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-NOTIF-001** | ユーザーは自分宛の通知一覧を閲覧できる | P0 | v1.0 継承 |
| **FR-NOTIF-002** | 通知一覧は最新 20 件まで取得し、Pull-to-Refresh で追加取得できる | P0 | v1.0 継承 |
| **FR-NOTIF-003** | 30 秒間隔の REST ポーリングを**残置**（WS 切断時のフォールバック） | P1 | **UAT 修正**: 30s 維持 |
| **FR-NOTIF-004** | 通知をタップすると既読化される（`POST /v1/notifications/{id}:read`） | P0 | v1.0 継承 |
| **FR-NOTIF-005** | 「すべて既読」ボタンで全通知を一括既読化できる | P1 | v1.0 継承 |
| **FR-NOTIF-006** | 通知をタップすると、関連する Work Item 詳細画面に遷移する | P0 | v1.0 継承 |
| **FR-NOTIF-007** | 未読通知の件数をアプリアイコンバッジに表示する | P1 | v1.0 継承 |
| **FR-NOTIF-008** | **WebSocket 接続が確立している場合、サーバ推送でリアルタイムに通知を受信する** | P0 | **UAT 新規** |
| **FR-NOTIF-009** | **WebSocket 切断時は 30s REST ポーリングにフォールバックする** | P0 | **UAT 新規** |
| **FR-NOTIF-010** | **OS レベルプッシュ通知（NotificationChannel）は実装しない** | — | UAT で確認 (FCM 不可, ローカル通知のみ将来 V1.2) |
| FR-NOTIF-011 | ❌ 通知のフィルタリング / 検索は実装しない（V1.2 候補） | — | 保留 |

### 5.5 FR-OFFLINE: オフライン機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-OFFLINE-001** | **ネットワーク接続喪失を `connectivity_plus` で検知し、`AppLifecycleState` と組み合わせて offline モードに入る** | P0 | **UAT 新規** |
| **FR-OFFLINE-002** | **オフライン状態では、過去に閲覧した Work Item / Board / Comment をローカル SQLite から表示する** | P0 | **UAT 新規** |
| **FR-OFFLINE-003** | **オフライン状態でも Work Item の編集 / コメント投稿 / 状態遷移操作が可能** | P0 | **UAT 新規** |
| **FR-OFFLINE-004** | **オフライン中の操作は SyncQueue（Drift テーブル）にローカル保存される** | P0 | **UAT 新規** |
| **FR-OFFLINE-005** | **接続回復時、SyncQueue のアイテムが順次同期される** | P0 | **UAT 新規** |
| **FR-OFFLINE-006** | **同期中に競合が発生した場合、ユーザーに競合解決 UI を提示し、勝側を選択させる** | P0 | **UAT 新規** |
| **FR-OFFLINE-007** | **オフライン状態は UI 上に明示的に表示**（バナー / アプリアイコン色変化） | P0 | **UAT 新規** |
| **FR-OFFLINE-008** | **ローカル DB は SQLCipher で暗号化する** | P0 | **UAT 新規, 機微データ保護** |
| **FR-OFFLINE-009** | **ローカル DB の容量上限は 50MB とし、上限到達時は古い Work Item から自動削除** | P1 | **UAT 新規** |
| **FR-OFFLINE-010** | **手動同期ボタン: Settings / 各画面から明示的に同期トリガ可能** | P1 | **UAT 新規** |
| **FR-OFFLINE-011** | **同期状態（synced / syncing / failed / conflict 件数）をアプリ内で可視化** | P1 | **UAT 新規** |

### 5.6 FR-PROJ: プロジェクト選択機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-PROJ-001** | ユーザーは自分がメンバーであるプロジェクトの一覧を閲覧できる | P0 | v1.0 継承 |
| **FR-PROJ-002** | プロジェクトをタップすると該当プロジェクトのボード画面に遷移する | P0 | v1.0 継承 |
| **FR-PROJ-003** | プロジェクト一覧は最終アクセス時刻でソート表示される | P1 | v1.0 継承 |
| FR-PROJ-004 | ❌ プロジェクトの新規作成は実装しない（Web で実施） | — | 保留 |

### 5.7 FR-SETTINGS: 設定機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-SETTINGS-001** | ユーザーはテーマ（light / dark / system）を切り替えられる | P0 | v1.0 継承 |
| **FR-SETTINGS-002** | ユーザーはログアウトできる | P0 | v1.0 継承 (FR-AUTH-005) |
| **FR-SETTINGS-003** | 設定画面にアプリバージョン / ビルド番号 / **同期状態** / **未同期件数** が表示される | P0 | **UAT 拡張** |
| **FR-SETTINGS-004** | 設定画面に**「手動同期」ボタン**が表示される | P0 | **UAT 新規** |
| **FR-SETTINGS-005** | 設定画面に**「キャッシュクリア」ボタン**が表示される（ローカル DB + 同期キュー削除） | P0 | **UAT 新規** |
| **FR-SETTINGS-006** | 設定画面に**「ログ送信」ボタン**が表示される（クラッシュログ + API 失敗ログ） | P1 | **UAT 新規** |
| **FR-SETTINGS-007** | 設定画面に新バージョン通知バナー（`GET /v1/app-version`）が表示される | P2 | v1.0 継承 |
| **FR-SETTINGS-008** | 設定画面に**WS 接続状態**（接続中 / 切断 / 再接続中）が表示される | P1 | **UAT 新規** |

### 5.8 FR-NAV: ナビゲーション機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-NAV-001** | アプリ起動時にログイン状態を確認し、未ログインならログイン画面、ログイン済みなら最終アクセス画面を表示する | P0 | v1.0 継承 |
| **FR-NAV-002** | 認証必須画面には認証ガードが働き、未ログイン状態でアクセスするとログイン画面にリダイレクトされる | P0 | v1.0 継承 |
| **FR-NAV-003** | アプリがバックグラウンドから復帰した時、認証トークンの有効性を確認し、無効ならログイン画面に遷移する | P0 | v1.0 継承 |
| **FR-NAV-004** | アプリ起動時間（cold start → ボード表示まで）は 1.5 秒以内 | P1 | v1.0 継承 |
| **FR-NAV-005** | **ネットワーク接続状態に応じて画面下部に「オフライン」バナーが表示される** | P0 | **UAT 新規** |

### 5.9 FR-WS: WebSocket 推送機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-WS-001** | **ログイン成功時、自動的に `wss://star.internal:8080/api/v1/ws` への接続を確立する** | P0 | **UAT 新規** |
| **FR-WS-002** | **接続時に `subprotocol: star.v1` + `Authorization: Bearer <jwt>` を設定** | P0 | **UAT 新規, per `api-design.md` §4** |
| **FR-WS-003** | **接続時に subscribe メッセージを送信し、`work_item` / `notification` リソースタイプを購読** | P0 | **UAT 新規, §11 G-16 拍板待ち** |
| **FR-WS-004** | **サーバから ping を受信した場合、60s 以内に pong を返す** | P0 | **UAT 新規, per `api-design.md` §4.5** |
| **FR-WS-005** | **接続切断時、指数バックオフ（1s → 3s → 10s）で自動再接続** | P0 | **UAT 新規** |
| **FR-WS-006** | **再接続成功時、subscribe を再送する** | P0 | **UAT 新規** |
| **FR-WS-007** | **WebSocket 接続は TLS 1.2+ で暗号化**（内網限定 cleartext は不可、WS のみ HTTPS 必須） | P0 | **UAT 新規, セキュリティ要件** |
| **FR-WS-008** | **WebSocket 接続状態（接続中 / 切断 / 再接続中 / エラー）が `AppLifecycleState` と連動** | P0 | **UAT 新規** |

---

## §6 非機能要件（Non-Functional Requirements）

### 6.1 性能要件（NFR-PERF）

| ID | 要件 | 目標値 | 出典 |
|---|---|---|---|
| NFR-PERF-001 | アプリ cold start 時間（Pixel 6 想定） | ≤ 1.5s | v1.0 継承 |
| NFR-PERF-002 | ボード表示時間（API 取得 + レンダリング） | ≤ 2.0s | v1.0 継承 |
| NFR-PERF-003 | API 呼び出し P95 応答時間（内網） | ≤ 200ms | v1.0 継承 |
| NFR-PERF-004 | メモリ使用量（idle 時） | ≤ 120 MB | v1.0 継承 |
| NFR-PERF-005 | APK サイズ（リリースビルド、Obfuscate 後） | ≤ 40 MB | **UAT 修正** (Drift/SQLCipher で +10MB) |
| NFR-PERF-006 | バッテリー消費（30 分アクティブ利用） | ≤ 5% | v1.0 継承 |
| **NFR-PERF-007** | **オフライン状態の UI 応答時間（ローカル DB クエリ）** | **≤ 100ms** | **UAT 新規** |
| **NFR-PERF-008** | **WebSocket 再接続時間（切断から復元まで）** | **≤ 5s** | **UAT 新規** |
| **NFR-PERF-009** | **同期キュー内 1 件同期時間（local → server）** | **≤ 500ms** | **UAT 新規** |
| **NFR-PERF-010** | **WebSocket 推送受信から UI 更新まで** | **≤ 200ms** | **UAT 新規** |

### 6.2 可用性要件（NFR-AVAIL）

| ID | 要件 | 目標値 | 出典 |
|---|---|---|---|
| NFR-AVAIL-001 | アプリクラッシュ率 | ≤ 0.1% | v1.0 継承 |
| NFR-AVAIL-002 | 致命的バグ修正 SLA | 24 時間 | v1.0 継承 |
| **NFR-AVAIL-003** | **ネットワーク断絶時のオフライン稼働率** | **≥ 99%** | **UAT 新規** |
| **NFR-AVAIL-004** | **WebSocket 接続維持率（セッション中）** | **≥ 95%** | **UAT 新規** |

### 6.3 セキュリティ要件（NFR-SEC）

| ID | 要件 | 出典 |
|---|---|---|
| NFR-SEC-001 | access_token / refresh_token は Android Keystore 暗号化して保存する | v1.0 継承 |
| NFR-SEC-002 | REST 通信は MVP 段階では cleartext HTTP（内網限定）;外網公開時は HTTPS 必須 | v1.0 継承 |
| **NFR-SEC-003** | **WebSocket 通信は WSS (TLS 1.2+) 必須、cleartext WS は不可** | **UAT 新規, FCM 不可のため WS は HTTPS 化** |
| NFR-SEC-004 | 通信先は `network_security_config.xml` で `star.internal` ドメインのみ cleartext 許可、他は拒否 | v1.0 継承 |
| NFR-SEC-005 | ログイン画面でスクリーンショット無効化フラグ（`FLAG_SECURE`）を設定する | v1.0 継承 |
| NFR-SEC-006 | ログアウト時にローカル資格情報 + オフラインキャッシュを完全削除する | **UAT 拡張** |
| NFR-SEC-007 | APK は `obfuscate` + `split-debug-info` ビルドを必須とする | v1.0 継承 |
| NFR-SEC-008 | 外部 SDK / 解析サービス（Firebase / Crashlytics / AppsFlyer 等）は使用禁止 | ADR-0021 |
| NFR-SEC-009 | API Key 等の秘匿情報はコード / 設定ファイルにハードコード禁止 | AGENTS.md §4 #5 |
| NFR-SEC-010 | tenant_id はクライアントから送信せず、API Gateway が JWT から抽出 | `api-design.md` §1.8 |
| **NFR-SEC-011** | **ローカル DB（Drift）は SQLCipher で暗号化する** | **UAT 新規** |
| **NFR-SEC-012** | **ログ送信時、トークン / パスワード / PII を自動 redact する** | **UAT 新規** |

### 6.4 保守性要件（NFR-MAINT）

| ID | 要件 | 出典 |
|---|---|---|
| NFR-MAINT-001 | `flutter analyze --fatal-infos` 0 warning | v1.0 継承 |
| NFR-MAINT-002 | `dart format` 100% pass | v1.0 継承 |
| NFR-MAINT-003 | ユニットテストカバレッジ: ≥ 70% | v1.0 継承 |
| NFR-MAINT-004 | ウィジェットテスト: 主要画面 100% カバレッジ | v1.0 継承 |
| NFR-MAINT-005 | Lint ルール: `very_good_analysis` 採用 | v1.0 継承 |
| NFR-MAINT-006 | コードレビュー: 5 域独立 Lead 承認必須 | AGENTS.md §4 #3 |
| **NFR-MAINT-007** | **統合テスト: 主要 UAT シナリオ 100% カバレッジ** | **UAT 新規** |
| **NFR-MAINT-008** | **オフライン / 競合解決 / WS 切断 シナリオのカバレッジ** | **UAT 新規** |

### 6.5 移植性要件（NFR-PORT）

| ID | 要件 | 出典 |
|---|---|---|
| NFR-PORT-001 | Android: minSdk 24, targetSdk 34 | v1.0 継承 |
| NFR-PORT-002 | 異なる画面サイズ（4.7"〜6.7"）で正しくレイアウトされる | v1.0 継承 |
| NFR-PORT-003 | ❌ iOS 対応は V2 | v1.0 継承 |

### 6.6 ユーザビリティ要件（NFR-USE）

| ID | 要件 | 出典 |
|---|---|---|
| NFR-USE-001 | タップターゲット ≥ 48dp | v1.0 継承 |
| NFR-USE-002 | ライト/ダーク両モード対応 | v1.0 継承 |
| NFR-USE-003 | ネットワークエラー時のリトライ UI 提供 | v1.0 継承 |
| NFR-USE-004 | 空状態（Empty State）のイラスト + 案内文提供 | v1.0 継承 |
| NFR-USE-005 | ローディング中は進捗インジケータ表示 | v1.0 継承 |
| **NFR-USE-006** | **同期中の進捗（プログレスバー / 件数表示）を表示** | **UAT 新規** |
| **NFR-USE-007** | **競合解決 UI は差分を左右に並べて表示、ユーザーが明示的に選択** | **UAT 新規** |

---

## §7 業務フロー / ユースケース

### 7.1 UC-001: ログイン（v1.0 継承）

```
[User] → アプリ起動 → 未ログイン状態
   → ログイン画面で email + password 入力
   → 「ログイン」ボタンタップ
   → Dio: POST /v1/auth/login
   → Backend: 認証 → access_token + refresh_token + user + tenant 返却
   → flutter_secure_storage に token 暗号化保存
   → AuthState = Authenticated(user, tenant)
   → 最終アクセス画面に遷移（Projects → Project → Board）
   → WebSocket 接続確立 (FR-WS-001)
```

### 7.2 UC-002: ボード閲覧（v1.0 継承 + オフライン拡張）

```
[User] → ログイン済み
   → プロジェクト選択画面 → プロジェクトタップ
   → go_router: /projects/:id/board
   → BoardController.fetch(): 優先 Drift ローカル → なければ REST GET
   → Board 表示（横スクロール Columns + Cards）
   → 30s REST ポーリング (FR-NOTIF-003) + WS 推送 (FR-NOTIF-008)
   → WS 受信時、差分更新 (FR-BOARD-008)
```

### 7.3 UC-003: Work Item 詳細閲覧（v1.0 継承）

```
[User] → ボード画面でカードタップ
   → go_router: /work-items/:id
   → WorkItemController.fetch(): Drift ローカル → REST GET
   → 3 タブ表示（Overview / Comments / Transitions）
   → 「Web で開く」ボタンタップ → Star Web の /work-items/{id} を外部ブラウザ起動
```

### 7.4 UC-004: 通知閲覧（v1.0 継承 + WS 拡張）

```
[User] → アプリ起動 / バックグラウンド復帰
   → Notifications タブ開く
   → NotificationsController.fetch(): REST GET 20件
   → WebSocket 接続中なら、推送受信で即時追加
   → 30s ポーリング (WS 切断時のフォールバック)
   → 通知タップ → POST /v1/notifications/{id}:read → 該当 Work Item 詳細画面遷移
   → アプリアイコンバッジに未読件数表示
```

### 7.5 UC-005: ログアウト（v1.0 拡張）

```
[User] → Settings 画面 → ログアウトボタン
   → 未同期 SyncQueue 件数チェック
   → 未同期 > 0 → 確認ダイアログ (FR-AUTH-007)
      → ユーザー「破棄」: 全削除してログアウト
      → ユーザー「キャンセル」: 戻る
   → flutter_secure_storage 全 key 削除
   → Drift DB 全テーブル削除
   → POST /v1/auth/logout (best-effort)
   → WebSocket 切断
   → AuthState = Unauthenticated → /login
```

### 7.6 UC-006: 状態遷移実行（UAT 新規）

```
[User] → Work Item 詳細 → Transitions タブ
   → 遷移可能状態一覧表示
   → 遷移先タップ → 確認ダイアログ
   → 確認 → POST /v1/work-items/{id}:transition (Idempotency-Key 必須, `api-design.md` §1.6)
   
   【オンライン】
   → 200 OK → WorkItem 新状態を表示
   → WS 推送 (FR-WS-008) で他クライアントへ伝播
   
   【オフライン】
   → SyncQueue に追記 (FR-OFFLINE-004)
   → Optimistic UI 更新（即座に新状態表示）
   → UI 上に「未同期」バッジ表示
   → 接続回復時に自動同期 (FR-OFFLINE-005)
   → 競合時 (FR-OFFLINE-006):
     → 409 Conflict 受領
     → ConflictResolver: サーバ版 / ローカル版をユーザーに提示
     → ユーザー選択 (サーバ優先 / ローカル優先 / マージ)
     → 再送または破棄
```

### 7.7 UC-007: オフライン編集（UAT 新規）

```
[User] → 地下鉄で作業中、ネットワーク断
   → UI 上に「オフライン」バナー表示 (FR-OFFLINE-007)
   → Work Item の priority を「High」に変更
     → Drift ローカル DB を即座に更新
     → SyncQueue に追記
     → Optimistic UI
   → 別の Work Item にコメント投稿
     → 同様にローカル保存 + SyncQueue
   → 地下を出て接続回復
   → 接続性監視 (FR-OFFLINE-001) が recovery 検知
   → SyncQueue を順次同期
   → 各リクエストに Idempotency-Key 付与
   → 全て成功 → バナー消える、未同期バッジ消える
   → 一部失敗 / 競合 → ユーザー通知 + 競合解決 UI
```

### 7.8 UC-008: WebSocket 推送受信（UAT 新規）

```
[Backend] → Work Item 状態変更イベント発生
   → WS Service: 当該テナントの全 WS クライアントに推送
   → [App] → WS 受信
   → JSON parse → event type 判定
     → 'work_item.updated': BoardController が該当 WorkItem を更新 → UI 反映
     → 'work_item.commented': Comments タブにバッジ表示
     → 'notification.new': NotificationsController がリスト先頭に追加 → UI 反映
   → 200ms 以内 (NFR-PERF-010)
```

### 7.9 例外フロー（v1.0 継承 + 拡張）

| 例外 | 挙動 |
|---|---|
| ネットワーク接続なし | 「オフライン」バナー + ボード画面は Drift ローカル データ + SyncQueue バッジ + Pull-to-Refresh 無効化 |
| API 500 エラー | 「サーバーエラー」+ リトライボタン (オンライン時) |
| API 401（refresh 失敗含む） | 自動ログアウト → ログイン画面 |
| API 403 | 「アクセス権限がありません」+ 戻るボタン |
| API 404 | 「Work Item が見つかりません」+ 戻るボタン |
| API 409 競合 | 競合解決 UI (NFR-USE-007) |
| トークン期限切れ + refresh 失敗 | 自動ログアウト + ログイン画面 + 「セッションの有効期限が切れました」トースト |
| **WebSocket 接続失敗** | **フォールバック: 30s REST ポーリングで稼働継続** (FR-NOTIF-009) |
| **WebSocket 接続中だが推送遅延 30s 超** | **接続再構築** (FR-WS-005) |
| **同期キュー 100 件超** | **古い順に自動 drop + ユーザー通知** |
| **SQLite 容量 50MB 到達** | **古い Work Item から削除 + ユーザー通知** (FR-OFFLINE-009) |

---

## §8 データ要件（高レベル、DDL なし）

### 8.1 アプリ内部データ（クライアント側のみ）

| データ | 種別 | 保持場所 | TTL | 暗号化 |
|---|---|---|---|---|
| access_token | 資格情報 | flutter_secure_storage (Keystore) | 15 分 | ✅ |
| refresh_token | 資格情報 | flutter_secure_storage (Keystore) | 7 日 | ✅ |
| user（id, name, avatar_url） | 業務データ | flutter_secure_storage (JSON) | refresh まで | ✅ |
| tenant（id, name） | 業務データ | flutter_secure_storage (JSON) | refresh まで | ✅ |
| テーマ設定 | ユーザー設定 | flutter_secure_storage | 永続 | ✅ |
| **Work Item キャッシュ** | **業務データ** | **Drift/SQLite (SQLCipher)** | **最終アクセス + 7 日** | **✅ NFR-SEC-011** |
| **Board キャッシュ** | **業務データ** | **Drift/SQLite** | **最終アクセス + 7 日** | **✅** |
| **Comment キャッシュ** | **業務データ** | **Drift/SQLite** | **最終アクセス + 7 日** | **✅** |
| **SyncQueue** | **業務データ** | **Drift/SQLite** | **同期完了まで** | **✅** |
| **Conflict 解決待ち** | **業務データ** | **Drift/SQLite** | **ユーザー解決まで** | **✅** |
| 通知リスト | 業務データ | メモリ + Drift | WS / 30s ポーリング | 一部 |

### 8.2 Drift テーブル概要（DDL 詳細は `03-detailed-design.md` §6.3）

| テーブル | 主キー | 用途 |
|---|---|---|
| `cached_work_items` | id | 閲覧済み Work Item キャッシュ |
| `cached_boards` | project_id | プロジェクトボード設定 |
| `cached_columns` | id | ボード列 |
| `cached_comments` | id | コメント履歴 |
| `cached_notifications` | id | 通知履歴 |
| `sync_queue` | auto-increment id | オフライン操作の同期キュー |
| `conflict_reports` | id | 競合解決待ちアイテム |

### 8.3 サーバ側データ

`01-requirements.md` v1.0 §8.2 と同じ（`tenant`, `user`, `project`, `board`, `work_item`, `comment`, `notification`）。

**UAT で新規に必要なサーバ側データ**:
- `idempotency_keys` テーブル（既存? V1.0 経由で確認要、§11 G-17）
- WS 接続管理（既存 / V1.0 経由で確認要、§11 G-18）

---

## §9 インターフェース要件（高レベル、詳細スキーマは §11 / 詳細設計書）

### 9.1 REST API エンドポイント一覧（UAT で 13 → 20 個に拡張）

| # | Method | パス | 認証 | 用途 | UAT 区分 |
|---|---|---|---|---|---|
| 1 | POST | `/v1/auth/login` | Anonymous | ログイン | v1.0 継承 |
| 2 | POST | `/v1/auth/refresh` | Authenticated | トークンリフレッシュ | v1.0 継承 |
| 3 | POST | `/v1/auth/logout` | Authenticated | ログアウト | v1.0 継承 |
| 4 | GET | `/v1/users/me` | Authenticated | 自分の情報取得 | v1.0 継承 |
| 5 | GET | `/v1/tenants/current` | Authenticated | 自分のテナント取得 | v1.0 継承 |
| 6 | GET | `/v1/projects/{id}/board` | Policy | ボード設定取得 | v1.0 継承 |
| 7 | GET | `/v1/work-items?project_id=&...` | Policy | Work Item リスト | v1.0 継承 |
| 8 | GET | `/v1/work-items/{id}` | Policy | Work Item 詳細 | v1.0 継承 |
| 9 | GET | `/v1/work-items/{id}/transitions` | Policy | 遷移可能状態 | v1.0 継承 |
| 10 | GET | `/v1/work-items/{id}/comments` | Policy | コメント一覧 | v1.0 継承 |
| 11 | GET | `/v1/notifications?read=false` | Authenticated | 通知一覧 | v1.0 継承 |
| 12 | POST | `/v1/notifications/{id}:read` | Authenticated | 単条既読化 | v1.0 継承 |
| 13 | POST | `/v1/notifications/mark-all-read` | Authenticated | 全既読化 | v1.0 継承 |
| **14** | **PATCH** | **`/v1/work-items/{id}`** | **Policy(`work_item:update`) + If-Match** | **Work Item 部分更新 (priority/assignee/description/due_date)** | **UAT 新規** |
| **15** | **POST** | **`/v1/work-items/{id}:transition`** | **Policy + Idempotency-Key** | **状態遷移実行** | **UAT 新規** |
| **16** | **POST** | **`/v1/work-items/{id}/comments`** | **Policy(`comment:create`) + Idempotency-Key** | **コメント投稿** | **UAT 新規** |
| **17** | **POST** | **`/v1/comments/{id}` (PATCH/DELETE)** | **Policy** | **コメント編集/削除** (MVP は作成のみ) | **UAT 部分** |
| **18** | **GET** | **`/v1/app-version`** | **Anonymous** | **最新アプリバージョン取得** (G-02) | **UAT 拡張** |
| **19** | **POST** | **`/v1/sync/batch`** | **Policy** | **オフライン操作のバッチ同期 (DRYRUN/preview)** | **UAT 検討 (§11 G-19)** |
| **20** | **GET** | **`/v1/work-items/{id}/attachments`** | **Policy** | **添付ファイル一覧 (V1.2 で download/upload)** | **UAT 部分** |

### 9.2 WebSocket エンドポイント（UAT 新規）

| 項目 | 値 |
|---|---|
| URL | `wss://star.internal:8080/api/v1/ws` (HTTPS 必須、NFR-SEC-003) |
| Subprotocol | `star.v1`（強制、不一致 → 握手失敗） |
| 認証 | `Sec-WebSocket-Protocol: star.v1` + `Authorization: Bearer <jwt>` |
| tenant_id | JWT claim から抽出（query / subscribe で送らない） |
| Heartbeat | サーバ 30s ごとに ping、クライアント 60s 以内に pong |
| 最大同時 Subscription | 100 / Connection |
| 購読 resource_types | `work_item`, `notification`（§11 G-16 で拍板待ち） |

**WS メッセージ形式** (per `api-design.md` §4.4):

```json
// Client → Server: subscribe
{
  "type": "subscribe",
  "id": "sub-001",
  "filter": {
    "resource_types": ["work_item", "notification"],
    "project_id": "prj_xxx"
  }
}

// Server → Client: push event
{
  "type": "event",
  "id": "evt-001",
  "resource_type": "work_item",
  "resource_id": "01HYYY...",
  "action": "updated",
  "data": { /* work_item snapshot */ }
}
```

### 9.3 画面遷移仕様（v1.0 継承 + UAT 拡張）

```
/login (public)
  → /projects (auth)
    → /projects/:id/board (auth + project member)
      → /work-items/:id (auth)
        → /work-items/:id/edit (auth, UAT 新規)
        → /work-items/:id/transitions (auth, UAT 新規)
        → /work-items/:id/comments (auth, UAT 新規)
  
  → /notifications (auth)
    → /work-items/:id (deep link)
  
  → /settings (auth)
    → /settings/sync (auth, UAT 新規: 同期状態詳細)
    → /settings/conflicts (auth, UAT 新規: 競合解決キュー)
  
  → /sync-conflict/:conflict_id (auth, UAT 新規: 競合解決画面)
```

---

## §10 運用・保守要件（v1.0 拡張）

### 10.1 APK 配布

- **内網ファイル共有** + 新バージョン通知バナー (FR-SETTINGS-007)

### 10.2 ログ収集（UAT 拡張）

- **外部解析サービス禁止**（ADR-0021）
- 自社クラッシュレポート: アプリ内 `try/catch` でローカルのログファイルに書き出し
- **ログ送信機能** (FR-SETTINGS-006): ユーザーが手動でログファイル送信
- **自動 redact** (NFR-SEC-012): トークン / パスワード / PII を送信前に除去

### 10.3 監視

- サーバ側メトリクス（API 応答時間、エラー率、WS 接続数）は Backend 側 Grafana
- クライアント側: **接続率 / 同期成功率 / 競合発生率** を匿名集計（V1.2 で詳細化）

### 10.4 バックアップ

- オフラインキャッシュは失われても再取得可能なため、バックアップ不要
- ただし、**未同期 SyncQueue は失われるとユーザー操作が失われる**（重要）

---

## §11 セキュリティ要件（v1.0 §11 + UAT 拡張）

| ID | 脅威 | 対策 | 出典 |
|---|---|---|---|
| SEC-001 | トークン盗難 | Android Keystore 暗号化 + FLAG_SECURE | NFR-SEC-001/005 |
| SEC-002 | 中間者攻撃 | cleartext HTTP（内網限定）+ ドメイン制限 | NFR-SEC-002/004 |
| SEC-003 | デバイス紛失 | ログアウトでローカル完全削除 + 同期キュー削除 | NFR-SEC-006 |
| SEC-004 | バックドア SDK | 外部 SDK 全面禁止、AGP 依存関係レビュー必須 | ADR-0021 |
| SEC-005 | APK 改ざん | 内網署名 + `network_security_config.xml` | NFR-SEC-004 |
| SEC-006 | コード解析 | `obfuscate` + `split-debug-info` | NFR-SEC-007 |
| SEC-007 | 認証情報ハードコード | `--dart-define` 注入 | NFR-SEC-009 |
| SEC-008 | 不正 tenant アクセス | クライアントから tenant_id 送信禁止 | NFR-SEC-010 |
| **SEC-009** | **WS 中間者攻撃** | **WSS (TLS 1.2+) 必須 + cert pinning 検討** | **NFR-SEC-003** |
| **SEC-010** | **オフラインキャッシュ漏洩** | **SQLCipher + ログアウト時全削除** | **NFR-SEC-011 + NFR-SEC-006** |
| **SEC-011** | **同期キュー改ざん** | **Integrity check + Idempotency-Key** | **`api-design.md` §1.6** |
| **SEC-012** | **ログ漏洩** | **送信前自動 redact (NFR-SEC-012)** | **UAT 新規** |

### 11.1 認証トークン管理詳細（v1.0 継承 + 拡張）

- **access_token**: 有効期限 15 分、API リクエストの `Authorization: Bearer` に使用
- **refresh_token**: 有効期限 7 日、access_token 再発行にのみ使用
- **WS 接続時の auth**: WS upgrade 時の `Authorization: Bearer` ヘッダで 1 度だけ認証、接続中は同一 token を使用
- **WS token 期限切れ**: WS 切断 → REST refresh → WS 再接続
- **保存場所**: `flutter_secure_storage` (Android Keystore バックエンド、API 23+)
- **ログアウト時の完全削除**: `flutter_secure_storage.deleteAll()` + `Drift` 全テーブル `DELETE`

### 11.2 通信セキュリティ（UAT 拡張）

| プロトコル | 暗号化 | 備考 |
|---|---|---|
| REST | HTTP（cleartext、内網限定） / HTTPS（外網） | MVP 段階は HTTP、V1.2 で HTTPS 移行検討 |
| WebSocket | **WSS (TLS 1.2+) 必須** | **cleartext WS は使用禁止** (NFR-SEC-003) |
| 証明書 | システム信頼ストア + V1.2 で pinning 検討 | 内網 CA は V1.2 で評価 |

### 11.3 オフラインキャッシュセキュリティ（UAT 新規）

- **SQLCipher** で DB 全体を AES-256 暗号化
- 暗号化キー: Keystore に保存 + 起動時取得
- ログアウト時: キー削除 + DB ファイル削除（次起動時に新規 DB 作成）

---

## §12 用語定義（v1.0 拡張）

| 用語 | 定義 | 出典 |
|---|---|---|
| **Work Item** | 作業の最小単位 | `requirements.md` §26 |
| **Board** | Work Item をカラム（状態）で表示するカンバンビュー | `api-design.md` §3.7 |
| **Column** | Board 内の一列、特定の状態に対応 | `api-design.md` §3.7 |
| **Tenant** | テナント（組織単位）の論理境界 | `requirements.md` §26 |
| **Project** | テナント内のプロジェクト単位 | `requirements.md` §26 |
| **UAT** | User Acceptance Test、本書のスコープ | IPA 標準 |
| **access_token** | 短時間有効な API 認証トークン（15 分） | `api-design.md` §1.12 |
| **refresh_token** | 長時間有効なトークン更新用トークン（7 日） | `api-design.md` §1.12 |
| **JWT** | JSON Web Token、本書では Bearer 認証に使用 | RFC 7519 |
| **RFC 7807** | Problem Details for HTTP APIs、エラーレスポンス形式 | IETF |
| **W3C Trace Context** | `traceparent` ヘッダ標準、分散トレーシング用 | W3C |
| **IPA** | 情報処理推進機構 | — |
| **DDD** | Domain-Driven Design、本書の上流設計で採用 | Eric Evans |
| **RLS** | Row Level Security、PostgreSQL の行レベルセキュリティ | `data-design.md` §4.1.4 |
| **SyncQueue** | オフライン中の操作を順次同期するための FIFO キュー（UAT 用語） | UAT 新規 |
| **Conflict Resolution** | 同一 Work Item のローカル変更とサーバ変更が衝突した際の解決プロセス | UAT 新規 |
| **Optimistic UI** | サーバ応答を待たず即座に UI を更新し、後で同期する UX パターン | UAT 新規 |
| **Idempotency-Key** | 同一リクエストの重複実行を防ぐための UUID キー | `api-design.md` §1.6 |
| **WSS** | WebSocket over TLS、本書では WSS 必須 (NFR-SEC-003) | RFC 6455 |
| **Drift** | Dart 向け type-safe SQL ライブラリ、本書でオフライン DB に使用 | drift package |
| **SQLCipher** | SQLite の透過的暗号化拡張 | sqlcipher.org |
| **subprotocol** | WebSocket の application-level protocol識別子、本書では `star.v1` | RFC 6455 |
| **MVP** | Minimum Viable Product（v1.0 スコープ、本書 v1.1 は UAT） | IPA 標準 |

---

## §13 既知の未解決事項

### 13.1 v1.0 から継承（一部は v1.1 で解決）

| ID | 項目 | 状態 | 拍板人 |
|---|---|---|---|
| G-01 | `/v1/auth/login` 等の認証エンドポイント実装状況 | 保留 | Ulysses |
| G-02 | `GET /v1/app-version` アップグレード通知 | 採用 (FR-SETTINGS-007) | SRE Lead |
| G-03 | `STAR_HOST`（内網ドメイン / IP + ポート） | 保留 | SRE Lead |
| **G-04** | **オフラインキャッシュ（SQLite/Drift）の V1.1 スコープ** | **✅ v1.1 で実装** | — |
| **G-05** | **プッシュ通知（自前 WebSocket）の V1.1 スコープ** | **✅ v1.1 で自前 WS 実装** | — |
| G-06 | Tablet / 横画面レイアウト | 保留 (V1.2) | 5 域 Lead (frontend) |
| G-07 | iOS V2 計画での Flutter 採用継続可否 | 保留 | Ulysses |
| **G-08** | **HTTPS / envoy 移行タイミング** | **保留 (REST は HTTP 維持、WS は WSS 必須)** | SRE Lead |
| G-09 | Device 三重バインディング | 保留 (V1.2) | Ulysses (安全) |
| G-10 | APK 内網 keystore 署名戦略 | 保留 | SRE Lead |
| G-11 | APK 配布チャネル | 保留 | Ulysses + SRE Lead |
| G-12 | 倉位置（`apps/star-mobile-flutter/` vs `frontend/mobile-flutter/`） | 保留 | 架構師 |
| **G-13** | **WebSocket 实时推送（ポーリング代替）** | **✅ v1.1 で WS 採用** | — |
| G-14 | 5 域独立 Lead 真实身份補簽 | 保留 (DDD Review 段階) | DDD Review Lead |
| G-15 | WBS 新增「Flutter MVP」項目 | 保留 | Ulysses |

### 13.2 v1.1 で新規追加

| ID | 項目 | 拍板人 | 影響 |
|---|---|---|---|
| **G-16** | WS サブスクリプション `resource_types` に `work_item` と `notification` が backend 実装に含まれているか | Ulysses + SRE Lead | 含まれていない場合 backend 拡張が必要（V1.1 ブロッカー） |
| **G-17** | `idempotency_keys` サーバ側テーブル実装 | SRE Lead | オフライン同期の冪等性担保 |
| **G-18** | WS 接続管理（接続数制限 / 不正接続検知）の backend 実装 | SRE Lead | セキュリティ + 性能 |
| **G-19** | バッチ同期エンドポイント `/v1/sync/batch` の要否 | 架構師 + 5 域 Lead | 同期キュー逐次 vs バッチ |
| **G-20** | 競合解決戦略（サーバ優先 / ローカル優先 / ユーザー選択） | 5 域 Lead (work-item) + PM | UX 影響大 |
| **G-21** | オフラインキャッシュ TTL（7 日）の妥当性 | 5 域 Lead (work-item) | 業務パターン次第 |
| **G-22** | SQLCipher 鍵管理（Keystore 連携 / ローテーション） | Ulysses (安全) | 鍵漏洩時の影響 |
| **G-23** | WS reconnect backoff の最大リトライ回数 | SRE Lead | 暴走防止 |
| **G-24** | 同期キュー 上限（100 件超の drop 戦略） | 5 域 Lead (work-item) | ユーザー操作消失 |
| **G-25** | ログ送信時のサイズ上限 / 自動送信トリガ | Ulysses (安全) | プライバシー + UX |

---

## §14 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v1.0 | 2026-09-02 16:14 JST | 架構師 (Mavis 接手 agent per DEC-008) | IPA 標準初版: read-only MVP 範囲, FR 30 件 + NFR 22 件 | 2026-09-02 15:52 JST Ulysses「完成设计文档撰写」+ 16:09 JST「要符合日本IPA标准」 |
| **v1.1** | 2026-09-02 16:27 JST | 架構師 (Mavis 接手 agent per DEC-008) | **UAT 全面拡張**: §2 In Scope に核心写 + オフライン + 推送追加, §2 Out of Scope から該当削除, §5 FR 30→58 件 (核心写 FR-WORK-007/008/009/010/011, オフライン FR-OFFLINE-001~011, WS FR-WS-001~008, 通知 FR-NOTIF-008/009 追加), §6 NFR 22→37 件追加, §7 UC 5→8 件追加, §8 データ要件 Drift 7 テーブル追加, §9 エンドポイント 13→20 + WS 仕様, §10 ログ送信追加, §11 セキュリティ 4 脅威追加, §12 用語 14→22 件, §13 既知未解決 G-04/G-05/G-13 解決 + G-16~G-25 新規追加 | 2026-09-02 16:27 JST Ulysses 拍板 UAT 範囲 + 自建 WS 推送 (questionnaire 答: full_uat + self_ws) |
| **v1.2** | 2026-09-02 16:54 JST | 架構師 (Mavis 接手 agent per DEC-008) | **§16 Capability Audit 増補**: v1.1 設計が現在 backend システムで実際に使用可能かを 3 段階 (✅ 完全実装 / 🟠 路由在 501 / ❌ 未実装) で監査;Ulysses 2026-09-02 16:40 JST 発令「app の設計が現在システム内の既に書かれた機能を使用できることを確保する」に対応;結論: v1.1 仮定 20 REST endpoint + WS + auth のうち**現在 backend に実装済みなのは 0 個**、MCP tool 3 個 (stdio) のみ;P2 backend 完了まで実運用不可だが設計自体は P2 完了後の姿として保持;G-XX を backend 依存 / client 環境 / 拍板待ち の 3 区分に再分類 | 2026-09-02 16:40 JST Ulysses「app 的设计要确保能使用当前系统内已经写好的功能」発令;per 守門 #1+#8+#12 (禁回溯叙事 + BAS git 实证) で backend 実装状況を直接 grep (`crates/star-api-rest/src/routes/*.rs` + `crates/star-mcp/src/tools/*.rs` + `crates/star-sse/src/lib.rs` + commit `9c46a1c`/`c8f6dc7`/`d71b63f`) した結果 22 路由全部 501 / 13 MCP tool stub / WS 無し / auth 無し / SSE 4 event types のみを実証 |

---

## §16 Capability Audit — Backend 現状整合 (per 2026-09-02 16:54 JST 増補)

> **追加トリガ**: 2026-09-02 16:40 JST Ulysses 発令「app 的设计要确保能使用当前系统内已经写好的功能」
> **監査方法**: backend crate ソース + git log 直接 grep (per 守門 #1+#8+#12 禁回溯叙事 + BAS git 实证)
> **監査実行**: 2026-09-02 16:50 JST by 架構師 (Mavis 接手)

### 16.1 監査サマリー

| 状態 | 件数 | 該当 |
|---|---|---|
| ✅ **完全実装 (production 利用可)** | 3 件 | MCP tool: `get_workspace` / `get_worktree` / `get_issue` (commit `9c46a1c` 「Phase F.2 tool 真实数据源接入」) — ただし **stdio のみ**、Flutter HTTP client からは不可 |
| 🟠 **路由在 501 業務ロジック未実装** | 22 件 | `crates/star-api-rest` 全ルート (commit `c8f6dc7` 「Phase L 骨架」), `src/routes/*.rs` 全関数 `RestError::not_implemented()` |
| 🟡 **MCP tool stub (in-memory mock)** | 13 件 | `crates/star-mcp/src/tools/*.rs` 12 + 1 (per commit `d71b63f` 「12 tool 留 P2 缺 service」), 内部で `mock_response()` 返す |
| 🟠 **SSE 存在 but EventType 限定** | 1 件 | `crates/star-sse/src/lib.rs:18-26` `enum EventType { MergeRequest, Pipeline, AgentState, WorktreeChange }` — **work_item / notification 無し** |
| ❌ **完全未実装** | 多数 | WebSocket (SSE のみ) / auth (login/refresh/logout) / PATCH work-item 業務 logic / POST `:transition` / POST comments / idempotency_keys テーブル / OAuth 2.0 / Device 三重バインド |

### 16.2 主要 FR 別 状態 (§5 機能要件の監査)

> 監査対象: v1.1 §5 FR 58 件。**現時点で backend 動作確認できるのは 0 件 (HTTP REST 経由)**。3 件 (MCP stdio) は backend 内では動作するが Flutter から到達不可。

| FR ID | 概要 | 状態 | 根拠 (git 実証) |
|---|---|---|---|
| FR-AUTH-001 | メール+パス ログイン | ❌ | 22 路由に `/v1/auth/login` 無し |
| FR-AUTH-002 | access/refresh token 受領 | ❌ | login 未実装 |
| FR-AUTH-003 | refresh token 自動更新 | ❌ | login 未実装、middleware は stub |
| FR-AUTH-004 | refresh 失敗時 logout | ❌ | 同上 |
| FR-AUTH-005 | ログアウト + ローカル削除 | ❌ | `/v1/auth/logout` 無し |
| FR-AUTH-006 | Keystore 暗号化 | (client のみ) ✅ | flutter_secure_storage で実装可 |
| FR-AUTH-007 | 未同期 SyncQueue 確認 | (client のみ) | SyncQueue 設計依存 |
| FR-BOARD-001 | ボード閲覧 | 🟠 | `/api/v1/projects/{id}/board` 無し (22 路由に無し) |
| FR-BOARD-002~008 | ボード UI | (client のみ) ✅ | UI 実装可 |
| FR-WORK-001 | Work Item 詳細閲覧 | 🟠 | `/api/v1/work-items/{id}` 路由在 501 (`routes/work_items.rs:21`) |
| FR-WORK-002~006 | 詳細 UI | (client のみ) ✅ | UI 実装可 |
| FR-WORK-007 | PATCH 優先度/担当者/期限 | 🟠 | `/api/v1/work-items/{id}` PATCH 路由在 501 (`routes/work_items.rs:31`) |
| FR-WORK-008 | コメント投稿 | ❌ | `/api/v1/work-items/{id}/comments` POST 無し |
| FR-WORK-009 | 状態遷移実行 | ❌ | `/api/v1/work-items/{id}:transition` 無し |
| FR-WORK-010 | オフライン編集 | (client のみ) ✅ | Drift + SyncQueue で実装可 |
| FR-WORK-011 | WS 推送反映 | ❌ | WS 未実装 |
| FR-NOTIF-001 | 通知一覧 | 🟠 | 22 路由に `/v1/notifications` 無し |
| FR-NOTIF-002~007 | UI + 30s ポーリング | (client のみ) ✅ | UI 実装可、ポーリングは mock で動作 |
| FR-NOTIF-008 | WS 推送 | ❌ | WS 未実装 |
| FR-NOTIF-009 | WS 切断時 REST フォールバック | ❌ | WS 未実装 |
| FR-OFFLINE-001~011 | オフライン全機能 | (client のみ) ✅ | Drift + SQLCipher + SyncQueue 設計のみで完結、backend 依存なし |
| FR-WS-001~008 | WebSocket 全機能 | ❌ | WS backend 未実装 |
| FR-PROJ / FR-SETTINGS / FR-NAV | UI + 設定 | (client のみ) ✅ | UI 実装可 |

**統計**: 58 FR 中、client のみ で実装可 = **27 件 (47%)**、backend 待ち = **31 件 (53%)**

### 16.3 何时使用可能か (P2 backend 依存度)

| 区分 | 実運用開始条件 | 影響範囲 |
|---|---|---|
| 3 MCP tool (read, stdio) | **今すぐ (local agent のみ)** | Flutter HTTP client からは到達不可 (MCP stdio はローカルプロセス間通信) |
| 22 REST 路由 (501) | P2 backend 完了後 | 13 FR 待ち (FR-BOARD-001, FR-WORK-001, FR-NOTIF-001 等) |
| 13 MCP tool stub (mock) | P2 backend 完了後 | 該当 MCP tool 経由の FR (今回は未使用) |
| WebSocket | backend WS 実装待ち | FR-WS-001~008, FR-NOTIF-008, FR-WORK-011 |
| Auth (login/refresh/logout) | backend auth 実装待ち | FR-AUTH-001~005 |
| 核心写 (PATCH / :transition / comments) | P2 完了後 (業務ロジック実装) | FR-WORK-007~009 |
| **Client-only 機能 (オフライン/UI/キャッシュ)** | **今すぐ (Flutter 側のみで完結)** | FR-OFFLINE-001~011, FR-BOARD-002~008, FR-WORK-002~006 + 010, FR-NOTIF-002~007, FR-PROJ-001~003, FR-SETTINGS-001~008, FR-NAV-001~005, FR-AUTH-006~007 |

### 16.4 G-XX 再評価 (§13 既知未解決 + 本監査による再分類)

| ID | v1.1 内容 | v1.2 再評価 | 区分 |
|---|---|---|---|
| G-01 | `/v1/auth/login` 端点存在性 | **P2 backend 待ち** (確認済: 22 路由に無し) | backend |
| G-02 | `/v1/app-version` | **P2 backend 待ち** | backend |
| G-03 | `STAR_HOST` | SRE Lead 拍板 (内網環境) | client 環境 |
| G-04 | オフラインキャッシュ | v1.1 採用 (**実装可**) | client (resolved) |
| G-05 | 推送 (自前 WS) | **P2 backend 待ち** (SSE も work_item event 無し) | backend |
| G-06 | Tablet / 横画面 | V1.2 候補 | 将来 |
| G-07 | iOS V2 | 保留 | 将来 |
| G-08 | HTTPS / envoy 移行 | MVP は HTTP 維持、WS は WSS 必須だが backend WS 自体未実装 | backend 依存 |
| G-09 | Device 三重バインディング | V1.2 候補 | 将来 |
| G-10 | APK 署名 | SRE Lead 拍板 | client 環境 |
| G-11 | APK 配布チャネル | Ulysses + SRE Lead 拍板 | client 環境 |
| G-12 | 倉位置 | 架構師 拍板 | client 環境 |
| G-13 | WebSocket 推送 | **P2 backend 待ち** | backend |
| G-14 | 5 域 Lead 真人補簽 | DDD Review 段階 | 将来 |
| G-15 | WBS 新增「Flutter MVP」 | Ulysses 拍板 | 計画 |
| G-16 | WS resource_types work_item/notification | **P2 backend 待ち** (SSE EventType も 4 つのみ) | backend |
| G-17 | `idempotency_keys` テーブル | **P2 backend 待ち** | backend |
| G-18 | WS 接続管理 | **P2 backend 待ち** | backend |
| G-19 | `/v1/sync/batch` エンドポイント | **P2 backend 待ち** | backend |
| G-20 | 競合解決戦略 | 5 域 Lead + PM 拍板 (client 設計) | client 設計 |
| G-21 | オフラインキャッシュ TTL | 5 域 Lead 拍板 (client 設計) | client 設計 |
| G-22 | SQLCipher 鍵管理 | Ulysses 拍板 (client 設計) | client 設計 |
| G-23 | WS reconnect 回数 | SRE Lead 拍板 (client 設計) | client 設計 |
| G-24 | 同期キュー 上限 | 5 域 Lead 拍板 (client 設計) | client 設計 |
| G-25 | ログ送信サイズ | Ulysses 拍板 (client 設計) | client 設計 |

**統計**: 25 G-XX 中、**P2 backend 待ち = 9 件 (36%)**、client 環境/設計 拍板 = 10 件 (40%)、将来計画 = 6 件 (24%)。

### 16.5 v1.1 設計の妥当性 (本監査後の評価)

> **結論**: v1.1 設計は **P2 backend 完了後の目標態** として保持する。今すぐの実装は不可だが、設計自体は妥当。

**妥当な点**:
- Client-only 機能 (UI / オフライン / キャッシュ / 競合解決) は backend 待たず実装可
- P2 backend 完了後に v1.1 範囲がそのまま稼働する設計整合性
- 5 域独立 Lead 構造 (per 8/21 JST) / token-OLU (per STAR-OLU-001) / IPA 標準章立て / ADR-0021 零廠商合作 等の守門と整合

**修正不要の点** (v1.1 維持):
- §5 FR 58 件 (実装可否は §16 で個別明示、設計自体は妥当)
- §6 NFR 37 件 (性能 / セキュリティ / 保守性 目標)
- §7 UC 8 件 (業務フロー)
- §8 データ要件 (Drift 7 テーブル, client 完結)
- §9 REST 20 endpoint + WS (P2 完了後に動作)
- §10-§15 (運用 / セキュリティ / 用語 / 既知未解決 / 承認欄)

**要注記の点** (運用時):
- Mobile app 開発は P2 backend 進捗と並走可能 (client 27 FR 先行実装可)
- 統合テストは P2 backend 完了まで mock + 単体テストで代替
- 5 域 Lead (work-item / notification / auth / frontend) の真人補簽は backend 側と合同で P2 段階に実施

### 16.6 推奨実装順序 (per 本監査)

| 順序 | 区分 | 開始条件 | 推定 token |
|---|---|---|---|
| 1 | **Client-only 機能 27 FR 先行実装** (UI / オフライン / Drift / 競合解決) | 即時 (P2 待ち不要) | ~2.0M |
| 2 | **P2 backend 待ち FR 9 件** (REST + WS + auth) | P2 backend 完了後 | ~3.0M |
| 3 | **統合テスト + DDD Review** | 1 + 2 完了後 | ~0.5M |
| **合計** | — | — | **~5.5M** (v1.1 推計 5.0-6.0M と整合) |

**P2 backend との並走モデル**:
- Phase A (即時, client 先行): 27 FR 実装、mock データで動作確認
- Phase B (P2 完了後, 結合): 残 31 FR 実装、E2E 統合テスト、5 域 Lead 合同 DDD Review

---

## §15 承認欄（5 角色）

| 角色 | 簽字 | 日付 | 備註 |
|---|---|---|---|
| 架構師 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽 (per 8/27 19:39 JST + 21:59 JST 三次強化授權) |
| SRE Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;5 域独立真实身份 (per 8/21 JST) DDD Review 段階で補充 |
| 平台 Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| 評審主持 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| PM | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
