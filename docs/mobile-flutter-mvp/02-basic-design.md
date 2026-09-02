# Star Mobile Flutter MVP — 基本設計書

> **基準**: 日本 IPA（情報処理推進機構）SEC ソフトウェア基本設計書 標準章立て
> **作成日**: 2026-09-02
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **上流要件定義書**: `D:\Star\docs\mobile-flutter-mvp\01-requirements.md` v1.0
> **Pre-IPA 草稿**: `docs/architecture/2026-09-02-upgrade/spec/mobile/01-flutter-mvp-design.md` v0.1 (commit `bd4998e`)

---

## §1 目的

本文書は、Star Mobile Flutter MVP の**基本設計**を定義する。`01-requirements.md` v1.0 で定義した要件（FR-AUTH-001〜FR-NOTIF-009、計 30 機能要件 + 22 非機能要件）を満たすための**システムアーキテクチャ・コンポーネント構成・データフロー・外部インターフェース概要・状態遷移・デプロイメント構成**を記述する。

本書のスコープ：
- ✅ モジュール分割 / 依存方向 / 責務境界
- ✅ コンポーネント間相互作用（シーケンス概要）
- ✅ データモデル概要（クラス / ER 概要、DDL なし）
- ✅ 外部 API インターフェース概要（エンドポイント + 認証方式）
- ✅ セキュリティ境界（高レベル）
- ✅ 状態遷移（Work Item の状態機のみ、Auth state / HTTP state は詳細設計）
- ✅ デプロイメント構成
- ❌ コードレベルのクラス詳細（→ 詳細設計書 §4）
- ❌ テストケース詳細（→ 詳細設計書 §10）
- ❌ ビルドコマンド詳細（→ 詳細設計書 §8）

---

## §2 適用範囲

### 2.1 文書階層における位置

```
[上流] docs/requirements.md v2.0          (Star プラットフォーム全体要件)
       docs/mobile-flutter-mvp/01-requirements.md v1.0  (本 MVP 要件, 本書の上流)
       
[本書] docs/mobile-flutter-mvp/02-basic-design.md v1.0 (本 MVP 基本設計)
       
[下流] docs/mobile-flutter-mvp/03-detailed-design.md v1.0  (本 MVP 詳細設計)
       docs/mobile-flutter-mvp/{10-implementation-report,99-changelog}.md (実装時に作成)
```

### 2.2 In Scope / Out of Scope

`01-requirements.md` §2 と同じ。MVP は Android 限定・Read-Only・online-only である。

---

## §3 前提条件・制約事項

### 3.1 技術的前提

| # | 前提 | 出典 |
|---|---|---|
| 1 | Flutter 3.24+ / Dart 3.5+ が開発環境で利用可能 | 本書新規 |
| 2 | Android Studio / VS Code + Flutter 拡張で開発 | 本書新規 |
| 3 | Backend API（13 エンドポイント）が production で稼働 | 要件 §3.1 |
| 4 | `flutter_secure_storage` 9.x が Android Keystore を使用可能（API 23+） | 本書新規 |
| 5 | `Hive` 2.x は Android 上で安定動作（Kotlin 互換） | 本書新規 |
| 6 | `Dio` 5.7+ の Interceptor chain で refresh token 自動化可能 | 本書新規 |

### 3.2 組織的制約

- 5 域独立 Lead（work-item / board / notification / auth / frontend）が個別承認（8/21 JST）
- Mavis 接手代簽 Ulysses（per 8/27 19:39 JST + 21:59 JST 三次強化）
- token-OLU 制（1 SRE·周 = 1.2M、`STAR-OLU-001.md` v0.1）
- AGENTS.md §4 13 項守門全部適用

### 3.3 環境変数制約

- 秘匿情報（API endpoint / future API key）は `.env` に書かない
- ビルド時に `--dart-define=KEY=VALUE` で注入
- 開発環境では `flutter run --dart-define-from-file=dev.json`（dev.json は .gitignore）

---

## §4 システムアーキテクチャ

