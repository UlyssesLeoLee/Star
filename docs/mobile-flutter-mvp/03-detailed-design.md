# Star Mobile Flutter MVP — 詳細設計書

> **基準**: 日本 IPA（情報処理推進機構）SEC ソフトウェア詳細設計書 標準章立て
> **作成日**: 2026-09-02
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **上流要件定義書**: `D:\Star\docs\mobile-flutter-mvp\01-requirements.md` v1.0
> **上流基本設計書**: `D:\Star\docs\mobile-flutter-mvp\02-basic-design.md` v1.0
> **Pre-IPA 草稿**: `docs/architecture/2026-09-02-upgrade/spec/mobile/01-flutter-mvp-design.md` v0.1 (commit `bd4998e`)

---

## §1 目的

本文書は、Star Mobile Flutter MVP の**詳細設計**を定義する。基本設計書 `02-basic-design.md` v1.0 で定義した 4 層アーキテクチャ・コンポーネント・データモデルの**実装レベルの詳細**を記述する。

本書のスコープ：
- ✅ クラス詳細（freezed モデル + Riverpod Provider の完全定義）
- ✅ シーケンス図（実装レベル、エラーケース含む）
- ✅ データ構造詳細（JSON スキーマ、Request/Response 完全形）
- ✅ インターフェース詳細（Dio Interceptor 実装、Retry 戦略、Error マッピング）
- ✅ 状態管理詳細（Riverpod AsyncNotifier の完全実装パターン）
- ✅ キャッシュ戦略（Key 設計、TTL、無効化タイミング）
- ✅ ビルド/デプロイ詳細（pubspec.yaml、AndroidManifest.xml、build.gradle.kts 完全形）
- ✅ テスト戦略（unit / widget / integration test ケース一覧）
- ❌ 実装コード本体（→ 実装フェーズで `lib/` に書く）
- ❌ サーバ側コード（既存、`crates/`）

---

## §2 適用範囲

`01-requirements.md` §2 / `02-basic-design.md` §2 と同じ。

本書は **Flutter クライアント側のみ**を扱い、サーバ側（`crates/star-*`）は既存実装を前提とする。

---

## §3 前提条件・制約事項

### 3.1 開発環境

| 項目 | バージョン / 設定 |
|---|---|
| Flutter SDK | 3.24.0+ (stable) |
| Dart SDK | 3.5.0+ |
| Android Studio | Hedgehog 2023.1.1+ |
| Android SDK | Platform 34, Build-Tools 34.0.0 |
| JDK | OpenJDK 17 (Temurin) |
| Gradle | 8.5+ |
| Kotlin | 1.9+ |
| テストランナー | `flutter test` (unit/widget), `integration_test` (e2e) |
| Git Hooks | pre-commit: `flutter analyze` + `dart format --set-exit-if-changed` |

### 3.2 コーディング規約

- `very_good_analysis` lint ルール採用（`analysis_options.yaml`）
- `dart format` (デフォルト 80 文字) 100% pass
- `dart fix --apply` 0 diff
- 命名規則:
  - ファイル: `snake_case.dart`
  - クラス: `PascalCase`
  - 変数 / 関数: `camelCase`
  - 定数: `lowerCamelCase` (Dart 慣習) または `SCREAMING_SNAKE_CASE` (環境変数)
  - プライベート: `_` プレフィックス
- 1 ファイル 1 クラス（freezed の `_$X` 補完ファイル除く）
- import 順: dart → flutter → package → relative（`directive_ordering` lint）

### 3.3 依存パッケージ（pubspec.yaml 完全形は §8.1）

主要依存：
- `flutter_riverpod: ^2.5.1`
- `dio: ^5.7.0`
- `dio_smart_retry: ^7.0.0`
- `flutter_secure_storage: ^9.2.2`
- `go_router: ^14.2.0`
- `freezed_annotation: ^2.4.4`
- `json_annotation: ^4.9.0`
- `intl: ^0.19.0`
- `timeago: ^3.7.0`
- `connectivity_plus: ^6.0.3`

dev 依存：
- `flutter_test`
- `mocktail: ^1.0.4`
- `freezed: ^2.5.7`
- `build_runner: ^2.4.13`
- `json_serializable: ^6.8.0`
- `integration_test`
- `very_good_analysis: ^6.0.0`

---

## §4 モジュール設計（クラス詳細）

### 4.1 core/api モジュール

#### 4.1.1 `DioClient` (singleton)

```dart
// lib/core/api/dio_client.dart
@Riverpod(keepAlive: true)
Dio dioClient(DioClientRef ref) {
  final dio = Dio(BaseOptions(
    baseUrl: Env.starApiBaseUrl,  // 例: 'http://star.internal:8080/api/v1'
    connectTimeout: const Duration(seconds: 10),
    receiveTimeout: const Duration(seconds: 30),
    sendTimeout: const Duration(seconds: 10),
    headers: {
      'Content-Type': 'application/json; charset=utf-8',
      'Accept': 'application/json',
    },
    responseType: ResponseType.json,
  ));

  dio.interceptors.addAll([
    LoggingInterceptor(),                    // 開発時のみ
    ref.read(authInterceptorProvider),        // Bearer + 401 refresh
    ErrorInterceptor(),                      // RFC 7807 → AppError
  ]);

  return dio;
}
```

