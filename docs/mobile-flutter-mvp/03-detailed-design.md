# Star Mobile Flutter MVP — 詳細設計書

> **基準**: 日本 IPA（情報処理推進機構）SEC ソフトウェア詳細設計書 標準章立て
> **作成日**: 2026-09-02
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **バージョン**: v1.1 (UAT 完全版, 2026-09-02 16:27 JST)
> **前身**: v1.0 (read-only, commit `6bd6aa2`, 2026-09-02 16:14 JST) → UAT 範囲追加により v1.1 へ全面書き換え
> **上流要件定義書**: `D:\Star\docs\mobile-flutter-mvp\01-requirements.md` v1.1
> **上流基本設計書**: `D:\Star\docs\mobile-flutter-mvp\02-basic-design.md` v1.1
> **Pre-IPA 草稿**: `docs/architecture/2026-09-02-upgrade/spec/mobile/01-flutter-mvp-design.md` v0.1 (commit `bd4998e`)

---

## §1 目的

本文書は、Star Mobile Flutter MVP **UAT レベル** の詳細設計を定義する。基本設計書 `02-basic-design.md` v1.1 で定義した 4 層 + Sync Engine アーキテクチャ、22 コンポーネント、18 ドメインモデルの**実装レベルの詳細**を記述する。

**v1.0 → v1.1 の主要変更点**:
- ❌ read-only → ✅ 写操作 (PATCH / POST :transition / POST comments)
- ❌ メモリのみ → ✅ Drift + SQLCipher + SyncQueue + ConflictResolver
- ❌ REST ポーリングのみ → ✅ WebSocket + REST フォールバック

**本書追加章節**:
- §4.7-§4.12 6 つの新モジュール (WebSocket / Drift DB / Sync Engine / 写操作 3 種)
- §5.4-§5.6 3 つの新シーケンス (WS / オフライン編集 / 競合解決)
- §6.3 Drift 7 テーブル DDL 完全形
- §6.4 5 つの新 JSON スキーマ (PATCH / POST :transition / POST comments / sync batch / WS messages)
- §7.5-§7.7 3 つの新状態管理パターン
- §9 テスト戦略全面拡張 (オフライン / 競合 / WS)

---

## §2 適用範囲

`01-requirements.md` v1.1 §2 / `02-basic-design.md` v1.1 §2 と同じ。

---

## §3 前提条件・制約事項

### 3.1 開発環境（v1.0 拡張）

| 項目 | バージョン / 設定 |
|---|---|
| Flutter SDK | 3.24.0+ (stable) |
| Dart SDK | 3.5.0+ |
| Android Studio | Hedgehog 2023.1.1+ |
| Android SDK | Platform 34, Build-Tools 34.0.0 |
| Android NDK | 25.1.8937393 (SQLCipher 用) |
| JDK | OpenJDK 17 (Temurin) |
| Gradle | 8.5+ |
| Kotlin | 1.9+ |

### 3.2 依存パッケージ（pubspec.yaml 完全形は §8.1）

**主要追加** (UAT 拡張):
- `drift: ^2.20.0`
- `drift_flutter: ^0.2.0`
- `sqlcipher_flutter_libs: ^0.6.0`
- `web_socket_channel: ^3.0.0`
- `connectivity_plus: ^6.0.3` (v1.0 継承)
- `uuid: ^4.5.0` (Idempotency-Key 用)
- `collection: ^1.18.0` (List/Set utilities)

**既存 v1.0 継承**:
- `flutter_riverpod: ^2.5.1`
- `dio: ^5.7.0` + `dio_smart_retry: ^7.0.0`
- `flutter_secure_storage: ^9.2.2`
- `go_router: ^14.2.0`
- `freezed_annotation: ^2.4.4` + `json_annotation: ^4.9.0`
- `intl: ^0.19.0` + `timeago: ^3.7.0`

**dev 追加**:
- `drift_dev: ^2.20.0`

### 3.3 コーディング規約（v1.0 継承）

- `very_good_analysis` lint ルール
- `dart format` 100% pass
- 命名規則: snake_case (file) / PascalCase (class) / camelCase (var/func)
- 1 ファイル 1 クラス（freezed 補完除く）

### 3.4 追加規約（UAT 拡張）

- **Idempotency-Key**: 全ての write 系 API で UUID v7 を生成、SyncQueueItem.payload に保存
- **Optimistic UI**: 全ての write 操作で API 応答前に UI 更新
- **Conflict Resolution**: 409 Conflict 時は必ず ConflictResolver に委譲 (try-catch しない)
- **WS 再接続**: 指数バックオフ 1s→3s→10s、最大 5 回で PushError 状態

---

## §4 モジュール設計（クラス詳細）

§4.1〜§4.6 は v1.0 継承（変更箇所のみ更新）、§4.7〜§4.12 が UAT 新規。

### 4.1-4.6 略 (v1.0 継承、変更なし)

DioClient / 3 Interceptors / TokenStorage / AuthState / AuthController / BoardController / NotificationsController / 6 freezed model は v1.0 と同じ。詳細実装は `commit 6bd6aa2` 時点の `03-detailed-design.md` 参照。

### 4.7 core/ws モジュール（UAT 新規）

#### 4.7.1 `WebSocketService`

