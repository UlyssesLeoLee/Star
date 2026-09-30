/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx",type:"file",language:"tsx"}),
  (suite:Function {name:"HookPolicyPage unauthenticated state tests",type:"function",signature:"describe('HookPolicyPage', ...)",visibility:"private",complexity:"simple"}),
  (failClosed:Function {name:"unauthenticated Hook page case",type:"function",signature:"it('does not show seeded policies without a host session', ...)",visibility:"private",complexity:"simple"}),
  (render:Function {name:"Testing Library render",type:"function",signature:"render(<HookPolicyPage />)",visibility:"private",complexity:"simple"}),
  (file)-[:CONTAINS]->(suite),(suite)-[:CONTAINS]->(failClosed),(failClosed)-[:CALLS]->(render);
*/

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import HookPolicyPage from "./HookPolicyPage";

describe("HookPolicyPage", () => {
  it("does not show seeded policies without a host session", () => {
    render(<HookPolicyPage />);

    expect(screen.getByTestId("hook-auth-required")).toBeInTheDocument();
    expect(screen.getByText(/当前不显示策略样例/)).toBeInTheDocument();
    expect(screen.queryByTestId("hook-policy-page")).not.toBeInTheDocument();
  });
});
