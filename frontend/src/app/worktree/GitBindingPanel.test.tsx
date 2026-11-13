/*
@cypher schema=1 source_sha256=68e2468cc6306081f3b2309f010bc206fac1ba6fa6c4f36b4f454c1f11195010
MERGE (self:File {path:"frontend/src/app/worktree/GitBindingPanel.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/app/worktree/GitBindingPanel.test.tsx::GitBindingPanel",kind:"suite"})
MERGE (panel:Symbol {id:"frontend/src/app/worktree/GitBindingPanel.tsx::GitBindingPanel",kind:"component"})
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(panel)
@endcypher
*/
// per WBS §23.34: GitBindingPanel 组件测试 (未绑定 → provider 卡; 绑定 → 列表 + 解绑)
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { I18nProvider } from "@/lib/i18n";
import { GitBindingPanel } from "./GitBindingPanel";
import { resetGitBindingsForTest, getGitBindings } from "@/lib/group/gitBindingStore";

function renderPanel() {
  return render(<I18nProvider initialLanguage="zh-CN"><GitBindingPanel /></I18nProvider>);
}

describe("GitBindingPanel (WBS §23.34)", () => {
  beforeEach(() => {
    resetGitBindingsForTest();
    window.localStorage.clear();
  });
  afterEach(() => {
    cleanup();
    resetGitBindingsForTest();
    window.localStorage.clear();
  });

  it("shows the 3 provider cards when nothing is bound", () => {
    renderPanel();
    expect(screen.getByTestId("git-provider-github")).toBeTruthy();
    expect(screen.getByTestId("git-provider-gitlab")).toBeTruthy();
    expect(screen.getByTestId("git-provider-self-host")).toBeTruthy();
  });

  it("binds a GitHub repository through the form", () => {
    renderPanel();
    fireEvent.click(screen.getByTestId("git-provider-github"));
    fireEvent.change(screen.getByTestId("git-form-full-name"), { target: { value: "UlyssesLeoLee/Star" } });
    fireEvent.change(screen.getByTestId("git-form-local-path"), { target: { value: "D:/Star" } });
    fireEvent.click(screen.getByTestId("git-form-submit"));

    const bindings = getGitBindings();
    expect(bindings).toHaveLength(1);
    expect(bindings[0].provider).toBe("github");
    expect(bindings[0].base_url).toBe("https://github.com");
    expect(bindings[0].local_path).toBe("D:/Star");
  });

  it("requires base_url for self-hosted Git", () => {
    renderPanel();
    fireEvent.click(screen.getByTestId("git-provider-self-host"));
    fireEvent.change(screen.getByTestId("git-form-full-name"), { target: { value: "me/myproject" } });
    fireEvent.change(screen.getByTestId("git-form-local-path"), { target: { value: "E:/src" } });
    fireEvent.click(screen.getByTestId("git-form-submit"));
    expect(screen.getByTestId("git-form-error")).toBeTruthy();
    expect(getGitBindings()).toHaveLength(0);

    fireEvent.change(screen.getByTestId("git-form-base-url"), { target: { value: "https://git.example.com" } });
    fireEvent.click(screen.getByTestId("git-form-submit"));
    expect(getGitBindings()).toHaveLength(1);
    expect(getGitBindings()[0].base_url).toBe("https://git.example.com");
  });

  it("rejects malformed full_name (no owner/repo)", () => {
    renderPanel();
    fireEvent.click(screen.getByTestId("git-provider-github"));
    fireEvent.change(screen.getByTestId("git-form-full-name"), { target: { value: "no-slash" } });
    fireEvent.change(screen.getByTestId("git-form-local-path"), { target: { value: "D:/x" } });
    fireEvent.click(screen.getByTestId("git-form-submit"));
    expect(screen.getByTestId("git-form-error")).toHaveTextContent(/owner\/repo/);
    expect(getGitBindings()).toHaveLength(0);
  });

  it("lists bindings and removes one", () => {
    renderPanel();
    fireEvent.click(screen.getByTestId("git-provider-github"));
    fireEvent.change(screen.getByTestId("git-form-full-name"), { target: { value: "a/b" } });
    fireEvent.change(screen.getByTestId("git-form-local-path"), { target: { value: "D:/a" } });
    fireEvent.click(screen.getByTestId("git-form-submit"));

    expect(screen.getByTestId("git-binding-list")).toBeTruthy();
    const id = getGitBindings()[0].id;
    fireEvent.click(screen.getByTestId(`git-binding-remove-${id}`));
    expect(getGitBindings()).toHaveLength(0);
  });
});