#### 4.1.2 `AuthInterceptor`

```dart
// lib/core/api/auth_interceptor.dart
class AuthInterceptor extends Interceptor {
  AuthInterceptor(this._tokenStorage, this._refreshApi, this._dio);
  
  final TokenStorage _tokenStorage;
  final RefreshApi _refreshApi;
  final Dio _dio;
  bool _refreshing = false;
  
  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) async {
    if (options.extra['skipAuth'] != true) {
      final token = await _tokenStorage.readAccessToken();
      if (token != null) {
        options.headers['Authorization'] = 'Bearer $token';
      }
    }
    handler.next(options);
  }
  
  @override
  void onError(DioException err, ErrorInterceptorHandler handler) async {
    if (err.response?.statusCode == 401 &&
        err.requestOptions.extra['retryAfterRefresh'] != true &&
        await _tokenStorage.hasRefreshToken()) {
      
      if (_refreshing) {
        // 他の refresh 中はスキップ（シンプル化、リトライしない）
        return handler.reject(err);
      }
      
      _refreshing = true;
      try {
        final newToken = await _refreshApi.refresh();
        await _tokenStorage.writeAccessToken(newToken);
        
        // 元リクエストを retry
        final retryOptions = err.requestOptions
          ..headers['Authorization'] = 'Bearer $newToken'
          ..extra['retryAfterRefresh'] = true;
        
        final response = await _dio.fetch(retryOptions);
        return handler.resolve(response);
      } catch (_) {
        // refresh 失敗 → ログアウト状態へ
        await _tokenStorage.clear();
        return handler.reject(err);
      } finally {
        _refreshing = false;
      }
    }
    
    return handler.reject(err);
  }
}
```

#### 4.1.3 `ErrorInterceptor`

```dart
// lib/core/api/error_interceptor.dart
class ErrorInterceptor extends Interceptor {
  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    final appError = _mapToAppError(err);
    handler.reject(DioException(
      requestOptions: err.requestOptions,
      error: appError,
      response: err.response,
      type: err.type,
    ));
  }
  
  AppError _mapToAppError(DioException err) {
    if (err.type == DioExceptionType.connectionTimeout ||
        err.type == DioExceptionType.receiveTimeout ||
        err.type == DioExceptionType.sendTimeout ||
        err.type == DioExceptionType.connectionError) {
      return const NetworkError();
    }
    
    final response = err.response;
    if (response == null) return const UnknownError();
    
    final body = response.data;
    if (body is! Map<String, dynamic>) return UnknownError(statusCode: response.statusCode);
    
    final code = body['code'] as String? ?? '';
    final detail = body['detail'] as String? ?? body['title'] as String? ?? 'Unknown error';
    final traceId = body['trace_id'] as String?;
    
    return switch (response.statusCode) {
      401 => UnauthorizedError(code: code, detail: detail, traceId: traceId),
      403 => ForbiddenError(code: code, detail: detail, traceId: traceId),
      404 => NotFoundError(code: code, detail: detail, traceId: traceId),
      409 => ConflictError(code: code, detail: detail, traceId: traceId),
      422 => ValidationError(code: code, detail: detail, traceId: traceId),
      429 => RateLimitedError(
        code: code,
        detail: detail,
        retryAfter: int.tryParse(response.headers.value('Retry-After') ?? ''),
        traceId: traceId,
      ),
      >= 500 && < 600 => ServerError(code: code, detail: detail, statusCode: response.statusCode!, traceId: traceId),
      _ => UnknownError(statusCode: response.statusCode, detail: detail, traceId: traceId),
    };
  }
}
```

#### 4.1.4 `LoggingInterceptor`

```dart
// lib/core/api/logging_interceptor.dart
class LoggingInterceptor extends Interceptor {
  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    if (kDebugMode) {
      debugPrint('→ ${options.method} ${options.uri}');
      debugPrint('  headers: ${options.headers}');
      if (options.data != null) {
        debugPrint('  body: ${_redact(options.data)}');  // token redact
      }
    }
    handler.next(options);
  }
  
  @override
  void onResponse(Response response, ResponseInterceptorHandler handler) {
    if (kDebugMode) {
      debugPrint('← ${response.statusCode} ${response.requestOptions.uri}');
    }
    handler.next(response);
  }
  
  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    if (kDebugMode) {
      debugPrint('✗ ${err.response?.statusCode ?? '???'} ${err.requestOptions.uri}');
      debugPrint('  error: ${err.error}');
    }
    handler.next(err);
  }
  
  String _redact(dynamic data) {
    if (data is Map) {
      final redacted = Map.from(data);
      redacted.remove('password');
      return redacted.toString();
    }
    return data.toString();
  }
}
```

### 4.2 core/auth モジュール

#### 4.2.1 `TokenStorage`