```dart
// lib/core/ws/websocket_service.dart
@Riverpod(keepAlive: true)
class WebSocketService extends _$WebSocketService {
  WebSocketChannel? _channel;
  StreamSubscription? _subscription;
  Timer? _heartbeatTimer;
  Timer? _reconnectTimer;
  int _reconnectAttempt = 0;
  static const _maxReconnectAttempts = 5;
  static const _backoffSchedule = [Duration(seconds: 1), Duration(seconds: 3), Duration(seconds: 10)];
  
  @override
  PushState build() {
    ref.onDispose(() {
      _subscription?.cancel();
      _heartbeatTimer?.cancel();
      _reconnectTimer?.cancel();
      _channel?.sink.close();
    });
    return const PushDisconnected();
  }
  
  /// 接続開始
  Future<void> connect() async {
    if (state is PushConnected || state is PushConnecting) return;
    
    state = const PushConnecting();
    
    final token = await ref.read(tokenStorageProvider).readAccessToken();
    if (token == null) {
      state = const PushDisconnected();
      return;
    }
    
    final wssUrl = Uri.parse('${Env.wssBaseUrl}/api/v1/ws');
    
    try {
      _channel = WebSocketChannel.connect(
        wssUrl,
        protocols: ['star.v1'],
        headers: {'Authorization': 'Bearer $token'},
      );
      
      // 接続確立
      await _channel!.ready;
      
      // subscribe 送信
      _sendSubscribe();
      
      // メッセージ受信
      _subscription = _channel!.stream.listen(
        _onMessage,
        onError: _onError,
        onDone: _onDone,
      );
      
      // heartbeat
      _heartbeatTimer = Timer.periodic(const Duration(seconds: 30), (_) {
        // サーバ ping 待機 (60s 以内 pong 期待)
      });
      
      _reconnectAttempt = 0;
      state = PushConnected(subscribedResources: ['work_item', 'notification']);
    } catch (e) {
      _scheduleReconnect();
    }
  }
  
  void _sendSubscribe() {
    _channel?.sink.add(jsonEncode({
      'type': 'subscribe',
      'id': 'sub-${Uuid().v4()}',
      'filter': {
        'resource_types': ['work_item', 'notification'],
        // project_id は JWT スコープから自動抽出 (サーバ側)
      },
    }));
  }
  
  void _onMessage(dynamic raw) {
    try {
      final json = jsonDecode(raw as String) as Map<String, dynamic>;
      final type = json['type'] as String;
      
      switch (type) {
        case 'event':
          _handleEvent(json);
          break;
        case 'ping':
          _channel?.sink.add(jsonEncode({'type': 'pong', 'ts': json['ts']}));
          break;
        case 'pong':
          // heartbeat ack
          break;
      }
    } catch (_) {
      // 不正メッセージは無視
    }
  }
  
  void _handleEvent(Map<String, dynamic> event) {
    final resourceType = event['resource_type'] as String;
    final resourceId = event['resource_id'] as String;
    final action = event['action'] as String;
    final data = event['data'] as Map<String, dynamic>;
    
    final pushEvent = switch (resourceType) {
      'work_item' when action == 'updated' => PushEvent.workItemUpdated(
        WorkItem.fromJson(data),
      ),
      'work_item' when action == 'commented' => PushEvent.workItemCommented(
        resourceId,
        Comment.fromJson(data),
      ),
      'notification' when action == 'new' => PushEvent.notificationNew(
        Notification.fromJson(data),
      ),
      _ => PushEvent.unknown(resourceType, action),
    };
    
    // PushEventRouter に委譲
    ref.read(pushEventRouterProvider).dispatch(pushEvent);
  }
  
  void _onError(Object error, StackTrace stack) {
    _scheduleReconnect();
  }
  
  void _onDone() {
    _scheduleReconnect();
  }
  
  void _scheduleReconnect() {
    state = PushReconnecting(
      attempt: _reconnectAttempt + 1,
      nextBackoff: _backoffSchedule[_reconnectAttempt.clamp(0, 2)],
    );
    
    if (_reconnectAttempt >= _maxReconnectAttempts) {
      state = const PushError(message: '再接続上限到達、REST ポーリングにフォールバック');
      return;
    }
    
    _reconnectTimer = Timer(_backoffSchedule[_reconnectAttempt.clamp(0, 2)], () {
      _reconnectAttempt++;
      connect();
    });
  }
  
  void disconnect() {
    _subscription?.cancel();
    _heartbeatTimer?.cancel();
    _reconnectTimer?.cancel();
    _channel?.sink.close();
    _channel = null;
    state = const PushDisconnected();
  }
}
```

#### 4.7.2 `PushEventRouter` (UAT 新規)

```dart
// lib/core/ws/push_event_router.dart
@Riverpod(keepAlive: true)
class PushEventRouter extends _$PushEventRouter {
  @override
  void build() {}
  
  void dispatch(PushEvent event) {
    switch (event) {
      case WorkItemUpdated(:final workItem):
        // BoardController + WorkItemController に通知
        ref.read(boardListControllerProvider(workItem.projectId).notifier)
          .onWorkItemUpdated(workItem);
        ref.read(workItemControllerProvider(workItem.id).notifier)
          .onWorkItemUpdated(workItem);
      case WorkItemCommented(:final workItemId, :final comment):
        ref.read(commentsControllerProvider(workItemId).notifier)
          .onCommentAdded(comment);
      case NotificationNew(:final notification):
        ref.read(notificationsControllerProvider.notifier)
          .onNotificationReceived(notification);
      case Unknown():
        // 無視
    }
  }
}
```

### 4.8 core/db モジュール（UAT 新規）

#### 4.8.1 `OfflineDatabase` (Drift + SQLCipher)

```dart
// lib/core/db/offline_database.dart
part 'offline_database.g.dart';

@DriftDatabase(
  tables: [
    CachedWorkItems,
    CachedBoards,
    CachedColumns,
    CachedComments,
    CachedNotifications,
    SyncQueue,
    ConflictReports,
  ],
)
class OfflineDatabase extends _$OfflineDatabase {
  OfflineDatabase(super.e);
  
  @override
  int get schemaVersion => 1;
  
  @override
  MigrationStrategy get migration => MigrationStrategy(
    onCreate: (m) => m.createAll(),
    beforeOpen: (details) async {
      await customStatement('PRAGMA foreign_keys = ON');
    },
  );
  
  // ====== Cached Work Items ======
  
  Future<List<CachedWorkItem>> getAllCachedWorkItems(String projectId) {
    return (select(cachedWorkItems)
      ..where((t) => t.projectId.equals(projectId))
      ..orderBy([(t) => OrderingTerm.desc(t.updatedAt)]))
      .get();
  }
  
  Future<CachedWorkItem?> getCachedWorkItem(String id) {
    return (select(cachedWorkItems)..where((t) => t.id.equals(id))).getSingleOrNull();
  }
  
  Future<void> upsertWorkItem(CachedWorkItem item) {
    return into(cachedWorkItems).insertOnConflictUpdate(item);
  }
  
  Future<void> upsertWorkItems(List<CachedWorkItem> items) async {
    await batch((b) {
      for (final item in items) {
        b.insert(cachedWorkItems, item, mode: InsertMode.insertOrReplace);
      }
    });
  }
  
  // ====== Sync Queue ======
  
  Future<int> enqueueSyncItem(SyncQueueCompanion item) {
    return into(syncQueue).insert(item);
  }
  
  Future<List<SyncQueueData>> getPendingSyncItems({int limit = 20}) {
    return (select(syncQueue)
      ..where((t) => t.status.equals('pending'))
      ..orderBy([(t) => OrderingTerm.asc(t.createdAt)])
      ..limit(limit))
      .get();
  }
  
  Future<void> updateSyncItemStatus(int id, String status, {String? lastError}) {
    return (update(syncQueue)..where((t) => t.id.equals(id))).write(
      SyncQueueCompanion(
        status: Value(status),
        lastError: Value(lastError),
      ),
    );
  }
  
  Future<int> pendingSyncCount() async {
    final count = countAll(filter: syncQueue.status.equals('pending'));
    final query = selectOnly(syncQueue)..addColumns([count]);
    final row = await query.getSingle();
    return row.read(count) ?? 0;
  }
  
  // ====== Conflict Reports ======
  
  Future<void> insertConflictReport(ConflictReportsCompanion report) {
    return into(conflictReports).insert(report);
  }
  
  Future<List<ConflictReportData>> getUnresolvedConflicts() {
    return (select(conflictReports)
      ..where((t) => t.resolution.equals('pending')))
      .get();
  }
  
  // ====== Cache Management ======
  
  Future<int> deleteOldCacheItems(DateTime threshold) {
    return (delete(cachedWorkItems)..where((t) => t.cachedAt.isSmallerThanValue(threshold))).go();
  }
  
  Future<void> clearAll() async {
    await batch((b) {
      b.deleteAll(cachedWorkItems);
      b.deleteAll(cachedBoards);
      b.deleteAll(cachedColumns);
      b.deleteAll(cachedComments);
      b.deleteAll(cachedNotifications);
      b.deleteAll(syncQueue);
      b.deleteAll(conflictReports);
    });
  }
}
```

#### 4.8.2 Drift テーブル定義（DDL 完全形）

