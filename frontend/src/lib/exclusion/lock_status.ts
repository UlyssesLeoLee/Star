/**
 * LockStatusTracker - 4 层锁状态聚合 (per Star-EI F-12)
 *
 * per docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §6.2
 *
 * 4 层持锁数 / 等待数 / 超时数聚合, 用于 UI 顶部角标
 */

export type LockTier = "L0_UI" | "L1_TopAgent" | "L2_SubAgent" | "L3_Domain";

export interface TierLockStats {
  held: number;
  waiting: number;
  timeout: number;
}

export class LockStatusTracker {
  private stats: Map<LockTier, TierLockStats> = new Map();

  constructor() {
    // 初始化 4 层默认 0
    for (const tier of ["L0_UI", "L1_TopAgent", "L2_SubAgent", "L3_Domain"] as LockTier[]) {
      this.stats.set(tier, { held: 0, waiting: 0, timeout: 0 });
    }
  }

  /** 增加持锁数 (per F-12 + F-13) */
  public incrementHeld(tier: LockTier): void {
    const s = this.stats.get(tier);
    if (s) s.held += 1;
  }

  /** 增加等待数 */
  public incrementWaiting(tier: LockTier): void {
    const s = this.stats.get(tier);
    if (s) s.waiting += 1;
  }

  /** 增加超时数 */
  public incrementTimeout(tier: LockTier): void {
    const s = this.stats.get(tier);
    if (s) s.timeout += 1;
  }

  /** 释放持锁 */
  public decrementHeld(tier: LockTier): void {
    const s = this.stats.get(tier);
    if (s && s.held > 0) s.held -= 1;
  }

  /** 获取某层状态 */
  public getStats(tier: LockTier): TierLockStats | undefined {
    return this.stats.get(tier);
  }

  /** 获取所有层聚合 */
  public getAllStats(): Map<LockTier, TierLockStats> {
    return new Map(this.stats);
  }

  /** 总持锁数 (4 层) */
  public totalHeld(): number {
    let total = 0;
    for (const s of this.stats.values()) total += s.held;
    return total;
  }
}