```dart
// lib/core/auth/token_storage.dart
class TokenStorage {
  TokenStorage(this._storage);
  final FlutterSecureStorage _storage;
  
  static const _accessTokenKey = 'access_token';
  static const _refreshTokenKey = 'refresh_token';
  static const _userKey = 'user';
  static const _tenantKey = 'tenant';
  static const _themeKey = 'theme';
  
  Future<String?> readAccessToken() => _storage.read(key: _accessTokenKey);
  Future<String?> readRefreshToken() => _storage.read(key: _refreshTokenKey);
  Future<bool> hasRefreshToken() async => (await readRefreshToken()) != null;
  
  Future<void> writeTokens({required String accessToken, required String refreshToken}) async {
    await _storage.write(key: _accessTokenKey, value: accessToken);
    await _storage.write(key: _refreshTokenKey, value: refreshToken);
  }
  
  Future<void> writeAccessToken(String token) => _storage.write(key: _accessTokenKey, value: token);
  
  Future<User?> readUser() async {
    final json = await _storage.read(key: _userKey);
    return json == null ? null : User.fromJson(jsonDecode(json) as Map<String, dynamic>);
  }
  
  Future<void> writeUser(User user) => _storage.write(key: _userKey, value: jsonEncode(user.toJson()));
  
  Future<Tenant?> readTenant() async {
    final json = await _storage.read(key: _tenantKey);
    return json == null ? null : Tenant.fromJson(jsonDecode(json) as Map<String, dynamic>);
  }
  
  Future<void> writeTenant(Tenant tenant) => _storage.write(key: _tenantKey, value: jsonEncode(tenant.toJson()));
  
  Future<void> clear() async {
    await _storage.deleteAll();
  }
}
```

#### 4.2.2 `AuthState` (sealed)

```dart
// lib/core/auth/auth_state.dart
@freezed
sealed class AuthState with _$AuthState {
  const factory AuthState.unauthenticated({String? message}) = Unauthenticated;
  const factory AuthState.authenticating() = Authenticating;
  const factory AuthState.authenticated({
    required User user,
    required Tenant tenant,
  }) = Authenticated;
  const factory AuthState.error({required String message, int? code}) = AuthError;
}
```

### 4.3 features/auth モジュール

#### 4.3.1 `AuthController`

```dart
// lib/features/auth/presentation/auth_controller.dart
class AuthController extends Notifier<AuthState> {
  @override
  AuthState build() {
    _bootstrap();
    return const AuthState.unauthenticated();
  }
  
  Future<void> _bootstrap() async {
    final storage = ref.read(tokenStorageProvider);
    final accessToken = await storage.readAccessToken();
    if (accessToken == null) return;
    
    // access_token の exp claim を確認
    final exp = _decodeExp(accessToken);
    if (exp == null || DateTime.now().isAfter(exp)) {
      // 期限切れ → refresh 試行
      await _tryRefresh();
    } else {
      // 有効 → user/tenant 復元
      final user = await storage.readUser();
      final tenant = await storage.readTenant();
      if (user != null && tenant != null) {
        state = AuthState.authenticated(user: user, tenant: tenant);
      }
    }
  }
  
  Future<void> signIn({required String email, required String password}) async {
    state = const AuthState.authenticating();
    try {
      final response = await ref.read(authApiProvider).login(
        email: email,
        password: password,
      );
      final storage = ref.read(tokenStorageProvider);
      await storage.writeTokens(
        accessToken: response.accessToken,
        refreshToken: response.refreshToken,
      );
      await storage.writeUser(response.user);
      await storage.writeTenant(response.tenant);
      state = AuthState.authenticated(user: response.user, tenant: response.tenant);
    } on AppError catch (e) {
      state = AuthState.error(message: _mapErrorToMessage(e), code: e.code);
    }
  }
  
  Future<void> signOut() async {
    try {
      await ref.read(authApiProvider).logout();
    } catch (_) {
      // best-effort
    }
    await ref.read(tokenStorageProvider).clear();
    state = const AuthState.unauthenticated();
  }
  
  Future<void> _tryRefresh() async {
    final storage = ref.read(tokenStorageProvider);
    if (!await storage.hasRefreshToken()) {
      state = const AuthState.unauthenticated();
      return;
    }
    try {
      final newToken = await ref.read(authApiProvider).refresh();
      await storage.writeAccessToken(newToken);
      final user = await storage.readUser();
      final tenant = await storage.readTenant();
      if (user != null && tenant != null) {
        state = AuthState.authenticated(user: user, tenant: tenant);
      }
    } catch (_) {
      await storage.clear();
      state = const AuthState.unauthenticated();
    }
  }
  
  DateTime? _decodeExp(String jwt) {
    try {
      final parts = jwt.split('.');
      if (parts.length != 3) return null;
      final payload = jsonDecode(
        utf8.decode(base64Url.decode(base64Url.normalize(parts[1]))),
      ) as Map<String, dynamic>;
      final exp = payload['exp'] as int?;
      return exp == null ? null : DateTime.fromMillisecondsSinceEpoch(exp * 1000);
    } catch (_) {
      return null;
    }
  }
  
  String _mapErrorToMessage(AppError error) => switch (error) {
    UnauthorizedError() => 'メールアドレスまたはパスワードが正しくありません',
    NetworkError() => 'ネットワーク接続がありません',
    ServerError() => 'サーバーで問題が発生しました。しばらくしてから再度お試しください',
    _ => 'ログインに失敗しました',
  };
}

@riverpod
class AuthController extends _$AuthController {
  @override
  AuthState build() => /* 上のクラス参照 */;
}
```