```dart
// lib/core/db/tables.dart

class CachedWorkItems extends Table {
  TextColumn get id => text()();
  TextColumn get projectId => text()();
  TextColumn get title => text()();
  TextColumn get status => text()();
  TextColumn get priority => text()();
  TextColumn get assigneeUserId => text().nullable()();
  TextColumn get assigneeDisplayName => text().nullable()();
  TextColumn get assigneeAvatarUrl => text().nullable()();
  TextColumn get description => text().nullable()();
  TextColumn get reporterUserId => text().nullable()();
  DateTimeColumn get dueDate => dateTime().nullable()();
  TextColumn get repositoryIdsJson => text().nullable()();
  TextColumn get worktreeIdsJson => text().nullable()();
  DateTimeColumn get createdAt => dateTime()();
  DateTimeColumn get updatedAt => dateTime()();
  DateTimeColumn get cachedAt => dateTime()();
  
  @override
  Set<Column> get primaryKey => {id};
}

class CachedBoards extends Table {
  TextColumn get projectId => text()();
  TextColumn get boardId => text()();
  TextColumn get boardType => text()();
  DateTimeColumn get cachedAt => dateTime()();
  
  @override
  Set<Column> get primaryKey => {projectId};
}

class CachedColumns extends Table {
  TextColumn get id => text()();
  TextColumn get projectId => text()();
  TextColumn get stateId => text()();
  TextColumn get name => text()();
  IntColumn get order => integer()();
  
  @override
  Set<Column> get primaryKey => {id};
}

class CachedComments extends Table {
  TextColumn get id => text()();
  TextColumn get workItemId => text()();
  TextColumn get authorUserId => text()();
  TextColumn get authorDisplayName => text().nullable()();
  TextColumn get authorAvatarUrl => text().nullable()();
  TextColumn get body => text()();
  TextColumn get mentionsJson => text().nullable()();
  DateTimeColumn get createdAt => dateTime()();
  DateTimeColumn get cachedAt => dateTime()();
  
  @override
  Set<Column> get primaryKey => {id};
}

class CachedNotifications extends Table {
  TextColumn get id => text()();
  TextColumn get recipientUserId => text()();
  TextColumn get eventType => text()();
  TextColumn get payloadJson => text()();
  DateTimeColumn get sentAt => dateTime()();
  DateTimeColumn get readAt => dateTime().nullable()();
  DateTimeColumn get cachedAt => dateTime()();
  
  @override
  Set<Column> get primaryKey => {id};
}

class SyncQueue extends Table {
  IntColumn get id => integer().autoIncrement()();
  TextColumn get kind => text()();  // 'edit' | 'comment' | 'transition'
  TextColumn get resourceId => text()();  // work_item_id 等
  TextColumn get payloadJson => text()();
  TextColumn get idempotencyKey => text().unique()();
  TextColumn get status => text()();  // 'pending' | 'in_progress' | 'failed' | 'dropped'
  IntColumn get retryCount => integer().withDefault(const Constant(0))();
  TextColumn get lastError => text().nullable()();
  DateTimeColumn get createdAt => dateTime()();
  DateTimeColumn get lastAttemptAt => dateTime().nullable()();
  
  @override
  Set<Column> get primaryKey => {id};
}

class ConflictReports extends Table {
  TextColumn get id => text()();
  TextColumn get workItemId => text()();
  TextColumn get serverVersionJson => text()();
  TextColumn get localVersionJson => text()();
  TextColumn get conflictedFieldsJson => text()();
  TextColumn get resolution => text()();  // 'pending' | 'server' | 'local' | 'merge'
  DateTimeColumn get detectedAt => dateTime()();
  DateTimeColumn get resolvedAt => dateTime().nullable()();
  
  @override
  Set<Column> get primaryKey => {id};
}
```

#### 4.8.3 Drift Database 初期化（SQLCipher + Keystore 連携）

```dart
// lib/core/db/database_provider.dart
@Riverpod(keepAlive: true)
Future<OfflineDatabase> offlineDatabase(OfflineDatabaseRef ref) async {
  // SQLCipher 鍵を Keystore から取得
  final key = await ref.read(dbKeyProvider.future);
  
  // データベースファイルパス
  final dbDir = await getApplicationDocumentsDirectory();
  final dbFile = File(p.join(dbDir.path, 'star_offline.db'));
  
  // Drift データベースを開く
  return OfflineDatabase(
    NativeDatabase.createInBackground(
      dbFile,
      setup: (db) {
        db.execute("PRAGMA key = '$key'");
      },
    ),
  );
}

@Riverpod(keepAlive: true)
Future<String> dbKey(DbKeyRef ref) async {
  const keyId = 'db_encryption_key_v1';
  const storage = FlutterSecureStorage(
    aOptions: AndroidOptions(encryptedSharedPreferences: true),
  );
  
  // 鍵取得 or 生成
  String? key = await storage.read(key: keyId);
  if (key == null) {
    // 32 バイト (256 bit) ランダム鍵を生成
    final random = Random.secure();
    final bytes = List<int>.generate(32, (_) => random.nextInt(256));
    key = base64Url.encode(bytes);
    await storage.write(key: keyId, value: key);
  }
  return key;
}
```

### 4.9 core/sync モジュール（UAT 新規）

#### 4.9.1 `SyncQueueService`

```dart
// lib/core/sync/sync_queue_service.dart
@Riverpod(keepAlive: true)
class SyncQueueService extends _$SyncQueueService {
  @override
  void build() {
    // 接続性変化を監視
    ref.listen<ConnectivityState>(connectivityWatcherProvider, (prev, next) async {
      if (next == ConnectivityState.online) {
        await drainQueue();
      }
    });
  }
  
  /// 同期キュー追加
  Future<void> enqueue({
    required String kind,
    required String resourceId,
    required Map<String, dynamic> payload,
  }) async {
    final db = await ref.read(offlineDatabaseProvider.future);
    await db.enqueueSyncItem(SyncQueueCompanion.insert(
      kind: kind,
      resourceId: resourceId,
      payloadJson: jsonEncode(payload),
      idempotencyKey: _generateIdempotencyKey(),
      status: 'pending',
      createdAt: DateTime.now(),
    ));
    ref.invalidate(syncStatusControllerProvider);
  }
  
  /// 同期キュー全件処理
  Future<void> drainQueue() async {
    final db = await ref.read(offlineDatabaseProvider.future);
    final items = await db.getPendingSyncItems(limit: 20);
    
    for (final item in items) {
      try {
        await db.updateSyncItemStatus(item.id, 'in_progress');
        await _processItem(item);
        await db.updateSyncItemStatus(item.id, 'success');
      } on ConflictError catch (e) {
        // 競合 → ConflictResolver
        await _handleConflict(item, e);
      } on AppError catch (e) {
        // リトライ
        if (item.retryCount < 3) {
          await db.updateSyncItemStatus(item.id, 'pending', lastError: e.toString());
          // retry_count + 1
        } else {
          await db.updateSyncItemStatus(item.id, 'dropped', lastError: e.toString());
        }
      }
    }
    
    ref.invalidate(syncStatusControllerProvider);
  }
  
  Future<void> _processItem(SyncQueueData item) async {
    final payload = jsonDecode(item.payloadJson) as Map<String, dynamic>;
    final dio = ref.read(dioClientProvider);
    final idemKey = item.idempotencyKey;
    
    switch (item.kind) {
      case 'edit':
        await dio.patch(
          '/v1/work-items/${item.resourceId}',
          data: payload,
          options: Options(headers: {'Idempotency-Key': idemKey}),
        );
        break;
      case 'transition':
        await dio.post(
          '/v1/work-items/${item.resourceId}:transition',
          data: payload,
          options: Options(headers: {'Idempotency-Key': idemKey}),
        );
        break;
      case 'comment':
        await dio.post(
          '/v1/work-items/${item.resourceId}/comments',
          data: payload,
          options: Options(headers: {'Idempotency-Key': idemKey}),
        );
        break;
    }
  }
  
  Future<void> _handleConflict(SyncQueueData item, ConflictError e) async {
    final db = await ref.read(offlineDatabaseProvider.future);
    final conflictResolver = ref.read(conflictResolverProvider.notifier);
    final localVersion = jsonDecode(item.payloadJson) as Map<String, dynamic>;
    final serverVersion = e.serverVersion;
    
    await conflictResolver.createReport(
      workItemId: item.resourceId,
      localVersion: localVersion,
      serverVersion: serverVersion,
      conflictedFields: e.conflictedFields,
    );
    
    await db.updateSyncItemStatus(item.id, 'failed', lastError: 'Conflict');
  }
  
  String _generateIdempotencyKey() {
    // UUID v7
    return const UuidV7().generate();
  }
}
```

