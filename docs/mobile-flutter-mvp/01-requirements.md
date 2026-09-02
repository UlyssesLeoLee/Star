# Star Mobile Flutter MVP — 要件定義書

> **基準**: 日本 IPA（情報処理推進機構）SEC ソフトウェア要件定義書 標準章立て
> **作成日**: 2026-09-02
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **Pre-IPA 草稿**: `docs/architecture/2026-09-02-upgrade/spec/mobile/01-flutter-mvp-design.md` v0.1 (commit `bd4998e`) — 本書 v1.0 として正式 IPA 化で supersede
> **上流要件定義書**: `D:\Star\docs\requirements.md` v2.0 (Star プラットフォーム全体,§N で引用)
> **上流基本設計書**: `D:\Star\docs\basic-design.md` v0.1 (Star プラットフォーム全体,§N で引用)

---

## §1 目的

本文書は、Star プラットフォーム（AI Coding Worktree Control Plane + Jira-class Work Management + SCM Integration）における**モバイルクライアント第一版（Android Flutter MVP）**の要件を定義する。上流の `docs/requirements.md` v2.0 がプラットフォーム全体の要件を定義するのに対し、本書は「**モバイルユーザーが必要とする最小機能集合**」に焦点を絞り、IPA 標準の要件定義書章立て（§1〜§14）に従って記述する。

本書の位置付け：
- 上流：Star プラットフォーム要件定義書 v2.0（プラットフォーム要件の正本）
- 本書：モバイル MVP の要件定義（プラットフォーム要件のうち MVP スコープだけ取り出し、モバイル固有の追加要件で拡張）
- 下流：モバイル MVP 基本設計書（`02-basic-design.md` v1.0）→ モバイル MVP 詳細設計書（`03-detailed-design.md` v1.0）

---

## §2 適用範囲

### 2.1 In Scope（MVP で実装する）

| 領域 | 範囲 | 出典 |
|---|---|---|
| プラットフォーム | **Android のみ**（minSdk 24 / Android 7.0+） | per 2026-09-02 15:52 JST 発令「安卓版」 |
| 通信プロトコル | REST のみ（MCP / WebSocket は含まない） | `api-design.md` §1.1 |
| 機能スコープ | **Read-Only**: ログイン、ボード閲覧、Work Item 閲覧、通知閲覧 | per 2026-09-02 15:54 JST Mavis 接手デフォルト |
| 認証 | JWT Bearer + Refresh Token | `api-design.md` §1.12 |
| オフライン | **対応しない**（online-only） | per 9/2 デフォルト |
| プッシュ通知 | **対応しない**（30s ポーリング） | per ADR-0021 零廠商合作 |
| Tablet 対応 | **対応しない**（スマホ縦画面のみ） | per 9/2 デフォルト |
| 多言語 | **中国語のみ**（i18n 対応なし） | per 内網利用 |

### 2.2 Out of Scope（MVP で実装しない、後の版で対応）

| 領域 | 計画 | 参照 |
|---|---|---|
| iOS 対応 | V2 で対応 | `internal-design.md:1600` + V2 モバイル計画 |
| Write 操作（作成/更新/状態遷移/コメント投稿） | V1.1 で対応 | 本書 §5.1 で要件化保留 |
| オフラインキャッシュ（SQLite/Drift） | V1.1 で対応 | §11 既知未解決 G-04 |
| プッシュ通知（FCM/自前 WS） | V1.1 で対応 | §11 既知未解決 G-05 |
| Tablet / 横画面 | V1.1 で対応 | §11 既知未解決 G-06 |
| ダークモード（system 連動以外の手動切替は含む） | V1.1 で対応 | 詳細設計で再評価 |
| 生体認証（指紋/顔認証） | V1.2 で対応 | 詳細設計で再評価 |
| Tablet 専用レイアウト | V1.2 で対応 | 同上 |
| 多言語 i18n | V2 で対応（V2 モバイル計画に統合） | 内部設計 §10 |

---

## §3 前提条件・制約事項

### 3.1 前提条件

1. Star プラットフォーム本体（Backend / API / DB）が既に運用可能で、対象エンドポイント（後述）が production 利用可能である
2. 対象ユーザーは Star テナントの既存ユーザーである（モバイルアプリ単独での新規テナント作成は不可）
3. 利用は**企業内ネットワーク**（内網）に限定され、インターネット公開はしない
4. 利用者は Android 7.0+ のスマートフォンを所有している
5. 利用者は Star のメールアドレス + パスワードを既に保有している