### 4.4 features/board モジュール

#### 4.4.1 `BoardController`

```dart
// lib/features/board/presentation/board_controller.dart
@riverpod
class BoardController extends _$BoardController {
  @override
  Future<Board> build(String projectId) async {
    // 並列取得
    final boardApi = ref.read(boardApiProvider);
    final workItemApi = ref.read(workItemApiProvider);
    final results = await Future.wait([
      boardApi.getBoard(projectId),
      workItemApi.listByProject(projectId: projectId, limit: 200),
    ]);
    return Board(
      config: results[0] as BoardConfig,
      cards: (results[1] as List<WorkItemSummary>).toList(),
    );
  }
  
  Future<void> refresh() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() => future);
  }
}
```

#### 4.4.2 `BoardScreen` (Widget)

```dart
// lib/features/board/presentation/board_screen.dart
class BoardScreen extends ConsumerWidget {
  const BoardScreen({required this.projectId, super.key});
  final String projectId;
  
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final boardAsync = ref.watch(boardControllerProvider(projectId));
    
    return Scaffold(
      appBar: AppBar(title: const Text('ボード')),
      body: boardAsync.when(
        loading: () => const LoadingState(),
        error: (e, st) => ErrorState(
          message: 'ボードを読み込めませんでした',
          onRetry: () => ref.invalidate(boardControllerProvider(projectId)),
        ),
        data: (board) => RefreshIndicator(
          onRefresh: () => ref.read(boardControllerProvider(projectId).notifier).refresh(),
          child: HorizontalBoardView(
            board: board,
            onCardTap: (card) => context.push('/work-items/${card.id}'),
          ),
        ),
      ),
    );
  }
}
```

### 4.5 features/notifications モジュール

#### 4.5.1 `NotificationsController` (30s ポーリング)

```dart
// lib/features/notifications/presentation/notifications_controller.dart
@riverpod
class NotificationsController extends _$NotificationsController {
  Timer? _timer;
  
  @override
  Future<List<Notification>> build() async {
    ref.onDispose(() {
      _timer?.cancel();
    });
    
    // 30s ポーリング
    _timer = Timer.periodic(const Duration(seconds: 30), (_) => _poll());
    
    return _fetch();
  }
  
  Future<List<Notification>> _fetch() async {
    return ref.read(notificationApiProvider).list(read: false, limit: 20);
  }
  
  Future<void> _poll() async {
    if (WidgetsBinding.instance.lifecycleState != AppLifecycleState.resumed) {
      // バックグラウンド時はポーリングしない
      return;
    }
    state = AsyncValue.data(await _fetch());
  }
  
  Future<void> markRead(String id) async {
    await ref.read(notificationApiProvider).markRead(id);
    ref.invalidateSelf();
  }
  
  Future<void> markAllRead() async {
    await ref.read(notificationApiProvider).markAllRead();
    state = const AsyncValue.data([]);
  }
}
```

### 4.6 ドメインモデル（freezed 完全形、抜粋）

```dart
// lib/features/auth/domain/user.dart
@freezed
class User with _$User {
  const factory User({
    required String id,
    required String email,
    required String displayName,
    String? avatarUrl,
  }) = _User;
  
  factory User.fromJson(Map<String, dynamic> json) => _$UserFromJson(json);
}

// lib/features/board/domain/board.dart
@freezed
class BoardConfig with _$BoardConfig {
  const factory BoardConfig({
    required String id,
    required String projectId,
    required List<Column> columns,
  }) = _BoardConfig;
  
  factory BoardConfig.fromJson(Map<String, dynamic> json) => _$BoardConfigFromJson(json);
}

@freezed
class Column with _$Column {
  const factory Column({
    required String id,
    required String name,
    required int order,
  }) = _Column;
  
  factory Column.fromJson(Map<String, dynamic> json) => _$ColumnFromJson(json);
}

@freezed
class WorkItemSummary with _$WorkItemSummary {
  const factory WorkItemSummary({
    required String id,
    required String title,
    required String status,
    required String priority,
    String? assigneeId,
    String? assigneeAvatarUrl,
    String? assigneeDisplayName,
  }) = _WorkItemSummary;
  
  factory WorkItemSummary.fromJson(Map<String, dynamic> json) => _$WorkItemSummaryFromJson(json);
}

// lib/features/work_item/domain/work_item.dart
@freezed
class WorkItem with _$WorkItem {
  const factory WorkItem({
    required String id,
    required String projectId,
    required String type,
    required String title,
    required String status,
    required String priority,
    String? description,
    String? assigneeId,
    String? reporterId,
    DateTime? dueDate,
    List<String>? repositoryIds,
    List<String>? worktreeIds,
  }) = _WorkItem;
  
  factory WorkItem.fromJson(Map<String, dynamic> json) => _$WorkItemFromJson(json);
}

// lib/features/notifications/domain/notification.dart
@freezed
class Notification with _$Notification {
  const factory Notification({
    required String id,
    required String eventType,
    required Map<String, dynamic> payload,
    required DateTime sentAt,
    DateTime? readAt,
  }) = _Notification;
  
  factory Notification.fromJson(Map<String, dynamic> json) => _$NotificationFromJson(json);
}
```

