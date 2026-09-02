# Star Mobile Flutter MVP — 基本設計書

> **基準**: 日本 IPA（情報処理推進機構）SEC ソフトウェア基本設計書 標準章立て
> **作成日**: 2026-09-02
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **バージョン**: v1.1 (UAT 完全版, 2026-09-02 16:27 JST)
> **前身**: v1.0 (read-only, commit `6bd6aa2`, 2026-09-02 16:14 JST) → UAT 範囲追加により v1.1 へ全面書き換え
> **上流要件定義書**: `D:\Star\docs\mobile-flutter-mvp\01-requirements.md` v1.1
> **Pre-IPA 草稿**: `docs/architecture/2026-09-02-upgrade/spec/mobile/01-flutter-mvp-design.md` v0.1 (commit `bd4998e`)

---

## §1 目的

本文書は、Star Mobile Flutter MVP の **UAT レベル** の基本設計を定義する。要件定義書 `01-requirements.md` v1.1 で定義した要件（FR-AUTH-001〜FR-WS-008、計 58 機能要件 + 37 非機能要件）を満たすための**システムアーキテクチャ・コンポーネント構成・データフロー・外部インターフェース概要・状態遷移・デプロイメント構成**を記述する。

**v1.0 → v1.1 の主要変更点**:
- ❌ 読み取り専用 → ✅ 核心写操作（状態遷移 / コメント / フィールド編集）
- ❌ メモリキャッシュ → ✅ オフラインキャッシュ（Drift/SQLite + SQLCipher + SyncQueue + 競合解決）
- ❌ 30s REST ポーリング → ✅ 自前 WebSocket + REST フォールバック

---

## §2 適用範囲

### 2.1 文書階層における位置

```
[上流] docs/requirements.md v2.0              (Star プラットフォーム全体要件)
       docs/mobile-flutter-mvp/01-requirements.md v1.1  (本 UAT 要件, 本書の上流)

[本書] docs/mobile-flutter-mvp/02-basic-design.md v1.1  (本 UAT 基本設計)

[下流] docs/mobile-flutter-mvp/03-detailed-design.md v1.1  (本 UAT 詳細設計)
       docs/mobile-flutter-mvp/{10-implementation-report,99-changelog}.md (実装時に作成)
```

### 2.2 In Scope / Out of Scope

`01-requirements.md` v1.1 §2 と同じ。MVP → UAT 拡張範囲を反映。

---

## §3 前提条件・制約事項

### 3.1 技術的前提（v1.0 拡張）

| # | 前提 | 出典 |
|---|---|---|
| 1 | Flutter 3.24+ / Dart 3.5+ が開発環境で利用可能 | v1.0 継承 |
| 2 | Android Studio / VS Code + Flutter 拡張で開発 | v1.0 継承 |
| 3 | Backend API（20 エンドポイント）+ WebSocket Service が production で稼働 | **UAT 拡張** |
| 4 | `flutter_secure_storage` 9.x が Android Keystore を使用可能 | v1.0 継承 |
| 5 | `drift` 2.x + `drift_flutter` 0.2.x が Android/iOS 両対応 | **UAT 拡張** |
| 6 | `sqlcipher` 4.x が Android NDK 上で動作 | **UAT 拡張** |
| 7 | `web_socket_channel` 3.x が WSS 接続可能 | **UAT 拡張** |
| 8 | `connectivity_plus` 6.x で接続性監視可能 | v1.0 継承 |

### 3.2 組織的制約

- 5 域独立 Lead（work-item / board / notification / auth / frontend / realtime）個別承認（8/21 JST）
- Mavis 接手代簽 Ulysses（per 8/27 19:39 JST + 21:59 JST 三次強化）
- token-OLU 制（1 SRE·周 = 1.2M、`STAR-OLU-001.md` v0.1）
- AGENTS.md §4 13 項守門全部適用

### 3.3 環境変数制約

- 秘匿情報は `.env` に書かない
- ビルド時 `--dart-define=KEY=VALUE` で注入
- 開発環境では `flutter run --dart-define-from-file=dev.json`（dev.json は .gitignore）

### 3.4 同期キュー容量制約

- 1 ユーザーあたり最大 100 件（超過時は古い順に drop + 通知）
- 1 件あたり最大 10KB

---

## §4 システムアーキテクチャ

### 4.1 全体アーキテクチャ図（UAT 拡張）

```
┌────────────────────────────────────────────────────────────────────┐
│  Android Flutter App (本 UAT MVP)                                   │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Presentation Layer (UI)                                      │  │
│  │  - Material 3 Screens (8 画面)                                │  │
│  │  - Widgets (Board Card / Notification Tile / Sync Banner)     │  │
│  │  - go_router (宣言的ルーティング)                              │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                          ↕ (Riverpod)                                │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Application Layer (Controllers)                               │  │
│  │  - AuthController                                             │  │
│  │  - BoardController                                            │  │
│  │  - WorkItemController (READ + WRITE)                          │  │
│  │  - CommentsController (READ + POST)                           │  │
│  │  - TransitionsController                                      │  │
│  │  - NotificationsController                                    │  │
│  │  - SyncStatusController (NEW)                                 │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                          ↕                                            │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Domain Layer (Models / States)                                │  │
│  │  - User / Tenant / Board / Column / WorkItem (Read)           │  │
│  │  - WorkItemEditCommand / TransitionCommand / CommentCreate    │  │
│  │  - SyncQueueItem / ConflictReport / PushEvent                 │  │
│  │  - AuthState (sealed) / Result<T,E> / SyncState (sealed)      │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                          ↕                                            │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Data Layer                                                     │  │
│  │  ┌──────────────────────────────────────────────────────┐    │  │
│  │  │ Repositories (interface)                              │    │  │
│  │  │  - AuthRepository / BoardRepository                   │    │  │
│  │  │  - WorkItemRepository (read + write)                  │    │  │
│  │  │  - CommentRepository / NotificationRepository         │    │  │
│  │  │  - SyncRepository (queue + conflict)                  │    │  │
│  │  └──────────────────────────────────────────────────────┘    │  │
│  │  ┌──────────────────────────────────────────────────────┐    │  │
│  │  │ Remote Sources (REST + WS)                            │    │  │
│  │  │  - AuthApi / UserApi / BoardApi / WorkItemApi          │    │  │
│  │  │  - CommentApi / NotificationApi                        │    │  │
│  │  │  - WebSocketService (NEW, wss://star.internal:8080)   │    │  │
│  │  │  - DioClient + 3 Interceptors                          │    │  │
│  │  └──────────────────────────────────────────────────────┘    │  │
│  │  ┌──────────────────────────────────────────────────────┐    │  │
│  │  │ Local Sources (Offline Cache) (NEW)                    │    │  │
│  │  │  - OfflineDatabase (Drift, SQLCipher)                  │    │  │
│  │  │    7 tables: cached_work_items / cached_boards /        │    │  │
│  │  │    cached_columns / cached_comments / cached_notifs /  │    │  │
│  │  │    sync_queue / conflict_reports                       │    │  │
│  │  └──────────────────────────────────────────────────────┘    │  │
│  │  ┌──────────────────────────────────────────────────────┐    │  │
│  │  │ Sync Engine (NEW)                                      │    │  │
│  │  │  - SyncQueueService: enqueue / dequeue / retry          │    │  │
│  │  │  - ConflictResolver: detect / prompt / resolve          │    │  │
│  │  │  - ConnectivityWatcher: online/offline 状態             │    │  │
│  │  └──────────────────────────────────────────────────────┘    │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                      │
└────────────────────────────────────────────────────────────────────┘
                          ↕ HTTP REST (cleartext) / WSS (TLS)
                          ↕ Authorization: Bearer <jwt>
┌────────────────────────────────────────────────────────────────────┐
│  Star プラットフォーム (Backend, 既存 + UAT 拡張)                    │
│  API Gateway → work-core → PostgreSQL                               │
│  WebSocket Service (axum-WS) ←→ NATS JetStream → work-core events  │
└────────────────────────────────────────────────────────────────────┘
```