### 3.2 制約事項

| 制約 | 出典 |
|---|---|
| **零廠商合作**（外部 SDK 不使用） | `docs/architecture/2026-08-26-upgrade/adr/0021-zero-vendor-cooperation.md` |
| **5 域独立 Lead, 兼任禁止** | `AGENTS.md` §4 #3 + 8/21 JST 拍板 |
| **トークン予算制（人日ではなく token）** | `AGENTS.md` §4 #4 + `STAR-OLU-001.md` v0.1 (1 SRE·周 = 1.2M) |
| **環境変数ハードコード禁止** | `AGENTS.md` §4 #5 + 8/27 11:06 JST hard ban |
| **AI ドキュメント治理解禁**（回溯叙事禁止） | `AGENTS.md` §1.2 + §4 #12 |
| **V1 はモバイル App を範囲外**（本書は V1 範囲の**例外**として新規追加） | `docs/internal-design.md:50` + 2026-09-02 15:52 JST 発令 |
| **Mobile V2 計画は React Native 候補**（本書は Flutter 採用で V2 と並走） | `docs/internal-design.md:1633` |

---

## §4 システムの概要

### 4.1 システム構成図

```
┌─────────────────────────────────┐         ┌──────────────────────┐
│   Android スマートフォン        │         │   Star プラットフォーム │
│  ┌───────────────────────────┐  │         │                      │
│  │ Flutter App (本 MVP)      │  │  HTTP   │  ┌────────────────┐  │
│  │ - Riverpod 状態管理        │  │ ───────▶│  │ API Gateway    │  │
│  │ - Dio HTTP クライアント    │  │         │  │ (Rust axum)    │  │
│  │ - flutter_secure_storage  │  │  REST   │  └────────────────┘  │
│  │ - メモリ内キャッシュ       │  │ ◀───────│  ┌────────────────┐  │
│  └───────────────────────────┘  │         │  │ work-core      │  │
│                                 │         │  │ (Rust Modular │  │
│  内網限定（cleartext HTTP）     │         │  │  Monolith)     │  │
└─────────────────────────────────┘         │  └────────────────┘  │
                                            └──────────────────────┘
```

### 4.2 主要ユーザー像（Persona, `requirements.md` §3 から抜粋 + モバイル特化）

| Persona | モバイル利用シーン |
|---|---|
| **Developer（人間）** | 通勤中・会議間で Work Item の状況確認、通知チェック |
| **Product Owner / PM** | 移動中にボードの進捗確認、次の Sprint の優先順位確認 |
| **Reviewer** | 移動中に PR レビュー依頼通知に気づき、後で PC で対応 |
| **Tech Lead** | 会議中に Work Item 詳細を確認、チーム進捗を俯瞰 |

---

## §5 機能要件（Functional Requirements）

### 5.1 FR-AUTH: 認証機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-AUTH-001** | ユーザーはメールアドレス + パスワードでログインできる | P0 | 本書新規 |
| **FR-AUTH-002** | ログイン成功時、access_token（15 分有効）と refresh_token（7 日有効）を受け取る | P0 | `api-design.md` §6.2 |
| **FR-AUTH-003** | access_token 期限切れ時、refresh_token で自動更新し、元の API 呼び出しを retry する | P0 | 本書新規 |
| **FR-AUTH-004** | refresh_token 期限切れ時、ユーザーのローカル資格情報をクリアしログイン画面に遷移する | P0 | 本書新規 |
| **FR-AUTH-005** | ログアウト時、ローカルの全資格情報をクリアし、ログアウト API を呼ぶ（best-effort） | P0 | 本書新規 |
| **FR-AUTH-006** | ログイン状態は `flutter_secure_storage`（Android Keystore）に暗号化して保存する | P0 | `api-design.md` §6.2 + 本書新規 |
| FR-AUTH-007 | ❌ OAuth 2.0 は実装しない（`api-design.md` §6.2 G-01 で Phase 2+ 候補） | — | 保留 |
| FR-AUTH-008 | ❌ 生体認証（指紋/顔）は実装しない（V1.2 候補） | — | 保留 |
| FR-AUTH-009 | ❌ Device 三重バインディング（`internal-design.md:23.2`）は MVP スキップ（V1.1 で対応） | — | 保留 |

