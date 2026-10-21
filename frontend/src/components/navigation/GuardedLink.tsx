/*
@cypher schema=1 source_sha256=4d227a4ced7419fce69229cfc390709d12254711c9764acba661c2036201cd25
MERGE (self:File {path:"frontend/src/components/navigation/GuardedLink.tsx"})
MERGE (link:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::GuardedLink",kind:"component"})
MERGE (navigate:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::handleNavigate",kind:"function"})
MERGE (useGuard:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::useUnsavedNavigationGuard",kind:"function"})
MERGE (confirm:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::confirmNavigation",kind:"function"})
MERGE (self)-[:DEFINES]->(link)
MERGE (self)-[:DEFINES]->(navigate)
MERGE (self)-[:IMPORTS]->(useGuard)
MERGE (link)-[:CALLS]->(useGuard)
MERGE (link)-[:USES]->(navigate)
MERGE (navigate)-[:CALLS]->(confirm)
@endcypher
*/
"use client";

import Link from "next/link";
import { useCallback, type ComponentProps } from "react";
import { useUnsavedNavigationGuard } from "@/lib/navigation/UnsavedNavigationGuard";

type GuardedLinkProps = ComponentProps<typeof Link>;

export function GuardedLink({ onNavigate, ...props }: GuardedLinkProps) {
  const { confirmNavigation } = useUnsavedNavigationGuard();
  const handleNavigate = useCallback<NonNullable<GuardedLinkProps["onNavigate"]>>((event) => {
    if (!confirmNavigation()) {
      event.preventDefault();
      return;
    }
    onNavigate?.(event);
  }, [confirmNavigation, onNavigate]);

  return <Link {...props} onNavigate={handleNavigate} />;
}