### 4.2 アーキテクチャパターン

| 項目 | 選択 | 理由 |
|---|---|---|
| **全体** | クリーンアーキテクチャ 4 層 + Sync Engine 副層 | テスタビリティ + 依存方向 + 同期ロジック隔離 |
| **状態管理** | Riverpod 2.5+ | 既存 |
| **非同期** | `async/await` + `Stream` | 既存 |
| **データ永続化** | `flutter_secure_storage` (token) + `Drift/SQLCipher` (業務 + sync) | **UAT 拡張**: オフライン対応 |
| **ナビゲーション** | `go_router` 14+ | 既存 |
| **JSON 解析** | `freezed` + `json_serializable` | 既存 |
| **HTTP 通信** | `Dio` 5.7+ + `dio_smart_retry` | 既存 |
| **WebSocket** | `web_socket_channel` 3.x + 独自 `WebSocketService` ラッパー | **UAT 新規** |
| **DB** | `drift` 2.x (type-safe SQL) + `drift_flutter` (Flutter 統合) | **UAT 新規** |
| **暗号化 DB** | `sqlcipher_flutter_libs` (Android NDK) | **UAT 新規** |

### 4.3 モジュール分割（feature-first + 横串 sync 層）

```
lib/
├── main.dart                  # ProviderScope + MaterialApp.router
├── app/                       # アプリ全体設定
│   ├── app.dart               # MaterialApp.router
│   ├── router.dart            # go_router 設定
│   └── theme.dart             # Material 3 ColorScheme
├── core/                      # アプリ全体共通基盤
│   ├── api/                   # Dio + 3 Interceptors (v1.0 継承)
│   ├── auth/                  # Token 管理
│   ├── ws/                    # WebSocket 接続管理 (UAT 新規)
│   ├── db/                    # Drift database + SQLCipher (UAT 新規)
│   ├── sync/                  # SyncQueue + ConflictResolver (UAT 新規)
│   ├── connectivity/          # connectivity_plus ラッパー (UAT 新規)
│   ├── env/                   # 環境設定
│   └── result/                # Result<T, AppError> sealed
├── features/                  # 機能別（垂直スライス）
│   ├── auth/                  # FR-AUTH
│   ├── board/                 # FR-BOARD
│   ├── work_item/             # FR-WORK (READ + WRITE)
│   ├── comments/              # FR-WORK-008 (UAT 新規ディレクトリ)
│   ├── transitions/           # FR-WORK-009 (UAT 新規ディレクトリ)
│   ├── notifications/         # FR-NOTIF (REST + WS)
│   ├── projects/              # FR-PROJ
│   ├── settings/              # FR-SETTINGS
│   └── sync_conflicts/        # FR-OFFLINE-006 競合解決画面 (UAT 新規)
└── shared/                    # 共通 UI コンポーネント
    ├── widgets/               # EmptyState / ErrorState / LoadingState / SyncBanner
    └── utils/                 # date_format / priority_color
```

**依存方向** (UAT 拡張):

```
Presentation → Application → Domain ← Data
                                 ↑
                          (Domain のみが Data に依存)

Data 層 = Remote Sources + Local Sources + Sync Engine
                ↓
         (全て Domain interface 経由)
```

**重要な追加依存** (UAT 拡張):
- `WebSocketService` は `Domain.PushEvent` を expose
- `OfflineDatabase` は `Domain` model を read/write
- `SyncQueueService` は `Domain.SyncQueueItem` を enqueue/dequeue
- `ConflictResolver` は `Domain.ConflictReport` を生成

### 4.4 オフライン/オンライン データ取得戦略（UAT 新規）

| データ種別 | 戦略 | 理由 |
|---|---|---|
| **Work Item 一覧** | ① Drift ローカル → ② REST 取得 → ③ マージ | オフライン即応 + 最新性 |
| **Work Item 詳細** | ① Drift ローカル → ② REST 取得（force_fresh=true） | 編集結果を反映 |
| **コメント** | ① Drift ローカル → ② REST 取得（増分） | 編集中の未同期コメント含む |
| **通知** | ① Drift ローカル → ② REST 取得 → ③ WS 推送 | WS 切断時 REST フォールバック |
| **ボード設定** | ① Drift ローカル → ② REST 取得 | ほぼ静的なので cache 優先 |

**Optimistic UI**:
- 編集操作 (FR-WORK-007/008/009) は即座に Drift 更新 + UI 反映
- サーバ応答は後から非同期で反映
- 競合時 (409 Conflict) は ConflictResolver に委譲

---

## §5 コンポーネント設計

### 5.1 主要コンポーネント一覧（v1.0 13 → v1.1 22 個に拡張）

