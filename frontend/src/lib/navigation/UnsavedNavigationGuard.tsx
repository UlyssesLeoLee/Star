/*
@cypher schema=1 source_sha256=173c47b2d2e9f06eafc88ba428e3381ed806c7a5b6e7645cb2a0d88e893a02d1
MERGE (self:File {path:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx"})
MERGE (provider:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::UnsavedNavigationGuardProvider",kind:"component"})
MERGE (register:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::registerUnsavedNavigationGuard",kind:"function"})
MERGE (confirm:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::confirmNavigation",kind:"function"})
MERGE (activeGuard:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::activeGuard",kind:"variable"})
MERGE (value:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::value",kind:"variable"})
MERGE (self)-[:DEFINES]->(provider)
MERGE (self)-[:DEFINES]->(register)
MERGE (self)-[:DEFINES]->(confirm)
MERGE (provider)-[:DEFINES]->(activeGuard)
MERGE (provider)-[:DEFINES]->(value)
MERGE (provider)-[:USES]->(register)
MERGE (provider)-[:USES]->(confirm)
MERGE (register)-[:WRITES]->(activeGuard)
MERGE (confirm)-[:READS]->(activeGuard)
@endcypher
*/
"use client";

import { createContext, useCallback, useContext, useMemo, useRef, type ReactNode } from "react";

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
  const activeGuard = useRef<ActiveNavigationGuard | null>(null);

  const registerUnsavedNavigationGuard = useCallback((message: string) => {
    const token = Symbol("unsaved-navigation-guard");
    activeGuard.current = { token, message };
    return () => {
      if (activeGuard.current?.token === token) activeGuard.current = null;
    };
  }, []);

  const confirmNavigation = useCallback(() => {
    const guard = activeGuard.current;
    if (!guard || typeof window === "undefined") return true;
    return window.confirm(guard.message);
  }, []);

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

export function useUnsavedNavigationGuard() {
  return useContext(UnsavedNavigationGuardContext);
}