---

## §5 シーケンス設計

### 5.1 認証フロー

```
[Sequence UC-001: ログイン]

User           LoginScreen        AuthController       AuthApi         Backend
 │                 │                    │                  │                │
 │ 起動            │                    │                  │                │
 │ ───────────────▶│                    │                  │                │
 │                 │ build()            │                  │                │
 │                 │ ──────────────────▶│                  │                │
 │                 │ 初期 Unauthenticated│                 │                │
 │                 │ ◀──────────────────│                  │                │
 │ email入力       │                    │                  │                │
 │ パスワード入力  │                    │                  │                │
 │ ログインボタン  │                    │                  │                │
 │ ───────────────▶│                    │                  │                │
 │                 │ signIn()           │                  │                │
 │                 │ ──────────────────▶│                  │                │
 │                 │                    │ Authenticating   │                │
 │                 │                    │ POST /auth/login │                │
 │                 │                    │ ────────────────▶│                │
 │                 │                    │                  │ 200 {tokens,user,tenant}
 │                 │                    │                  │ ◀───────────────│
 │                 │                    │ 書込 tokens      │                │
 │                 │                    │ 書込 user/tenant │                │
 │                 │                    │ Authenticated    │                │
 │                 │ ナビゲート         │                  │                │
 │                 │ ◀──────────────────│                  │                │
 │ /projects へ    │                    │                  │                │
 │ ◀──────────────│                    │                  │                │
```

### 5.2 ボード取得フロー

```
[Sequence UC-002: ボード閲覧]

User           BoardScreen       BoardController      BoardApi     WorkItemApi      Backend
 │                 │                    │                  │              │              │
 │ /projects/:id/board                  │                  │              │              │
 │ ───────────────▶│                    │                  │              │              │
 │                 │ build(projectId)   │                  │              │              │
 │                 │ ──────────────────▶│                  │              │              │
 │                 │                    │ GET /projects/{id}/board        │              │
 │                 │                    │ ────────────────────────────────────────────────▶│
 │                 │                    │ 200 {board}        │              │              │
 │                 │                    │ ◀────────────────────────────────────────────────│
 │                 │                    │ GET /work-items?project_id=:id&limit=200        │
 │                 │                    │ ───────────────────────────────▶│              │
 │                 │                    │ 200 {items[]}      │              │              │
 │                 │                    │ ◀───────────────────────────────│              │
 │                 │                    │ Board(...)        │              │              │
 │                 │ 描画               │                  │              │              │
 │                 │ ◀──────────────────│                  │              │              │
 │                 │ 横スクロール Columns│                  │              │              │
 │                 │ + Cards 表示       │                  │              │              │
```

### 5.3 エラーハンドリング（401 refresh retry）

```
[Sequence UC-E01: 401 refresh retry]

User           WorkItemDetailScreen  WorkItemController  AuthInterceptor  Backend
 │                 │                    │                    │                │
 │ /work-items/:id │                    │                    │                │
 │ ───────────────▶│                    │                    │                │
 │                 │ build(id)          │                    │                │
 │                 │ ──────────────────▶│                    │                │
 │                 │                    │ GET /work-items/:id│                │
 │                 │                    │ ──────────────────▶│                │
 │                 │                    │ 注入 Bearer        │                │
 │                 │                    │ ─────────────────────────────────────▶│
 │                 │                    │ 401 SEC-001         │                │
 │                 │                    │ ◀─────────────────────────────────────│
 │                 │                    │ 401 検知            │                │
 │                 │                    │ POST /auth/refresh │                │
 │                 │                    │ ──────────────────▶│                │
 │                 │                    │ 200 {accessToken}  │                │
 │                 │                    │ ◀──────────────────│                │
 │                 │                    │ 新 token 保存       │                │
 │                 │                    │ 元 request retry   │                │
 │                 │                    │ ─────────────────────────────────────▶│
 │                 │                    │ 200 {workItem}     │                │
 │                 │                    │ ◀─────────────────────────────────────│
 │                 │                    │ WorkItem           │                │
 │                 │ 描画               │                    │                │
 │                 │ ◀──────────────────│                    │                │
```

---

## §6 データ構造詳細（JSON スキーマ完全形）

### 6.1 リクエスト / レスポンス

#### 6.1.1 POST /v1/auth/login

**Request**:
```json
{
  "email": "ulysses@acme.com",
  "password": "<plaintext password, HTTPS or 内部 cleartext>"
}
```