#### 4.9.2 `ConflictResolver`

```dart
// lib/core/sync/conflict_resolver.dart
@Riverpod(keepAlive: true)
class ConflictResolver extends _$ConflictResolver {
  @override
  void build() {}
  
  Future<void> createReport({
    required String workItemId,
    required Map<String, dynamic> localVersion,
    required Map<String, dynamic> serverVersion,
    required List<String> conflictedFields,
  }) async {
    final db = await ref.read(offlineDatabaseProvider.future);
    await db.insertConflictReport(ConflictReportsCompanion.insert(
      id: Uuid().v4(),
      workItemId: workItemId,
      serverVersionJson: jsonEncode(serverVersion),
      localVersionJson: jsonEncode(localVersion),
      conflictedFieldsJson: jsonEncode(conflictedFields),
      resolution: 'pending',
      detectedAt: DateTime.now(),
    ));
    
    // ユーザー通知
    ref.read(syncStatusControllerProvider.notifier).onConflictCreated();
  }
  
  /// ユーザー解決 (UI から呼ばれる)
  Future<void> resolve({
    required String reportId,
    required String resolution,  // 'server' | 'local' | 'merge'
    Map<String, dynamic>? mergedVersion,
  }) async {
    final db = await ref.read(offlineDatabaseProvider.future);
    
    // 1. レポートを resolved に更新
    final report = await (db.select(db.conflictReports)..where((t) => t.id.equals(reportId))).getSingle();
    
    switch (resolution) {
      case 'server':
        // サーバ版を採用 → ローカル変更を破棄
        final serverVersion = jsonDecode(report.serverVersionJson) as Map<String, dynamic>;
        await ref.read(workItemControllerProvider(report.workItemId).notifier)
          .applyServerVersion(serverVersion);
        break;
      case 'local':
        // ローカル版を採用 → 再送
        final localVersion = jsonDecode(report.localVersionJson) as Map<String, dynamic>;
        await ref.read(syncQueueServiceProvider.notifier).enqueue(
          kind: 'edit',  // kind は context 依存、本来は元 kind を保持
          resourceId: report.workItemId,
          payload: localVersion,
        );
        break;
      case 'merge':
        if (mergedVersion == null) throw ArgumentError('merge には mergedVersion 必須');
        await ref.read(syncQueueServiceProvider.notifier).enqueue(
          kind: 'edit',
          resourceId: report.workItemId,
          payload: mergedVersion,
        );
        break;
    }
    
    // 2. レポートを resolved に更新
    await (db.update(db.conflictReports)..where((t) => t.id.equals(reportId))).write(
      ConflictReportsCompanion(
        resolution: Value(resolution),
        resolvedAt: Value(DateTime.now()),
      ),
    );
    
    ref.invalidate(syncStatusControllerProvider);
  }
}
```

#### 4.9.3 `ConnectivityWatcher`

```dart
// lib/core/connectivity/connectivity_watcher.dart
@Riverpod(keepAlive: true)
Stream<ConnectivityState> connectivityWatcher(ConnectivityWatcherRef ref) {
  return Connectivity().onConnectivityChanged.map((event) {
    if (event == ConnectivityResult.wifi || event == ConnectivityResult.mobile) {
      return ConnectivityState.online;
    }
    return ConnectivityState.offline;
  });
}

enum ConnectivityState { online, offline }
```

### 4.10 features/work_item 拡張（UAT 新規）

#### 4.10.1 `WorkItemWriteService` (PATCH 部分更新)

```dart
// lib/features/work_item/data/work_item_write_service.dart
@Riverpod(keepAlive: true)
WorkItemWriteApi workItemWriteApi(WorkItemWriteApiRef ref) {
  final dio = ref.watch(dioClientProvider);
  return WorkItemWriteApi(dio);
}

class WorkItemWriteApi {
  WorkItemWriteApi(this._dio);
  final Dio _dio;
  
  /// PATCH /v1/work-items/{id}
  /// Idempotency-Key 必須 (api-design §1.6)
  Future<WorkItem> patchWorkItem({
    required String id,
    required WorkItemEditCommand command,
    String? ifMatch,  // ETag
  }) async {
    final idempotencyKey = const UuidV7().generate();
    try {
      final response = await _dio.patch(
        '/v1/work-items/$id',
        data: command.toJson(),
        options: Options(
          headers: {
            'Idempotency-Key': idempotencyKey,
            if (ifMatch != null) 'If-Match': ifMatch,
          },
        ),
      );
      return WorkItem.fromJson(response.data as Map<String, dynamic>);
    } on DioException catch (e) {
      if (e.response?.statusCode == 409) {
        throw ConflictError(
          serverVersion: (e.response?.data as Map)['server_version'] as Map<String, dynamic>,
          conflictedFields: (e.response?.data as Map)['conflicted_fields'] as List<String>,
        );
      }
      rethrow;
    }
  }
}
```

#### 4.10.2 `WorkItemController` 拡張 (PATCH 操作)

```dart
// lib/features/work_item/presentation/work_item_controller.dart
@riverpod
class WorkItemController extends _$WorkItemController {
  @override
  Future<WorkItem> build(String workItemId) async {
    // 1. Drift ローカル から即座に返す（オフライン対応）
    final local = await ref.read(offlineDatabaseProvider.future).then(
      (db) => db.getCachedWorkItem(workItemId),
    );
    if (local != null) {
      state = AsyncValue.data(_fromCachedWorkItem(local));
    }
    
    // 2. REST 取得（最新）
    final remote = await ref.read(workItemApiProvider).getWorkItem(workItemId);
    
    // 3. Drift 保存
    await ref.read(offlineDatabaseProvider.future).then(
      (db) => db.upsertWorkItem(_toCachedWorkItem(remote)),
    );
    
    return remote;
  }
  
  /// PATCH 部分更新 (オフライン対応)
  Future<void> patchFields(Map<String, dynamic> patch) async {
    final current = state.valueOrNull;
    if (current == null) return;
    
    // 1. Optimistic UI
    final optimistic = current.copyWithFields(patch);
    state = AsyncValue.data(optimistic);
    
    // 2. Drift 即時更新
    await ref.read(offlineDatabaseProvider.future).then(
      (db) => db.upsertWorkItem(_toCachedWorkItem(optimistic)),
    );
    
    // 3. SyncQueue enqueue
    await ref.read(syncQueueServiceProvider.notifier).enqueue(
      kind: 'edit',
      resourceId: current.id,
      payload: patch,
    );
  }
  
  void onWorkItemUpdated(WorkItem updated) {
    final current = state.valueOrNull;
    if (current == null || current.id != updated.id) return;
    state = AsyncValue.data(updated);
  }
}
```