### 4.1 全体アーキテクチャ図

```
┌──────────────────────────────────────────────────────────────┐
│  Android Flutter App (本 MVP)                                 │
│                                                                │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Presentation Layer (UI)                              │   │
│  │  - Material 3 Screens (6 画面)                       │   │
│  │  - Widgets (Board Card / Notification Tile / etc.)   │   │
│  │  - go_router (宣言的ルーティング)                    │   │
│  └──────────────────────────────────────────────────────┘   │
│                          ↕ (Riverpod)                          │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Application Layer (Controllers)                       │   │
│  │  - AuthController                                     │   │
│  │  - BoardController                                    │   │
│  │  - WorkItemController                                 │   │
│  │  - NotificationsController                            │   │
│  │  - ProjectListController                              │   │
│  │  - SettingsController                                 │   │
│  └──────────────────────────────────────────────────────┘   │
│                          ↕                                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Domain Layer (Models / States)                        │   │
│  │  - User / Tenant / Board / Column / WorkItem          │   │
│  │    / Comment / Transition / Notification              │   │
│  │  - AuthState (sealed) / Result<T,E>                   │   │
│  └──────────────────────────────────────────────────────┘   │
│                          ↕                                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Data Layer (Repositories + Storage)                   │   │
│  │  - AuthRepository / BoardRepository / ...             │   │
│  │  - DioClient + 3 Interceptors                         │   │
│  │  - TokenStorage (flutter_secure_storage)              │   │
│  │  - In-memory cache (no SQLite for MVP)                │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                                │
└──────────────────────────────────────────────────────────────┘
                          ↕ HTTP REST (cleartext)
                          ↕ Authorization: Bearer <jwt>
┌──────────────────────────────────────────────────────────────┐
│  Star プラットフォーム (Backend, 既存)                         │
│  API Gateway → work-core → PostgreSQL                       │
└──────────────────────────────────────────────────────────────┘
```

### 4.2 アーキテクチャパターン

| 項目 | 選択 | 理由 |
|---|---|---|
| **全体** | クリーンアーキテクチャ 4 層（Presentation / Application / Domain / Data） | テスタビリティ + 依存方向の明確化 |
| **状態管理** | Riverpod 2.5+ (NOT Bloc / Provider) | compile-time safe + 強型 + 冷起動性能 |
| **依存性注入** | Riverpod Provider 経由（GetIt 不使用） | Riverpod で完結、追加 DI ライブラリ不要 |
| **非同期** | Dart `async/await` + `Stream` のみ（RxDart 不使用） | Dart 標準で十分 |
| **データ永続化** | `flutter_secure_storage` (token) + メモリ内キャッシュ | MVP 要件として online-only |
| **ナビゲーション** | `go_router` 14+ | 宣言的、deep link 対応（V1.1 拡張余地） |
| **JSON 解析** | `freezed` + `json_serializable` | 不変モデル + copyWith + sealed unions |
| **HTTP 通信** | `Dio` 5.7+ + `dio_smart_retry` | Interceptor chain + retry/backoff |

### 4.3 モジュール分割（feature-first）

```
lib/
├── main.dart                  # ProviderScope + MaterialApp.router
├── app/                       # アプリ全体設定
│   ├── app.dart               # MaterialApp.router
│   ├── router.dart            # go_router 設定
│   └── theme.dart             # Material 3 ColorScheme
├── core/                      # アプリ全体共通基盤
│   ├── api/                   # Dio クライアント + Interceptor
│   ├── auth/                  # Token 管理
│   ├── env/                   # 環境設定
│   └── result/                # Result<T, AppError> sealed
├── features/                  # 機能別（垂直スライス）
│   ├── auth/                  # FR-AUTH
│   ├── board/                 # FR-BOARD
│   ├── work_item/             # FR-WORK
│   ├── notifications/         # FR-NOTIF
│   ├── projects/              # FR-PROJ
│   └── settings/              # FR-SETTINGS
└── shared/                    # 共通 UI コンポーネント
    ├── widgets/               # EmptyState / ErrorState / LoadingState
    └── utils/                 # date_format / priority_color
```

