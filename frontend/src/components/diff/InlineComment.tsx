/**
 * InlineComment — ULYS-165 FR-ORCA-035 单条 annotation 卡片.
 *
 * Per docs/ecosystem-survey/orca-design-survey.md §11 line 483-485:
 *   在 Diff 上写注释 → 回喂 Agent
 *
 * **本文件 (ULYS-165 v2)** — 适配 PR #84 (ULYS-201) 的 DiffAnnotation DTO:
 *   - 移除 status 字段
 *   - 用 author: 'user' | 'agent' 替代
 *
 * 用法: PR 详情 / Agent 详情 / 悬浮展开
 */

import type { DiffAnnotationView, DiffAnnotationAuthor } from "./DiffAnnotator";

interface InlineCommentProps {
  annotation: DiffAnnotationView;
  onDelete?: (comment_id: string) => void;
}

const AUTHOR_LABEL: Record<DiffAnnotationAuthor, string> = {
  user: "User",
  agent: "Agent",
};

export function InlineComment({ annotation, onDelete }: InlineCommentProps) {
  return (
    <div
      className="inline-comment"
      data-testid={`inline-comment-${annotation.comment_id}`}
    >
      <div className="inline-comment-header">
        <span className="inline-comment-file">{annotation.file_path}</span>
        <span className="inline-comment-line">
          L{annotation.line_start}-{annotation.line_end}
        </span>
        <span className={`inline-comment-author author-${annotation.author}`}>
          {AUTHOR_LABEL[annotation.author]}
        </span>
        {onDelete && (
          <button
            className="inline-comment-delete"
            onClick={() => onDelete(annotation.comment_id)}
            aria-label="Delete annotation"
            data-testid={`inline-comment-delete-${annotation.comment_id}`}
          >
            ×
          </button>
        )}
      </div>
      <div className="inline-comment-body">{annotation.body}</div>
      <div className="inline-comment-meta">
        <time dateTime={annotation.created_at}>
          {new Date(annotation.created_at).toLocaleString()}
        </time>
      </div>
    </div>
  );
}