#### 4.10.3 `TransitionsController` (UAT 新規)

```dart
// lib/features/transitions/presentation/transitions_controller.dart
@riverpod
class TransitionsController extends _$TransitionsController {
  @override
  Future<List<Transition>> build(String workItemId) async {
    return ref.read(workItemApiProvider).getTransitions(workItemId);
  }
  
  Future<void> execute({
    required String workItemId,
    required String toState,
  }) async {
    // 1. Optimistic UI
    final workItem = ref.read(workItemControllerProvider(workItemId)).valueOrNull;
    if (workItem != null) {
      final optimistic = workItem.copyWith(status: toState);
      ref.read(workItemControllerProvider(workItemId).notifier)
        .onWorkItemUpdated(optimistic);
    }
    
    // 2. SyncQueue enqueue
    await ref.read(syncQueueServiceProvider.notifier).enqueue(
      kind: 'transition',
      resourceId: workItemId,
      payload: {'to_state': toState},
    );
  }
}
```

#### 4.10.4 `CommentsController` (UAT 新規)

```dart
// lib/features/comments/presentation/comments_controller.dart
@riverpod
class CommentsController extends _$CommentsController {
  @override
  Future<List<Comment>> build(String workItemId) async {
    // 1. Drift ローカル
    final local = await ref.read(offlineDatabaseProvider.future).then(
      (db) => db.getCachedComments(workItemId),
    );
    if (local.isNotEmpty) {
      state = AsyncValue.data(local);
    }
    
    // 2. REST 取得
    final remote = await ref.read(workItemApiProvider).getComments(workItemId);
    return remote;
  }
  
  Future<void> postComment({
    required String workItemId,
    required String body,
    List<String>? mentions,
  }) async {
    final tempId = 'temp-${Uuid().v4()}';
    final optimistic = Comment(
      id: tempId,
      authorUserId: 'me',
      authorDisplayName: 'You',
      body: body,
      mentions: mentions,
      createdAt: DateTime.now(),
    );
    
    // 1. Optimistic UI
    state = AsyncValue.data([...state.valueOrNull ?? [], optimistic]);
    
    // 2. Drift 即時更新
    await ref.read(offlineDatabaseProvider.future).then(
      (db) => db.upsertComment(_toCachedComment(optimistic, workItemId)),
    );
    
    // 3. SyncQueue enqueue
    await ref.read(syncQueueServiceProvider.notifier).enqueue(
      kind: 'comment',
      resourceId: workItemId,
      payload: {'body': body, 'mentions': mentions ?? []},
    );
  }
  
  void onCommentAdded(Comment comment) {
    final current = state.valueOrNull ?? [];
    if (current.any((c) => c.id == comment.id)) return;
    state = AsyncValue.data([comment, ...current]);
  }
}
```

### 4.11 features/settings 拡張（UAT 新規）

#### 4.11.1 `SyncStatusController`

```dart
// lib/features/settings/presentation/sync_status_controller.dart
@riverpod
class SyncStatusController extends _$SyncStatusController {
  @override
  SyncState build() {
    ref.listen(connectivityWatcherProvider, (prev, next) {
      if (next.value == ConnectivityState.offline) {
        state = const SyncOffline();
      }
    });
    return const SyncIdle();
  }
  
  Future<void> manualSync() async {
    state = const SyncInProgress(remainingItems: 0, totalItems: 0);
    await ref.read(syncQueueServiceProvider.notifier).drainQueue();
    final remaining = await ref.read(offlineDatabaseProvider.future).then(
      (db) => db.pendingSyncCount(),
    );
    state = remaining == 0
      ? SyncSuccess(at: DateTime.now())
      : SyncFailed(error: '$remaining 件未同期');
  }
  
  void onConflictCreated() {
    final conflicts = ref.read(offlineDatabaseProvider.future).then(
      (db) => db.getUnresolvedConflicts(),
    );
    // 簡略化: 1 件以上あれば SyncConflicts
    state = const SyncConflicts(reports: []);
  }
}
```

### 4.12 shared/widgets 拡張（UAT 新規）

#### 4.12.1 `SyncBanner` (画面下部バナー)

```dart
// lib/shared/widgets/sync_banner.dart
class SyncBanner extends ConsumerWidget {
  const SyncBanner({super.key});
  
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final syncState = ref.watch(syncStatusControllerProvider);
    final connectivity = ref.watch(connectivityWatcherProvider).valueOrNull;
    
    if (connectivity == ConnectivityState.offline) {
      return _OfflineBanner();
    }
    
    return switch (syncState) {
      SyncInProgress(:final remainingItems, :final totalItems) => _SyncingBanner(
        remaining: remainingItems,
        total: totalItems,
      ),
      SyncConflicts() => _ConflictBanner(
        onTap: () => context.push('/settings/conflicts'),
      ),
      SyncFailed(:final error) => _ErrorBanner(message: error),
      _ => const SizedBox.shrink(),
    };
  }
}

class _OfflineBanner extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      color: Colors.orange.shade100,
      padding: const EdgeInsets.all(8),
      child: const Row(
        children: [
          Icon(Icons.cloud_off, color: Colors.orange),
          SizedBox(width: 8),
          Text('オフライン — 変更は後で同期されます'),
        ],
      ),
    );
  }
}
```

---

## §5 シーケンス設計（v1.0 継承 + UAT 拡張）

### 5.1-5.3 略 (v1.0 継承)

### 5.4 WebSocket 接続確立（UAT 新規、v1.1 専用）

(本書に §4.7.1 でコード + §2-basic §5.2.1 で図示済み)

### 5.5 オフライン編集 + 接続回復同期（UAT 新規、v1.1 専用）

(本書に §4.9.1 でコード + §2-basic §5.2.2 で図示済み)

### 5.6 競合解決（UAT 新規、v1.1 専用）

(本書に §4.9.2 でコード + §2-basic §5.2.4 で図示済み)

---

## §6 データ構造詳細（JSON スキーマ完全形）

### 6.1 v1.0 継承 (略)

13 エンドポイントの JSON スキーマは v1.0 と同じ。

### 6.2 v1.1 新規エンドポイント

#### 6.2.1 PATCH /v1/work-items/{id}

**Request** (Idempotency-Key 必須 + If-Match 任意):
```http
PATCH /v1/work-items/01HYYY...
Authorization: Bearer eyJ...
Idempotency-Key: 018e7c5e-2b6f-7c4d-9a8b-1234567890ab
If-Match: "v3"
Content-Type: application/json; charset=utf-8

{
  "priority": "High",
  "assignee_user_id": "01HZZZ...",
  "description": "Updated description...",
  "due_date": "2026-09-15T00:00:00Z"
}
```

**Response 200**:
```json
{
  "id": "01HYYY...",
  "priority": "High",
  "assignee_user_id": "01HZZZ...",
  "description": "Updated description...",
  "due_date": "2026-09-15T00:00:00Z",
  "version": 4,
  "updated_at": "2026-09-02T16:30:00Z"
}
```