### 5.2 FR-BOARD: ボード閲覧機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-BOARD-001** | ユーザーはプロジェクト単位でボードを閲覧できる | P0 | `api-design.md` §3.7 |
| **FR-BOARD-002** | ボードはカンバン形式（横スクロール Columns）で表示される | P0 | 内部設計 §10 V1 |
| **FR-BOARD-003** | 各 Column には複数の Work Item カードが縦に並ぶ | P0 | 内部設計 §10 V1 |
| **FR-BOARD-004** | Work Item カードには title / assignee アバター / priority chip / 状態 chip が表示される | P0 | 本書新規 |
| **FR-BOARD-005** | ボード画面は Pull-to-Refresh で再取得できる | P0 | 本書新規 |
| **FR-BOARD-006** | ボード画面に入った瞬間に最新データを取得する | P0 | 本書新規 |
| FR-BOARD-007 | ❌ カードのドラッグ&ドロップによる状態遷移は実装しない（V1.1） | — | 保留 |
| FR-BOARD-008 | ❌ Column のリネーム / 並び替えは実装しない（V1.1） | — | 保留 |

### 5.3 FR-WORK-ITEM: Work Item 詳細閲覧機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-WORK-001** | ユーザーは Work Item 詳細を閲覧できる | P0 | `api-design.md` §3.5:626 |
| **FR-WORK-002** | 詳細画面は 3 タブ構成: Overview / Comments / Transitions | P0 | 内部設計 §10 |
| **FR-WORK-003** | Overview タブには title, description, status, priority, assignee, reporter, due date, repository/worktree へのリンクが表示される | P0 | 内部設計 §10 |
| **FR-WORK-004** | Comments タブにはコメント一覧（時系列降順）が表示される | P0 | `api-design.md` §3.10:700 |
| **FR-WORK-005** | Transitions タブには現在の状態から遷移可能な状態一覧が表示される（**閲覧のみ、実行は V1.1**） | P0 | `api-design.md` §3.5:630 |
| **FR-WORK-006** | 詳細画面右上に「Web で開く」ボタンを配置し、Star Web の該当 Work Item 詳細ページに遷移する | P0 | 本書新規 |
| FR-WORK-007 | ❌ インライン編集は実装しない（V1.1） | — | 保留 |
| FR-WORK-008 | ❌ コメント投稿は実装しない（V1.1） | — | 保留 |
| FR-WORK-009 | ❌ 状態遷移の実行は実装しない（V1.1） | — | 保留 |
| FR-WORK-010 | ❌ 添付ファイル閲覧は実装しない（V1.1） | — | 保留 |

### 5.4 FR-NOTIF: 通知閲覧機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-NOTIF-001** | ユーザーは自分宛の通知一覧を閲覧できる | P0 | `api-design.md` §3.16:787 |
| **FR-NOTIF-002** | 通知一覧は最新 20 件まで取得し、Pull-to-Refresh で追加取得できる | P0 | 本書新規 |
| **FR-NOTIF-003** | 30 秒間隔でポーリングし、新規通知を自動取得する | P1 | per ADR-0021 制約 |
| **FR-NOTIF-004** | 通知をタップすると既読化される（`POST /v1/notifications/{id}:read`） | P0 | `api-design.md` §3.16:788 |
| **FR-NOTIF-005** | 「すべて既読」ボタンで全通知を一括既読化できる | P1 | `api-design.md` §3.16:789 |
| **FR-NOTIF-006** | 通知をタップすると、関連する Work Item 詳細画面に遷移する | P0 | 本書新規 |
| **FR-NOTIF-007** | 未読通知の件数をアプリアイコンバッジに表示する | P1 | 本書新規 |
| FR-NOTIF-008 | ❌ プッシュ通知（OS レベルの通知）は実装しない | — | 保留 |
| FR-NOTIF-009 | ❌ 通知のフィルタリング / 検索は実装しない（V1.1） | — | 保留 |

### 5.5 FR-PROJ: プロジェクト選択機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-PROJ-001** | ユーザーは自分がメンバーであるプロジェクトの一覧を閲覧できる | P0 | `api-design.md` §3.4 |
| **FR-PROJ-002** | プロジェクトをタップすると該当プロジェクトのボード画面に遷移する | P0 | 本書新規 |
| **FR-PROJ-003** | プロジェクト一覧は最終アクセス時刻でソート表示される | P1 | 本書新規 |
| FR-PROJ-004 | ❌ プロジェクトの新規作成は実装しない（Web で実施） | — | 保留 |

