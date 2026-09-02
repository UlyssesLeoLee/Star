# Star Mobile Flutter MVP — 実装仕様書 (Phase A)

> **基準**: 日本 IPA（情報処理推進機構）SEC ソフトウェア実装仕様書 標準章立て
> **作成日**: 2026-09-02
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **バージョン**: v1.0
> **上位文書**: `01-requirements.md` v1.2.1 / `02-basic-design.md` v1.2.1 / `03-detailed-design.md` v1.2.1
> **本書の位置**: IPA 4 段組 (要件 → 基本 → 詳細 → **実装**) の最終段。**Phase A (client のみ 7 モジュール)** に絞り、Phase B (backend 待ち 5 モジュール) は別 v1.1 として保留。
> **trigger**: 2026-09-02 17:16 JST Ulysses 発令「撰写spec」

---

## §0 目的

本文書は、`01-requirements.md` v1.2.1 §16 Capability Audit で分類された **Phase A (client のみ 7 モジュール)** の実装仕様を定義する。Phase B (backend 待ち 5 モジュール) は P2 backend 完了後に本書 v1.1 として拡張する。

**Phase A 7 モジュール**:
1. **OfflineDatabase** (Drift 7 テーブル + SQLCipher)
2. **SyncQueueService** (FIFO + retry + Idempotency-Key)
3. **ConflictResolver** (409 UI 提示 + ユーザー選択)
4. **ConnectivityWatcher** (online/offline 検知)
5. **SyncStatusController** (SyncState 状態管理)
6. **SyncBanner** (画面下部バナー UI)
7. **LogRedactor** (PII / token redact)

**Phase B 5 モジュール (v1.1 保留)**: WebSocketService / PushEventRouter / WorkItemWriteService / CommentsController / TransitionsController

---

## §1 適用範囲

### 1.1 Phase A in-scope (本書のスコープ)

