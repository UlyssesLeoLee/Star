"use client";

/**
 * DocsTab — F-04 运维文档 端到端 UI 组件
 * (per docs/briefs/ops-f04-docs-impl.md
 *  + docs/basic-design/OPS-BASIC-DESIGN-001.md §3.4 F-04
 *  + docs/requirements/SRS-STAR-OPS-001.md v0.1 §4 F-04)
 *
 * 范围:
 * - 5 类别分组: SRS / BAS / DET / 报告 / 其他 (per brief §2.1, 跟 SRS-001 §4 F-04 卡片)
 * - 文档链接列表: 路径 + 标题 + 类别 tag + updated_at
 * - useQuery 60s 轮询 (文档列表变更频率低, 守门 #6 v2 retriable 自动 retry)
 * - i18n 3 语言 (zh-CN / en / ja), 走 useTranslation
 * - 守门 #1 R-05: 仅 docs/ 4 子目录, 真实 walkdir 扫描
 * - 守门 #11 缺标比错标: 文档为空 → 显示 "暂无可用文档" 而非空白
 */

import { useQuery } from "@tanstack/react-query";
import { FileText, RefreshCw, AlertTriangle, FolderOpen, Clock } from "lucide-react";
import { listDocs, type DocRef, OpsApiError } from "@/lib/ops-api";
import { useTranslation } from "@/lib/i18n";

const OPS_API_URL = process.env.NEXT_PUBLIC_OPS_URL || "http://localhost:8090";

/** 5 类别顺序 (per brief §2.1, 跟 SRS-001 §4 F-04 卡片: SRS / BAS / DET / 报告 / 其他) */
const CATEGORY_ORDER = ["SRS", "BAS", "DET", "Report", "Other"] as const;
type CategoryName = (typeof CATEGORY_ORDER)[number];

export function DocsTab() {
  const { t } = useTranslation();

  // 调 /api/ops/docs 真实端点 (per F-04 端到端, walkdir 扫 4 docs/ 子目录)
  const docsQuery = useQuery({
    queryKey: ["ops-docs-list"],
    queryFn: () => listDocs(),
    refetchInterval: 60_000, // 60s 轮询, 文档列表变更频率低
    retry: (failureCount, err) => {
      if (err instanceof OpsApiError) {
        return err.retriable && failureCount < 3;
      }
      return failureCount < 3;
    },
  });

  // 兼容 ops_api 旧返回 (data.data.data, 跟 F-02 LogAnalysis 走 OpsResponse 包装)
  // 跟 ops_api.rs::docs_list 一致: 返 Vec<DocRef> directly (无中间 DocRef[] wrapper)
  const allDocs: DocRef[] = (() => {
    const data = docsQuery.data?.data;
    if (Array.isArray(data)) {
      return data as DocRef[];
    }
    // 兼容 backend 老格式 (data.data: DocRef[])
    if (data && typeof data === "object" && "data" in data && Array.isArray((data as { data: unknown }).data)) {
      return (data as { data: DocRef[] }).data;
    }
    return [];
  })();

  // 按 5 类别分组 (per brief §2.1, SRS-001 §4 F-04 卡片)
  const grouped: Record<CategoryName, DocRef[]> = {
    SRS: [],
    BAS: [],
    DET: [],
    Report: [],
    Other: [],
  };
  for (const d of allDocs) {
    const cat = d.category as CategoryName;
    if (cat in grouped) {
      grouped[cat].push(d);
    } else {
      grouped.Other.push(d);
    }
  }

  return (
    <div className="space-y-4">
      {/* === 标题 + Refresh 状态 === */}
      <div className="anime-panel anime-chamfer p-[21px]">
        <div className="flex items-center gap-2 mb-2">
          <FileText className="w-4 h-4 text-accent" />
          <h3 className="text-title font-bold">{t.opsConsole.docsTitle}</h3>
          <span className="anime-hud-tag ml-auto">F-04</span>
          {docsQuery.isFetching && (
            <RefreshCw className="w-3 h-3 animate-spin text-ink-mute" />
          )}
        </div>
        <p className="text-xs text-ink-mute font-mono">
          {t.opsConsole.docsSubtitle}
        </p>
      </div>

      {/* === 错误状态 === */}
      {docsQuery.isError && (
        <div className="anime-panel anime-chamfer p-[21px]">
          <div className="flex items-center gap-2 text-error text-sm">
            <AlertTriangle className="w-4 h-4" />
            <span>{(docsQuery.error as Error).message}</span>
          </div>
        </div>
      )}

      {/* === 5 类别分组卡片 === */}
      {!docsQuery.isError && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {CATEGORY_ORDER.map((cat) => {
            const items = grouped[cat];
            const label = (() => {
              switch (cat) {
                case "SRS":
                  return t.opsConsole.docsCategorySrs;
                case "BAS":
                  return t.opsConsole.docsCategoryBas;
                case "DET":
                  return t.opsConsole.docsCategoryDet;
                case "Report":
                  return t.opsConsole.docsCategoryReport;
                case "Other":
                  return t.opsConsole.docsCategoryOther;
              }
            })();
            return (
              <CategoryCard
                key={cat}
                category={cat}
                label={label}
                items={items}
                emptyHint={t.opsConsole.docsEmpty}
              />
            );
          })}
        </div>
      )}

      {/* === meta.hint 提示 (守门 #1 R-05) === */}
      {docsQuery.data && (
        <p className="text-[10px] text-ink-mute italic font-mono">
          {docsQuery.data.meta.hint}
        </p>
      )}

      {/* === 远端 URL (调试用) === */}
      <p className="text-[10px] text-ink-mute font-mono">
        {OPS_API_URL}/api/ops/docs
      </p>
    </div>
  );
}

function CategoryCard({
  category,
  label,
  items,
  emptyHint,
}: {
  category: CategoryName;
  label: string;
  items: DocRef[];
  emptyHint: string;
}) {
  return (
    <div className="anime-panel anime-chamfer p-4">
      <div className="flex items-center gap-2 mb-3">
        <FolderOpen className="w-4 h-4 text-accent" />
        <h4 className="text-sm font-semibold text-ink">{label}</h4>
        <span className="text-[10px] font-mono text-ink-mute ml-auto">{category}</span>
        <span className="anime-hud-tag">{items.length}</span>
      </div>

      {items.length === 0 ? (
        <p className="text-xs text-ink-mute font-mono py-4 text-center">{emptyHint}</p>
      ) : (
        <ul className="space-y-2">
          {items.map((d) => (
            <li key={d.path} className="text-sm">
              <a
                href={`/${d.path}`}
                className="flex items-start gap-2 p-2 rounded hover:bg-bg-soft/50 transition-colors group"
                title={d.path}
              >
                <FileText className="w-3 h-3 text-ink-mute mt-0.5 flex-shrink-0" />
                <div className="flex-1 min-w-0">
                  <p className="text-ink group-hover:text-accent transition-colors truncate">
                    {d.title}
                  </p>
                  <p className="text-[10px] text-ink-mute font-mono truncate">{d.path}</p>
                </div>
                <div className="flex items-center gap-1 text-[10px] text-ink-mute font-mono flex-shrink-0">
                  <Clock className="w-3 h-3" />
                  <span>{formatDate(d.updated_at)}</span>
                </div>
              </a>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** 格式化 ISO 8601 UTC 日期为短格式 (YYYY-MM-DD) */
function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    if (isNaN(d.getTime())) return iso;
    return d.toISOString().slice(0, 10);
  } catch {
    return iso;
  }
}
