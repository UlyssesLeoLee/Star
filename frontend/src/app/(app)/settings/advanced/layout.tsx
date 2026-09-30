/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/(app)/settings/advanced/layout.tsx",type:"file",language:"tsx"}),
  (layout:Function {name:"AdvancedSettingsLayout",type:"function",signature:"AdvancedSettingsLayout({children})",visibility:"public",complexity:"moderate"}),
  (tabs:Variable {name:"tabs",type:"variable",language:"typescript"}),
  (advanced:Variable {name:"advanced",type:"variable",language:"typescript"}),
  (useTranslation:Function {name:"useTranslation",type:"function",language:"typescript",visibility:"imported",complexity:"simple"}),
  (pathname:Variable {name:"pathname",type:"variable",language:"typescript"}),
  (file)-[:CONTAINS]->(layout),(file)-[:CONTAINS]->(tabs),(file)-[:CONTAINS]->(advanced),
  (layout)-[:USES]->(tabs),(layout)-[:USES]->(pathname),(layout)-[:CALLS]->(useTranslation),(layout)-[:USES]->(advanced);
*/

"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Blocks, Plug, ShieldCheck, Sparkles } from "lucide-react";
import type { ReactNode } from "react";
import { clsx } from "clsx";
import { useTranslation } from "@/lib/i18n";

export default function AdvancedSettingsLayout({ children }: { children: ReactNode }) {
  const pathname = usePathname() ?? "";
  const { t } = useTranslation();
  const advanced = t.navModules["advanced-settings"];
  const tabs = [
    { id: "skills", label: "Skills", href: "/settings/advanced/skills", icon: Sparkles },
    { id: "hooks", label: "Hooks", href: "/settings/advanced/hooks", icon: ShieldCheck },
    { id: "mcp", label: "MCP", href: "/settings/advanced/mcp", icon: Plug },
    { id: "plugins", label: "Plugins", href: "/settings/advanced/plugins", icon: Blocks },
  ] as const;

  return (
    <section className="mx-auto w-full max-w-[1440px] p-4 md:p-6" data-testid="advanced-settings">
      <header className="mb-5 flex flex-wrap items-start justify-between gap-3">
        <div>
          <p className="text-[10px] font-mono uppercase tracking-[0.22em] text-accent">{advanced.categoryLabel}</p>
          <h1 className="mt-1 text-2xl font-semibold text-ink">{advanced.label}</h1>
          <p className="mt-1 max-w-3xl text-sm text-ink-dim">{advanced.description}</p>
        </div>
        <Link href="/settings" className="text-xs text-ink-mute underline decoration-line underline-offset-4 hover:text-ink">{t.navModules.settings.label}</Link>
      </header>

      <nav aria-label="高级设置导航" className="mb-5 flex gap-1 overflow-x-auto border-b border-line" data-testid="advanced-settings-tabs">
        {tabs.map(({ id, label, href, icon: Icon }) => {
          const active = pathname === href || pathname.startsWith(`${href}/`);
          return (
            <Link
              key={id}
              href={href}
              aria-current={active ? "page" : undefined}
              data-testid={`advanced-settings-tab-${id}`}
              className={clsx(
                "inline-flex shrink-0 items-center gap-2 border-b-2 px-4 py-3 text-sm transition-colors",
                active ? "border-accent text-accent" : "border-transparent text-ink-dim hover:border-line hover:text-ink",
              )}
            >
              <Icon size={15} />
              <span>{label}</span>
            </Link>
          );
        })}
      </nav>
      {children}
    </section>
  );
}