- 7 モジュールの Dart 実装 (per `03-detailed-design.md` v1.2.1 §4.7-4.12 の Dart コード完全版)
- 倉構造 / pubspec.yaml / build.gradle.kts の Phase A 部分
- 単体テスト (≥ 70% カバレッジ)
- 統合テスト (Phase A 範囲: offline / conflict / SQLCipher)
- CI パイプライン (`.github/workflows/`)
- 開発環境セットアップ手順
- サブエージェント brief (per 守門 #20)
- モックデータ seed (`test/fixtures/`)
- 検証手順 (`flutter test` / `flutter analyze` / `flutter build apk`)

### 1.2 Phase A out-of-scope (v1.1 / V1.2 / V2 で実装)

- 5 モジュール (Phase B) — P2 backend 待ち
- iOS 対応 — V2 計画 (per `internal-design.md:1600`)
- Tablet / 横画面レイアウト — V1.2 候補
- 添付ファイル Upload/Download — V1.2 候補
- 多言語 i18n — V2 計画
- 生体認証 — V1.2 候補
- OAuth 2.0 — V1.2 候補
- Device 三重バインディング — V1.2 候補

### 1.3 想定開発環境

| 項目 | バージョン |
|---|---|
| Flutter SDK | 3.24.0+ stable |
| Dart SDK | 3.5.0+ |
| Android Studio | Hedgehog 2023.1.1+ |
| Android NDK | 25.1.8937393 (SQLCipher 用) |
| JDK | OpenJDK 17 (Temurin) |
| Gradle | 8.5+ |
| エミュレータ | Android API 24+ (実機テスト推奨) |

---

## §2 プロジェクト構造 (Phase A のみ)

### 2.1 倉位置

**2 つの選択肢** (G-12 拍板待ち、デフォルトは `apps/star-mobile-flutter/`):

| 選択肢 | パス | 推奨度 |
|---|---|---|
| **A. `apps/star-mobile-flutter/`** | 独立 monorepo | ★★★ 推奨 (frontend/ と並列、CI 独立) |
| B. `frontend/mobile-flutter/` | 既存 frontend 配下 | ⭐ 既存 frontend 知識流用可 |

**デフォルト: A** (理由: P3 WBS #5 「9 wt merge」パターンと整合、独立 CI、独立 5 域 Lead review)

### 2.2 ディレクトリ構造 (Phase A のみ)

```
apps/star-mobile-flutter/
├── pubspec.yaml                          # 3.1 参照
├── analysis_options.yaml                 # very_good_analysis + custom
├── android/                              # Android 設定 (3.2 参照)
│   ├── app/
│   │   ├── build.gradle.kts             # NDK 25.1.8937393 + SQLCipher 4.5.4
│   │   └── src/main/AndroidManifest.xml # INTERNET + cleartext (MVP)
│   └── ...
├── lib/
│   ├── main.dart
│   ├── app/
│   │   ├── app.dart                     # MaterialApp.router
│   │   ├── router.dart                  # go_router
│   │   └── theme.dart
│   ├── core/
│   │   ├── auth/
│   │   │   └── token_storage.dart       # Phase A: 枠のみ (Phase B で auth 接続)
│   │   ├── connectivity/
│   │   │   └── connectivity_watcher.dart # §3 モジュール 4
│   │   ├── db/
│   │   │   ├── offline_database.dart    # §3 モジュール 1
│   │   │   ├── tables.dart              # Drift 7 テーブル DDL
│   │   │   └── database_provider.dart   # SQLCipher 鍵管理
│   │   ├── sync/
│   │   │   ├── sync_queue_service.dart  # §3 モジュール 2
│   │   │   └── conflict_resolver.dart   # §3 モジュール 3
│   │   └── utils/
│   │       └── log_redactor.dart        # §3 モジュール 7
│   ├── features/
│   │   └── settings/
│   │       └── presentation/
│   │           └── sync_status_controller.dart  # §3 モジュール 5
│   └── shared/
│       └── widgets/
│           └── sync_banner.dart         # §3 モジュール 6
├── test/
│   ├── core/
│   │   ├── db/offline_database_test.dart
│   │   ├── sync/sync_queue_service_test.dart
│   │   ├── sync/conflict_resolver_test.dart
│   │   ├── connectivity/connectivity_watcher_test.dart
│   │   └── utils/log_redactor_test.dart
│   ├── features/settings/sync_status_controller_test.dart
│   ├── shared/widgets/sync_banner_test.dart
│   └── fixtures/
│       ├── sync_queue_samples.json
│       ├── conflict_samples.json
│       └── work_item_samples.json
├── integration_test/
│   ├── offline_edit_test.dart
│   ├── conflict_test.dart
│   └── security_logout_test.dart        # ローカル DB 削除確認
├── docs/
│   └── briefs/                          # 守門 #20: サブエージェント brief
│       ├── 01-offline-database.md
│       ├── 02-sync-queue-service.md
│       ├── 03-conflict-resolver.md
│       ├── 04-connectivity-watcher.md
│       ├── 05-sync-status-controller.md
│       ├── 06-sync-banner.md
│       └── 07-log-redactor.md
├── scripts/                              # 守門 #19: Python 化
│   ├── automation/
│   │   ├── dispatcher.py                # サブエージェント dispatch
│   │   ├── drift_gen.py                 # Drift コード生成
│   │   ├── db_seed.py                   # モックデータ生成
│   │   └── verify.py                    # Phase A 検証
│   └── pre-commit.sh
├── .github/workflows/
│   ├── build-mobile.yml                 # CI: analyze + test + build
│   └── integration-test.yml             # 実機テスト
└── README.md
```

### 2.3 依存関係 (Phase A のみ)

```yaml
# apps/star-mobile-flutter/pubspec.yaml
dependencies:
  flutter:
    sdk: flutter
  flutter_riverpod: ^2.5.1
  freezed_annotation: ^2.4.4
  json_annotation: ^4.9.0
  intl: ^0.19.0
  # Phase A 固有
  drift: ^2.20.0
  drift_flutter: ^0.2.0
  sqlcipher_flutter_libs: ^0.6.0
  flutter_secure_storage: ^9.2.2   # SQLCipher 鍵管理 (Phase A で必要, 03-detailed §4.8.3 参照)
  connectivity_plus: ^6.0.3
  uuid: ^4.5.0
  path_provider: ^2.1.4
  path: ^1.9.0
  # (Phase B で dio / web_socket_channel 追加)

dev_dependencies:
  flutter_test:
    sdk: flutter
  mocktail: ^1.0.4
  freezed: ^2.5.7
  build_runner: ^2.4.13
  json_serializable: ^6.8.0
  very_good_analysis: ^6.0.0
  drift_dev: ^2.20.0
```

---

## §3 モジュール別実装手順 (Phase A 7 個)

各モジュールに: 概要 / 依存 / テスト / 完了基準 (DoD) を定義。

### 3.1 モジュール 1: OfflineDatabase

#### 概要

Drift + SQLCipher で 7 テーブル (cached_work_items / cached_boards / cached_columns / cached_comments / cached_notifications / sync_queue / conflict_reports) を管理。`03-detailed-design.md` v1.2.1 §4.8.1-4.8.3 完全 Dart コードあり。

#### 依存

- `drift: ^2.20.0`
- `drift_flutter: ^0.2.0`
- `sqlcipher_flutter_libs: ^0.6.0`
- `flutter_secure_storage: ^9.2.2` (鍵管理用 — Phase A 枠のみ)
- `path_provider: ^2.1.4`
- `path: ^1.9.0`

#### 実装手順

1. `pubspec.yaml` 追加
2. `android/app/build.gradle.kts` NDK 25.1.8937393 + SQLCipher 4.5.4 追加
3. `core/db/tables.dart` 7 テーブル定義 (`03-detailed-design.md` v1.2.1 §4.8.2 参照)
4. `core/db/offline_database.dart` `@DriftDatabase` + DAO メソッド 7 テーブル分
5. `core/db/database_provider.dart` SQLCipher 鍵管理 (`03-detailed-design.md` v1.2.1 §4.8.3 参照)
6. `dart run build_runner build --delete-conflicting-outputs` で `.g.dart` 生成
7. 単体テスト (`test/core/db/offline_database_test.dart`)

#### テスト

- 7 テーブル CRUD
- SQLCipher 暗号化確認 (DB ファイル生データを `strings` コマンドで覗いて平文が無いこと)
- migration onCreate
- 鍵ローテーション (将来)

#### DoD (Definition of Done)

- ✅ 7 テーブル すべて CRUD テスト pass
- ✅ SQLCipher 暗号化テスト pass (平文無し)
- ✅ `flutter analyze --fatal-infos` 0 err
- ✅ `dart run build_runner build` 成功
- ✅ `flutter test test/core/db/` 100% pass

---

### 3.2 モジュール 2: SyncQueueService

#### 概要

FIFO キュー + Idempotency-Key (UUID v7) + retry/backoff。`03-detailed-design.md` v1.2.1 §4.9.1 コード完全版。

#### 依存

- モジュール 1 (OfflineDatabase)
- `uuid: ^4.5.0`
- (Phase B で `dio: ^5.7.0` 追加)

#### 実装手順

1. `core/sync/sync_queue_service.dart` `@Riverpod(keepAlive: true)` クラス
2. `enqueue()` / `drainQueue()` / `_processItem()` メソッド
3. 単体テスト (`test/core/sync/sync_queue_service_test.dart`)

#### テスト

- enqueue で SyncQueue に追加される
- drainQueue で pending → success 遷移
- 409 Conflict 受信時 ConflictResolver 委譲
- retry 上限 (3 回) 超過で `dropped` 遷移
- 100 件上限超過時 drop (将来実装)

#### DoD

- ✅ enqueue / drainQueue テスト pass
- ✅ Idempotency-Key UUID v7 生成
- ✅ 409 委譲テスト pass
- ✅ retry 上限テスト pass

---

### 3.3 モジュール 3: ConflictResolver

#### 概要

409 Conflict 受信時、サーバ版 / ローカル版をユーザーに提示し、選択結果で `applyServerVersion` / 再送 / マージを実行。`03-detailed-design.md` v1.2.1 §4.9.2 コード完全版。

#### 依存

- モジュール 1 (OfflineDatabase, conflict_reports テーブル)
- モジュール 2 (SyncQueueService, 再送用)

#### 実装手順

1. `core/sync/conflict_resolver.dart` `@Riverpod(keepAlive: true)`
2. `createReport()` / `resolve(server|local|merge)` メソッド
3. `applyServerVersion()` (Phase B で実装、Phase A は mock)
4. 単体テスト

#### テスト

- createReport で conflict_reports に保存
- resolve(server) でローカル破棄 → サーバ版適用 (mock)
- resolve(local) で SyncQueue 再 enqueue
- resolve(merge) で merged payload 再 enqueue

#### DoD

- ✅ createReport / resolve テスト pass
- ✅ applyServerVersion は Phase A で mock 動作
- ✅ 3 つの resolution 戦略 (server / local / merge) テスト pass

---

### 3.4 モジュール 4: ConnectivityWatcher

#### 概要

`connectivity_plus` 6.x のラッパー。online / offline 状態を提供。`03-detailed-design.md` v1.2.1 §4.9.3 コード完全版。

#### 依存

- `connectivity_plus: ^6.0.3`

#### 実装手順

1. `core/connectivity/connectivity_watcher.dart` `@Riverpod(keepAlive: true)` StreamProvider
2. `Connectivity().onConnectivityChanged` を `ConnectivityState.online/offline` に map
3. 単体テスト (mocktail で `Connectivity` mock)

#### テスト

- online / offline 状態遷移
- Stream が正しく emit する
- 起動時の初期状態

#### DoD

- ✅ online / offline 遷移テスト pass
- ✅ StreamProvider テスト pass

---

### 3.5 モジュール 5: SyncStatusController

#### 概要

SyncState (sealed) を管理し、UI に可視化。`03-detailed-design.md` v1.2.1 §4.11.1 コード完全版。

#### 依存

- モジュール 1 (OfflineDatabase, pending count)
- モジュール 4 (ConnectivityWatcher)

#### 実装手順

1. `features/settings/presentation/sync_status_controller.dart` `@riverpod` AsyncNotifier
2. `SyncState` sealed 定義 (Idle / InProgress / Success / Failed / Conflicts / Offline)
3. `manualSync()` メソッド (SyncQueueService.drainQueue 呼び出し)
4. 単体テスト

#### テスト

- 状態遷移 (Idle → InProgress → Success)
- 競合発生時 SyncConflicts 遷移
- オフライン時 SyncOffline 遷移
- manualSync 動作

#### DoD

- ✅ 状態遷移テスト pass
- ✅ manualSync テスト pass
- ✅ 競合 / オフライン 状態テスト pass

---

### 3.6 モジュール 6: SyncBanner

#### 概要

画面下部バナー UI。`03-detailed-design.md` v1.2.1 §4.12.1 コード完全版。

#### 依存

- モジュール 4 (ConnectivityWatcher)
- モジュール 5 (SyncStatusController)

#### 実装手順

1. `shared/widgets/sync_banner.dart` `ConsumerWidget`
2. SyncState / ConnectivityState に応じて 3 パターン表示
   - オフライン: オレンジバナー「オフライン — 変更は後で同期されます」
   - 同期中: 進捗バナー
   - 競合: 赤バナー「競合 N 件 — 解決」ボタン
3. ウィジェットテスト

#### テスト

- オフライン時 バナー表示
- 同期中時 進捗表示
- 競合時 赤バナー + タップで `/settings/conflicts` 遷移
- 正常時 バナー非表示

#### DoD

- ✅ 3 パターンバナー表示テスト pass
- ✅ タップ遷移テスト pass

---

### 3.7 モジュール 7: LogRedactor

#### 概要

ログ送信前 PII / token / password を自動 redact (regex + 構造化フィールド)。**本モジュールは 03-detailed v1.2.1 に独立した section がない** (v1.1 修订履歴 + §13 Implementation Feasibility で言及のみ)、**本仕様書 §3.7 で新規定義**。NFR-SEC-012 対応。

#### 依存

- なし (pure Dart)

#### 実装手順

1. `core/utils/log_redactor.dart` クラス
2. redact ルール:
   - `Authorization: Bearer ...` → `Authorization: Bearer <redacted>`
   - `password` フィールド → `<redacted>`
   - email (`xxx@yyy.zzz`) → `<email_redacted>`
   - display_name → `<name_redacted>`
   - comment body → `<body_redacted>`
3. 単体テスト

#### テスト

- Authorization ヘッダ redact
- 5 種類の PII redact
- 入れ子構造 (Map of Map) redact
- redact しないキー (`X-Request-Id` 等) 通過

#### DoD

- ✅ 5 種類 PII redact テスト pass
- ✅ 入れ子構造テスト pass
- ✅ ホワイトリスト通過テスト pass

---

## §4 データシーディング (Mock Data)

Phase A は backend 接続なし、モックデータで開発・テスト。`scripts/automation/db_seed.py` で生成。

### 4.1 サンプル Work Item (`test/fixtures/work_item_samples.json`)

```json
[
  {
    "id": "wi-001",
    "project_id": "prj-test",
    "title": "OAuth 2.0 実装",
    "status": "IN_PROGRESS",
    "priority": "High",
    "assignee_user_id": "user-001",
    "created_at": "2026-09-01T10:00:00Z",
    "updated_at": "2026-09-02T12:30:00Z"
  },
  {
    "id": "wi-002",
    "project_id": "prj-test",
    "title": "通知中心 UI 改善",
    "status": "TODO",
    "priority": "Medium",
    "assignee_user_id": "user-002",
    "created_at": "2026-09-01T11:00:00Z",
    "updated_at": "2026-09-02T13:00:00Z"
  }
]
```

### 4.2 サンプル SyncQueueItem (`test/fixtures/sync_queue_samples.json`)

```json
[
  {
    "kind": "edit",
    "resource_id": "wi-001",
    "payload": {"priority": "High"},
    "idempotency_key": "018e7c5e-2b6f-7c4d-9a8b-1234567890ab",
    "status": "pending"
  }
]
```

### 4.3 サンプル ConflictReport (`test/fixtures/conflict_samples.json`)

```json
[
  {
    "id": "cr-001",
    "work_item_id": "wi-001",
    "server_version": {"priority": "Medium", "version": 3},
    "local_version": {"priority": "High", "version": 2},
    "conflicted_fields": ["priority"],
    "resolution": "pending"
  }
]
```

---

## §5 開発環境セットアップ

> **Phase A 注記**: Phase A は backend 接続なし (per Capability Audit §16.6)。`STAR_HOST` / `WSS_HOST` 環境変数は Phase B まで不要。release ビルド時の `--dart-define=STAR_HOST` は将来の Phase B 用プレースホルダ。

### 5.1 初回セットアップ手順

```bash
# 1. 倉作成 (G-12 拍板待ち、デフォルト apps/star-mobile-flutter/)
mkdir -p apps/star-mobile-flutter
cd apps/star-mobile-flutter

# 2. Flutter init
flutter create --org com.star --project-name star_mobile .

# 3. 依存追加
# pubspec.yaml を §2.3 の内容に書き換え

flutter pub get

# 4. Drift コード生成
dart run build_runner build --delete-conflicting-outputs

# 5. 環境変数設定 (開発)
cat > dev.json <<'EOF'
{
  "STAR_HOST": "http://star.local:8080",
  "WSS_HOST": "ws://star.local:8080",
  "DB_KEY_ID": "dev_v1"
}
EOF

# 6. テスト実行
flutter test

# 7. APK ビルド
flutter build apk --debug \
  --dart-define-from-file=dev.json
```

### 5.2 日常開発コマンド

```bash
# コード生成 (Drift / Freezed)
dart run build_runner build --delete-conflicting-outputs

# Lint
flutter analyze

# フォーマット
dart format lib test

# テスト
flutter test
flutter test --coverage  # カバレッジ

# 統合テスト (実機 / エミュレータ)
flutter test integration_test/

# ビルド (debug)
flutter build apk --debug --dart-define-from-file=dev.json

# ビルド (release, Phase A 段階)
flutter build apk --release \
  --dart-define=STAR_HOST=http://star.internal:8080 \
  --dart-define=DB_KEY_ID=internal_v1 \
  --obfuscate --split-debug-info=build/symbols/
```

---

## §6 CI / ビルドパイプライン

### 6.1 GitHub Actions ワークフロー

`.github/workflows/build-mobile.yml`:

```yaml
name: Build Mobile (Phase A)

on:
  pull_request:
    paths: [apps/star-mobile-flutter/**]
  push:
    branches: [main]
    paths: [apps/star-mobile-flutter/**]

jobs:
  analyze-and-test:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: apps/star-mobile-flutter
    steps:
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with:
          channel: stable
      - run: flutter pub get
      - run: dart run build_runner build --delete-conflicting-outputs
      - run: flutter analyze --fatal-infos
      - run: dart format --set-exit-if-changed lib test
      - run: flutter test --coverage
      - uses: actions/upload-artifact@v4
        with:
          name: coverage
          path: apps/star-mobile-flutter/coverage/lcov.info

  build-apk:
    needs: analyze-and-test
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: apps/star-mobile-flutter
    steps:
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with:
          channel: stable
      - run: flutter pub get
      - run: dart run build_runner build --delete-conflicting-outputs
      - run: flutter build apk --release --dart-define=STAR_HOST=http://star.local:8080 --obfuscate
      - uses: actions/upload-artifact@v4
        with:
          name: app-release
          path: apps/star-mobile-flutter/build/app/outputs/flutter-apk/app-release.apk
```

### 6.2 必須 CI チェック

すべて exit 0 必須 (per `AGENTS.md` §4 #1 守門):

- `flutter pub get` 成功
- `dart run build_runner build` 成功
- `flutter analyze --fatal-infos` 0 err
- `dart format --set-exit-if-changed` 0 diff
- `flutter test` 100% pass
- `flutter build apk --release` 成功

---

## §7 テスト戦略

### 7.1 Phase A テストレベル

| レベル | 範囲 | ツール | カバレッジ目標 |
|---|---|---|---|
| 単体テスト (Unit) | 7 モジュール + 関連ヘルパー | `flutter test` + `mocktail` | **≥ 70%** |
| ウィジェットテスト (Widget) | SyncBanner + LogRedactor 関連 | `flutter test` | 関連 widget 100% |
| 統合テスト (Integration) | offline / conflict / logout | `integration_test` | **3 シナリオ** |

### 7.2 Phase A 必須テストケース

| ファイル | テスト |
|---|---|
| `test/core/db/offline_database_test.dart` | 7 テーブル CRUD + SQLCipher 暗号化 + migration |
| `test/core/sync/sync_queue_service_test.dart` | enqueue + drainQueue + 409 委譲 + retry |
| `test/core/sync/conflict_resolver_test.dart` | createReport + resolve(server/local/merge) |
| `test/core/connectivity/connectivity_watcher_test.dart` | online / offline 遷移 + Stream |
| `test/features/settings/sync_status_controller_test.dart` | 状態遷移 + manualSync + 競合 / オフライン |
| `test/shared/widgets/sync_banner_test.dart` | 3 パターン + タップ遷移 |
| `test/core/utils/log_redactor_test.dart` | 5 種類 PII + 入れ子 + ホワイトリスト |
| `integration_test/offline_edit_test.dart` | オフライン編集 → 接続回復 → 同期成功 |
| `integration_test/conflict_test.dart` | 409 Conflict → 競合解決 UI → 再送 |
| `integration_test/security_logout_test.dart` | ログアウト → Drift DB 削除確認 |

### 7.3 カバレッジ目標

- Line coverage: **≥ 70%**
- Branch coverage: **≥ 60%**
- `flutter test --coverage` + `lcov` で計測、CI で 70% 未満 fail

---

## §8 サブエージェント brief (per 守門 #20)

> **守門 #20**: サブエージェント dispatch 前に必先 `automation/dispatcher.py brief(...)` で brief を `docs/briefs/<task_id>.md` に書く。**RPC 不可信頼** (10 background task `net::ERR_CONNECTION_CLOSED` でも status 報告 `succeeded` の事例あり、per AGENTS.md §4 #9)。本書は 7 モジュール分の brief テンプレートを含む。

### 8.1 brief フォーマット

```markdown
# Brief: <モジュール名>

## 目標
- 実装するモジュール: <名前>
- 適用範囲: Phase A
- 関連設計: `03-detailed-design.md` v1.2.1 §<章>

## 依存
- 入力: <上流モジュール>
- 出力: <下流モジュール>
- 外部: <ライブラリ>

## DoD (Definition of Done)
- ✅ 単体テスト ≥ 70%
- ✅ `flutter analyze` 0 err
- ✅ Commit hash: <待証>
- ✅ 単体テスト commit 短碼 を git log -p --follow <file> で実証
- ✅ AGENTS.md §4 13 項守門 すべて pass

## 禁止事項
- ❌ 編造 BAS 引用 (per 守門 #12)
- ❌ 仕事外範囲 (Phase B モジュール含む)
- ❌ 単独 commit せず status="succeeded" 偽報告 (per 守門 #9)

## 完了報告
- 完了時: `<task_id>.status.json` を `docs/briefs/<task_id>.status.json` に書く
- commit hash + `git log -p --follow` 検証結果を明記
```

### 8.2 7 モジュール brief 一覧

| Task ID | モジュール | 場所 | 推定 token |
|---|---|---|---|
| `01-offline-database` | OfflineDatabase | `docs/briefs/01-offline-database.md` | ~0.5M |
| `02-sync-queue-service` | SyncQueueService | `docs/briefs/02-sync-queue-service.md` | ~0.4M |
| `03-conflict-resolver` | ConflictResolver | `docs/briefs/03-conflict-resolver.md` | ~0.3M |
| `04-connectivity-watcher` | ConnectivityWatcher | `docs/briefs/04-connectivity-watcher.md` | ~0.2M |
| `05-sync-status-controller` | SyncStatusController | `docs/briefs/05-sync-status-controller.md` | ~0.2M |
| `06-sync-banner` | SyncBanner | `docs/briefs/06-sync-banner.md` | ~0.2M |
| `07-log-redactor` | LogRedactor | `docs/briefs/07-log-redactor.md` | ~0.2M |
| **合計** | — | — | **~2.0M** |

### 8.3 dispatcher.py 使用例 (per 守門 #19 + #20)

```bash
cd scripts/automation
python dispatcher.py brief --task 01-offline-database --module OfflineDatabase --design-ref "03-detailed-design.md v1.2.1 §4.8"

# 完了後
python dispatcher.py dispatch --task 01-offline-database --worker explorer
# → explorer が brief を読んで実装、commit hash を返す
# → Mavis 接手が git log -p --follow <file> で実証 (per 守門 #9)
```

---

## §9 検証手順 (Phase A 完了基準)

### 9.1 ローカル検証

```bash
cd apps/star-mobile-flutter

# 1. Lint / フォーマット
flutter analyze --fatal-infos
dart format --set-exit-if-changed lib test
# 期待: exit 0, 0 err, 0 diff

# 2. 単体テスト
flutter test --coverage
# 期待: 全 pass, coverage ≥ 70%

# 3. 統合テスト (エミュレータ起動後)
flutter test integration_test/
# 期待: offline_edit / conflict / security_logout 3 シナリオ pass

# 4. ビルド
flutter build apk --release --obfuscate
# 期待: APK 生成, size ≤ 25 MB (Phase A は backend 機能なし、軽量)

# 5. 検証スクリプト
python scripts/automation/verify.py
# 期待: 7 モジュール実装 + 7 テストファイル + pubspec + AndroidManifest + CI workflow すべて存在
```

### 9.2 CI 検証 (per §6.1)

`.github/workflows/build-mobile.yml` の `analyze-and-test` + `build-apk` ジョブ両方 pass。

### 9.3 5 域 Lead レビュー (per 8/21 JST)

- work-item 域: 競合解決 UI / SyncQueue 確認
- board 域: (Phase A は board 関連なし、Phase B 待ち)
- notification 域: (Phase A は notification なし、Phase B 待ち)
- auth 域: 鍵管理 / LogRedactor 確認
- frontend 域: SyncBanner UI / Theme 確認

各域 Lead の Mavis 接手代簽で進行可 (per 8/27 19:39 JST)、DDD Review 段階で 5 域 Lead 真人補簽。

---

## §10 既知の制限事項

### 10.1 Phase A 段階の制限 (backend 未実装)

- **ログイン不可**: Phase A は auth UI / ログイン画面なし、起動時「未認証」状態で OfflineDatabase のみ動作
- **REST API 呼び出し不可**: SyncQueueService は enqueue のみ、drainQueue は Phase B で HTTP 呼び出し実装
- **WebSocket 接続なし**: ConnectivityWatcher は offline 検知のみ、PushState は Phase B
- **通知受信不可**: 通知テーブル cache のみ、サーバ推送は Phase B
- **競合解決 UI はモック**: ConflictResolutionScreen は Phase B で実装、Phase A は conflict_reports テーブル保存のみ

### 10.2 性能上の注意

- SQLCipher 起動時 鍵取得 + DB オープン: **約 200-500ms** (NFR-PERF-001 cold start ≤ 1.5s 余裕あり)
- Drift 7 テーブル初期 migration: 数十 ms
- 同期キュー 100 件上限: 超過時 drop 通知 (G-24 拍板待ち)

### 10.3 セキュリティ上の注意

- 開発時の Keystore 鍵は `dev.json` に保存、`.gitignore` 追加必須
- リリース時の鍵は `--dart-define=DB_KEY_ID=internal_v1` で注入、CI secret 経由
- ログ送信機能 (Phase B 予定) で LogRedactor 必須

---

## §11 Phase B への拡張計画 (v1.1 本書, P2 backend 完了後)

| 追加モジュール | 依存 | 開始条件 |
|---|---|---|
| WebSocketService | backend WS 実装 | P2 backend WS 完成 |
| PushEventRouter | WebSocketService | 同上 |
| WorkItemWriteService | `PATCH /v1/work-items/{id}` 実装 | P2 backend REST 業務ロジック完成 |
| CommentsController | `POST /v1/work-items/{id}/comments` 実装 | 同上 |
| TransitionsController | `POST /v1/work-items/{id}:transition` 実装 | 同上 |

Phase B 着手時、本書 v1.1 として §3 を 5 モジュール追加、§6 CI に backend 接続チェック追加、§7 テスト戦略拡張。

---

## §12 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v1.0 | 2026-09-02 17:16 JST | 架構師 (Mavis 接手 agent per DEC-008) | IPA 4 段組 (要件→基本→詳細→実装) 最終段初版: Phase A 7 モジュール限定 (OfflineDatabase / SyncQueueService / ConflictResolver / ConnectivityWatcher / SyncStatusController / SyncBanner / LogRedactor); 倉位置 / pubspec / 開発環境 / CI / テスト / サブエージェント brief / 検証手順 / 既知制限; Phase B 5 モジュール (WS / Write / Comments / Transitions / PushEventRouter) は v1.1 保留 | 2026-09-02 17:16 JST Ulysses「撰写spec」発令; 上位 3 doc (要件/基本/詳細 v1.2.1) の IPA 4 段組 完成済を受けて実装仕様書着手 |
| **v1.0.1** | 2026-09-02 17:22 JST | 架構師 (Mavis 接手 agent per DEC-008) | **self-review 3 件 patch**: (1) §2.3 pubspec.yaml 漏列 `flutter_secure_storage: ^9.2.2` 追加 (03-detailed §4.8.3 と整合); (2) §6.1 CI workflow `build-apk` job 重複 `actions/checkout@v4` step 削除 (YAML 修正); (3) §3.7 LogRedactor 引用 `03-detailed §4.12.1` を修正 — §4.12.1 は SyncBanner, LogRedactor は 03-detailed に独立 section なし、本仕様書 §3.7 で新規定義; 副: §5 Phase A 注記追加 (STAR_HOST 不要明示) | 2026-09-02 17:22 JST Ulysses「自审」発令; self-review skill 適用 (lens: バグ / hallucinated APIs / ripple effects / consistency / leftovers); cross-doc 検証で 3 HIGH/MEDIUM issues 発見・修正 |

---

## §13 承認欄 (5 角色, AGENTS.md §3 + 8/21 JST 5 域独立)

| 角色 | 簽字 | 日付 | 備註 |
|---|---|---|---|
| 架構師 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽 (per 8/27 19:39 JST + 21:59 JST 三次強化授權) |
| SRE Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;5 域独立真实身份 (per 8/21 JST) DDD Review 段階で補充 |
| 平台 Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| 評審主持 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| PM | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
