/*
@cypher schema=1 source_sha256=0b12614a3f7866fa02004bbb0eadff01c2562b2ea3f1d79adbbc8eb2ad9346e0
MERGE (self:File {path:"frontend/src/components/nav/SidebarShortcuts.tsx"})
MERGE (rail:Symbol {id:"frontend/src/components/nav/SidebarShortcuts.tsx::SidebarShortcuts",kind:"component"})
MERGE (row:Symbol {id:"frontend/src/components/nav/SidebarShortcuts.tsx::ShortcutModule",kind:"component"})
MERGE (nav:Symbol {id:"frontend/src/lib/nav/navStore.ts::useNavStore",kind:"hook"})
MERGE (registry:File {path:"frontend/src/lib/nav/registry.ts"})
MERGE (link:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::GuardedLink",kind:"component"})
MERGE (self)-[:DEFINES]->(rail)
MERGE (self)-[:DEFINES]->(row)
MERGE (self)-[:IMPORTS]->(nav)
MERGE (self)-[:IMPORTS]->(registry)
MERGE (self)-[:IMPORTS]->(link)
MERGE (rail)-[:CALLS]->(nav)
MERGE (rail)-[:CALLS]->(row)
MERGE (rail)-[:READS]->(registry)
MERGE (rail)-[:CALLS]->(link)
MERGE (row)-[:CALLS]->(link)
@endcypher
*/
"use client";

import { clsx } from "clsx";
import { Bell, LayoutGrid, MessageSquare, Plus, X } from "lucide-react";
import { usePathname, useSearchParams } from "next/navigation";
import { useEffect, useId, useRef, useState } from "react";
import { GuardedLink } from "@/components/navigation/GuardedLink";
import { useModuleTranslation, useTranslation } from "@/lib/i18n";
import { moduleHrefMatches } from "@/lib/nav/activeMatch";
import { useNavStore } from "@/lib/nav/navStore";
import { MODULE_MAP, NAV_GROUPS, type ModuleDefinition } from "@/lib/nav/registry";