**依存方向**:
```
Presentation → Application → Domain ← Data
                                 ↑
                          (Domain のみが Data に依存)
```
- Domain 層は外部依存ゼロ（pure Dart + freezed）
- Data 層は Domain の interface（Repository）に依存
- Application 層は Domain のみに依存
- Presentation 層は Application + Domain に依存

---

## §5 コンポーネント設計

### 5.1 主要コンポーネント一覧

| コンポーネント | 責務 | 配置 |
|---|---|---|
| **DioClient** | Dio シングルトン、baseUrl、interceptor チェーン | `core/api/dio_client.dart` |
| **AuthInterceptor** | Bearer token 注入、401 検知 → refresh → retry | `core/api/auth_interceptor.dart` |
| **ErrorInterceptor** | RFC 7807 → `AppException` sealed に変換 | `core/api/error_interceptor.dart` |
| **LoggingInterceptor** | 開発時のみログ出力、リリース時 no-op | `core/api/logging_interceptor.dart` |
| **TokenStorage** | Keystore 経由の token 永続化 | `core/auth/token_storage.dart` |
| **AuthController** | 認証状態管理（StateNotifier<AuthState>） | `features/auth/presentation/auth_controller.dart` |
| **BoardController** | ボード取得 + キャッシュ | `features/board/presentation/board_controller.dart` |
| **WorkItemController** | 詳細取得（3 タブ並列） | `features/work_item/presentation/work_item_controller.dart` |
| **NotificationsController** | 30s ポーリング | `features/notifications/presentation/notifications_controller.dart` |

### 5.2 主要シーケンス図

#### 5.2.1 ログイン + ボード表示

```
User          LoginScreen    AuthController    DioClient       AuthInterceptor    Backend
 │                │                │                │                  │                │
 │ email+pass tap │                │                │                  │                │
 │ ──────────────▶│                │                │                  │                │
 │                │ signIn()       │                │                  │                │
 │                │ ──────────────▶│                │                  │                │
 │                │                │ POST /login    │                  │                │
 │                │                │ ──────────────▶│                  │                │
 │                │                │                │ ───────────────────────────────────▶│
 │                │                │                │ 200 {tokens,user,tenant}           │
 │                │                │                │ ◀───────────────────────────────────│
 │                │                │ save tokens    │                  │                │
 │                │                │ ──────────────▶│                  │                │
 │                │                │ AuthState.Authenticated          │                │
 │                │                │                │                  │                │
 │                │ /projects     │                │                  │                │
 │                │ ─────────────▶│                │                  │                │
 │                │                │                │                  │                │
 │                │                │ GET /projects  │                  │                │
 │                │                │ ──────────────▶│                  │                │
 │                │                │                │ inject Bearer    │                │
 │                │                │                │ ───────────────────────────────────▶│
 │                │                │                │ 200 [...]                         │
 │                │                │                │ ◀───────────────────────────────────│
 │                │ BoardScreen   │                │                  │                │
 │                │ ─────────────▶│                │                  │                │
```

#### 5.2.2 access_token 期限切れ → refresh → retry

```
Caller        AuthInterceptor    DioClient    Backend
 │                │                │                │
 │ GET /work-items│                │                │
 │ ──────────────▶│                │                │
 │                │ inject Bearer  │                │
 │                │ ──────────────▶│                │
 │                │                │ ──────────────▶│
 │                │                │ 401 SEC-001    │
 │                │ ◀──────────────│                │
 │                │ detect 401    │                │
 │                │ POST /refresh │                │
 │                │ ──────────────▶│                │
 │                │                │ ──────────────▶│
 │                │                │ 200 {new token}│
 │                │ ◀──────────────│                │
 │                │ save new token│                │
 │                │ retry original│                │
 │                │ ──────────────▶│                │
 │                │                │ ──────────────▶│
 │                │                │ 200 [...]      │
 │                │ ◀──────────────│                │
```

