/*
@cypher schema=1 source_sha256=0ce079cd4bcd069fe258e56edc31caf42f3d1e1b66eb44f460a7c7d0afb82d58
MERGE (self:File {path:"frontend/src/components/navigation/GuardedLink.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/navigation/GuardedLink.test.tsx::GuardedLink navigation tests",kind:"suite"})
MERGE (navigationCase:Symbol {id:"frontend/src/components/navigation/GuardedLink.test.tsx::cancels a supported internal route when the user stays and permits it after confirmation case",kind:"test"})
MERGE (cleanupCase:Symbol {id:"frontend/src/components/navigation/GuardedLink.test.tsx::removes the active prompt when the draft is cleared or its owner unmounts case",kind:"test"})
MERGE (externalCase:Symbol {id:"frontend/src/components/navigation/GuardedLink.test.tsx::does not prompt for external links or links opened in a new tab case",kind:"test"})
MERGE (callbackCase:Symbol {id:"frontend/src/components/navigation/GuardedLink.test.tsx::runs app-shell navigation callbacks only after the guard approves case",kind:"test"})
MERGE (contentCase:Symbol {id:"frontend/src/components/navigation/GuardedLink.test.tsx::guards arbitrary same-origin content anchors before their click effects and routes once after approval case",kind:"test"})
MERGE (exemptionsCase:Symbol {id:"frontend/src/components/navigation/GuardedLink.test.tsx::leaves external, modified, new-tab, download, hash-only, and normalized same-route anchors alone case",kind:"test"})
MERGE (draft:Symbol {id:"frontend/src/components/navigation/GuardedLink.test.tsx::ActiveDraftGuard",kind:"component"})
MERGE (link:Symbol {id:"frontend/src/components/navigation/GuardedLink.tsx::GuardedLink",kind:"component"})
MERGE (provider:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::UnsavedNavigationGuardProvider",kind:"component"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(navigationCase)
MERGE (suite)-[:DEFINES]->(cleanupCase)
MERGE (suite)-[:DEFINES]->(externalCase)
MERGE (suite)-[:DEFINES]->(callbackCase)
MERGE (suite)-[:DEFINES]->(contentCase)
MERGE (suite)-[:DEFINES]->(exemptionsCase)
MERGE (suite)-[:DEFINES]->(draft)
MERGE (navigationCase)-[:TESTS]->(link)
MERGE (cleanupCase)-[:TESTS]->(link)
MERGE (externalCase)-[:TESTS]->(link)
MERGE (callbackCase)-[:TESTS]->(link)
MERGE (contentCase)-[:TESTS]->(provider)
MERGE (exemptionsCase)-[:TESTS]->(provider)
MERGE (draft)-[:CALLS]->(provider)
@endcypher
*/
import { useEffect } from "react";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { GuardedLink } from "@/components/navigation/GuardedLink";
import { UnsavedNavigationGuardProvider, useUnsavedNavigationGuard } from "@/lib/navigation/UnsavedNavigationGuard";

const navigation = vi.hoisted(() => ({ push: vi.fn() }));

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: navigation.push }),
}));

vi.mock("next/link", () => ({
  default: ({ href, target, children, onNavigate, onClick, ...props }: any) => (
    <a
      href="#"
      data-destination={String(href)}
      target={target}
      {...props}
      onClick={(event) => {
        onClick?.(event);
        if (event.defaultPrevented || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || target === "_blank") return;
        const destination = new URL(String(href), window.location.origin);
        if (destination.origin !== window.location.origin) return;
        event.preventDefault();
        let blocked = false;
        onNavigate?.({ preventDefault: () => { blocked = true; event.preventDefault(); } });
        if (!blocked) navigation.push(String(href));
      }}
    >
      {children}
    </a>
  ),
}));

function ActiveDraftGuard({ active = true }: { active?: boolean }) {
  const { registerUnsavedNavigationGuard } = useUnsavedNavigationGuard();
  useEffect(() => active ? registerUnsavedNavigationGuard("Unsaved Hook policy draft") : undefined, [active, registerUnsavedNavigationGuard]);
  return null;
}

