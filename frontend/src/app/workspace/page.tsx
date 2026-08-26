"use client";

import { useStore } from "@/lib/store";
import { ListPage } from "@/lib/page-builders";
import { Briefcase } from "lucide-react";
import { StatusPill } from "@/components/StatusPill";

export default function WorkspacePage() {
  const workspaces = useStore((s) => s.workspaces);
  return (
    <ListPage
      title="Workspaces"
      subtitle="Workspace + Member 模型。default_branch_policy 决定 PR 合并策略(fast-forward-only / allow-non-ff)。"
      icon={<Briefcase className="text-accent" size={20} />}
      track="E"
      items={workspaces}
      searchKeys={["name"]}
      columns={[
        { key: "id", label: "ID", width: "100px", render: (w) => <span className="font-mono text-xs">{w.id}</span> },
        { key: "name", label: "Name", render: (w) => <span className="font-medium">{w.name}</span> },
        { key: "project", label: "Project", render: (w) => <span className="font-mono text-xs text-info">{w.project_id}</span> },
        { key: "kind", label: "Kind", render: (w) => <StatusPill value={w.kind} size="xs" /> },
        { key: "members", label: "Members", render: (w) => <span className="font-mono text-xs">{w.member_ids.length}</span> },
        { key: "policy", label: "Branch policy", render: (w) => (
          <StatusPill value={w.default_branch_policy === "fast-forward-only" ? "ff-only" : "allow-non-ff"} size="xs" />
        )},
      ]}
    />
  );
}
