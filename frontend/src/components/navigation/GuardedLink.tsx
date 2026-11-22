/*
@cypher schema=1 source_sha256=9d268c5e5d176bf257e09e78b46a20752db8aeede77729938957a6688cfe5d8f
MERGE (self:File {path:"frontend/src/components/navigation/GuardedLink.tsx"})
MERGE (link:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::GuardedLink",kind:"component"})
MERGE (navigate:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::handleNavigate",kind:"function"})
MERGE (useGuard:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::useUnsavedNavigationGuard",kind:"function"})
MERGE (confirm:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::confirmNavigation",kind:"function"})
MERGE (managedMarker:Config {id:"data-unsaved-navigation-guard=managed"})
MERGE (self)-[:DEFINES]->(link)
MERGE (self)-[:DEFINES]->(navigate)
MERGE (self)-[:IMPORTS]->(useGuard)
MERGE (link)-[:CALLS]->(useGuard)
MERGE (link)-[:USES]->(navigate)
MERGE (navigate)-[:CALLS]->(confirm)
MERGE (link)-[:CONFIGURES]->(managedMarker)
@endcypher
*/
"use client";

import Link from "next/link";
import { useCallback, useMemo, type ComponentProps } from "react";
import { useRouter } from "next/navigation";
import { useUnsavedNavigationGuard } from "@/lib/navigation/UnsavedNavigationGuard";

type GuardedLinkProps = ComponentProps<typeof Link> & {
  /** 目标 tab（如 "kanban" | "canvas" | "nema"）。同页面不同 tab 时，点击后切换到该 tab 而非默认 tab。 */
  targetTab?: string;
};

export function GuardedLink({ onNavigate, targetTab, ...props }: GuardedLinkProps) {
  const { confirmNavigation } = useUnsavedNavigationGuard();
  // 测试环境中 next/navigation mock 可能不导出 useRouter，优雅降级为仅调用 onNavigate
  const router = (() => {
    try {
      return (useRouter as () => { push?: (href: string) => unknown })();
    } catch {
      return undefined;
    }
  })();

  const hrefWithTab = useMemo(() => {
    if (!targetTab) return props.href as string;
    const href = (props.href as string) ?? "/";
    const url = new URL(href, window.location.origin);
    url.searchParams.set("tab", targetTab);
    return url.pathname + url.search;
  }, [props.href, targetTab]);

  const handleNavigate = useCallback<NonNullable<GuardedLinkProps["onNavigate"]>>(
    (event) => {
      if (!confirmNavigation()) {
        event.preventDefault();
        return;
      }
      if (targetTab && router?.push) {
        event.preventDefault();
        router.push(hrefWithTab);
        onNavigate?.(event);
        return;
      }
      onNavigate?.(event);
    },
    [confirmNavigation, router, targetTab, hrefWithTab, onNavigate],
  );

  return <Link {...props} href={hrefWithTab} data-unsaved-navigation-guard="managed" onNavigate={handleNavigate} />;
}