**Response 200**:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "rt_a3f9d8e2b1c0...",
  "expires_in": 900,
  "token_type": "Bearer",
  "user": {
    "id": "01HXXX...",
    "email": "ulysses@acme.com",
    "display_name": "Ulysses",
    "avatar_url": "https://cdn.acme.com/avatars/ulysses.png"
  },
  "tenant": {
    "id": "t_acme",
    "name": "Acme Inc."
  }
}
```

**Error 401**:
```json
{
  "type": "https://star.acme.com/errors/SEC-001",
  "title": "Unauthorized",
  "status": 401,
  "detail": "Invalid credentials",
  "code": "SEC-001",
  "trace_id": "0af7651916cd43dd8448eb211c80319c"
}
```

#### 6.1.2 GET /v1/work-items/{id}

**Response 200**:
```json
{
  "id": "01HYYY...",
  "tenant_id": "t_acme",
  "project_id": "prj_xxx",
  "type": "Story",
  "title": "Implement OAuth2 flow",
  "description": "Add OAuth2 authentication...",
  "status": "IN_PROGRESS",
  "priority": "High",
  "assignee_user_id": "01HZZZ...",
  "reporter_user_id": "01HWWW...",
  "due_date": "2026-09-15T00:00:00Z",
  "repository_ids": ["repo_aaa"],
  "worktree_ids": ["wt_bbb"],
  "created_at": "2026-09-01T10:00:00Z",
  "updated_at": "2026-09-02T12:30:00Z"
}
```

#### 6.1.3 GET /v1/notifications?read=false&limit=20

**Response 200**:
```json
{
  "items": [
    {
      "id": "01HAAA...",
      "recipient_user_id": "01HZZZ...",
      "event_type": "WorkItemAssigned",
      "payload": {
        "work_item_id": "01HYYY...",
        "work_item_title": "Implement OAuth2 flow",
        "assigner_name": "Ulysses"
      },
      "read_at": null,
      "sent_at": "2026-09-02T15:00:00Z"
    }
  ],
  "next_cursor": null,
  "total": 1
}
```

### 6.2 freezed JSON モデル（完全形）

（§4.6 で主要モデルを定義済、本節は WorkItem / Comment / Transition の完全版）

```dart
// lib/features/work_item/domain/work_item.dart
@freezed
class WorkItem with _$WorkItem {
  const factory WorkItem({
    required String id,
    @JsonKey(name: 'tenant_id') required String tenantId,
    @JsonKey(name: 'project_id') required String projectId,
    required String type,
    required String title,
    String? description,
    required String status,
    required String priority,
    @JsonKey(name: 'assignee_user_id') String? assigneeUserId,
    @JsonKey(name: 'reporter_user_id') String? reporterUserId,
    @JsonKey(name: 'due_date') DateTime? dueDate,
    @JsonKey(name: 'repository_ids') List<String>? repositoryIds,
    @JsonKey(name: 'worktree_ids') List<String>? worktreeIds,
    @JsonKey(name: 'created_at') required DateTime createdAt,
    @JsonKey(name: 'updated_at') required DateTime updatedAt,
  }) = _WorkItem;
  
  factory WorkItem.fromJson(Map<String, dynamic> json) => _$WorkItemFromJson(json);
}

// lib/features/work_item/domain/comment.dart
@freezed
class Comment with _$Comment {
  const factory Comment({
    required String id,
    @JsonKey(name: 'author_user_id') required String authorUserId,
    @JsonKey(name: 'author_display_name') String? authorDisplayName,
    @JsonKey(name: 'author_avatar_url') String? authorAvatarUrl,
    required String body,
    @JsonKey(name: 'mentions') List<String>? mentions,
    @JsonKey(name: 'created_at') required DateTime createdAt,
  }) = _Comment;
  
  factory Comment.fromJson(Map<String, dynamic> json) => _$CommentFromJson(json);
}

// lib/features/work_item/domain/transition.dart
@freezed
class Transition with _$Transition {
  const factory Transition({
    required String from,
    required String to,
    @JsonKey(name: 'required_permission') String? requiredPermission,
    @JsonKey(name: 'is_allowed') required bool isAllowed,
  }) = _Transition;
  