### 5.3 状態管理モデル

#### 5.3.1 AuthState（sealed）

```dart
sealed class AuthState {}
class Unauthenticated extends AuthState { final String? message; }
class Authenticating extends AuthState {}
class Authenticated extends AuthState {
  final User user;
  final Tenant tenant;
}
class AuthError extends AuthState { final String message; final int? code; }
```

#### 5.3.2 BoardState（AsyncNotifier）

```dart
class BoardController extends AsyncNotifier<Board> {
  Future<Board> build(String projectId) async {
    // 並列取得: board config + work items list
    final results = await Future.wait([
      _boardRepository.fetchBoard(projectId),
      _workItemRepository.listByProject(projectId),
    ]);
    return Board(
      config: results[0] as BoardConfig,
      cards: results[1] as List<WorkItemSummary>,
    );
  }
  
  Future<void> refresh() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() => build(_projectId));
  }
}
```

---

## §6 データモデル概要

### 6.1 ドメインモデル一覧（freezed）

| モデル | 主要フィールド | 出典 |
|---|---|---|
| `User` | id, email, displayName, avatarUrl | `api-design.md` §3.2 |
| `Tenant` | id, name | `api-design.md` §3.2 |
| `AuthTokens` | accessToken, refreshToken, expiresAt | `api-design.md` §6.2 |
| `Project` | id, tenantId, name, slug, lastAccessedAt | `api-design.md` §3.4 |
| `BoardConfig` | id, projectId, columns: List<Column> | `api-design.md` §3.7:668 |
| `Column` | id, stateId, name, order, workItemIds | `api-design.md` §3.7 |
| `WorkItemSummary` | id, title, status, priority, assignee | `api-design.md` §3.5:624 |
| `WorkItem` | 全フィールド（Summary + description, type, repo 等） | `api-design.md` §3.5:626 |
| `Comment` | id, author, body, createdAt, mentions | `api-design.md` §3.10:700 |
| `Transition` | from, to, requiredPermission, isAllowed | `api-design.md` §3.5:630 |
| `Notification` | id, eventType, payload, readAt, sentAt | `api-design.md` §3.16:787 |
| `AppError` (sealed) | Network / Unauthorized / Forbidden / NotFound / Server / Unknown | RFC 7807 準拠 |

### 6.2 ER 概要（概念レベル、DDL は `data-design.md` v0.2 §4）

```
Tenant ─┬─ User ─┬─ AuthSession (token)        [サーバ側]
        │        ├─ Notification
        │        └─ AuthState (クライアント側キャッシュ)
        │
        └─ Project ─┬─ BoardConfig ─ Column ─ WorkItem
                    ├─ WorkItem ─┬─ Comment
                    │            ├─ Transition
                    │            └─ AcceptanceCriterion (V1.1)
                    └─ Sprint (V1.1)
```

**本 MVP が読み取るテーブル**（read-only）:
- tenant, user, project, board, board_column, work_item, comment, notification

**本 MVP が書き込むテーブル**: なし（read-only MVP）

---

## §7 外部インターフェース設計概要

### 7.1 API エンドポイント（要件 §9 の 13 個を実装で叩く）

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

### 7.2 共通ヘッダ

| ヘッダ | 値 | 送信元 | 備考 |
|---|---|---|---|
| `Authorization` | `Bearer <access_token>` | AuthInterceptor | 要件 §1.12 |
| `Content-Type` | `application/json; charset=utf-8` | Dio 自動 | `api-design.md` §1.10 |
| `User-Agent` | `StarMobile/1.0.0 (Android)` | Dio 自動 | Backend 監査用 |
| `traceparent` | W3C 形式 | Dio 自動 | `api-design.md` §1.9 |
| `X-Request-Id` | UUID v7 | Dio Interceptor | Backend ログ相関 |

**送信しない**:
- `X-Tenant-Id` (Gateway が JWT から抽出、`api-design.md` §1.8)

### 7.3 エラーレスポンス処理

