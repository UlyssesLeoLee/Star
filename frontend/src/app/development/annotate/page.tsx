// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/app/development/annotate/page.tsx
//
// /development/annotate — Diff 标注工作台 (per FR-ORCA-035 §11 v1.0, ULYS-211)。
//
// 挂载 DiffAnnotator 的可导航入口: agent 生成的 diff 在此逐行 drop comment,
// 再经 feed_to_agent 回到 agent session 的下一轮 prompt。
//
// 缺标 (per 守门 #11): diff 正文来自 src/mocks/data/annotations.ts —— 后端
// 尚无"取 worktree 变更 diff"的接口 (见 lib/api/annotations.ts 文件头)。
// annotation 本身走真实 REST client (annotationsApi), 不是假数据。

"use client";

import { MessageSquarePlus } from "lucide-react";

import { DiffAnnotator } from "@/components/DiffAnnotator";
import { PageHeader } from "@/components/PageHeader";
import {
  DEMO_AGENT_RUN_ID,
  DEMO_DIFF_LINES,
  DEMO_FILE_PATH,
} from "@/mocks/data/annotations";

export default function DiffAnnotatePage() {
  return (
    <div className="mx-auto max-w-5xl" data-testid="diff-annotate-page">
      <PageHeader
        title="Diff 标注"
        subtitle="在 AI 生成的 diff 上逐行 drop comment; 标注作为 structured feedback 回到 agent session 的下一轮 prompt (FR-ORCA-035)。"
        icon={<MessageSquarePlus className="text-accent" size={20} />}
        track="D"
        count={DEMO_FILE_PATH}
      />

      <div className="card p-3">
        <DiffAnnotator
          filePath={DEMO_FILE_PATH}
          agentRunId={DEMO_AGENT_RUN_ID}
          lines={DEMO_DIFF_LINES}
        />
      </div>
    </div>
  );
}