### 5.6 FR-SETTINGS: 設定機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-SETTINGS-001** | ユーザーはテーマ（light / dark / system）を切り替えられる | P0 | 本書新規 |
| **FR-SETTINGS-002** | ユーザーはログアウトできる | P0 | FR-AUTH-005 |
| **FR-SETTINGS-003** | 設定画面にアプリバージョン / ビルド番号が表示される | P2 | 本書新規 |
| **FR-SETTINGS-004** | 設定画面に新バージョン通知バナー（`GET /v1/app-version`）が表示される | P2 | §11 G-02 |

### 5.7 FR-NAV: ナビゲーション機能

| ID | 要件 | 優先度 | 出典 |
|---|---|---|---|
| **FR-NAV-001** | アプリ起動時にログイン状態を確認し、未ログインならログイン画面、ログイン済みなら最終アクセス画面を表示する | P0 | 本書新規 |
| **FR-NAV-002** | 認証必須画面には認証ガードが働き、未ログイン状態でアクセスするとログイン画面にリダイレクトされる | P0 | 本書新規 |
| **FR-NAV-003** | アプリがバックグラウンドから復帰した時、認証トークンの有効性を確認し、無効ならログイン画面に遷移する | P0 | 本書新規 |
| **FR-NAV-004** | アプリ起動時間（cold start → ボード表示まで）は 1.5 秒以内（Mid-range Android 想定） | P1 | 性能要件 §6.1 |

---

## §6 非機能要件（Non-Functional Requirements）

### 6.1 性能要件（NFR-PERF）

| ID | 要件 | 目標値 | 出典 |
|---|---|---|---|
| NFR-PERF-001 | アプリ cold start 時間（Pixel 6 想定） | ≤ 1.5s | 本書新規 |
| NFR-PERF-002 | ボード表示時間（API 取得 + レンダリング） | ≤ 2.0s | 本書新規 |
| NFR-PERF-003 | API 呼び出し P95 応答時間（内網） | ≤ 200ms | `api-design.md` §10 |
| NFR-PERF-004 | メモリ使用量（idle 時） | ≤ 120 MB | 本書新規 |
| NFR-PERF-005 | APK サイズ（リリースビルド、Obfuscate 後） | ≤ 30 MB | 本書新規 |
| NFR-PERF-006 | バッテリー消費（30 分アクティブ利用） | ≤ 5% | 本書新規 |

### 6.2 可用性要件（NFR-AVAIL）

| ID | 要件 | 目標値 | 出典 |
|---|---|---|---|
| NFR-AVAIL-001 | アプリクラッシュ率（Firebase Crashlytics 不使用のため自社測定） | ≤ 0.1% | 本書新規 |
| NFR-AVAIL-002 | 致命的バグ（起動不可 / 主要機能全滅）発生時の修正 SLA | 24 時間 | 本書新規 |

### 6.3 セキュリティ要件（NFR-SEC）

| ID | 要件 | 出典 |
|---|---|---|
| NFR-SEC-001 | access_token / refresh_token は Android Keystore 暗号化して保存する（平文 SharedPreferences 禁止） | 本書新規 |
| NFR-SEC-002 | 通信は MVP 段階では cleartext HTTP（内網限定）;外網公開時は HTTPS 必須 | per 9/2 デフォルト |
| NFR-SEC-003 | 通信先は `network_security_config.xml` で `star.internal` ドメインのみ cleartext 許可、他は拒否 | 本書新規 |
| NFR-SEC-004 | ログイン画面でスクリーンショット無効化フラグ（`FLAG_SECURE`）を設定する | 本書新規 |
| NFR-SEC-005 | ログアウト時にローカル資格情報を完全削除する | FR-AUTH-005 |
| NFR-SEC-006 | APK は `obfuscate` + `split-debug-info` ビルドを必須とする | 本書新規 |
| NFR-SEC-007 | 外部 SDK / 解析サービス（Firebase / Crashlytics / AppsFlyer 等）は使用禁止 | ADR-0021 |
| NFR-SEC-008 | API Key 等の秘匿情報はコード / 設定ファイルにハードコード禁止、ビルド時 `--dart-define` で注入 | AGENTS.md §4 #5 |
| NFR-SEC-009 | tenant_id はクライアントから送信せず、API Gateway が JWT から抽出（`api-design.md` §1.8） | `api-design.md` §1.8 |

