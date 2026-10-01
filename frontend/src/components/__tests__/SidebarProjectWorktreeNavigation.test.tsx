// CYPHER STRUCTURAL MANIFEST
// CREATE
//   (file:File {name:"SidebarProjectWorktreeNavigation.test.tsx",type:"file",language:"typescript"}),
//   (module:Module {name:"SidebarProjectWorktreeNavigation",type:"module",language:"typescript"}),
//   (suite:Function {name:"Project Worktree Index sidebar navigation suite",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
//   (test:Function {name:"project scope opens the authorized Project Worktree Index route",type:"function",language:"typescript",visibility:"private",complexity:"simple"}),
//   (projectId:Variable {name:"projectId",type:"variable",language:"typescript"}),
//   (mockUsePathname:Function {name:"mockUsePathname",type:"function",language:"typescript"}),
//   (mockUseSearchParams:Function {name:"mockUseSearchParams",type:"function",language:"typescript"}),
//   (navigationMockFactory:Function {name:"navigationMockFactory",type:"function",language:"typescript"}),
//   (usePathnameMock:Function {name:"usePathnameMock",type:"function",language:"typescript"}),
//   (useSearchParamsMock:Function {name:"useSearchParamsMock",type:"function",language:"typescript"}),
//   (describe:Function {name:"describe",type:"function",language:"typescript"}),
//   (it:Function {name:"it",type:"function",language:"typescript"}),
//   (viMock:Function {name:"vi.mock",type:"function",language:"typescript"}),
//   (viFn:Function {name:"vi.fn",type:"function",language:"typescript"}),
//   (render:Function {name:"render",type:"function",language:"typescript"}),
//   (fireEvent:Function {name:"fireEvent.click",type:"function",language:"typescript"}),
//   (getByTestId:Function {name:"screen.getByTestId",type:"function",language:"typescript"}),
//   (expect:Function {name:"expect",type:"function",language:"typescript"}),
//   (toHaveAttribute:Function {name:"toHaveAttribute",type:"function",language:"typescript"}),
//   (urlSearchParams:Function {name:"URLSearchParams",type:"function",language:"typescript"}),
//   (encodeURIComponent:Function {name:"encodeURIComponent",type:"function",language:"typescript"}),
//   (Sidebar:Function {name:"Sidebar",type:"function",language:"typescript"}),
//   (I18nProvider:Function {name:"I18nProvider",type:"function",language:"typescript"}),
//   (useNavStore:Function {name:"useNavStore.setState",type:"function",language:"typescript"}),
//   (file)-[:CONTAINS]->(module),(module)-[:CONTAINS]->(suite),(suite)-[:CONTAINS]->(test),
//   (module)-[:CONTAINS]->(projectId),(module)-[:CONTAINS]->(mockUsePathname),(module)-[:CONTAINS]->(mockUseSearchParams),
//   (module)-[:CONTAINS]->(navigationMockFactory),(navigationMockFactory)-[:CALLS]->(mockUsePathname),(navigationMockFactory)-[:CALLS]->(mockUseSearchParams),
//   (navigationMockFactory)-[:CALLS]->(usePathnameMock),(navigationMockFactory)-[:CALLS]->(useSearchParamsMock),(navigationMockFactory)-[:CALLS]->(viFn),
//   (usePathnameMock)-[:CALLS]->(mockUsePathname),(useSearchParamsMock)-[:CALLS]->(mockUseSearchParams),
//   (mockUsePathname)-[:CALLS]->(viFn),(mockUseSearchParams)-[:CALLS]->(viFn),(mockUseSearchParams)-[:CALLS]->(urlSearchParams),
//   (module)-[:CALLS]->(viMock),(viMock)-[:CALLS]->(navigationMockFactory),
//   (suite)-[:CALLS]->(describe),(suite)-[:CALLS]->(it),(test)-[:CALLS]->(useNavStore),(test)-[:CALLS]->(render),
//   (test)-[:CALLS]->(I18nProvider),(test)-[:CALLS]->(Sidebar),(test)-[:CALLS]->(fireEvent),(test)-[:CALLS]->(getByTestId),
//   (test)-[:CALLS]->(expect),(test)-[:CALLS]->(toHaveAttribute),(test)-[:CALLS]->(mockUsePathname),(test)-[:CALLS]->(mockUseSearchParams),
//   (test)-[:CALLS]->(urlSearchParams),(test)-[:CALLS]->(encodeURIComponent),(test)-[:USES]->(projectId);

import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { Sidebar } from "../Sidebar";
import { useNavStore } from "@/lib/nav/navStore";
import { I18nProvider } from "@/lib/i18n";

const projectId = "prj-authorized-123";
const mockUsePathname = vi.fn(() => "/worktree");
const mockUseSearchParams = vi.fn(() => new URLSearchParams(""));

vi.mock("next/navigation", () => ({
  usePathname: () => mockUsePathname(),
  useSearchParams: () => mockUseSearchParams(),
}));

describe("Project Worktree Index sidebar navigation", () => {
  it("project scope opens the authorized Project Worktree Index route", () => {
    useNavStore.setState({
      sidebarItemIds: ["inbox", "issues", "projects", "agents"],
      pinnedViewIds: ["kanban", "timeline"],
      headerTabIds: ["inbox", "issues", "projects", "agents", "analytics"],
      sidebarFold: "expanded",
      sidebarScope: "main",
      selectedProjectId: projectId,
      isMatrixOpen: false,
    });

    render(
      <I18nProvider initialLanguage="zh-CN">
        <Sidebar />
      </I18nProvider>,
    );
    fireEvent.click(screen.getByTestId("sidebar-scope-project"));

    const link = screen.getByTestId("sidebar-project-worktree-index");
    expect(link).toHaveAttribute(
      "href",
      `/worktree?project_id=${encodeURIComponent(projectId)}`,
    );
    expect(link).toHaveAttribute("data-active", "true");
  });
});