**Response 409 Conflict**:
```json
{
  "type": "https://star.acme.com/errors/CONFLICT",
  "title": "Conflict",
  "status": 409,
  "detail": "Work item has been modified since you last read it",
  "code": "CONFLICT-001",
  "trace_id": "...",
  "server_version": {
    "priority": "Medium",
    "description": "Server version (different from your local change)",
    "updated_at": "2026-09-02T16:25:00Z",
    "version": 3
  },
  "conflicted_fields": ["priority", "description"]
}
```

#### 6.2.2 POST /v1/work-items/{id}:transition

**Request** (Idempotency-Key 必須):
```http
POST /v1/work-items/01HYYY...:transition
Idempotency-Key: 018e7c5e-2b6f-7c4d-9a8b-1234567890ab

{
  "to_state": "IN_PROGRESS"
}
```

**Response 200**: `WorkItem` (新状態)

#### 6.2.3 POST /v1/work-items/{id}/comments

**Request** (Idempotency-Key 必須):
```http
POST /v1/work-items/01HYYY.../comments
Idempotency-Key: 018e7c5e-2b6f-7c4d-9a8b-1234567890ab

{
  "body": "Looking into this now",
  "mentions": ["01HWWW..."]
}
```

**Response 201**: `Comment`

### 6.3 Drift テーブル DDL（生成形、SQLCipher 暗号化）

```sql
-- Cached Work Items
CREATE TABLE cached_work_items (
  id TEXT NOT NULL PRIMARY KEY,
  project_id TEXT NOT NULL,
  title TEXT NOT NULL,
  status TEXT NOT NULL,
  priority TEXT NOT NULL,
  assignee_user_id TEXT,
  assignee_display_name TEXT,
  assignee_avatar_url TEXT,
  description TEXT,
  reporter_user_id TEXT,
  due_date INTEGER,  -- DateTime
  repository_ids_json TEXT,
  worktree_ids_json TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  cached_at INTEGER NOT NULL
);
CREATE INDEX idx_cached_work_items_project ON cached_work_items(project_id);
CREATE INDEX idx_cached_work_items_updated ON cached_work_items(updated_at DESC);

-- Sync Queue
CREATE TABLE sync_queue (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  kind TEXT NOT NULL,
  resource_id TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  idempotency_key TEXT NOT NULL UNIQUE,
  status TEXT NOT NULL,
  retry_count INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  created_at INTEGER NOT NULL,
  last_attempt_at INTEGER
);
CREATE INDEX idx_sync_queue_status ON sync_queue(status, created_at);

-- Conflict Reports
CREATE TABLE conflict_reports (
  id TEXT NOT NULL PRIMARY KEY,
  work_item_id TEXT NOT NULL,
  server_version_json TEXT NOT NULL,
  local_version_json TEXT NOT NULL,
  conflicted_fields_json TEXT NOT NULL,
  resolution TEXT NOT NULL,
  detected_at INTEGER NOT NULL,
  resolved_at INTEGER
);
CREATE INDEX idx_conflict_reports_resolution ON conflict_reports(resolution);
```

### 6.4 WebSocket メッセージ形式

(`02-basic-design.md` v1.1 §7.2 と同一、完全形はそちら参照)

---

## §7 状態管理詳細

### 7.1-7.3 略 (v1.0 継承 + v1.1 拡張は §4 モジュールコードに記載)

### 7.4 編集操作の状態管理パターン (UAT 新規)

```dart
// Controller パターン: Optimistic UI + SyncQueue
class WorkItemController extends _$WorkItemController {
  Future<void> patchFields(Map<String, dynamic> patch) async {
    final current = state.valueOrNull;
    if (current == null) return;
    
    // 1. 即座に UI 反映 (Optimistic)
    final optimistic = current.copyWithFields(patch);
    state = AsyncValue.data(optimistic);
    
    // 2. Drift 即時更新 (オフライン対応)
    await _updateLocal(optimistic);
    
    // 3. SyncQueue enqueue (同期は非同期)
    await ref.read(syncQueueServiceProvider.notifier).enqueue(
      kind: 'edit',
      resourceId: current.id,
      payload: patch,
    );
    
    // 4. 同期は SyncQueueService が connectivity 監視して自動実行
    // 5. 成功時は state を最新に更新
    // 6. 競合時は ConflictResolver に委譲
  }
}
```

### 7.5 Sync Engine パターン (UAT 新規)

```dart
// SyncQueueService: 接続性監視 + 自動同期
@Riverpod(keepAlive: true)
class SyncQueueService extends _$SyncQueueService {
  @override
  void build() {
    ref.listen(connectivityWatcherProvider, (prev, next) {
      if (next.value == ConnectivityState.online) {
        drainQueue();  // fire-and-forget
      }
    });
  }
  
  Future<void> drainQueue() async { /* ... §4.9.1 ... */ }
}
```

### 7.6 WebSocket 状態管理パターン (UAT 新規)

```dart
// WebSocketService: 状態は sealed PushState で表現
@Riverpod(keepAlive: true)
class WebSocketService extends _$WebSocketService {
  @override
  PushState build() => const PushDisconnected();
  
  // 状態変化を listen して UI 反映
}

// UI 側
class WsStatusIndicator extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(webSocketServiceProvider);
    return switch (state) {
      PushConnected() => const Icon(Icons.cloud_done, color: Colors.green),
      PushDisconnected() => const Icon(Icons.cloud_off, color: Colors.grey),
      PushReconnecting() => const Icon(Icons.sync, color: Colors.orange),
      PushError() => const Icon(Icons.error, color: Colors.red),
      PushConnecting() => const Icon(Icons.sync, color: Colors.blue),
    };
  }
}
```

### 7.7 Conflict Resolution UI パターン (UAT 新規)

```dart
class ConflictResolutionScreen extends ConsumerWidget {
  const ConflictResolutionScreen({required this.reportId, super.key});
  final String reportId;
  
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final report = ref.watch(conflictReportProvider(reportId));
    return report.when(
      loading: () => const LoadingState(),
      error: (e, st) => ErrorState(message: 'Failed to load conflict'),
      data: (r) {
        final serverVersion = jsonDecode(r.serverVersionJson) as Map<String, dynamic>;
        final localVersion = jsonDecode(r.localVersionJson) as Map<String, dynamic>;
        final conflictedFields = (jsonDecode(r.conflictedFieldsJson) as List).cast<String>();
        
        return Scaffold(
          appBar: AppBar(title: const Text('競合解決')),
          body: Row(
            children: [
              Expanded(child: _VersionColumn(title: 'サーバ版', version: serverVersion, fields: conflictedFields)),
              Expanded(child: _VersionColumn(title: 'ローカル版', version: localVersion, fields: conflictedFields)),
            ],
          ),
          bottomNavigationBar: Row(
            children: [
              TextButton(
                onPressed: () => ref.read(conflictResolverProvider.notifier).resolve(
                  reportId: reportId, resolution: 'server',
                ),
                child: const Text('サーバ版を採用'),
              ),
              TextButton(
                onPressed: () => ref.read(conflictResolverProvider.notifier).resolve(
                  reportId: reportId, resolution: 'local',
                ),
                child: const Text('ローカル版を採用'),
              ),
            ],
          ),
        );
      },
    );
  }
}
```

---

## §8 ビルド / デプロイ詳細

### 8.1 pubspec.yaml 完全形 (UAT 拡張)

