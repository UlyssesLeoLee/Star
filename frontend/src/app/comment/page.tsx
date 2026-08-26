"use client";

import { useStore } from "@/lib/store";
import { ListPage } from "@/lib/page-builders";
import { MessageSquare } from "lucide-react";
import { StatusPill } from "@/components/StatusPill";

export default function CommentPage() {
  const comments = useStore((s) => s.comments);
  return (
    <ListPage
      title="Comments"
      subtitle="跨实体评论(支持 work_item / pr / context_packet / agent_session)。thread_root 用于嵌套回复,mentions 用于 @通知。"
      icon={<MessageSquare className="text-accent" size={20} />}
      track="D"
      items={comments}
      searchKeys={["body"]}
      columns={[
        { key: "id", label: "ID", width: "100px", render: (c) => <span className="font-mono text-xs">{c.id}</span> },
        { key: "target", label: "Target", render: (c) => (
          <span className="font-mono text-xs">
            <StatusPill value={c.target_kind} size="xs" />
            <span className="ml-1.5 text-info">{c.target_id}</span>
          </span>
        )},
        { key: "author", label: "Author", render: (c) => <span className="font-mono text-xs">{c.author_id}</span> },
        { key: "body", label: "Body", render: (c) => <span className="text-ink-dim text-xs line-clamp-2">{c.body}</span> },
        { key: "mentions", label: "@mentions", render: (c) => <span className="font-mono text-xs">{c.mentions.length || "—"}</span> },
        { key: "when", label: "Created", render: (c) => <span className="text-ink-dim text-xs">{new Date(c.created_at).toLocaleString()}</span> },
      ]}
    />
  );
}