| コンポーネント | 責務 | 配置 | UAT 区分 |
|---|---|---|---|
| **DioClient** | Dio シングルトン、baseUrl、interceptor チェーン | `core/api/dio_client.dart` | v1.0 継承 |
| **AuthInterceptor** | Bearer token 注入、401 検知 → refresh → retry | `core/api/auth_interceptor.dart` | v1.0 継承 |
| **ErrorInterceptor** | RFC 7807 → `AppError` sealed に変換 | `core/api/error_interceptor.dart` | v1.0 継承 |
| **LoggingInterceptor** | 開発時のみログ出力、リリース時 no-op | `core/api/logging_interceptor.dart` | v1.0 継承 |
| **TokenStorage** | Keystore 経由の token 永続化 | `core/auth/token_storage.dart` | v1.0 継承 |
| **AuthController** | 認証状態管理 | `features/auth/presentation/auth_controller.dart` | v1.0 継承 |
| **BoardController** | ボード取得 + キャッシュ | `features/board/presentation/board_controller.dart` | v1.0 継承 |
| **WorkItemController** | 詳細取得 (3 タブ並列) | `features/work_item/presentation/work_item_controller.dart` | v1.0 継承 |
| **NotificationsController** | 30s ポーリング | `features/notifications/presentation/notifications_controller.dart` | v1.0 拡張 (WS 受信追加) |
| **WebSocketService** | **WSS 接続管理 + reconnect + subscribe** | `core/ws/websocket_service.dart` | **UAT 新規** |
| **OfflineDatabase** | **Drift データベース (7 テーブル) + SQLCipher** | `core/db/offline_database.dart` | **UAT 新規** |
| **SyncQueueService** | **同期キューの enqueue / dequeue / retry** | `core/sync/sync_queue_service.dart` | **UAT 新規** |
| **ConflictResolver** | **競合検出 + ユーザー解決 UI 提示** | `core/sync/conflict_resolver.dart` | **UAT 新規** |
| **ConnectivityWatcher** | **オンライン/オフライン 状態監視** | `core/connectivity/connectivity_watcher.dart` | **UAT 新規** |
| **SyncStatusController** | **同期状態の可視化** | `features/settings/presentation/sync_status_controller.dart` | **UAT 新規** |
| **WorkItemWriteService** | **Work Item 部分更新 (priority/assignee/description/due_date)** | `features/work_item/data/work_item_write_service.dart` | **UAT 新規** |
| **CommentsController** | **コメント投稿 (POST)** | `features/comments/presentation/comments_controller.dart` | **UAT 新規** |
| **TransitionsController** | **状態遷移実行 (POST :transition)** | `features/transitions/presentation/transitions_controller.dart` | **UAT 新規** |
| **ConflictResolutionScreen** | **競合解決 UI (3 候補から選択)** | `features/sync_conflicts/presentation/conflict_resolution_screen.dart` | **UAT 新規** |
| **SyncBanner** | **オフライン / 同期中 / 競合待ち の画面下部バナー** | `shared/widgets/sync_banner.dart` | **UAT 新規** |
| **PushEventRouter** | **WS 受信イベントを各 Controller に振り分け** | `core/ws/push_event_router.dart` | **UAT 新規** |
| **LogRedactor** | **ログ送信前 PII / token redact** | `core/utils/log_redactor.dart` | **UAT 新規** |

### 5.2 主要シーケンス図

#### 5.2.1 ログイン + WebSocket 接続確立 (UAT 拡張)

```
User       LoginScreen  AuthController   AuthApi     TokenStorage  WebSocketService     Backend
 │             │             │              │              │                │               │
 │ 起動        │             │              │              │                │               │
 │ ───────────▶│             │              │              │                │               │
 │             │ build()     │              │              │                │               │
 │             │ ───────────▶│              │              │                │               │
 │             │             │ POST /auth/login           │                │               │
 │             │             │ ──────────────────────────▶│                │               │
 │             │             │              │ 200 {tokens,user,tenant}     │               │
 │             │             │              │ ◀────────────────────────────│               │
 │             │             │ save tokens  │              │                │               │
 │             │             │ ──────────────────────────▶│                │               │
 │             │             │              │              │ connect WSS   │               │
 │             │             │              │              │ ──────────────▶│               │
 │             │             │              │              │                │ WS Upgrade    │
 │             │             │              │              │                │ ──────────────▶│
 │             │             │              │              │                │ 101 Switching │
 │             │             │              │              │                │ ◀──────────────│
 │             │             │              │              │ subscribe      │               │
 │             │             │              │              │ (work_item,notification)         │
 │             │             │              │              │ ──────────────▶│               │
 │             │             │              │              │                │ 200 OK         │
 │             │             │              │              │ ◀──────────────│               │
 │             │             │ AuthState.Authenticated      │                │               │
 │             │ /projects   │              │              │                │               │
 │             │ ◀───────────│              │              │                │               │
```

#### 5.2.2 オフライン状態遷移 + 接続回復後同期 (UAT 新規)

```
User      WorkItemDetailScreen  WorkItemController  SyncQueueService  OfflineDatabase  ConnectivityWatcher  Backend
 │              │                    │                    │                  │                   │                │
 │ 地下鉄に乗る │                    │                    │                  │                   │                │
 │              │                    │                    │                  │ 接続喪失検知       │                │
 │              │                    │                    │                  │ ──────────────────▶│                │
 │              │                    │                    │                  │                   │                │
 │              │                    │                    │                  │ (オフライン中)     │                │
 │              │                    │                    │                  │                   │                │
 │ 状態遷移実行 │                    │                    │                  │                   │                │
 │ ────────────▶│                    │                    │                  │                   │                │
 │              │ transitionExecute() │                    │                  │                   │                │
 │              │ ──────────────────▶│                    │                  │                   │                │
 │              │                    │ 1. Drift 即座更新  │                  │                   │                │
 │              │                    │ 2. SyncQueue 追記 │                  │                   │                │
 │              │                    │ ──────────────────▶│                  │                   │                │
 │              │                    │ 3. Optimistic UI   │                  │                   │                │
 │              │                    │ 4. Idempotency-Key 生成              │                   │                │
 │              │                    │                    │                  │                   │                │
 │ 地下を出る   │                    │                    │                  │ 接続回復検知       │                │
 │              │                    │                    │                  │ ◀──────────────────│                │
 │              │                    │                    │ 同期開始         │                   │                │
 │              │                    │                    │ ◀─────────────────│                   │                │
 │              │                    │                    │ POST /work-items/{id}:transition      │                │
 │              │                    │                    │ (Idempotency-Key)                     │                │
 │              │                    │                    │ ────────────────────────────────────────────────────────▶│
 │              │                    │                    │ 200 OK (新状態)                                       │
 │              │                    │                    │ ◀────────────────────────────────────────────────────────│
 │              │                    │                    │ SyncQueue dequeue                                       │
 │              │                    │                    │ (Drift を最新サーバ状態に更新)                         │
 │              │                    │ 5. UI をサーバ結果で更新                 │                                │
 │              │ ◀──────────────────│                    │                                                  │
```

