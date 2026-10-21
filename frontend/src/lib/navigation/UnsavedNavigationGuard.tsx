/*
@cypher schema=1 source_sha256=3a4a1eef020eb061c142c59011d881fd06ffa8b6df44c256308a2fcd8ca2e905
MERGE (self:File {path:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx"})
MERGE (provider:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::UnsavedNavigationGuardProvider",kind:"component"})
MERGE (contentNavigation:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::guardContentAnchorNavigation",kind:"function"})
MERGE (normalizedSearch:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::normalizedSearch",kind:"function"})
MERGE (sameRoute:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::sameRouteNavigation",kind:"function"})
MERGE (router:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::router",kind:"variable"})
MERGE (routerHook:ExternalService {name:"next/navigation.useRouter"})
MERGE (routerPush:ExternalService {name:"next/navigation.AppRouterInstance.push"})
MERGE (register:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::registerUnsavedNavigationGuard",kind:"function"})
MERGE (confirm:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::confirmNavigation",kind:"function"})
MERGE (activeGuard:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::activeGuard",kind:"variable"})
MERGE (activeGuardToken:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::activeGuardToken",kind:"variable"})
MERGE (value:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::value",kind:"variable"})
MERGE (clickEvent:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::document_click_event",kind:"event"})
MERGE (self)-[:IMPORTS]->(routerHook)
MERGE (self)-[:DEFINES]->(provider)
MERGE (self)-[:DEFINES]->(contentNavigation)
MERGE (self)-[:DEFINES]->(normalizedSearch)
MERGE (self)-[:DEFINES]->(sameRoute)
MERGE (self)-[:DEFINES]->(register)
MERGE (self)-[:DEFINES]->(confirm)
MERGE (provider)-[:DEFINES]->(activeGuard)
MERGE (provider)-[:DEFINES]->(activeGuardToken)
MERGE (provider)-[:DEFINES]->(router)
MERGE (provider)-[:DEFINES]->(value)
MERGE (provider)-[:USES]->(register)
MERGE (provider)-[:USES]->(confirm)
MERGE (provider)-[:USES]->(routerHook)
MERGE (provider)-[:CALLS]->(contentNavigation)
MERGE (provider)-[:SUBSCRIBES]->(clickEvent)
MERGE (contentNavigation)-[:HANDLES]->(clickEvent)
MERGE (contentNavigation)-[:CALLS]->(confirm)
MERGE (contentNavigation)-[:CALLS]->(sameRoute)
MERGE (contentNavigation)-[:CALLS]->(routerPush)
MERGE (sameRoute)-[:CALLS]->(normalizedSearch)
MERGE (register)-[:WRITES]->(activeGuard)
MERGE (register)-[:WRITES]->(activeGuardToken)
MERGE (confirm)-[:READS]->(activeGuard)
@endcypher
*/
"use client";

import { useRouter } from "next/navigation";
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";

type ActiveNavigationGuard = { token: symbol; message: string };

interface UnsavedNavigationGuardContextValue {
  registerUnsavedNavigationGuard: (message: string) => () => void;
  confirmNavigation: () => boolean;
}

const UnsavedNavigationGuardContext = createContext<UnsavedNavigationGuardContextValue>({
  registerUnsavedNavigationGuard: () => () => undefined,
  confirmNavigation: () => true,
});

export function UnsavedNavigationGuardProvider({ children }: { children: ReactNode }) {
  const router = useRouter();
  const activeGuard = useRef<ActiveNavigationGuard | null>(null);
  const [activeGuardToken, setActiveGuardToken] = useState<symbol | null>(null);

  const registerUnsavedNavigationGuard = useCallback((message: string) => {
    const token = Symbol("unsaved-navigation-guard");
    activeGuard.current = { token, message };
    setActiveGuardToken(token);
    return () => {
      if (activeGuard.current?.token !== token) return;
      activeGuard.current = null;
      setActiveGuardToken(null);
    };
  }, []);

  const confirmNavigation = useCallback(() => {
    const guard = activeGuard.current;
    if (!guard || typeof window === "undefined") return true;
    return window.confirm(guard.message);
  }, []);

  useEffect(() => {
    if (!activeGuardToken) return;

    const guardContentAnchorNavigation = (event: MouseEvent) => {
      if (event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
      if (!(event.target instanceof Element)) return;

      const anchor = event.target.closest<HTMLAnchorElement>("a[href]");
      if (!anchor || anchor.dataset.unsavedNavigationGuard === "managed") return;
      if (anchor.hasAttribute("download")) return;
      const target = anchor.getAttribute("target")?.trim().toLowerCase();
      if (target && target !== "_self") return;

      let destination: URL;
      try {
        destination = new URL(anchor.href, window.location.href);
      } catch {
        return;
      }
      if (destination.origin !== window.location.origin || sameRouteNavigation(destination, new URL(window.location.href))) return;

      if (!confirmNavigation()) {
        event.preventDefault();
        event.stopPropagation();
        return;
      }

      // Route through the public App Router API so accepted internal anchors stay
      // in the SPA and do not trigger a second native beforeunload prompt.
      event.preventDefault();
      router.push(`${destination.pathname}${destination.search}${destination.hash}`);
    };

    document.addEventListener("click", guardContentAnchorNavigation, true);
    return () => document.removeEventListener("click", guardContentAnchorNavigation, true);
  }, [activeGuardToken, confirmNavigation, router]);

  const value = useMemo(
    () => ({ registerUnsavedNavigationGuard, confirmNavigation }),
    [registerUnsavedNavigationGuard, confirmNavigation],
  );

  return (
    <UnsavedNavigationGuardContext.Provider value={value}>
      {children}
    </UnsavedNavigationGuardContext.Provider>
  );
}

function normalizedSearch(url: URL): string {
  return [...url.searchParams.entries()]
    .map(([key, value], index) => ({ key, value, index }))
    .sort((left, right) => left.key.localeCompare(right.key) || left.index - right.index)
    .map(({ key, value }) => `${key}\u0000${value}`)
    .join("\u0001");
}

function sameRouteNavigation(destination: URL, current: URL): boolean {
  return destination.pathname === current.pathname && normalizedSearch(destination) === normalizedSearch(current);
}

export function useUnsavedNavigationGuard() {
  return useContext(UnsavedNavigationGuardContext);
}
