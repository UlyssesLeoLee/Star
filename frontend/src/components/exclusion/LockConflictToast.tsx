/**
 * LockConflictToast - 错乱 toast (per F-12 + F-14)
 */

import * as React from "react";

interface LockConflictToastProps {
  taskId: string;
  heldBy: string;
  retryAfterSeconds: number;
  onRetry: () => void;
  onCancel: () => void;
}

export function LockConflictToast({
  taskId,
  heldBy,
  retryAfterSeconds,
  onRetry,
  onCancel,
}: LockConflictToastProps): React.ReactElement {
  return (
    <div className="lock-conflict-toast" data-testid="lock-conflict-toast" role="alert">
      <div className="lock-conflict-toast__title">⚠️ task {taskId} 正在被 {heldBy} 操作</div>
      <div className="lock-conflict-toast__body">
        请稍后重试 (建议 {retryAfterSeconds}s 后) 或联系 {heldBy} 协调
      </div>
      <div className="lock-conflict-toast__actions">
        <button onClick={onRetry} data-testid="lock-conflict-toast-retry">重试</button>
        <button onClick={onCancel} data-testid="lock-conflict-toast-cancel">取消</button>
      </div>
    </div>
  );
}

export default LockConflictToast;