describe("GuardedLink", () => {
  beforeEach(() => {
    navigation.push.mockReset();
    window.history.replaceState({}, "", "/");
    vi.stubGlobal("confirm", vi.fn().mockReturnValue(false));
  });
  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    window.history.replaceState({}, "", "/");
  });

  it("cancels a supported internal route when the user stays and permits it after confirmation", () => {
    const confirm = window.confirm as ReturnType<typeof vi.fn>;
    render(
      <UnsavedNavigationGuardProvider>
        <ActiveDraftGuard />
        <GuardedLink href="/inbox">Inbox</GuardedLink>
      </UnsavedNavigationGuardProvider>,
    );

    const link = screen.getByRole("link", { name: "Inbox" });
    fireEvent.click(link);
    expect(confirm).toHaveBeenCalledOnce();
    expect(navigation.push).not.toHaveBeenCalled();

    confirm.mockReturnValue(true);
    fireEvent.click(link);
    expect(confirm).toHaveBeenCalledTimes(2);
    expect(navigation.push).toHaveBeenCalledOnce();
  });

  it("removes the active prompt when the draft is cleared or its owner unmounts", () => {
    const confirm = window.confirm as ReturnType<typeof vi.fn>;
    const view = render(
      <UnsavedNavigationGuardProvider>
        <ActiveDraftGuard />
        <GuardedLink href="/inbox">Inbox</GuardedLink>
      </UnsavedNavigationGuardProvider>,
    );
    const link = screen.getByRole("link", { name: "Inbox" });
    fireEvent.click(link);
    expect(confirm).toHaveBeenCalledOnce();
    expect(navigation.push).not.toHaveBeenCalled();

    view.rerender(
      <UnsavedNavigationGuardProvider>
        <ActiveDraftGuard active={false} />
        <GuardedLink href="/inbox">Inbox</GuardedLink>
      </UnsavedNavigationGuardProvider>,
    );
    fireEvent.click(link);
    expect(confirm).toHaveBeenCalledOnce();
    expect(navigation.push).toHaveBeenCalledOnce();

    view.rerender(
      <UnsavedNavigationGuardProvider>
        <ActiveDraftGuard />
        <GuardedLink href="/inbox">Inbox</GuardedLink>
      </UnsavedNavigationGuardProvider>,
    );
    view.rerender(
      <UnsavedNavigationGuardProvider>
        <GuardedLink href="/inbox">Inbox</GuardedLink>
      </UnsavedNavigationGuardProvider>,
    );
    fireEvent.click(link);
    expect(confirm).toHaveBeenCalledOnce();
    expect(navigation.push).toHaveBeenCalledOnce();
  });

  it("does not prompt for external links or links opened in a new tab", () => {
    const confirm = window.confirm as ReturnType<typeof vi.fn>;
    render(
      <UnsavedNavigationGuardProvider>
        <ActiveDraftGuard />
        <GuardedLink href="https://docs.example.test/" onClick={(event) => event.preventDefault()}>External help</GuardedLink>
        <GuardedLink href="/inbox" target="_blank" onClick={(event) => event.preventDefault()}>Open another tab</GuardedLink>
      </UnsavedNavigationGuardProvider>,
    );

    fireEvent.click(screen.getByRole("link", { name: "External help" }));
    fireEvent.click(screen.getByRole("link", { name: "Open another tab" }));
    expect(confirm).not.toHaveBeenCalled();
  });

  it("runs app-shell navigation callbacks only after the guard approves", () => {
    const confirm = window.confirm as ReturnType<typeof vi.fn>;
    const onNavigate = vi.fn();
    render(
      <UnsavedNavigationGuardProvider>
        <ActiveDraftGuard />
        <GuardedLink href="/inbox" onNavigate={onNavigate}>Inbox</GuardedLink>
      </UnsavedNavigationGuardProvider>,
    );
    const link = screen.getByRole("link", { name: "Inbox" });

    fireEvent.click(link);
    expect(onNavigate).not.toHaveBeenCalled();

    confirm.mockReturnValue(true);
    fireEvent.click(link);
    expect(onNavigate).toHaveBeenCalledOnce();
    expect(navigation.push).toHaveBeenCalledWith("/inbox");
  });

  it("guards arbitrary same-origin content anchors before their click effects and routes once after approval", () => {
    const confirm = window.confirm as ReturnType<typeof vi.fn>;
    const onClick = vi.fn();
    render(
      <UnsavedNavigationGuardProvider>
        <ActiveDraftGuard />
        <a href="/work-items/wi-7" onClick={onClick}>Open work item</a>
      </UnsavedNavigationGuardProvider>,
    );

    fireEvent.click(screen.getByRole("link", { name: "Open work item" }));
    expect(confirm).toHaveBeenCalledOnce();
    expect(onClick).not.toHaveBeenCalled();
    expect(navigation.push).not.toHaveBeenCalled();

    confirm.mockReturnValue(true);
    fireEvent.click(screen.getByRole("link", { name: "Open work item" }));
    expect(confirm).toHaveBeenCalledTimes(2);
    expect(onClick).toHaveBeenCalledOnce();
    expect(navigation.push).toHaveBeenCalledOnce();
    expect(navigation.push).toHaveBeenCalledWith("/work-items/wi-7");
  });

  it("leaves external, modified, new-tab, download, hash-only, and normalized same-route anchors alone", () => {
    const confirm = window.confirm as ReturnType<typeof vi.fn>;
    const onClick = vi.fn();
    window.history.replaceState({}, "", "/settings/advanced/hooks?a=1&b=2");
    render(
      <UnsavedNavigationGuardProvider>
        <ActiveDraftGuard />
        <a href="https://outside.example.test/page" onClick={(event) => { event.preventDefault(); onClick(); }}>External</a>
        <a href="/work-items/wi-7" target="_blank" onClick={(event) => { event.preventDefault(); onClick(); }}>New tab</a>
        <a href="/work-items/wi-7" download onClick={(event) => { event.preventDefault(); onClick(); }}>Download</a>
        <a href="/work-items/wi-7" onClick={(event) => { event.preventDefault(); onClick(); }}>Modified</a>
        <a href="/settings/advanced/hooks?a=1&b=2#rules" onClick={(event) => { event.preventDefault(); onClick(); }}>Hash only</a>
        <a href="/settings/advanced/hooks?b=2&a=1" onClick={(event) => { event.preventDefault(); onClick(); }}>Normalized query</a>
      </UnsavedNavigationGuardProvider>,
    );

    fireEvent.click(screen.getByRole("link", { name: "External" }));
    fireEvent.click(screen.getByRole("link", { name: "New tab" }));
    fireEvent.click(screen.getByRole("link", { name: "Download" }));
    fireEvent.click(screen.getByRole("link", { name: "Modified" }), { ctrlKey: true });
    fireEvent.click(screen.getByRole("link", { name: "Hash only" }));
    fireEvent.click(screen.getByRole("link", { name: "Normalized query" }));

    expect(confirm).not.toHaveBeenCalled();
    expect(navigation.push).not.toHaveBeenCalled();
    expect(onClick).toHaveBeenCalledTimes(6);
  });
});
