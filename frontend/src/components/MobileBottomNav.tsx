/*
@cypher schema=1 source_sha256=852a06a8ad888dee8012e19c5f9d0c3476f7be352bcbfd8e889c61c40a34f131
MERGE (self:File {path:"frontend/src/components/MobileBottomNav.tsx"})
MERGE (navigation:Symbol {id:"frontend/src/components/MobileBottomNav.tsx::MobileBottomNav",kind:"component"})
MERGE (projectTree:Symbol {id:"frontend/src/components/run/ProjectRunTree.tsx::ProjectRunTree",kind:"component"})
MERGE (pathname:ExternalService {name:"next/navigation usePathname"})
MERGE (translation:Symbol {id:"frontend/src/lib/i18n/useTranslation.ts::useTranslation",kind:"hook"})
MERGE (projectNavigationLabel:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.projectNavigation",kind:"property"})
MERGE (openDirectoryLabel:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.openProjectDirectory",kind:"property"})
MERGE (directoryLabel:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.projectDirectory",kind:"property"})
MERGE (directoryTitle:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.projectDirectoryTitle",kind:"property"})
MERGE (directoryHint:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.projectDirectoryHint",kind:"property"})
MERGE (closeDirectoryLabel:Symbol {id:"frontend/src/lib/i18n/dictionary.ts::Dictionary.sidebar.closeProjectDirectory",kind:"property"})
MERGE (self)-[:DEFINES]->(navigation)
MERGE (self)-[:IMPORTS]->(projectTree)
MERGE (self)-[:IMPORTS]->(pathname)
MERGE (self)-[:IMPORTS]->(translation)
MERGE (navigation)-[:CALLS]->(projectTree)
MERGE (navigation)-[:CALLS]->(pathname)
MERGE (navigation)-[:CALLS]->(translation)
MERGE (navigation)-[:READS]->(projectNavigationLabel)
MERGE (navigation)-[:READS]->(openDirectoryLabel)
MERGE (navigation)-[:READS]->(directoryLabel)
MERGE (navigation)-[:READS]->(directoryTitle)
MERGE (navigation)-[:READS]->(directoryHint)
MERGE (navigation)-[:READS]->(closeDirectoryLabel)
@endcypher
*/
"use client";

import { ProjectRunTree } from "@/components/run/ProjectRunTree";
import { usePathname } from "next/navigation";
import { useTranslation } from "@/lib/i18n";
import { ChevronDown, FolderTree } from "lucide-react";
import { useEffect, useRef, useState, type KeyboardEvent } from "react";

export function MobileBottomNav() {
  const pathname = usePathname() ?? "/";
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const closeRef = useRef<HTMLButtonElement>(null);
  const wasOpen = useRef(false);

  useEffect(() => {
    setOpen(false);
  }, [pathname]);

  useEffect(() => {
    if (open) {
      wasOpen.current = true;
      closeRef.current?.focus();
      return;
    }
    if (wasOpen.current) {
      wasOpen.current = false;
      triggerRef.current?.focus();
    }
  }, [open]);

  function handleKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (event.key === "Escape" && open) {
      event.preventDefault();
      setOpen(false);
    }
  }

  return (
    <>
      <nav
        data-testid="mobile-bottom-nav"
        aria-label={t.sidebar.projectNavigation}
        className="md:hidden fixed bottom-0 inset-x-0 border-t-2 border-[var(--cel-ink)] bg-bg/95 backdrop-blur-xl shadow-[0_-3px_0px_0px_var(--cel-ink)]"
        style={{ paddingBottom: "env(safe-area-inset-bottom)", zIndex: 9999 }}
      >
        <button
          ref={triggerRef}
          type="button"
          data-testid="mobile-nav-project"
          aria-label={t.sidebar.openProjectDirectory}
          aria-expanded={open}
          aria-controls="mobile-project-directory"
          onClick={() => setOpen((current) => !current)}
          className="flex h-14 w-full items-center justify-center gap-2 text-xs font-medium text-ink-dim hover:text-ink focus:outline focus:outline-2 focus:outline-accent"
        >
          <FolderTree size={18} aria-hidden="true" />
          <span>{t.sidebar.projectDirectoryTitle}</span>
          <ChevronDown
            size={14}
            aria-hidden="true"
            className={`transition-transform ${open ? "rotate-180" : ""}`}
          />
        </button>
      </nav>

      <section
        id="mobile-project-directory"
        data-testid="mobile-project-directory"
        aria-label={t.sidebar.projectDirectory}
        hidden={!open}
        onKeyDown={handleKeyDown}
        className="md:hidden fixed inset-x-0 bottom-14 z-[9998] max-h-[min(70dvh,32rem)] overflow-y-auto border-t-2 border-[var(--cel-ink)] bg-bg-soft p-3 shadow-[0_-4px_0px_0px_var(--cel-shadow-color)]"
        style={{ bottom: "calc(3.5rem + env(safe-area-inset-bottom))" }}
      >
        <div className="mb-2 flex items-center justify-between gap-3">
          <div>
            <h2 className="text-sm font-semibold">{t.sidebar.projectDirectoryTitle}</h2>
            <p className="text-[10px] text-ink-mute">
              {t.sidebar.projectDirectoryHint}
            </p>
          </div>
          <button
            ref={closeRef}
            type="button"
            data-testid="mobile-project-directory-close"
            onClick={() => setOpen(false)}
            className="shrink-0 rounded border border-line px-3 py-2 text-xs text-ink-dim hover:text-ink focus:outline focus:outline-2 focus:outline-accent"
          >
            {t.sidebar.closeProjectDirectory}
          </button>
        </div>
        {open && <ProjectRunTree collapsed={false} />}
      </section>
    </>
  );
}