#### 5.2.3 WebSocket 推送受信 (UAT 新規)

```
Backend   WebSocketService  PushEventRouter  BoardController  WorkItemController  NotificationsController
   │             │                │                  │                  │                       │
   │ Work Item   │                │                  │                  │                       │
   │ 状態変更    │                │                  │                  │                       │
   │ ────────────▶│                │                  │                  │                       │
   │ push event  │                │                  │                  │                       │
   │             │ JSON parse     │                  │                  │                       │
   │             │ ──────────────▶│                  │                  │                       │
   │             │                │ resource_type 判定                  │                       │
   │             │                │                  │                  │                       │
   │             │                │ 'work_item.updated'                 │                       │
   │             │                │ ──────────────────▶│                │                       │
   │             │                │                  │ 該当 WorkItem 更新│                       │
   │             │                │                  │ UI 反映 (差分)   │                       │
   │             │                │                  │                  │                       │
   │             │                │ 'notification.new'                  │                       │
   │             │                │ ──────────────────────────────────────────────────────────────▶│
   │             │                │                  │                  │                       │ 通知先頭に追加
   │             │                │                  │                  │                       │ UI 反映
```

#### 5.2.4 競合解決 (UAT 新規)

```
User    WorkItemDetailScreen  WorkItemController  Backend  SyncQueueService  ConflictResolver  ConflictResolutionScreen
 │              │                    │                │             │                   │                       │
 │ オフライン中 │                    │                │             │                   │                       │
 │ 編集         │                    │                │             │                   │                       │
 │              │                    │                │             │                   │                       │
 │ 接続回復     │                    │                │             │                   │                       │
 │              │                    │                │             │ 同期試行          │                       │
 │              │                    │                │             │ ────────────────▶│                       │
 │              │                    │                │             │ 409 Conflict      │                       │
 │              │                    │                │             │ (サーバ版: v3)     │                       │
 │              │                    │                │             │ 競合検出          │                       │
 │              │                    │                │             │ ConflictReport 作成                       │
 │              │                    │                │             │ ────────────────▶│                       │
 │              │                    │                │             │                   │                       │
 │              │                    │                │             │                   │ /sync-conflicts      │
 │              │                    │                │             │                   │ ────────────────────▶│
 │              │                    │                │             │                   │ サーバ版 v3 /       │
 │              │                    │                │             │                   │ ローカル版 v2 を    │
 │              │                    │                │             │                   │ 並べて表示           │
 │              │                    │                │             │                   │                       │
 │ 「サーバ版を採用」タップ          │                │             │                   │                       │
 │ ────────────────────────────────────────────────────────────────────────────────▶│                       │
 │              │                    │                │             │                   │ ユーザー選択記録     │
 │              │                    │                │             │                   │ ローカル v2 を破棄  │
 │              │                    │                │             │                   │ サーバ v3 を再取得  │
 │              │                    │ ◀─────────────────────────────────────────────────────────────────────────────│
 │              │ WorkItem v3 表示  │                │             │                   │                       │
```

### 5.3 状態管理モデル（v1.1 拡張）

#### 5.3.1 AuthState（v1.0 継承）

```dart
sealed class AuthState {}
class Unauthenticated extends AuthState { ... }
class Authenticating extends AuthState {}
class Authenticated extends AuthState {
  final User user;
  final Tenant tenant;
}
class AuthError extends AuthState { ... }
```

#### 5.3.2 SyncState（UAT 新規）

```dart
sealed class SyncState {}
class SyncIdle extends SyncState {}                              // 同期中ではない
class SyncInProgress extends SyncState {                          // 同期中
  final int remainingItems;
  final int totalItems;
}
class SyncSuccess extends SyncState { final DateTime at; }        // 最終成功時刻
class SyncFailed extends SyncState { final String error; }        // 最終失敗
class SyncConflicts extends SyncState {                            // 競合あり
  final List<ConflictReport> reports;
}
class SyncOffline extends SyncState {}                            // オフライン
```

#### 5.3.3 PushState（UAT 新規）

```dart
sealed class PushState {}
class PushDisconnected extends PushState { final DateTime? lastConnectedAt; }
class PushConnecting extends PushState {}
class PushConnected extends PushState {
  final List<String> subscribedResources;
}
class PushReconnecting extends PushState {
  final int attempt;
  final Duration nextBackoff;
}
class PushError extends PushState { final String message; }
```

#### 5.3.4 BoardController (UAT 拡張: SyncQueue 反映)

```dart
class BoardController extends AsyncNotifier<Board> {
  Future<Board> build(String projectId) async {
    // 1. Drift ローカル から即時ロード (オフライン対応)
    final localBoard = await ref.read(offlineDatabaseProvider).getBoard(projectId);
    if (localBoard != null) {
      state = AsyncValue.data(localBoard);
    }
    
    // 2. REST 取得 (最新)
    final remoteBoard = await _boardApi.getBoard(projectId);
    
    // 3. Drift 保存
    await ref.read(offlineDatabaseProvider).saveBoard(remoteBoard);
    
    // 4. 未同期バッジ付与 (SyncQueue 確認)
    final syncItems = await ref.read(syncQueueServiceProvider).pendingForProject(projectId);
    return remoteBoard.withPendingBadges(syncItems);
  }
}
```

---

## §6 データモデル概要

### 6.1 ドメインモデル一覧（v1.0 12 → v1.1 18 個に拡張）