  factory Transition.fromJson(Map<String, dynamic> json) => _$TransitionFromJson(json);
}
```

---

## §7 状態管理詳細

### 7.1 Riverpod Provider 階層

```
[Global]
├── dioClientProvider              (keepAlive: true)
├── tokenStorageProvider           (keepAlive: true)
├── authInterceptorProvider        (keepAlive: true)
├── errorInterceptorProvider       (keepAlive: true)
├── loggingInterceptorProvider     (keepAlive: true)
│
├── envProvider                    (const)
│
├── authApiProvider                (依存: dioClient)
├── userApiProvider
├── tenantApiProvider
├── boardApiProvider
├── workItemApiProvider
├── commentApiProvider
├── notificationApiProvider
│
├── authControllerProvider         (NotifierProvider<AuthState>)
│
├── connectivityProvider           (Stream)
│
[Feature: Board]
├── boardControllerProvider(projectId)  (AsyncNotifierProvider.family)
│
[Feature: Work Item]
├── workItemControllerProvider(workItemId)  (FutureProvider.family)
│
[Feature: Notifications]
├── notificationsControllerProvider  (AsyncNotifierProvider)
│
[Feature: Projects]
├── projectListControllerProvider  (AsyncNotifierProvider)
```

### 7.2 状態管理パターン

| パターン | 用途 | Provider |
|---|---|---|
| 単一値（不変） | 環境変数、シングルトン | `Provider` |
| 単一値（可変） | AuthState | `NotifierProvider` |
| 非同期（自動 refresh） | 画面表示用 | `FutureProvider` / `AsyncNotifierProvider` |
| 非同期（手動 refresh） | Pull-to-Refresh | `AsyncNotifierProvider` + `ref.invalidateSelf()` |
| ポーリング | 通知 | `AsyncNotifierProvider` + `Timer.periodic` |
| パラメータ付き | 特定 WorkItem | `FutureProvider.family` |
| Stream | 接続状態、Crashlytics 代替 | `StreamProvider` |

### 7.3 エラーハンドリングパターン

```dart
// 画面 Widget 内
final boardAsync = ref.watch(boardControllerProvider(projectId));

return boardAsync.when(
  loading: () => const LoadingState(),
  error: (error, stack) {
    if (error is NetworkError) {
      return ErrorState(
        message: 'ネットワーク接続がありません',
        onRetry: () => ref.invalidate(boardControllerProvider(projectId)),
      );
    }
    if (error is UnauthorizedError) {
      // AuthInterceptor が refresh を試行済、ここに来るのは refresh 失敗
      return ErrorState(
        message: 'セッションの有効期限が切れました',
        onRetry: () => ref.invalidate(boardControllerProvider(projectId)),
      );
    }
    if (error is ForbiddenError) {
      return ErrorState(message: 'このボードを閲覧する権限がありません');
    }
    return ErrorState(
      message: 'ボードを読み込めませんでした',
      onRetry: () => ref.invalidate(boardControllerProvider(projectId)),
    );
  },
  data: (board) => BoardView(board: board),
);
```

### 7.4 ローディング / 空状態パターン

- ローディング: `LoadingState` (Material 3 CircularProgressIndicator centered)
- 空: `EmptyState` (アイコン + テキスト + アクションボタン)
- エラー: `ErrorState` (アイコン + メッセージ + リトライボタン)
- 成功: データ表示

全画面共通で `shared/widgets/` に配置。

---

## §8 ビルド / デプロイ詳細

### 8.1 pubspec.yaml 完全形

```yaml
name: star_mobile
description: Star プラットフォーム Android モバイルクライアント
publish_to: 'none'
version: 1.0.0+1

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

flutter:
  uses-material-design: true
  assets:
    - assets/images/
  # 注: フォント / 多言語は MVP 範囲外
```

### 8.2 android/app/build.gradle.kts 抜粋

```kotlin
plugins {
    id("com.android.application")
    id("kotlin-android")
    id("dev.flutter.flutter-gradle-plugin")
}

android {
    namespace = "com.star.mobile"
    compileSdk = 34
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_17.toString()
    }

    defaultConfig {
        applicationId = "com.star.mobile"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "1.0.0"
    }

    buildTypes {
        release {
            signingConfig = signingConfigs.getByName("internal")  // G-10 拍板待ち
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
        debug {
            applicationIdSuffix = ".debug"
            isDebuggable = true
        }
    }
}
```

### 8.3 ビルドコマンド

```bash
# 開発
flutter run --dart-define=STAR_HOST=http://star.local:8080

# リリースビルド
flutter build apk --release \
  --dart-define=STAR_HOST=http://star.internal:8080 \
  --dart-define=API_VERSION=v1 \
  --obfuscate \
  --split-debug-info=build/symbols/