Backend 側 `RFC 7807` 形式:

```json
{
  "type": "https://star.acme.com/errors/SEC-001",
  "title": "Unauthorized",
  "status": 401,
  "detail": "Token has expired",
  "code": "SEC-001",
  "trace_id": "abc123"
}
```

`ErrorInterceptor` でこの形式を解析し、`AppError` sealed にマッピング:

| HTTP Status | code prefix | AppError 型 | クライアント挙動 |
|---|---|---|---|
| 401 | SEC-001 | `Unauthorized` | refresh 試行 → 失敗時 logout |
| 403 | SEC-002, SEC-003, SEC-007 | `Forbidden` | 「権限なし」ダイアログ + 戻る |
| 404 | *-001 | `NotFound` | 「見つかりません」+ 戻る |
| 409 | *-003 | `Conflict` | ダイアログ表示 |
| 422 | VAL-* | `ValidationError` | フォームエラー表示 |
| 429 | RATE-* | `RateLimited` | retry_after 後に自動 retry |
| 5xx | SRV-* | `ServerError` | リトライボタン + ログ収集 |
| タイムアウト | — | `NetworkError` | ネットワークエラーダイアログ |
| 接続不可 | — | `NetworkError` | 同上 |

---

## §8 セキュリティ設計

### 8.1 認証フロー

| ステップ | 処理 | 出典 |
|---|---|---|
| 1. 起動 | `flutter_secure_storage` から token 読込 | NFR-SEC-001 |
| 2. 検証 | access_token を decode、有効期限内なら `AuthState.Authenticated` | 要件 §7.1 |
| 3. 期限切れ | refresh_token で `POST /v1/auth/refresh` | FR-AUTH-003 |
| 4. refresh 失敗 | 資格情報全削除 → `AuthState.Unauthenticated` → ログイン画面 | FR-AUTH-004 |

### 8.2 トークン管理

| 項目 | 仕様 |
|---|---|
| 保管場所 | Android Keystore（`flutter_secure_storage` 経由） |
| 暗号化 | AES-256 (Keystore 標準) |
| アクセス制御 | `BIOMETRIC_STRONG` OR `DEVICE_CREDENTIAL` (API 30+) |
| バックアップ除外 | `android:allowBackup="false"` |

### 8.3 通信セキュリティ

| 項目 | 仕様 |
|---|---|
| プロトコル | HTTP/1.1（MVP cleartext） |
| ドメイン制限 | `network_security_config.xml` で `star.internal` のみ許可 |
| TLS ピン留め | V1.1 で実装（MVP スキップ） |
| 証明書検証 | システム信頼ストア使用 |

### 8.4 攻撃面

| 攻撃 | 対策 | 出典 |
|---|---|---|
| トークン盗難 | Keystore 暗号化 + FLAG_SECURE | NFR-SEC-001/004 |
| 中間者攻撃 | cleartext HTTP（内網限定）+ ドメイン制限 | NFR-SEC-002/003 |
| リプレイ攻撃 | Backend 側 `Idempotency-Key` 必須（書き込み時、MVP なし） | `api-design.md` §1.6 |
| バックドア SDK | 外部 SDK 全面禁止 | ADR-0021 |
| 認証情報ハードコード | `--dart-define` 注入のみ | NFR-SEC-008 |

---

## §9 性能・可用性設計

### 9.1 性能目標とアプローチ

| NFR | 目標 | アプローチ |
|---|---|---|
| NFR-PERF-001 cold start | ≤ 1.5s | main() の最小化 + lazy load features |
| NFR-PERF-002 board 表示 | ≤ 2.0s | Future.wait で board config + work items 並列取得 |
| NFR-PERF-003 API P95 | ≤ 200ms | レスポンス圧縮 (gzip) 期待 + 内網レイテンシ低 |
| NFR-PERF-004 メモリ | ≤ 120 MB | 画像キャッシュは API URL + インメモリのみ、永続キャッシュなし |
| NFR-PERF-005 APK | ≤ 30 MB | Flutter 標準 + asset 最小化 |
| NFR-PERF-006 バッテリー | ≤ 5% / 30min | ポーリングは 30s 間隔 + バックグラウンド停止 |