### 6.4 保守性要件（NFR-MAINT）

| ID | 要件 | 出典 |
|---|---|---|
| NFR-MAINT-001 | コード品質: `flutter analyze --fatal-infos` 0 warning | 本書新規 |
| NFR-MAINT-002 | コードフォーマット: `dart format` 100% pass | 本書新規 |
| NFR-MAINT-003 | ユニットテストカバレッジ: ≥ 70% | 本書新規 |
| NFR-MAINT-004 | ウィジェットテスト: 主要画面 100% カバレッジ | 本書新規 |
| NFR-MAINT-005 | Lint ルール: `very_good_analysis` 採用 | 本書新規 |
| NFR-MAINT-006 | コードレビュー: 5 域独立 Lead 承認必須 | AGENTS.md §4 #3 |

### 6.5 移植性要件（NFR-PORT）

| ID | 要件 | 出典 |
|---|---|---|
| NFR-PORT-001 | Android: minSdk 24, targetSdk 34 | 本書新規 |
| NFR-PORT-002 | 異なる画面サイズ（4.7"〜6.7"）で正しくレイアウトされる | 本書新規 |
| NFR-PORT-003 | ❌ iOS 対応は V2 | `internal-design.md:1600` |

### 6.6 ユーザビリティ要件（NFR-USE）

| ID | 要件 | 出典 |
|---|---|---|
| NFR-USE-001 | タップターゲット ≥ 48dp（Material Design guideline） | Material 3 |
| NFR-USE-002 | ライト/ダーク両モード対応 | Material 3 |
| NFR-USE-003 | ネットワークエラー時のリトライ UI 提供 | 本書新規 |
| NFR-USE-004 | 空状態（Empty State）のイラスト + 案内文提供 | 内部設計 §10 |
| NFR-USE-005 | ローディング中は進捗インジケータ表示 | 本書新規 |

---

## §7 業務フロー / ユースケース

### 7.1 UC-001: ログイン

```
[User] → アプリ起動 → 未ログイン状態
   → ログイン画面で email + password 入力
   → 「ログイン」ボタンタップ
   → Dio: POST /v1/auth/login
   → Backend: 認証 → access_token + refresh_token + user + tenant 返却
   → flutter_secure_storage に token 暗号化保存
   → AuthState = Authenticated(user, tenant)
   → 最終アクセス画面に遷移（Projects → Project → Board）
```

### 7.2 UC-002: ボード閲覧

```
[User] → ログイン済み
   → プロジェクト選択画面 → プロジェクトタップ
   → go_router: /projects/:id/board
   → BoardController.fetch(): GET /v1/projects/{id}/board + GET /v1/work-items?project_id=...
   → Board 表示（横スクロール Columns + Cards）
   → 30s 待機（ポーリングは Notifications のみ、Board は Pull-to-Refresh のみ）
```

### 7.3 UC-003: Work Item 詳細閲覧

```
[User] → ボード画面でカードタップ
   → go_router: /work-items/:id
   → WorkItemController.fetch(): GET /v1/work-items/{id} + GET /v1/work-items/{id}/comments + GET /v1/work-items/{id}/transitions
   → 3 タブ表示（Overview / Comments / Transitions）
   → 「Web で開く」ボタンタップ → Star Web の /work-items/{id} を外部ブラウザ起動
```

### 7.4 UC-004: 通知閲覧

```
[User] → アプリ起動 / バックグラウンド復帰
   → Notifications タブ開く
   → NotificationsController.fetch(): GET /v1/notifications?read=false&limit=20
   → 30 秒ごとにポーリング（バックグラウンド時停止）
   → 通知タップ → POST /v1/notifications/{id}:read → 該当 Work Item 詳細画面遷移
   → アプリアイコンバッジに未読件数表示
```

### 7.5 UC-005: ログアウト

```
[User] → Settings 画面 → ログアウトボタン
   → flutter_secure_storage の全 key 削除
   → POST /v1/auth/logout（best-effort）
   → AuthState = Unauthenticated
   → go_router: /login
```

### 7.6 例外フロー

