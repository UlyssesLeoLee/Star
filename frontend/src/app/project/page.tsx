"use client";

import { useStore } from "@/lib/store";
import { ListPage } from "@/lib/page-builders";
import { FolderTree } from "lucide-react";
import { StatusPill } from "@/components/StatusPill";
import { useTranslation } from "@/lib/i18n";

export default function ProjectPage() {
  const { t } = useTranslation();
  const projects = useStore((s) => s.projects);
  return (
    <ListPage
      title={t.pageTitles['/projects'].title}
      subtitle="租户下的工作单元,key 用作 work-item 编号前缀(PHYSIS-123)。"
      icon={<FolderTree className="text-accent" size={20} />}
      track="D"
      items={projects}
      searchKeys={["name", "key"]}
      columns={[
        { key: "key", label: "Key", width: "100px", render: (p) => <span className="font-mono text-info">{p.key}</span> },
        { key: "name", label: "Name", render: (p) => <span className="font-medium">{p.name}</span> },
        { key: "visibility", label: "Visibility", render: (p) => <StatusPill value={p.visibility} /> },
        { key: "owner", label: "Owner", render: (p) => <span className="font-mono text-xs">{p.owner_id}</span> },
        { key: "members", label: "Members", render: (p) => <span className="font-mono">{p.member_count}</span> },
        { key: "created", label: "Created", render: (p) => <span className="text-ink-dim text-xs">{new Date(p.created_at).toLocaleDateString()}</span> },
      ]}
    />
  );
}