### 9.2 キャッシュ戦略

| データ | キャッシュ場所 | 無効化タイミング |
|---|---|---|
| access_token | Keystore | refresh / logout |
| refresh_token | Keystore | logout |
| user / tenant | Keystore (JSON) | refresh / logout |
| ボード | メモリ（Riverpod） | 画面遷移で破棄 |
| Work Item 詳細 | メモリ | 同上 |
| 通知 | メモリ | 30s ポーリング更新 |

**MVP では永続キャッシュなし**（V1.1 で Drift/SQLite 追加、要件 §11 G-04）。

### 9.3 接続性管理

| 状態 | 検知 | 挙動 |
|---|---|---|
| WiFi / セルラー接続中 | `connectivity_plus` で定期確認 | 通常動作 |
| 接続喪失 | 接続イベント / API 呼び出し失敗 | ネットワークエラーバナー + 最終取得データ表示 |
| 接続回復 | 接続イベント | 自動 retry（次 API 呼び出し時） |

### 9.4 クラッシュレポート

- 外部 SDK 禁止（ADR-0021）のため Firebase Crashlytics 不使用
- 自社実装: `FlutterError.onError` + `PlatformDispatcher.instance.onError` でキャッチ
- ローカルファイル `/data/data/com.star.mobile/files/crash.log` に追記
- ユーザーが Settings → 「不具合報告」メニューから任意のタイミングでファイル送信

### 9.5 監視

- サーバ側メトリクスは Backend 側 Grafana で集約（クライアントからは送信しない）
- クライアント側クラッシュ率は手動集計（V1.1 で Prometheus exporter 検討）

---

## §10 状態遷移設計

### 10.1 Work Item 状態機（Backend 側、表示のみ）

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

**MVP 動作**: 状態表示 + 遷移可能状態リスト表示（FR-WORK-005）のみ。実行不可（V1.1）。

### 10.2 AuthState 状態機（クライアント側）

```
              ┌────────────────┐
              │ Bootstrapping  │ (アプリ起動時 0.5s)
              └────────┬───────┘
                       │ 初期化完了
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│Unauthenticated│ │Authenticating│ │Authenticated │
│  (default)    │ │ (login 中)   │ │ (user,tenant)│
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │
       │ login start    │ login ok       │ logout
       │                │                │ /401
       │                ▼                ▼
       │       ┌──────────────┐  ┌──────────────┐
       │       │Authenticated │  │Unauthenticated│
       │       └──────────────┘  └──────────────┘
       │                │
       │                │ login fail
       │                ▼
       │       ┌──────────────┐
       │       │  AuthError   │
       │       └──────────────┘
       │
       │ (default entry)
```

### 10.3 Network State（HTTP 呼び出し状態）

各 API 呼び出しは `AsyncValue<T>` (Riverpod) で表現:
- `AsyncValue.loading()` — 呼び出し中
- `AsyncValue.data(T)` — 成功
- `AsyncValue.error(AppError, StackTrace)` — 失敗

UI 側は `when(loading, error, data)` で分岐。

---

## §11 デプロイメント設計

### 11.1 APK ビルドパイプライン

```
[開発]  flutter run --dart-define=STAR_HOST=http://star.local:8080
         │
         ▼
[CI]    flutter build apk --release \
            --dart-define=STAR_HOST=http://star.internal:8080 \
            --dart-define=API_VERSION=v1 \
            --obfuscate \
            --split-debug-info=build/symbols/
         │
         ▼
[成果物] build/app/outputs/flutter-apk/app-release.apk
         build/symbols/  (デバッグシンボル、後で ProGuard mapping として保管)
         │
         ▼
[配布]  内網ファイル共有 (NAS/MinIO) にアップロード
         README に QR コード + URL 記載
```

### 11.2 Android 設定