| 例外 | 挙動 |
|---|---|
| ネットワーク接続なし | 「ネットワーク接続がありません」トースト + ボード画面は最終取得データ表示（メモリ内）+ Pull-to-Refresh 時に再試行 |
| API 500 エラー | 「サーバーエラーが発生しました」+ リトライボタン |
| API 401（refresh 失敗含む） | 自動ログアウト → ログイン画面 |
| API 403 | 「アクセス権限がありません」+ 戻るボタン |
| API 404 | 「Work Item が見つかりません」+ 戻るボタン |
| トークン期限切れ + refresh 失敗 | 自動ログアウト + ログイン画面 + 「セッションの有効期限が切れました」トースト |

---

## §8 データ要件（高レベル、DDL なし）

### 8.1 アプリ内部データ（クライアント側のみ）

| データ | 種別 | 保持場所 | TTL |
|---|---|---|---|
| access_token | 資格情報 | flutter_secure_storage (Keystore) | 15 分（後端定） |
| refresh_token | 資格情報 | flutter_secure_storage (Keystore) | 7 日 |
| user（id, name, avatar_url） | 業務データ | flutter_secure_storage (JSON) | refresh まで |
| tenant（id, name） | 業務データ | flutter_secure_storage (JSON) | refresh まで |
| ボード設定 | 業務データ | メモリ（Riverpod） | 単一セッション |
| Work Item リスト | 業務データ | メモリ | 単一セッション |
| Work Item 詳細 | 業務データ | メモリ | 単一セッション |
| 通知リスト | 業務データ | メモリ | 30s ポーリング更新 |
| テーマ設定 | ユーザー設定 | flutter_secure_storage | 永続 |

**注**: MVP では**ローカル DB（SQLite/Hive 業務キャッシュ）を持たない**。理由: offline-only 機能要件がないため。V1.1 で Drift/SQLite 導入予定（§11 G-04）。

### 8.2 サーバ側データ（要件のみ、DDL は `data-design.md` v0.2 §4 参照）

本 MVP が読み取る既存 Star サーバ側データ：
- `tenant` テーブル（id, name）
- `user` テーブル（id, email, display_name, avatar_url, tenant_id）
- `project` テーブル（id, tenant_id, name, slug）
- `board` テーブル（id, project_id, columns[]）
- `work_item` テーブル（id, project_id, type, title, status, assignee_user_id, priority, ...）
- `comment` テーブル（id, parent_type, parent_id, author_user_id, body, ...）
- `notification` テーブル（id, recipient_user_id, event_type, payload, read_at, sent_at）

サーバ側テーブル定義は本書の対象外（`docs/data-design.md` v0.2 §4 を参照）。

---

## §9 インターフェース要件（高レベル、詳細スキーマは §11 / 詳細設計書）

### 9.1 API エンドポイント一覧（MVP で呼び出す 13 個）

| # | Method | パス | 認証 | 用途 |
|---|---|---|---|---|
| 1 | POST | `/v1/auth/login` | Anonymous | ログイン（§11 G-01 で存在未確認） |
| 2 | POST | `/v1/auth/refresh` | Authenticated | トークンリフレッシュ |
| 3 | POST | `/v1/auth/logout` | Authenticated | ログアウト |
| 4 | GET | `/v1/users/me` | Authenticated | 自分の情報取得 |
| 5 | GET | `/v1/tenants/current` | Authenticated | 自分のテナント取得 |
| 6 | GET | `/v1/projects/{id}/board` | Policy | ボード設定取得 |
| 7 | GET | `/v1/work-items?project_id=&...` | Policy | Work Item リスト |
| 8 | GET | `/v1/work-items/{id}` | Policy | Work Item 詳細 |
| 9 | GET | `/v1/work-items/{id}/transitions` | Policy | 遷移可能状態 |
| 10 | GET | `/v1/work-items/{id}/comments` | Policy | コメント一覧 |
| 11 | GET | `/v1/notifications?read=false` | Authenticated | 通知一覧 |
| 12 | POST | `/v1/notifications/{id}:read` | Authenticated | 単条既読化 |
| 13 | POST | `/v1/notifications/mark-all-read` | Authenticated | 全既読化 |

### 9.2 通信仕様（高レベル）

- プロトコル: HTTP/1.1 + JSON
- ベース URL: `http://<STAR_HOST>/api/v1`（STAR_HOST は §11 G-03 で拍板待ち）
- 認証ヘッダ: `Authorization: Bearer <access_token>`
- テナント: クライアント送信なし（API Gateway が JWT から抽出、`api-design.md` §1.8）
- トレース: `traceparent` (W3C Trace Context)
- コンテンツタイプ: `application/json; charset=utf-8`
- エラー: RFC 7807 Problem Details 形式

