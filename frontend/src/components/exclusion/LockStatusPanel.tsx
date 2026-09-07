/**
 * LockStatusPanel - 详情面板 (per F-12)
 */

import * as React from "react";
import { LockStatusTracker, type LockTier } from "../../lib/exclusion/lock_status";
import { LockStatusBadge } from "./LockStatusBadge";

interface LockStatusPanelProps {
  tracker: LockStatusTracker;
}

const TIERS: LockTier[] = ["L0_UI", "L1_TopAgent", "L2_SubAgent", "L3_Domain"];

export function LockStatusPanel({ tracker }: LockStatusPanelProps): React.ReactElement {
  return (
    <div className="lock-status-panel" data-testid="lock-status-panel">
      <h3>4 层锁链状态</h3>
      {TIERS.map((tier) => {
        const stats = tracker.getStats(tier);
        if (!stats) return null;
        return (
          <div key={tier} className="lock-status-panel__row">
            <strong>{tier}</strong>
            <LockStatusBadge stats={stats} />
          </div>
        );
      })}
    </div>
  );
}

export default LockStatusPanel;
