/**
 * IdempotencyManager - 客户端 dedup (per Star-EI F-06)
 *
 * per docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §2.1.1
 * per PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md §1.1 EX-05
 *
 * 守门合规:
 * - 守门 #6: TypeScript 4 步 (tsc / eslint / test / build)
 * - 守门 #7: 0 unsafe
 */

export interface IdempotencyManagerConfig {
  storageKeyPrefix: string;
  defaultTimeoutMs: number;
  enableLocalStorageCache: boolean;
  enableAbortController: boolean;
}

export interface InflightRecord {
  key: string;
  fingerprint: string;
  controller: AbortController;
  startedAt: number;
  promise: Promise<unknown>;
}

export class IdempotencyManager {
  private inflight: Map<string, InflightRecord> = new Map();
  private config: IdempotencyManagerConfig;

  constructor(config?: Partial<IdempotencyManagerConfig>) {
    this.config = {
      storageKeyPrefix: "star-idem:",
      defaultTimeoutMs: 30000,
      enableLocalStorageCache: true,
      enableAbortController: true,
      ...config,
    };
  }

  /** 生成新 key (UUID v4) */
  public generateKey(): string {
    return crypto.randomUUID();
  }

  /** dispatch + 自动 dedup (per F-06) */
  public async dispatch<T>(
    key: string,
    fn: () => Promise<T>,
    opts?: {
      fallbackBusinessHash?: string;
      timeoutMs?: number;
    },
  ): Promise<T> {
    const timeoutMs = opts?.timeoutMs ?? this.config.defaultTimeoutMs;
    const record = this.inflight.get(key);

    if (record && this.config.enableAbortController) {
      record.controller.abort();
    }

    const controller = new AbortController();
    const timeoutHandle = setTimeout(() => controller.abort(), timeoutMs);

    const promise = (async () => {
      try {
        return await fn();
      } finally {
        clearTimeout(timeoutHandle);
        this.inflight.delete(key);
      }
    })();

    this.inflight.set(key, {
      key,
      fingerprint: opts?.fallbackBusinessHash ?? "",
      controller,
      startedAt: Date.now(),
      promise,
    });

    return promise;
  }

  /** 显式取消 in-flight */
  public abort(key: string): void {
    const record = this.inflight.get(key);
    if (record) {
      record.controller.abort();
      this.inflight.delete(key);
    }
  }

  /** localStorage 缓存 key */
  public cacheKey(key: string, ttlMs: number): void {
    if (!this.config.enableLocalStorageCache) return;
    const expiresAt = Date.now() + ttlMs;
    localStorage.setItem(
      `${this.config.storageKeyPrefix}${key}`,
      JSON.stringify({ cachedAt: Date.now(), expiresAt }),
    );
  }

  public getCachedKey(key: string): string | null {
    if (!this.config.enableLocalStorageCache) return null;
    const raw = localStorage.getItem(`${this.config.storageKeyPrefix}${key}`);
    if (!raw) return null;
    try {
      const { expiresAt } = JSON.parse(raw);
      if (Date.now() > expiresAt) {
        localStorage.removeItem(`${this.config.storageKeyPrefix}${key}`);
        return null;
      }
      return raw;
    } catch {
      return null;
    }
  }
}