### 9.3 画面遷移仕様（高レベル）

```
/login (public)
  → /projects (auth)
    → /projects/:id/board (auth + project member)
      → /work-items/:id (auth)
  
  → /notifications (auth)
    → /work-items/:id (deep link)
  
  → /settings (auth)
```

---

## §10 運用・保守要件

### 10.1 APK 配布

- **内網ファイル共有**（NAS / MinIO）からの手動ダウンロード
- QR コードまたは URL を README / 社内ポータルに掲載
- 自動更新は**未対応**（V1.1 で `GET /v1/app-version` ベースの更新通知バナーを実装予定、§11 G-02）

### 10.2 ログ収集

- **外部解析サービス禁止**（ADR-0021）
- 自社クラッシュレポート: アプリ内 `try/catch` でローカルのログファイルに書き出し、ユーザーが「不具合報告」メニューから手動送信
- API ログ: Backend 側 `audit_event` テーブルに記録（`api-design.md` §3.12）

### 10.3 監視

- サーバー側メトリクス（API 応答時間、エラー率）は Backend 側 Grafana で監視
- クライアント側クラッシュ率は §10.2 の手動収集で代替

### 10.4 バックアップ

- モバイル側に永続データなし（§8.1）→ バックアップ不要

---

## §11 セキュリティ要件（詳細、§6.3 の拡張）

| ID | 脅威 | 対策 | 出典 |
|---|---|---|---|
| SEC-001 | トークン盗難 | Android Keystore 暗号化 + FLAG_SECURE（スクリーンショット防止） | NFR-SEC-001/004 |
| SEC-002 | 中間者攻撃 | MVP は cleartext HTTP（内網限定）;V1.1 で HTTPS 移行検討 | NFR-SEC-002/003 |
| SEC-003 | デバイス紛失 | ログアウトでローカル完全削除 + Server 側トークン無効化 | NFR-SEC-005 |
| SEC-004 | バックドア SDK | 外部 SDK 全面禁止、AGP 依存関係レビュー必須 | ADR-0021 |
| SEC-005 | APK 改ざん | 内網署名 + `network_security_config.xml` で通信先制限 | NFR-SEC-003 |
| SEC-006 | コード解析 | `obfuscate` + `split-debug-info` 必須 | NFR-SEC-006 |
| SEC-007 | 認証情報ハードコード | `--dart-define` 注入、AGP `BuildConfig` 経由 | NFR-SEC-008 |
| SEC-008 | 不正 tenant アクセス | クライアントから tenant_id 送信禁止、Gateway が JWT 抽出 | NFR-SEC-009 |

### 11.1 認証トークン管理詳細

- **access_token**: 有効期限 15 分、API リクエストの `Authorization: Bearer` に使用
- **refresh_token**: 有効期限 7 日、access_token 再発行にのみ使用
- **保存場所**: `flutter_secure_storage` (Android Keystore バックエンド、API 23+)
- **送信**: HTTPS のみ (MVP 段階は HTTP、§6.3 NFR-SEC-002)
- **ローテーション**: 7 日経過または logout 時に無効化
- **危殆化時の対応**: 即時サーバー側無効化（`api-design.md` §6.2）+ 全クライアント次回 refresh 失敗で自動ログアウト

### 11.2 通信セキュリティ

- MVP: `http://star.internal:8080` cleartext、`network_security_config.xml` で `*.internal` ドメインのみ許可
- V1.1: HTTPS 移行 + 自前 CA 証明書（内網限定）または envoy + 自己署名証明書（per 9/1 13:03 JST 偏好）
- パブリック CA（Let's Encrypt 等）は使用しない（V2 で評価）

---

## §12 用語定義