```yaml
name: star_mobile
description: Star プラットフォーム Android モバイルクライアント (UAT)
publish_to: 'none'
version: 1.1.0+2

environment:
  sdk: '>=3.5.0 <4.0.0'
  flutter: '>=3.24.0'

dependencies:
  flutter:
    sdk: flutter
  flutter_riverpod: ^2.5.1
  dio: ^5.7.0
  dio_smart_retry: ^7.0.0
  flutter_secure_storage: ^9.2.2
  go_router: ^14.2.0
  freezed_annotation: ^2.4.4
  json_annotation: ^4.9.0
  intl: ^0.19.0
  timeago: ^3.7.0
  connectivity_plus: ^6.0.3
  cupertino_icons: ^1.0.8
  # UAT 新規
  drift: ^2.20.0
  drift_flutter: ^0.2.0
  sqlcipher_flutter_libs: ^0.6.0
  web_socket_channel: ^3.0.0
  uuid: ^4.5.0
  collection: ^1.18.0
  path_provider: ^2.1.4
  path: ^1.9.0

dev_dependencies:
  flutter_test:
    sdk: flutter
  integration_test:
    sdk: flutter
  mocktail: ^1.0.4
  freezed: ^2.5.7
  build_runner: ^2.4.13
  json_serializable: ^6.8.0
  very_good_analysis: ^6.0.0
  drift_dev: ^2.20.0

flutter:
  uses-material-design: true
  assets:
    - assets/images/
```

### 8.2 android/app/build.gradle.kts 抜粋 (UAT 拡張: NDK 追加)

```kotlin
android {
    namespace = "com.star.mobile"
    compileSdk = 34
    ndkVersion = "25.1.8937393"  // SQLCipher 用
    // ... (v1.0 と同じ)
}

dependencies {
    // SQLCipher ネイティブライブラリ
    implementation("net.zetetic:sqlcipher-android:4.5.4")
    implementation("androidx.sqlite:sqlite:2.4.0")
}
```

### 8.3 ビルドコマンド (UAT 拡張)

```bash
flutter build apk --release \
  --dart-define=STAR_HOST=http://star.internal:8080 \
  --dart-define=WSS_HOST=wss://star.internal:8080 \
  --dart-define=API_VERSION=v1.1 \
  --dart-define=DB_KEY_ID=internal_v1 \
  --obfuscate \
  --split-debug-info=build/symbols/
```

### 8.4 配布フロー (v1.0 継承 + UAT 拡張)

`02-basic-design.md` v1.1 §11.5 と同じ。

---

## §9 テスト戦略（UAT 拡張）

### 9.1 テストレベル（v1.0 継承 + 拡張）

| レベル | 範囲 | ツール | カバレッジ目標 |
|---|---|---|---|
| 単体テスト (Unit) | Repository, Controller, Interceptor, freezed model, **SyncQueueService, ConflictResolver, WebSocketService, OfflineDatabase** | `flutter test` + `mocktail` | ≥ 70% |
| ウィジェットテスト (Widget) | 6 + **2 (ConflictResolutionScreen, SyncBanner)** | `flutter test` | **8 画面 100%** |
| 統合テスト (Integration) | **オフライン編集 / 競合解決 / WS 切断シナリオ** | `integration_test` | **5 シナリオ** |

### 9.2 主要テストケース（UAT 拡張）

#### 9.2.1 単体テスト (UAT 追加)

| ファイル | テスト |
|---|---|
| `core/ws/websocket_service_test.dart` | 接続成功 / 401 切断 / 指数バックオフ reconnect / ping/pong / メッセージ解析 / 不正メッセージ無視 |
| `core/sync/sync_queue_service_test.dart` | enqueue / drainQueue / 競合時 ConflictResolver 委譲 / retry 上限 drop |
| `core/sync/conflict_resolver_test.dart` | createReport / resolve(server/local/merge) / UI 通知 |
| `core/db/offline_database_test.dart` | upsert / get / delete / migration / SQLCipher 暗号化確認 |
| `features/work_item/work_item_controller_test.dart` | build (Drift + REST マージ) / patchFields (Optimistic + SyncQueue) / onWorkItemUpdated (WS) |
| `features/comments/comments_controller_test.dart` | postComment / onCommentAdded |
| `features/transitions/transitions_controller_test.dart` | execute / 状態反映 |
| `core/utils/log_redactor_test.dart` | token / password / PII redact |

#### 9.2.2 ウィジェットテスト (UAT 追加)

| ファイル | テスト |
|---|---|
| `features/sync_conflicts/conflict_resolution_screen_test.dart` | サーバ版 / ローカル版 採用ボタン / 競合表示 |
| `shared/widgets/sync_banner_test.dart` | オフライン / 同期中 / 競合待ち 状態表示 |

#### 9.2.3 統合テスト (UAT 追加)

| ファイル | シナリオ |
|---|---|
| `integration_test/offline_edit_test.dart` | 1. ログイン → /board<br>2. ネットワーク OFF シミュレーション<br>3. Work Item priority 変更 → Optimistic UI<br>4. SyncQueue に enqueue 確認<br>5. ネットワーク ON シミュレーション<br>6. drainQueue 実行 → REST 送信<br>7. 成功確認 + 未同期バッジ消える |
| `integration_test/conflict_test.dart` | 1. ログイン → Work Item 詳細<br>2. ローカルで priority 変更<br>3. 別経路でサーバ側 priority 変更 (mock)<br>4. 同期 → 409 Conflict 受信<br>5. ConflictResolver 起動 → UI 表示<br>6. 「ローカル版を採用」タップ<br>7. 再送 → 成功確認 |
| `integration_test/ws_reconnect_test.dart` | 1. ログイン → WS 接続確立<br>2. WS 切断シミュレーション<br>3. PushReconnecting 状態確認<br>4. 指数バックオフ 再接続<br>5. PushConnected 復帰<br>6. 再 subscribe 確認 |
| `integration_test/offline_to_online_test.dart` | 1. オフラインで 3 件 編集 / コメント / 状態遷移<br>2. 接続回復<br>3. 順次同期<br>4. 全て成功確認 |
| `integration_test/security_logout_test.dart` | 1. ログイン → オフライン編集 enqueue 残<br>2. ログアウト<br>3. 確認ダイアログ「破棄」<br>4. 全データ削除確認 |

### 9.3 CI 統合（v1.0 継承 + UAT 拡張）

```bash
# 必須 CI ステップ
flutter pub get
dart run build_runner build --delete-conflicting-outputs  # Drift/Freezed
flutter analyze --fatal-infos
dart format --set-exit-if-changed lib test
flutter test --coverage
flutter build apk --release

# 統合テスト (手動 / デバイスファーム)
flutter test integration_test/
```

すべて exit 0 必須。

---

## §10 既知の未解決事項

`01-requirements.md` v1.1 §13 と同じ G-01〜G-25。本書で詳細化した項目:
- G-16: WS サブスクリプション resource_types backend 実装確認
- G-17: `idempotency_keys` サーバ側テーブル実装
- G-18: WS 接続管理 backend 実装
- G-19: バッチ同期エンドポイント要否
- G-20: 競合解決戦略（デフォルト = ユーザー選択 UI 提示）
- G-21: オフラインキャッシュ TTL（デフォルト 7 日）
- G-22: SQLCipher 鍵管理（デフォルト = Keystore 連携、ローテーションなし）
- G-23: WS reconnect 最大リトライ回数（デフォルト 5）
- G-24: 同期キュー上限（デフォルト 100 件）
- G-25: ログ送信サイズ上限（デフォルト 5MB）