| モデル | 主要フィールド | 出典 | UAT 区分 |
|---|---|---|---|
| `User` | id, email, displayName, avatarUrl | `api-design.md` §3.2 | v1.0 継承 |
| `Tenant` | id, name | `api-design.md` §3.2 | v1.0 継承 |
| `AuthTokens` | accessToken, refreshToken, expiresAt | `api-design.md` §6.2 | v1.0 継承 |
| `Project` | id, tenantId, name, slug, lastAccessedAt | `api-design.md` §3.4 | v1.0 継承 |
| `BoardConfig` | id, projectId, columns: List<Column> | `api-design.md` §3.7:668 | v1.0 継承 |
| `Column` | id, stateId, name, order, workItemIds | `api-design.md` §3.7 | v1.0 継承 |
| `WorkItemSummary` | id, title, status, priority, assignee, updatedAt | `api-design.md` §3.5:624 | v1.0 継承 |
| `WorkItem` | 全フィールド | `api-design.md` §3.5:626 | v1.0 継承 |
| `Comment` | id, author, body, createdAt, mentions | `api-design.md` §3.10:700 | v1.0 継承 |
| `Transition` | from, to, requiredPermission, isAllowed | `api-design.md` §3.5:630 | v1.0 継承 |
| `Notification` | id, eventType, payload, readAt, sentAt | `api-design.md` §3.16:787 | v1.0 継承 |
| `AppError` (sealed) | Network / Unauthorized / Conflict / ... | RFC 7807 準拠 | v1.0 継承 |
| **WorkItemEditCommand** | **id, patch fields, idempotencyKey, version** | **UAT 新規** |
| **CommentCreateCommand** | **workItemId, body, mentions, idempotencyKey** | **UAT 新規** |
| **TransitionCommand** | **workItemId, toState, idempotencyKey** | **UAT 新規** |
| **SyncQueueItem** | **id, kind (edit/comment/transition), payload, idempotencyKey, status, retryCount, createdAt** | **UAT 新規** |
| **ConflictReport** | **id, workItemId, serverVersion, localVersion, conflictedFields, detectedAt** | **UAT 新規** |
| **PushEvent** (sealed) | **WorkItemUpdated / WorkItemCommented / NotificationNew / Unknown** | **UAT 新規** |

### 6.2 ER 概要（概念レベル、DDL は `data-design.md` v0.2 §4）

```
[サーバ側 — 既存]

Tenant ─┬─ User ─┬─ AuthSession
        │        ├─ Notification
        │        └─ NotificationChannel
        │
        └─ Project ─┬─ BoardConfig ─ Column ─ WorkItem
                    ├─ WorkItem ─┬─ Comment
                    │            ├─ Transition (state machine)
                    │            └─ AcceptanceCriterion
                    └─ Sprint

[クライアント側 — 新規 (Drift/SQLCipher)]

cached_work_items
  └─ cached_comments (FK: work_item_id)

cached_boards
  └─ cached_columns (FK: board_id)

cached_notifications

sync_queue
  └─ kind: edit | comment | transition
  └─ status: pending | in_progress | failed | dropped
  └─ payload: JSON
  └─ idempotency_key: UUID v7
  └─ retry_count: int
  └─ last_error: text?

conflict_reports
  └─ work_item_id
  └─ server_version: JSON
  └─ local_version: JSON
  └─ conflicted_fields: JSON
  └─ resolution: server | local | merge | pending
```

**Drift テーブル DDL 詳細は `03-detailed-design.md` §6.3**。

### 6.3 サーバ側データ要件（UAT 拡張）

- `idempotency_keys` テーブル（既存? V1.0 経由で確認要、§11 G-17）
- WS 接続管理（既存 / V1.0 経由で確認要、§11 G-18）
- WS サブスクリプション resource_types に `work_item` と `notification` 追加（§11 G-16）

---

## §7 外部インターフェース設計概要

### 7.1 REST API エンドポイント（13 → 20）

| # | Method | パス | 認証 | 用途 | 詳細設計書 |
|---|---|---|---|---|---|
| 1 | POST | `/v1/auth/login` | Anon | ログイン | §5.1.1 |
| 2 | POST | `/v1/auth/refresh` | Auth | refresh | §5.1.2 |
| 3 | POST | `/v1/auth/logout` | Auth | logout | §5.1.3 |
| 4 | GET | `/v1/users/me` | Auth | 自分情報 | §5.2.1 |
| 5 | GET | `/v1/tenants/current` | Auth | テナント情報 | §5.2.2 |
| 6 | GET | `/v1/projects/{id}/board` | Policy | ボード設定 | §5.3.1 |
| 7 | GET | `/v1/work-items?project_id=...` | Policy | WI リスト | §5.3.2 |
| 8 | GET | `/v1/work-items/{id}` | Policy | WI 詳細 | §5.3.3 |
| 9 | GET | `/v1/work-items/{id}/transitions` | Policy | 遷移可能 | §5.3.4 |
| 10 | GET | `/v1/work-items/{id}/comments` | Policy | コメント | §5.3.5 |
| 11 | GET | `/v1/notifications?read=false` | Auth | 通知 | §5.4.1 |
| 12 | POST | `/v1/notifications/{id}:read` | Auth | 既読 | §5.4.2 |
| 13 | POST | `/v1/notifications/mark-all-read` | Auth | 全既読 | §5.4.3 |
| **14** | **PATCH** | **`/v1/work-items/{id}`** | **Policy + If-Match** | **部分更新** | **§5.3.6** |
| **15** | **POST** | **`/v1/work-items/{id}:transition`** | **Policy + Idempotency-Key** | **状態遷移** | **§5.3.7** |
| **16** | **POST** | **`/v1/work-items/{id}/comments`** | **Policy + Idempotency-Key** | **コメント投稿** | **§5.3.8** |
| **17** | **GET** | **`/v1/work-items/{id}/attachments`** | **Policy** | **添付一覧** | **§5.3.9 (V1.2 部分)** |
| **18** | **GET** | **`/v1/app-version`** | **Anonymous** | **最新バージョン** | **§5.5.1** |
| **19** | **POST** | **`/v1/sync/batch`** | **Policy** | **バッチ同期 (DRYRUN)** | **§5.6.1 (G-19 拍板待ち)** |
| **20** | **GET** | **`/v1/work-items/{id}/audit-events`** | **Policy** | **WI 監査ログ (V1.2 で)** | **§5.7.1 (将来)** |

### 7.2 WebSocket エンドポイント（UAT 新規）

| 項目 | 値 | 出典 |
|---|---|---|
| URL | `wss://star.internal:8080/api/v1/ws` | **UAT 必須 WSS** (NFR-SEC-003) |
| Subprotocol | `star.v1`（強制） | `api-design.md` §4 |
| 認証 | `Sec-WebSocket-Protocol: star.v1` + `Authorization: Bearer <jwt>` | `api-design.md` §4 |
| Heartbeat | サーバ 30s ping / クライアント 60s 以内 pong | `api-design.md` §4.5 |
| 最大同時 Subscription | 100 / Connection | `api-design.md` §4 |
| 購読 resource_types | `work_item`, `notification` | **UAT 要件 (§11 G-16 拍板待ち)** |

**メッセージ形式** (per `api-design.md` §4.4):

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
  "data": { /* resource snapshot */ }
}