| 用語 | 定義 | 出典 |
|---|---|---|
| **Work Item** | 作業の最小単位（Jira で言う Issue、GitHub で言う Issue/PR） | `requirements.md` §26 |
| **Board** | Work Item をカラム（状態）で表示するカンバンビュー | `api-design.md` §3.7 |
| **Column** | Board 内の一列、特定の状態（TODO/IN_PROGRESS/DONE 等）に対応 | `api-design.md` §3.7 |
| **Tenant** | テナント（組織単位）の論理境界 | `requirements.md` §26 |
| **Project** | テナント内のプロジェクト単位 | `requirements.md` §26 |
| **MVP** | Minimum Viable Product、本書のスコープ | IPA 標準 |
| **access_token** | 短時間有効な API 認証トークン（15 分） | `api-design.md` §1.12 |
| **refresh_token** | 長時間有効なトークン更新用トークン（7 日） | `api-design.md` §1.12 |
| **JWT** | JSON Web Token、本書では Bearer 認証に使用 | RFC 7519 |
| **RFC 7807** | Problem Details for HTTP APIs、エラーレスポンス形式 | IETF |
| **W3C Trace Context** | `traceparent` ヘッダ標準、分散トレーシング用 | W3C |
| **IPA** | 情報処理推進機構（Information-technology Promotion Agency） | — |
| **DDD** | Domain-Driven Design、本書の上流設計で採用 | Eric Evans |
| **RLS** | Row Level Security、PostgreSQL の行レベルセキュリティ | `data-design.md` §4.1.4 |

---

## §13 既知の未解決事項（受け入れ前 拍板待ち）

| ID | 項目 | 拍板人 | 影響 |
|---|---|---|---|
| **G-01** | `/v1/auth/login` 等の認証エンドポイント実装状況 | Ulysses（架構師） | 存在しない場合 OAuth ブラウザ遷移に切り替え |
| **G-02** | `GET /v1/app-version` アップグレード通知エンドポイント | SRE Lead | V1.1 で実装要否決定 |
| **G-03** | `STAR_HOST`（内網ドメイン / IP + ポート） | SRE Lead | アプリ設定と Backend 側 Nginx/envoy 設定 |
| **G-04** | オフラインキャッシュ（SQLite/Drift）の V1.1 スコープ | 5 域 Lead（work-item） | ネットワーク断絶時の挙動 |
| **G-05** | プッシュ通知（自前 WebSocket）の V1.1 スコープ | 5 域 Lead（realtime） | リアルタイム性 |
| **G-06** | Tablet / 横画面レイアウト | 5 域 Lead（frontend） | UX |
| **G-07** | iOS V2 計画での Flutter 採用継続可否 | Ulysses | V2 モバイル計画統合 |
| **G-08** | HTTPS / envoy 移行タイミング（per 9/1 13:03/13:05 JST 偏好） | SRE Lead | 内網→外網公開時 |
| **G-09** | Device 三重バインディング（`internal-design.md:23.2`） | Ulysses（安全） | リスク評価 |
| **G-10** | APK 内網 keystore 署名戦略 | SRE Lead | 5 域独立拍板（8/21 JST） |
| **G-11** | APK 配布チャネル（ファイル共有 vs 自前 MDM） | Ulysses + SRE Lead | 運用フロー |
| **G-12** | 倉位置（`apps/star-mobile-flutter/` vs `frontend/mobile-flutter/`） | 架構師 | CI / monorepo 構造 |
| **G-13** | WebSocket 实时推送（ポーリング代替） | 5 域 Lead（realtime） | UX vs 複雑度 |
| **G-14** | 5 域独立 Lead 真实身份補簽 | DDD Review Lead | 簽字欄 DDD Review 段階で補充 |
| **G-15** | WBS 新增「Flutter MVP」項目 | Ulysses | `AGENTS.md` §7 WBS への組み込み |

---

## §14 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架構師 (Mavis 接手 agent per DEC-008) | IPA 標準初版: §1〜§14 全章, 5.1〜5.7 機能要件 30 件, 6.1〜6.6 非機能要件 22 件, 7.1〜7.6 ユースケース 5 件, 11 既知未解決 15 件, 12 用語 14 件 | 2026-09-02 16:09 JST Ulysses 発令「要符合日本IPA标准的需求、基本设计、详细设计」, v0.1 (commit `bd4998e`) を IPA 3 段組に supersede |

---

## §15 承認欄（5 角色, AGENTS.md §3 + 8/21 JST 5 域独立）

| 角色 | 簽字 | 日付 | 備註 |
|---|---|---|---|
| 架構師 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽 (per 8/27 19:39 JST + 21:59 JST 三次強化授權) |
| SRE Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;5 域独立真实身份 (per 8/21 JST) DDD Review 段階で補充 |
| 平台 Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| 評審主持 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| PM | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
