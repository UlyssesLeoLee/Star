/*
@cypher schema=1 source_sha256=0cf700f5231fafd927b7ba1d28c43de44b01cfb1dec1dce7c9b703cd9f54f138
MERGE (self:File {path:"frontend/src/components/UserMenu.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/UserMenu.test.tsx::UserMenu navigation tests",kind:"suite"})
MERGE (stayCase:Symbol {id:"frontend/src/components/UserMenu.test.tsx::keeps the menu open when dirty-draft navigation is canceled case",kind:"test"})
MERGE (draft:Symbol {id:"frontend/src/components/UserMenu.test.tsx::DirtyDraft",kind:"component"})
MERGE (menu:Symbol {id:"frontend/src/components/UserMenu.tsx::UserMenu",kind:"component"})
MERGE (guard:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::UnsavedNavigationGuardProvider",kind:"component"})
MERGE (i18n:Symbol {id:"frontend/src/lib/i18n/I18nProvider.tsx::I18nProvider",kind:"component"})
MERGE (guardHook:Symbol {id:"frontend/src/lib/navigation/UnsavedNavigationGuard.tsx::useUnsavedNavigationGuard",kind:"function"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (self)-[:IMPORTS]->(menu)
MERGE (self)-[:IMPORTS]->(guard)
MERGE (self)-[:IMPORTS]->(i18n)
MERGE (self)-[:IMPORTS]->(guardHook)
MERGE (suite)-[:DEFINES]->(stayCase)
MERGE (suite)-[:DEFINES]->(draft)
MERGE (stayCase)-[:TESTS]->(menu)
MERGE (draft)-[:CALLS]->(guardHook)
@endcypher
*/
import { useEffect } from "react";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { UserMenu } from "@/components/UserMenu";
import { I18nProvider } from "@/lib/i18n";
import {
  UnsavedNavigationGuardProvider,
  useUnsavedNavigationGuard,
} from "@/lib/navigation/UnsavedNavigationGuard";

const navigation = vi.hoisted(() => ({ push: vi.fn() }));

vi.mock("next/link", () => ({
  default: ({ href, children, onNavigate, onClick, ...props }: any) => (
    <a
      href="#"
      data-destination={String(href)}
      {...props}
      onClick={(event) => {
        onClick?.(event);
        if (event.defaultPrevented) return;

        // Model Next.js Link's client navigation without letting jsdom perform
        // a document navigation for an anchor element.
        event.preventDefault();
        let canceled = false;
        onNavigate?.({
          preventDefault: () => {
            canceled = true;
          },
        });
        if (!canceled) navigation.push(String(href));
      }}
    >
      {children}
    </a>
  ),
}));

function DirtyDraft() {
  const { registerUnsavedNavigationGuard } = useUnsavedNavigationGuard();
  useEffect(
    () => registerUnsavedNavigationGuard("Unsaved Hook policy draft"),
    [registerUnsavedNavigationGuard],
  );
  return null;
}

describe("UserMenu guarded navigation", () => {
  beforeEach(() => {
    navigation.push.mockReset();
    vi.spyOn(window, "confirm").mockReturnValue(false);
  });

  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it("keeps the menu open when dirty-draft navigation is canceled", () => {
    const confirm = vi.mocked(window.confirm);
    render(
      <I18nProvider initialLanguage="en">
        <UnsavedNavigationGuardProvider>
          <DirtyDraft />
          <UserMenu />
        </UnsavedNavigationGuardProvider>
      </I18nProvider>,
    );

    fireEvent.click(screen.getByTestId("user-avatar"));
    const menu = screen.getByRole("menu");
    const profileLink = menu.querySelector<HTMLAnchorElement>(
      '[data-destination="/settings/profile"]',
    );
    expect(profileLink).not.toBeNull();

    fireEvent.click(profileLink!);
    expect(confirm).toHaveBeenCalledOnce();
    expect(navigation.push).not.toHaveBeenCalled();
    expect(screen.getByRole("menu")).toBe(menu);

    confirm.mockReturnValue(true);
    fireEvent.click(profileLink!);
    expect(confirm).toHaveBeenCalledTimes(2);
    expect(navigation.push).toHaveBeenCalledWith("/settings/profile");
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });
});