---

## §11 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v1.0 | 2026-09-02 16:14 JST | 架構師 (Mavis 接手 agent per DEC-008) | IPA 標準初版: read-only MVP 範囲 | v1.0 と同じ (read-only 範囲) |
| **v1.1** | 2026-09-02 16:27 JST | 架構師 (Mavis 接手 agent per DEC-008) | **UAT 全面拡張**: §3 依存追加 (drift / sqlcipher / web_socket_channel / uuid / collection / path_provider), §4 6 新規モジュール完全コード (WebSocketService / PushEventRouter / OfflineDatabase + 7 Drift テーブル / SyncQueueService / ConflictResolver / ConnectivityWatcher / WorkItemWriteService / CommentsController / TransitionsController / SyncStatusController / SyncBanner / LogRedactor), §5 3 シーケンス, §6 4 JSON スキーマ + Drift 7 テーブル DDL 完全形, §7 4 状態管理パターン, §8 pubspec.yaml + build.gradle.kts NDK 追加, §9 テスト 5 統合テストシナリオ追加 | 2026-09-02 16:27 JST Ulysses 拍板 UAT 範囲 + 自建 WS 推送 (questionnaire 答: full_uat + self_ws) |
| **v1.2** | 2026-09-02 16:54 JST | 架構師 (Mavis 接手 agent per DEC-008) | **§13 Implementation Feasibility 増補**: v1.1 §4 で定義した 12 新規 Dart モジュールのうち client のみで実装可能な 7 個 (OfflineDatabase / SyncQueueService / ConflictResolver / ConnectivityWatcher / SyncStatusController / SyncBanner / LogRedactor) と backend 待ち 5 個 (WebSocketService / PushEventRouter / WorkItemWriteService / CommentsController / TransitionsController) を明示;Phase A (client 先行, mock) + Phase B (P2 backend 完了後, 統合) 推奨;詳細 FR 別監査は `01-requirements.md` v1.2 §16 参照 | 2026-09-02 16:40 JST Ulysses「app 的设计要确保能使用当前系统内已经写好的功能」発令に対応; per 守門 #1+#8+#12 で `crates/star-mcp/src/tools/*.rs` + `crates/star-api-rest/src/routes/*.rs` + `crates/star-sse/src/lib.rs` を git 実証 (commit `9c46a1c` / `c8f6dc7` / `d71b63f`) |

---

## §13 Implementation Feasibility (per 2026-09-02 16:54 JST 増補)

> **FR / エンドポイント レベルの詳細監査は `01-requirements.md` v1.2 §16 + `02-basic-design.md` v1.2 §15 参照**。本書では v1.1 §4 で定義した Dart モジュール 12 個を **client のみで実装可 / backend 待ち** の 2 区分に分類する。

### 13.1 モジュール実装可否マトリクス

| モジュール | 配置 | 状態 | Phase | 理由 |
|---|---|---|---|---|
| **OfflineDatabase** (Drift 7 テーブル + SQLCipher) | `lib/core/db/` | ✅ **client のみ** | **Phase A** | 純 client 完結、backend 依存なし |
| **SyncQueueService** | `lib/core/sync/` | ✅ **client のみ** | **Phase A** | FIFO キュー + REST 呼び出し、HTTP 失敗時 retry |
| **ConflictResolver** | `lib/core/sync/` | ✅ **client のみ** | **Phase A** | 409 Conflict 受信後の UI 提示 + ユーザー選択 |
| **ConnectivityWatcher** | `lib/core/connectivity/` | ✅ **client のみ** | **Phase A** | connectivity_plus ラッパー |
| **SyncStatusController** | `lib/features/settings/` | ✅ **client のみ** | **Phase A** | SyncState 管理 + 通知 |
| **SyncBanner** | `lib/shared/widgets/` | ✅ **client のみ** | **Phase A** | 画面下部バナー UI |
| **LogRedactor** | `lib/core/utils/` | ✅ **client のみ** | **Phase A** | regex + 構造化 redact |
| **WebSocketService** | `lib/core/ws/` | 🟠 **backend 待ち** | **Phase B** | 接続先 `wss://...` 未実装 |
| **PushEventRouter** | `lib/core/ws/` | 🟠 **backend 待ち** | **Phase B** | WS 受信イベントに依存 |
| **WorkItemWriteService** | `lib/features/work_item/` | 🟠 **backend 待ち** | **Phase B** | `PATCH /v1/work-items/{id}` 501 |
| **CommentsController** (postComment) | `lib/features/comments/` | 🟠 **backend 待ち** | **Phase B** | `POST /comments` 未実装 |
| **TransitionsController** | `lib/features/transitions/` | 🟠 **backend 待ち** | **Phase B** | `POST :transition` 未実装 |

**統計**: 12 モジュール中、Phase A (client のみ) = **7 個 (58%)**、Phase B (backend 待ち) = **5 個 (42%)**。

### 13.2 Phase A 推奨実装順序 (即時着手可)

```
Week 1-2:  OfflineDatabase + Drift 7 テーブル DDL + SQLCipher 鍵管理
Week 3:    SyncQueueService + ConflictResolver + ConnectivityWatcher
Week 4:    SyncBanner + SyncStatusController + LogRedactor
Week 5:    Phase A 統合テスト (mock backend, in-memory)
```

推定 token: ~2.0M (4-5 週 @ 1.2M/週 STAR-OLU-001 基線)

### 13.3 Phase B 実装条件

- `crates/star-api-rest` 22 路由の **業務ロジック実装** (commit `c8f6dc7` 以降の P2 段階)
- `crates/star-mcp` 13 tool stub の **実データ化** (commit `d71b63f` 以降の P2 段階)
- **WebSocket backend 実装** (work_item / notification event type 追加)
- **auth 実装** (login/refresh/logout + JWT middleware)

### 13.4 統合テスト戦略

- **Phase A 期間**: mock backend (in-memory) で offline / sync / 競合解決シナリオをテスト
- **Phase B 着手時**: backend P2 進捗と並走で mock + 実 backend 切替テスト
- **Phase C (P2 完了後)**: E2E 統合テスト + 5 域 Lead 合同 DDD Review

### 13.5 实施リスク評価

| リスク | 影響 | 緩和策 |
|---|---|---|
| P2 backend 完了遅延 | Phase B 着手不可 | Phase A 単独で 27 FR 先行実装、MVP 縮小投入可能 |
| WebSocket 仕様変更 | `WebSocketService` リファクタ | `api-design.md` §4 を参照、§13 で再評価 |
| SQLCipher Android NDK 問題 | Drift 起動失敗 | `build.gradle.kts` NDK 25.1.8937393 固定 (per §8.2) |
| backend 401 / 403 仕様変更 | `AuthInterceptor` リファクタ | 既存 `AuthInterceptor` 抽象化済み (§4.1.2) |
| 5 域 Lead 真人補簽待ち | DDD Review ブロック | per 8/21 JST 5 域独立拍板、Mavis 接手代簽で進行可 |

---

## §12 承認欄

| 角色 | 簽字 | 日付 | 備註 |
|---|---|---|---|
| 架構師 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽 (per 8/27 19:39 JST + 21:59 JST 三次強化授權) |
| SRE Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;5 域独立真实身份 DDD Review 段階で補充 |
| 平台 Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| 評審主持 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| PM | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
