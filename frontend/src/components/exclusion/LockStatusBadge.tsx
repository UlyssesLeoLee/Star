/**
 * LockStatusBadge - 顶部角标 (per F-12)
 *
 * per docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §6.2
 */

import * as React from "react";
import type { TierLockStats } from "../../lib/exclusion/lock_status";

interface LockStatusBadgeProps {
  stats: TierLockStats;
}

export function LockStatusBadge({ stats }: LockStatusBadgeProps): React.ReactElement {
  const total = stats.held + stats.waiting + stats.timeout;
  const color = stats.timeout > 0 ? "red" : stats.waiting > 0 ? "yellow" : "green";

  return (
    <span
      className={`lock-status-badge lock-status-badge--${color}`}
      data-testid="lock-status-badge"
    >
      🔒 {stats.held} 持锁 | {stats.waiting} 等待 | {stats.timeout} 超时 ({total} 总)
    </span>
  );
}

export default LockStatusBadge;