// Server → Client: ping
{ "type": "ping", "ts": 1725265800 }

// Client → Server: pong
{ "type": "pong", "ts": 1725265800 }
```

### 7.3 共通ヘッダ（v1.0 継承 + Idempotency-Key 追加）

| ヘッダ | 値 | 送信元 | 備考 |
|---|---|---|---|
| `Authorization` | `Bearer <access_token>` | AuthInterceptor | 要件 §1.12 |
| `Content-Type` | `application/json; charset=utf-8` | Dio 自動 | `api-design.md` §1.10 |
| `User-Agent` | `StarMobile/1.0.0 (Android)` | Dio 自動 | Backend 監査用 |
| `traceparent` | W3C 形式 | Dio 自動 | `api-design.md` §1.9 |
| `X-Request-Id` | UUID v7 | Dio Interceptor | Backend ログ相関 |
| **`Idempotency-Key`** | **UUID v7** | **SyncQueueService** | **`api-design.md` §1.6 (UAT 必須)** |
| **`If-Match`** | **`<etag>`** | **WorkItemController** | **`api-design.md` §3.5:627 (楽観ロック)** |

### 7.4 エラーレスポンス処理（v1.0 + UAT 拡張）

| HTTP Status | code prefix | AppError 型 | UAT クライアント挙動 |
|---|---|---|---|
| 401 | SEC-001 | `Unauthorized` | refresh 試行 → 失敗時 logout |
| 403 | SEC-002, SEC-003, SEC-007 | `Forbidden` | 「権限なし」+ 戻る |
| 404 | *-001 | `NotFound` | 「見つかりません」+ 戻る |
| **409** | **CONFLICT-* / *-003** | **`Conflict` (UAT 新規)** | **ConflictResolver に委譲 + 競合解決 UI 起動** |
| 422 | VAL-* | `ValidationError` | フォームエラー表示 |
| 429 | RATE-* | `RateLimited` | retry_after 後に自動 retry |
| 5xx | SRV-* | `ServerError` | リトライボタン + ログ収集 |
| タイムアウト | — | `NetworkError` | オフライン判定 → 同期キュー保持 |
| 接続不可 | — | `NetworkError` | 同上 |
| **WebSocket 切断** | — | **`WsDisconnected` (UAT 新規)** | **REST 30s ポーリングにフォールバック** |
| **WebSocket 401** | **SEC-001** | **`WsAuthError` (UAT 新規)** | **REST refresh → WS 再接続** |

---

## §8 セキュリティ設計

### 8.1 認証フロー（v1.0 拡張）

| ステップ | 処理 | 出典 |
|---|---|---|
| 1. 起動 | `flutter_secure_storage` から token 読込 | NFR-SEC-001 |
| 2. 検証 | access_token の exp claim 確認 | 要件 §7.1 |
| 3. 期限切れ | refresh_token で `POST /v1/auth/refresh` | FR-AUTH-003 |
| 4. refresh 失敗 | 資格情報全削除 → Unauthenticated → ログイン画面 | FR-AUTH-004 |
| 5. **WS 接続** | **WSS 接続確立 (FR-WS-001)** | **UAT 新規** |
| 6. **WS 認証** | **Upgrade 時に Bearer token 検証** | **`api-design.md` §4** |
| 7. **WS 切断** | **指数バックオフ再接続 (1s→3s→10s) + REST フォールバック** | **FR-WS-005/006 + FR-NOTIF-009** |

### 8.2 トークン管理（v1.0 拡張）

| 項目 | 仕様 |
|---|---|
| 保管場所 | Android Keystore |
| 暗号化 | AES-256 (Keystore 標準) |
| アクセス制御 | `BIOMETRIC_STRONG` OR `DEVICE_CREDENTIAL` (API 30+) |
| バックアップ除外 | `android:allowBackup="false"` |
| **ログアウト時削除** | **flutter_secure_storage.deleteAll() + Drift 全テーブル削除** |

### 8.3 通信セキュリティ（UAT 拡張）

| プロトコル | 暗号化 | 必須性 | 備考 |
|---|---|---|---|
| REST | HTTP（cleartext、内網限定） / HTTPS（外網） | MVP は HTTP | NFR-SEC-002 |
| **WebSocket** | **WSS (TLS 1.2+) 必須** | **UAT 必須** | **NFR-SEC-003** |
| 証明書 | システム信頼ストア + V1.2 で pinning 検討 | 内網 CA は V1.2 で評価 | NFR-SEC-002 |

### 8.4 オフラインキャッシュセキュリティ（UAT 新規）

- **SQLCipher** で DB 全体を AES-256 暗号化 (NFR-SEC-011)
- 暗号化キー: Keystore に保存 + 起動時取得
- ログアウト時: キー削除 + DB ファイル削除

### 8.5 同期キューセキュリティ（UAT 新規）

- 各 SyncQueueItem は **Idempotency-Key (UUID v7)** を持つ
- サーバ側で重複検出（`idempotency_keys` テーブル想定、§11 G-17）
- 暗号化: SQLCipher で DB 全体暗号化（個別項目暗号化はしない）
- Integrity check: 各 payload の `sha256` を別カラムに保存（改ざん検知）

### 8.6 ログセキュリティ（UAT 新規）

- **LogRedactor** (NFR-SEC-012) で redact 対象:
  - `Authorization: Bearer ...` ヘッダ
  - パスワード (login request body)
  - PII (email, display_name, avatar_url, body of comment, etc.)
  - Idempotency-Key (X-Request-Id は許可)
- redact ルール: regex ベース + 構造化フィールド (freezed から自動抽出)
- redact ログは SHA-256 ハッシュに置換（デバッグ時に照合可能）

### 8.7 攻撃面

| 攻撃 | 対策 | 出典 |
|---|---|---|
| トークン盗難 | Keystore 暗号化 + FLAG_SECURE | NFR-SEC-001/005 |
| **REST 中間者攻撃** | cleartext HTTP（内網限定）+ ドメイン制限 | NFR-SEC-002/004 |
| **WS 中間者攻撃** | WSS 必須 + cert pinning 検討 | NFR-SEC-003 |
| リプレイ攻撃 | Idempotency-Key (UAT) + Backend 側 `idempotency_keys` | `api-design.md` §1.6 |
| バックドア SDK | 外部 SDK 全面禁止 | ADR-0021 |
| 認証情報ハードコード | `--dart-define` 注入 | NFR-SEC-009 |
| **ローカル DB 漏洩** | SQLCipher + ログアウト時削除 | NFR-SEC-011 + NFR-SEC-006 |
| **ログ漏洩** | 自動 redact | NFR-SEC-012 |

---

## §9 性能・可用性設計

### 9.1 性能目標とアプローチ（UAT 拡張）

| NFR | 目標 | アプローチ |
|---|---|---|
| NFR-PERF-001 cold start | ≤ 1.5s | Drift/SQLCipher 起動時 warmup + lazy load features |
| NFR-PERF-002 board 表示 | ≤ 2.0s | Drift ローカル 即応 + REST 差分更新 |
| NFR-PERF-003 API P95 | ≤ 200ms | gzip + 内網レイテンシ低 |
| NFR-PERF-004 メモリ | ≤ 150 MB | **UAT 修正** (Drift + WebSocket + SyncQueue で +30MB) |
| NFR-PERF-005 APK | ≤ 40 MB | **UAT 修正** (Drift/SQLCipher で +10MB) |
| NFR-PERF-006 バッテリー | ≤ 5% / 30min | WS 接続時は 30s ping/pong、切断時 REST |
| **NFR-PERF-007** | **オフライン UI 応答 ≤ 100ms** | **Drift indexed query** |
| **NFR-PERF-008** | **WS 再接続 ≤ 5s** | **指数バックオフ (1s→3s→10s)** |
| **NFR-PERF-009** | **同期 1 件 ≤ 500ms** | **Idempotency-Key + retry** |
| **NFR-PERF-010** | **WS 推送 → UI 更新 ≤ 200ms** | **Riverpod 直接 update + Stream 配信** |

### 9.2 キャッシュ戦略（v1.0 + UAT 拡張）

| データ | キャッシュ場所 | 無効化タイミング |
|---|---|---|
| access_token | Keystore | refresh / logout |
| refresh_token | Keystore | logout |
| user / tenant | Keystore (JSON) | refresh / logout |
| ボード | **Drift (SQLCipher)** + メモリ | REST fetch / WS push |
| Work Item 詳細 | **Drift** + メモリ | REST fetch / WS push / ローカル編集 |
| コメント | **Drift** + メモリ | REST fetch / ローカル投稿 / WS push |
| 通知 | **Drift** + メモリ | WS push / 30s ポーリング |
| 同期キュー | **Drift** | 同期完了 |
| 競合レポート | **Drift** | ユーザー解決 |

### 9.3 接続性管理（UAT 拡張）

| 状態 | 検知 | 挙動 |
|---|---|---|
| WiFi / セルラー接続中 | `connectivity_plus` | 通常動作 + WS 接続 |
| 接続喪失 | `connectivity_plus` + API 失敗 | オフラインバナー + 編集を SyncQueue に enqueue |
| 接続回復 | `connectivity_plus` | 自動同期 (SyncQueue 順次) + WS 再接続 |

### 9.4 クラッシュレポート（v1.0 継承 + UAT 拡張）

- 外部 SDK 禁止（ADR-0021）
- `FlutterError.onError` + `PlatformDispatcher.instance.onError` でキャッチ
- ローカルファイルに追記
- **ログ送信機能** (FR-SETTINGS-006) でユーザー手動送信
- **送信前 LogRedactor で redact** (NFR-SEC-012)

### 9.5 監視（UAT 拡張）

- サーバ側: Backend 側 Grafana
- **クライアント側集計** (匿名):
  - WS 接続成功率
  - 同期成功率 / 競合発生率
  - オフライン利用率
- V1.2 で Prometheus exporter 検討

---

## §10 状態遷移設計（v1.0 + UAT 拡張）

### 10.1 Work Item 状態機（v1.0 継承 + UAT 実行可能化）

```
              ┌──────────┐
              │  TODO    │
              └────┬─────┘
                   │ start
                   ▼
              ┌──────────┐
              │IN_PROGRESS│◀──┐
              └────┬─────┘    │ reopen
                   │ resolve  │
                   ▼          │
              ┌──────────┐    │
              │  DONE    │────┘
              └──────────┘
                   │ archive
                   ▼
              ┌──────────┐
              │ ARCHIVED │
              └──────────┘

