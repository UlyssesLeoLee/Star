/*
@cypher schema=1 source_sha256=placeholder
MERGE (self:File {path:"frontend/src/components/run/LaneContextMenu.tsx"})
MERGE (menu:Symbol {id:"frontend/src/components/run/LaneContextMenu.tsx::LaneContextMenu",kind:"component"})
MERGE (self)-[:DEFINES]->(menu)
@endcypher
*/
// LaneContextMenu — Lane 右键菜单组件 (2026-11-27)
// 提供 Lane 删除、运行时切换、Worktree 扫描、Branch 链接等管理功能
// 可通过右键或点击触发，支持独立页面或内联使用
"use client";

import { clsx } from "clsx";
import {
  Trash2,
  Monitor,
  ScanEye,
  GitBranch,
  XCircle,
  ExternalLink,
  Check,
  Loader2,
} from "lucide-react";
import { useState, useEffect, useRef } from "react";
import { canonicalLaneHref } from "@/lib/run/runDirectoryApi";
import { useTranslation } from "@/lib/i18n";

export interface LaneContextAction {
  id: string;
  label: string;
  icon: React.ComponentType<{ size?: number; className?: string }>;
  danger?: boolean;
  href?: string;
  onClick?: () => void;
  disabled?: boolean;
  separatorAfter?: boolean;
}

export interface LaneContextMenuProps {
  projectId: string;
  laneId: string;
  laneName: string;
  isOpen: boolean;
  position: { x: number; y: number };
  onClose: () => void;
  onDelete?: (laneId: string) => void;
  onSwitchRuntime?: (laneId: string) => void;
  onScanWorktrees?: (laneId: string) => void;
  onLinkBranch?: (laneId: string) => void;
  disabled?: boolean;
  className?: string;
}

export function LaneContextMenu({
  projectId,
  laneId,
  laneName,
  isOpen,
  position,
  onClose,
  onDelete,
  onSwitchRuntime,
  onScanWorktrees,
  onLinkBranch,
  disabled,
  className,
}: LaneContextMenuProps) {
  const { t } = useTranslation();
  const menuRef = useRef<HTMLDivElement>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);

  // 点击外部关闭
  useEffect(() => {
    if (!isOpen) return;
    function dismissOutside(event: PointerEvent) {
      if (event.target instanceof Node && !menuRef.current?.contains(event.target)) {
        onClose();
      }
    }
    function dismissEscape(event: KeyboardEvent) {
      if (event.key === "Escape") { onClose(); }
    }
    document.addEventListener("pointerdown", dismissOutside);
    document.addEventListener("keydown", dismissEscape);
    return () => {
      document.removeEventListener("pointerdown", dismissOutside);
      document.removeEventListener("keydown", dismissEscape);
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const actions: LaneContextAction[] = [
    {
      id: "switch-runtime",
      label: "切换运行时",
      icon: Monitor,
      onClick: () => { onSwitchRuntime?.(laneId); onClose(); },
      disabled,
    },
    {
      id: "scan-worktrees",
      label: "扫描本地 Worktree",
      icon: ScanEye,
      onClick: () => { onScanWorktrees?.(laneId); onClose(); },
      disabled,
    },
    {
      id: "link-branch",
      label: "链接 GitHub Branch",
      icon: GitBranch,
      onClick: () => { onLinkBranch?.(laneId); onClose(); },
      disabled,
    },
    {
      id: "open-lane",
      label: "打开 Lane 管理页",
      icon: ExternalLink,
      href: canonicalLaneHref(projectId, laneId),
    },
  ];

  return (
    <div
      ref={menuRef}
      role="menu"
      aria-label={`Lane: ${laneName}`}
      className={clsx(
        "fixed z-[9999] min-w-[200px] rounded-lg border-2 border-[var(--cel-ink,var(--color-border))] bg-[var(--cel-surface-card,var(--color-surface-2))] p-1.5 shadow-xl",
        className,
      )}
      style={{ left: position.x, top: position.y }}
    >
      <div className="flex items-center justify-between border-b border-line pb-1.5 mb-1.5 px-2">
        <span className="text-[10px] font-mono font-bold truncate text-[var(--cel-cyan,#00f0ff)]">{laneName}</span>
        <button
          type="button"
          onClick={onClose}
          className="text-ink-mute hover:text-ink p-0.5"
          aria-label="关闭菜单"
        >
          <XCircle size={12} />
        </button>
      </div>

      {confirmDelete ? (
        <div className="px-2 py-1.5" role="alertdialog" aria-label="确认删除 Lane">
          <p className="text-xs text-err mb-2">确定删除 Lane「{laneName}」？此操作不可撤销。</p>
          <div className="flex gap-1.5">
            <button
              type="button"
              className="flex-1 text-[10px] rounded border border-line px-2 py-1 hover:bg-bg-soft"
              onClick={() => setConfirmDelete(false)}
              disabled={deleting}
            >
              取消
            </button>
            <button
              type="button"
              className="flex-1 flex items-center justify-center gap-1 text-[10px] rounded border border-err/50 bg-err/10 text-err px-2 py-1 hover:bg-err/20 disabled:opacity-50"
              onClick={async () => {
                setDeleting(true);
                try {
                  onDelete?.(laneId);
                  onClose();
                } finally {
                  setDeleting(false);
                }
              }}
              disabled={deleting || disabled}
            >
              {deleting ? <Loader2 size={10} className="animate-spin" /> : <Trash2 size={10} />}
              确认删除
            </button>
          </div>
        </div>
      ) : (
        <>
          {/* Lane 操作列表 */}
          {actions.map((action, i) => {
            if (action.href) {
              return (
                <a
                  key={action.id}
                  href={action.href}
                  role="menuitem"
                  className={clsx(
                    "flex items-center gap-2 rounded px-2 py-1.5 text-xs transition-colors",
                    "hover:bg-[var(--cel-cyan,var(--accent-DEFAULT))]/10 hover:text-[var(--cel-cyan,var(--accent-DEFAULT))]",
                  )}
                  onClick={onClose}
                >
                  <action.icon size={13} />
                  <span className="flex-1">{action.label}</span>
                </a>
              );
            }
            return (
              <button
                key={action.id}
                type="button"
                role="menuitem"
                disabled={action.disabled}
                onClick={action.onClick}
                className={clsx(
                  "flex w-full items-center gap-2 rounded px-2 py-1.5 text-xs text-left transition-colors",
                  action.danger
                    ? "text-err hover:bg-err/10"
                    : "hover:bg-[var(--cel-cyan,var(--accent-DEFAULT))]/10 hover:text-[var(--cel-cyan,var(--accent-DEFAULT))]",
                  action.disabled && "opacity-50 cursor-not-allowed",
                )}
              >
                <action.icon size={13} />
                <span className="flex-1">{action.label}</span>
              </button>
            );
          })}

          {/* 删除 Lane (危险操作, 需确认) */}
          <div className="border-t border-line mt-1.5 pt-1.5">
            <button
              type="button"
              role="menuitem"
              disabled={disabled}
              onClick={() => setConfirmDelete(true)}
              className={clsx(
                "flex w-full items-center gap-2 rounded px-2 py-1.5 text-xs text-left text-err/80 hover:bg-err/10 hover:text-err transition-colors",
                disabled && "opacity-50 cursor-not-allowed",
              )}
            >
              <Trash2 size={13} />
              <span className="flex-1">删除 Lane</span>
            </button>
          </div>
        </>
      )}
    </div>
  );
}