export function SidebarShortcuts({ inDrawer = false, onCustomize }: {
  inDrawer?: boolean;
  onCustomize?: () => void;
}) {
  const { t } = useTranslation();
  const pathname = usePathname() ?? "/";
  const searchParams = useSearchParams();
  const panelId = useId();
  const container = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const [openMode, setOpenMode] = useState<"hover" | "click" | null>(null);
  const pinIds = useNavStore((s) => s.sidebarItemIds);
  const removePin = useNavStore((s) => s.removeSidebarItem);
  const openMatrix = useNavStore((s) => s.openMatrix);
  const open = openMode !== null;
  const pinnedModules = pinIds.flatMap((id) => {
    const item = MODULE_MAP.get(id);
    return item ? [item] : [];
  });
  const globalModules = NAV_GROUPS["home-personal"].flatMap((id) => {
    const item = MODULE_MAP.get(id);
    return item && !pinIds.includes(id) ? [item] : [];
  });

  useEffect(() => {
    if (!open) return;
    function dismissOutside(event: PointerEvent) {
      if (event.target instanceof Node && !container.current?.contains(event.target)) setOpenMode(null);
    }
    function dismissWithEscape(event: KeyboardEvent) {
      if (event.key !== "Escape") return;
      if (container.current?.contains(document.activeElement)) {
        event.preventDefault();
        trigger.current?.focus();
      }
      setOpenMode(null);
    }
    document.addEventListener("pointerdown", dismissOutside);
    document.addEventListener("keydown", dismissWithEscape);
    return () => {
      document.removeEventListener("pointerdown", dismissOutside);
      document.removeEventListener("keydown", dismissWithEscape);
    };
  }, [open]);

  function close() {
    setOpenMode(null);
  }

  const utilityLinks = [
    { id: "notifications", icon: Bell, label: t.sidebar.quickEntries.notifications, href: "/notification" },
    { id: "chat", icon: MessageSquare, label: t.sidebar.quickEntries.chat, href: "/inbox?type=chat" },
  ];

  return (
    <div
      ref={container}
      className={clsx("z-10", inDrawer ? "relative shrink-0" : "absolute left-full top-20")}
      data-testid="sidebar-shortcuts"
      onPointerEnter={(event) => {
        if (!inDrawer && event.pointerType !== "touch") setOpenMode((mode) => mode ?? "hover");
      }}
      onPointerLeave={() => {
        if (!container.current?.contains(document.activeElement)) {
          setOpenMode((mode) => mode === "hover" ? null : mode);
        }
      }}
      onKeyDown={(event) => {
        if (event.key !== "Escape" || !open) return;
        event.preventDefault();
        event.stopPropagation();
        close();
        trigger.current?.focus();
      }}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget)) close();
      }}
    >
      <button
        ref={trigger}
        type="button"
        title={t.sidebar.shortcuts.label}
        aria-label={t.sidebar.shortcuts.label}
        aria-expanded={open}
        aria-controls={panelId}
        data-testid="sidebar-shortcuts-trigger"
        onClick={() => setOpenMode((mode) => mode === "click" ? null : "click")}
        className={clsx(
          "grid h-9 place-items-center border border-line bg-bg-card text-ink-mute hover:text-accent focus-visible:outline focus-visible:outline-accent",
          inDrawer ? "w-9 rounded-md" : "w-7 rounded-r-md border-l-0",
          open && "text-accent",
        )}
      >
        <LayoutGrid size={15} aria-hidden="true" />
      </button>

      <nav
        id={panelId}
        hidden={!open}
        aria-label={t.sidebar.shortcuts.label}
        data-testid="sidebar-shortcuts-panel"
        className={clsx(
          "absolute w-52 overflow-y-auto rounded-md border border-line bg-bg-card p-2 shadow-soft",
          inDrawer ? "right-0 top-full max-h-[60dvh]" : "left-full top-0 max-h-[calc(100dvh-6rem)]",
        )}
      >
        {open && (
          <>
            {pinnedModules.length > 0 && (
              <ul className="space-y-0.5 border-b border-line pb-1" data-testid="sidebar-pinned-modules">
                {pinnedModules.map((item) => (
                  <ShortcutModule
                    key={item.id}
                    item={item}
                    active={moduleHrefMatches(item.href, pathname, searchParams)}
                    onNavigate={close}
                    onRemove={() => {
                      trigger.current?.focus();
                      removePin(item.id);
                    }}
                  />
                ))}
              </ul>
            )}
            <ul className="space-y-0.5" data-testid="sidebar-group-global">
              {globalModules.map((item) => (
                <ShortcutModule key={item.id} item={item} active={moduleHrefMatches(item.href, pathname, searchParams)} onNavigate={close} />
              ))}
              {utilityLinks.map(({ id, href, label, icon: Icon }) => (
                <li key={id}>
                  <GuardedLink href={href} onNavigate={close} className="flex items-center gap-2 rounded px-2 py-1.5 text-xs text-ink-dim hover:bg-bg-soft hover:text-ink" data-testid={`sidebar-quick-${id}`}>
                    <Icon size={14} aria-hidden="true" />{label}
                  </GuardedLink>
                </li>
              ))}
            </ul>
            <button
              type="button"
              data-testid="sidebar-add-shortcut"
              className="mt-1 flex w-full items-center gap-2 border-t border-line px-2 py-2 text-left text-xs text-accent hover:bg-bg-soft focus-visible:outline focus-visible:outline-accent"
              onClick={() => {
                trigger.current?.focus();
                close();
                onCustomize?.();
                openMatrix();
              }}
            >
              <Plus size={14} aria-hidden="true" />{t.sidebar.shortcuts.add}
            </button>
          </>
        )}
      </nav>
    </div>
  );
}

function ShortcutModule({ item, active, onNavigate, onRemove }: {
  item: ModuleDefinition;
  active: boolean;
  onNavigate: () => void;
  onRemove?: () => void;
}) {
  const { label } = useModuleTranslation(item);
  const { t } = useTranslation();
  const Icon = item.icon;
  const removeLabel = `${t.appMatrix.unpinFromSidebar} · ${label}`;
  return (
    <li className="flex items-center rounded hover:bg-bg-soft">
      <GuardedLink
        href={item.href}
        onNavigate={onNavigate}
        aria-current={active ? "page" : undefined}
        data-active={active}
        data-testid={`sidebar-item-${item.id}`}
        className={clsx("flex min-w-0 flex-1 items-center gap-2 rounded px-2 py-1.5 text-xs", active ? "bg-accent/10 font-medium text-accent" : "text-ink-dim hover:text-ink")}
      >
        <Icon size={14} className="shrink-0" aria-hidden="true" />
        <span className="min-w-0 flex-1 truncate">{label}</span>
        {item.count !== undefined && item.count > 0 && <span className="shrink-0 tabular-nums">{item.count}</span>}
      </GuardedLink>
      {onRemove && (
        <button type="button" onClick={onRemove} aria-label={removeLabel} title={removeLabel} data-testid={`sidebar-remove-${item.id}`} className="shrink-0 rounded p-1.5 text-ink-mute hover:text-err focus-visible:outline focus-visible:outline-accent">
          <X size={12} aria-hidden="true" />
        </button>
      )}
    </li>
  );
}