副状態: BLOCKED (任意の主状態に付与可)
```

**v1.0 → v1.1**: 状態表示のみ → **実行可能** (FR-WORK-009)

### 10.2 AuthState 状態機（v1.0 継承）

### 10.3 Network State（v1.0 継承）

### 10.4 SyncState 状態機（UAT 新規）

```
                ┌─────────┐
       起動     │  Idle   │
       ────────▶└────┬────┘
                    │ 編集 / 接続喪失中の再接続
                    ▼
            ┌────────────────┐
            │ SyncInProgress │
            │ (残り N / M 件) │
            └────┬───────────┘
                 │
        ┌────────┴────────┬─────────────┐
        ▼                 ▼             ▼
   ┌─────────┐      ┌──────────┐  ┌──────────┐
   │ Success │      │ Failed   │  │ Conflicts│
   │ (at=now)│      │ (error)  │  │ (N 件)   │
   └────┬────┘      └────┬─────┘  └────┬─────┘
        │                │             │
        │ idle に戻る    │ idle に戻る │ ユーザー
        │                │             │ 解決待ち
        ▼                ▼             ▼
   ┌─────────┐      ┌──────────┐  ┌──────────┐
   │  Idle   │      │  Idle    │  │  Idle    │
   └─────────┘      └──────────┘  └──────────┘

オフライン中: 全ての操作は sync_queue に enqueue され、SyncState は SyncOffline として表現
```

### 10.5 PushState 状態機（UAT 新規）

```
       ┌──────────────────┐
       │ PushDisconnected │ (起動時、ログアウト時)
       └────────┬─────────┘
                │ connect()
                ▼
       ┌──────────────────┐
       │ PushConnecting   │
       └────────┬─────────┘
                │ success
                ▼
       ┌──────────────────┐
       │ PushConnected    │◀─────────┐
       │ (subscribed:     │          │ 再接続成功
       │  [work_item,     │          │
       │   notification]) │          │
       └────┬─────────────┘          │
            │                        │
   接続失敗 / 認証失敗 / タイムアウト  │
            ▼                        │
   ┌──────────────────┐              │
   │ PushReconnecting │──────────────┘
   │ (attempt=N)      │
   └────────┬─────────┘
            │ 最大リトライ超過
            ▼
   ┌──────────────────┐
   │ PushError        │
   │ (fallback: REST) │
   └──────────────────┘