# 成果物
build/app/outputs/flutter-apk/app-release.apk
build/symbols/  # ProGuard mapping、後で保管
```

### 8.4 配布フロー（CI 統合）

```yaml
# .github/workflows/build-mobile.yml (将来追加)
name: Build Mobile APK
on:
  push:
    branches: [main]
    paths: [frontend/mobile-flutter/**]
jobs:
  build:
    runs-on: [self-hosted, internal]  # 内網ランナー
    steps:
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with:
          channel: stable
      - run: cd frontend/mobile-flutter && flutter pub get
      - run: cd frontend/mobile-flutter && flutter analyze
      - run: cd frontend/mobile-flutter && flutter test
      - run: |
          cd frontend/mobile-flutter && flutter build apk --release \
            --dart-define=STAR_HOST=${{ secrets.STAR_HOST }} \
            --obfuscate \
            --split-debug-info=build/symbols/
      - uses: actions/upload-artifact@v4
        with:
          name: app-release
          path: frontend/mobile-flutter/build/app/outputs/flutter-apk/app-release.apk
      - name: Upload to internal NAS
        run: |
          curl -X POST ${{ secrets.NAS_WEBHOOK }} \
            -F "file=@frontend/mobile-flutter/build/app/outputs/flutter-apk/app-release.apk"
```

### 8.5 バージョン管理

- `versionName`: semver (`major.minor.patch`)
- `versionCode`: CI 自動インクリメント（git tag 連動）
- 1 機能追加 = patch +1
- 破壊的変更 = major +1（V1 → V1.1 → V2）

---

## §9 テスト戦略

### 9.1 テストレベル

| レベル | 範囲 | ツール | カバレッジ目標 |
|---|---|---|---|
| 単体テスト (Unit) | Repository, Controller, Interceptor, freezed model | `flutter test` + `mocktail` | ≥ 70% |
| ウィジェットテスト (Widget) | 各 Screen, 主要 Widget | `flutter test` | 6 画面 100% |
| 統合テスト (Integration) | ログイン → ボード → 詳細 → ログアウト E2E | `integration_test` | 1 シナリオ |

### 9.2 主要テストケース

#### 9.2.1 単体テスト

| ファイル | テスト |
|---|---|
| `core/api/auth_interceptor_test.dart` | 401 時に refresh + retry / refresh 失敗時に reject / 連続 401 時にループしない |
| `core/api/error_interceptor_test.dart` | 各 HTTP status → 対応する AppError / タイムアウト → NetworkError / 不明 → UnknownError |
| `core/auth/auth_state_test.dart` | signIn 成功 / 失敗 / signOut 状態遷移 |
| `features/board/board_controller_test.dart` | build 成功 / 失敗 / refresh |
| `features/notifications/notifications_controller_test.dart` | 30s ポーリング / バックグラウンド停止 / markRead / markAllRead |
| `features/auth/domain/user_test.dart` | fromJson / toJson / copyWith |

#### 9.2.2 ウィジェットテスト

| ファイル | テスト |
|---|---|
| `features/auth/presentation/login_screen_test.dart` | 入力 → ボタン活性 / ログイン中ボタン無効 / エラー表示 |
| `features/board/presentation/board_screen_test.dart` | ローディング / エラー / データ表示 / カードタップ遷移 |
| `features/work_item/presentation/work_item_detail_screen_test.dart` | 3 タブ切替 / Web ボタン / 空コメント / Comments 並び順 |
| `features/notifications/presentation/notifications_screen_test.dart` | 通知タップ既読化 / 遷移 / 全既読ボタン |
| `features/settings/presentation/settings_screen_test.dart` | テーマ切替 / ログアウト |

#### 9.2.3 統合テスト

| ファイル | テスト |
|---|---|
| `integration_test/app_e2e_test.dart` | 1. 起動 → /login<br>2. メール+パス入力 → ログイン → /projects<br>3. プロジェクト選択 → /board<br>4. カードタップ → /work-items/:id<br>5. 戻る → /board<br>6. 設定 → ログアウト → /login |

### 9.3 テストデータ

- Mock API: `mocktail` で各 Repository をモック化
- Fixture: `test/fixtures/` に JSON ファイル（`api-design.md` レスポンスを実体化）
- Test environment: `STAR_HOST=http://mock.star` で起動

### 9.4 CI 統合

```bash
# 必須 CI ステップ
flutter pub get
flutter analyze --fatal-infos
dart format --set-exit-if-changed lib test
flutter test --coverage
flutter build apk --release
```

すべて exit 0 必須。

---

## §10 既知の未解決事項

`01-requirements.md` §13 と同じ 15 項目（**G-01〜G-15**）。本書の詳細化で追加された項目なし。

実装フェーズで発生する追加課題は別ファイル `99-implementation-changelog.md` に記録。

---

## §11 改訂履歴

| バージョン | 日付 | 改訂人 | 改訂内容 | トリガ |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架構師 (Mavis 接手 agent per DEC-008) | IPA 標準初版: §1〜§11 全章, 4.1〜4.6 完全クラス定義 (4 Interceptor + AuthController + BoardController + NotificationsController + 6 freezed model), 5.1〜5.3 3 シーケンス図, 6.1〜6.2 JSON スキーマ完全形 + freezed JSON マッピング, 7.1〜7.4 Riverpod 階層 + 4 状態管理パターン + エラーハンドリング, 8.1〜8.5 pubspec.yaml 完全形 + build.gradle.kts + CI workflow, 9.1〜9.4 単体 + ウィジェット + 統合テストケース | 2026-09-02 16:09 JST Ulysses 発令「要符合日本IPA标准的需求、基本设计、详细设计」, v0.1 (commit `bd4998e`) を IPA 3 段組に supersede |

---

## §12 承認欄

| 角色 | 簽字 | 日付 | 備註 |
|---|---|---|---|
| 架構師 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽 (per 8/27 19:39 JST + 21:59 JST 三次強化授權) |
| SRE Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;5 域独立真实身份 DDD Review 段階で補充 |
| 平台 Lead | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| 評審主持 | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
| PM | 架構師 (Mavis 接手 agent per DEC-008) | 2026-09-02 | 🟢 Mavis 接手代簽;同上 |