| 項目 | 値 |
|---|---|
| `applicationId` | `com.star.mobile` |
| `versionCode` | CI 自動 +1 |
| `versionName` | semver (例 1.0.0) |
| `minSdkVersion` | 24 (Android 7.0) |
| `targetSdkVersion` | 34 (Android 14) |
| `compileSdkVersion` | 34 |
| 署名 | 内網 keystore (G-10 で拍板) |
| 難読化 | `obfuscate` + `split-debug-info` |
| ProGuard | Flutter 標準 + アプリ独自ルール |

### 11.3 AndroidManifest.xml 重要項目

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
  <uses-permission android:name="android.permission.INTERNET" />
  <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />

  <application
    android:label="Star"
    android:icon="@mipmap/ic_launcher"
    android:allowBackup="false"               <!-- トークン流出防止 -->
    android:usesCleartextTraffic="true"        <!-- MVP HTTP 明文 -->
    android:networkSecurityConfig="@xml/network_security_config">
    <activity
      android:name=".MainActivity"
      android:exported="true"
      android:configChanges="orientation|keyboardHidden|keyboard|screenSize|locale|layoutDirection|fontScale|screenLayout|density|uiMode"
      android:hardwareAccelerated="true"
      android:windowSoftInputMode="adjustResize">
      <meta-data
        android:name="io.flutter.embedding.android.NormalTheme"
        android:resource="@style/NormalTheme" />
    </activity>
  </application>
</manifest>
```

### 11.4 network_security_config.xml

```xml
<?xml version="1.0" encoding="utf-8"?>
<network-security-config>
  <domain-config cleartextTrafficPermitted="true">
    <domain includeSubdomains="true">star.internal</domain>
    <domain includeSubdomains="true">star.local</domain>  <!-- 開発時 -->
  </domain-config>
  <base-config cleartextTrafficPermitted="false">
    <trust-anchors>
      <certificates src="system" />
    </trust-anchors>
  </base-config>
</network-security-config>
```

### 11.5 デプロイフロー

| 段階 | 作業 | 担当 |
|---|---|---|
| 1. 開発 | `flutter run` で実機デバッグ | 5 域 frontend Lead |
| 2. CI | PR → `flutter analyze` + `flutter test` + `flutter build apk` | CI bot |
| 3. 内網 UAT | SRE Lead 端末 + 5 域 Lead 5 端末で実機テスト | SRE + 5 域 |
| 4. 配布 | APK + SHA256 + 署名を内網 NAS に配置 + README 更新 | SRE |
| 5. 通知 | Slack `#star-mobile` で URL 共有 | SRE |
| 6. ユーザー導入 | QR コード読み取り or URL クリックで各自インストール | 全員 |

---

## §12 既知の未解決事項

`01-requirements.md` §13 と同じ 15 項目。**G-01〜G-15 は要件定義書と一意対応**し、本書で追加の未解決事項はない。

実装フェーズで発生する追加課題は `03-detailed-design.md` §11 に追記。

---

## §13 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架構師 (Mavis 接手 agent per DEC-008) | IPA 標準初版: §1〜§13 全章, 4 層クリーンアーキテクチャ + 6 機能モジュール, 13 コンポーネント, 2 つの主要シーケンス, 12 ドメインモデル, 13 API エンドポイント, 9 RFC 7807 エラー型, 6 性能 NFR マッピング, 3 状態機 (Work Item / Auth / Network), 11 デプロイメント設定 | 2026-09-02 16:09 JST Ulysses 発令「要符合日本IPA标准的需求、基本设计、详细设计」, v0.1 (commit `bd4998e`) を IPA 3 段組に supersede |

---

## §14 承認欄

| 角色 | 簽字 | 日付 | 備註 |
|---|---|---|---|
| 架構師 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽 (per 8/27 19:39 JST + 21:59 JST 三次強化授權) |
| SRE Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;5 域独立真实身份 DDD Review 段階で補充 |
| 平台 Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| 評審主持 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| PM | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