```

### 10.6 編集操作の状態機（UAT 新規）

```
              ┌─────────┐
   ユーザー   │ Pending │  SyncQueue に enqueue 済、UI Optimistic
   操作       │ (local) │
              └────┬────┘
                   │ 同期開始
                   ▼
              ┌─────────┐
              │InFlight │  REST リクエスト送信中
              │(network)│
              └────┬────┘
                   │
        ┌──────────┼──────────┐
        ▼          ▼          ▼
   ┌─────────┐ ┌─────────┐ ┌─────────┐
   │ Success │ │Conflict │ │ Failed  │
   │ (synced)│ │ (待解決)│ │ (retry) │
   └─────────┘ └─────────┘ └─────────┘
                              │ retry < 3
                              │ → Pending に戻る
                              │ retry >= 3
                              ▼
                          ┌─────────┐
                          │ Dropped │ (ユーザー通知)
                          └─────────┘
```

---

## §11 デプロイメント設計

### 11.1 APK ビルドパイプライン

```
[開発]  flutter run --dart-define=STAR_HOST=http://star.local:8080
                     --dart-define=WS_HOST=ws://star.local:8080 (HTTP)
                     --dart-define=WSS_HOST=wss://star.local:8080 (HTTPS)
         │
         ▼
[CI]    flutter build apk --release \
            --dart-define=STAR_HOST=http://star.internal:8080 \
            --dart-define=WSS_HOST=wss://star.internal:8080 \
            --dart-define=API_VERSION=v1 \
            --dart-define=DB_KEY_ID=internal_v1 \
            --obfuscate \
            --split-debug-info=build/symbols/
         │
         ▼
[成果物] build/app/outputs/flutter-apk/app-release.apk
         build/symbols/  (ProGuard mapping, 後で保管)
         │
         ▼
[配布]  内網ファイル共有 (NAS/MinIO) にアップロード
         README に QR コード + URL 記載
```

### 11.2 Android 設定（v1.0 継承）

| 項目 | 値 |
|---|---|
| `applicationId` | `com.star.mobile` |
| `versionCode` | CI 自動 +1 |
| `versionName` | semver (1.0.0 → 1.1.0) |
| `minSdkVersion` | 24 (Android 7.0) |
| `targetSdkVersion` | 34 (Android 14) |
| 署名 | 内網 keystore (G-10 拍板待ち) |
| 難読化 | `obfuscate` + `split-debug-info` |
| ProGuard | Flutter 標準 + アプリ独自ルール + SQLCipher keep ルール |

### 11.3 AndroidManifest.xml 重要項目

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
  <uses-permission android:name="android.permission.INTERNET" />
  <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
  <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />  <!-- 将来 V1.2 -->

  <application
    android:label="Star"
    android:icon="@mipmap/ic_launcher"
    android:allowBackup="false"
    android:usesCleartextTraffic="true"        <!-- MVP HTTP 明文（WS は別扱い） -->
    android:networkSecurityConfig="@xml/network_security_config">
    <activity ...> ... </activity>
  </application>
</manifest>
```

### 11.4 network_security_config.xml

```xml
<?xml version="1.0" encoding="utf-8"?>
<network-security-config>
  <!-- cleartext HTTP は star.internal のみ許可 (REST) -->
  <domain-config cleartextTrafficPermitted="true">
    <domain includeSubdomains="true">star.internal</domain>
    <domain includeSubdomains="true">star.local</domain>  <!-- 開発時 -->
  </domain-config>
  <!-- ベースは HTTPS / WSS のみ -->
  <base-config cleartextTrafficPermitted="false">
    <trust-anchors>
      <certificates src="system" />
    </trust-anchors>
  </base-config>
</network-security-config>
```

注: `cleartextTrafficPermitted="true"` は `http://` の REST のみに作用。WSS は `wss://` なので HTTPS 扱い、別途 cert 検証。

### 11.5 デプロイフロー（v1.0 継承）

| 段階 | 作業 | 担当 |
|---|---|---|
| 1. 開発 | `flutter run` で実機デバッグ | 5 域 frontend Lead |
| 2. CI | PR → `flutter analyze` + `flutter test` + `flutter build apk` | CI bot |
| 3. 内網 UAT | SRE Lead + 5 域 Lead + α ユーザー 10 名で実機テスト | 全 5 域 + SRE |
| 4. 配布 | APK + SHA256 + 署名を内網 NAS に配置 + README 更新 | SRE |
| 5. 通知 | Slack `#star-mobile` で URL 共有 | SRE |
| 6. ユーザー導入 | QR コード読み取り or URL クリックで各自インストール | 全員 |

**UAT 拡張**: 配布前に `flutter integration_test` を実機デバイスファームで実行 (CI bot)。

---

## §12 既知の未解決事項

`01-requirements.md` v1.1 §13 と完全同期。G-01〜G-25 すべてここに継承。

実装フェーズで発生する追加課題は `03-detailed-design.md` §11 + `99-implementation-changelog.md` に記録。

---

## §13 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v1.0 | 2026-09-02 16:14 JST | 架構師 (Mavis 接手 agent per DEC-008) | IPA 標準初版: read-only MVP 範囲, 4 層アーキテクチャ + 6 機能モジュール + 13 コンポーネント | v1.0 と同じ (read-only 範囲) |
| **v1.1** | 2026-09-02 16:27 JST | 架構師 (Mavis 接手 agent per DEC-008) | **UAT 全面拡張**: §4 4 層 + Sync Engine 副層, 22 コンポーネント (WebSocketService / OfflineDatabase / SyncQueueService / ConflictResolver / ConnectivityWatcher / SyncStatusController / WorkItemWriteService / CommentsController / TransitionsController / ConflictResolutionScreen / SyncBanner / PushEventRouter / LogRedactor 新規), 4 シーケンス図追加 (WS 接続 / オフライン編集 / WS 推送 / 競合解決), 18 ドメインモデル (6 コマンド系 + 同期/競合/推送系), 20 API エンドポイント, WS エンドポイント + メッセージ形式, 3 状態機追加 (SyncState / PushState / 編集操作), セキュリティ 4 脅威追加, 性能 NFR 4 件追加 | 2026-09-02 16:27 JST Ulysses 拍板 UAT 範囲 + 自建 WS 推送 (questionnaire 答: full_uat + self_ws) |

---

## §14 承認欄

| 角色 | 簽字 | 日付 | 備註 |
|---|---|---|---|
| 架構師 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽 (per 8/27 19:39 JST + 21:59 JST 三次強化授權) |
| SRE Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;5 域独立真实身份 DDD Review 段階で補充 |
| 平台 Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| 評審主持 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| PM | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
