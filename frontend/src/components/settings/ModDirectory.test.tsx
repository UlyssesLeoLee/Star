/*
@cypher schema=1 source_sha256=3fd52c64c02009c05bd62e6f2bcad377e1de151328529e0f4bbba0467beb2164
MERGE (self:File {path:"frontend/src/components/settings/ModDirectory.test.tsx"})
MERGE (modDirectory:Symbol {id:"frontend/src/components/settings/ModDirectory.tsx::ModDirectory",kind:"component"})
MERGE (suite:Symbol {id:"frontend/src/components/settings/ModDirectory.test.tsx::ModDirectory 本地 Mod 目录",kind:"suite"})
MERGE (badgeTest:Symbol {id:"frontend/src/components/settings/ModDirectory.test.tsx::renders honest local-preview badge and empty installed state",kind:"test"})
MERGE (browseTest:Symbol {id:"frontend/src/components/settings/ModDirectory.test.tsx::renders local directory mods with name type abilities version",kind:"test"})
MERGE (installTest:Symbol {id:"frontend/src/components/settings/ModDirectory.test.tsx::installs a directory mod into the installed list",kind:"test"})
MERGE (fileInstallTest:Symbol {id:"frontend/src/components/settings/ModDirectory.test.tsx::installs a valid mod from file input",kind:"test"})
MERGE (fileInvalidJsonTest:Symbol {id:"frontend/src/components/settings/ModDirectory.test.tsx::shows error when file content is not valid JSON",kind:"test"})
MERGE (fileMissingFieldsTest:Symbol {id:"frontend/src/components/settings/ModDirectory.test.tsx::shows error when file payload misses required fields",kind:"test"})
MERGE (templateTest:Symbol {id:"frontend/src/components/settings/ModDirectory.test.tsx::creates a skeleton mod from the template form",kind:"test"})
MERGE (templateDisabledTest:Symbol {id:"frontend/src/components/settings/ModDirectory.test.tsx::disables template button while name is empty",kind:"test"})
MERGE (self)-[:IMPORTS]->(modDirectory)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:DEFINES]->(badgeTest)
MERGE (suite)-[:DEFINES]->(browseTest)
MERGE (suite)-[:DEFINES]->(installTest)
MERGE (suite)-[:DEFINES]->(fileInstallTest)
MERGE (suite)-[:DEFINES]->(fileInvalidJsonTest)
MERGE (suite)-[:DEFINES]->(fileMissingFieldsTest)
MERGE (suite)-[:DEFINES]->(templateTest)
MERGE (suite)-[:DEFINES]->(templateDisabledTest)
MERGE (badgeTest)-[:TESTS]->(modDirectory)
MERGE (browseTest)-[:TESTS]->(modDirectory)
MERGE (installTest)-[:TESTS]->(modDirectory)
MERGE (fileInstallTest)-[:TESTS]->(modDirectory)
MERGE (fileInvalidJsonTest)-[:TESTS]->(modDirectory)
MERGE (fileMissingFieldsTest)-[:TESTS]->(modDirectory)
MERGE (templateTest)-[:TESTS]->(modDirectory)
MERGE (templateDisabledTest)-[:TESTS]->(modDirectory)
@endcypher
*/
import { afterEach, describe, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { ModDirectory } from "./ModDirectory";

function jsonFile(content: string): File {
  return new File([content], "test.mod.json", { type: "application/json" });
}

function pickFile(content: string) {
  const input = screen.getByTestId("mod-dir-file-install") as HTMLInputElement;
  fireEvent.change(input, { target: { files: [jsonFile(content)] } });
}

describe("ModDirectory 本地 Mod 目录", () => {
  afterEach(cleanup);

  it("renders honest local-preview badge and empty installed state", () => {
    render(<ModDirectory />);
    expect(screen.getByTestId("mod-directory")).toBeInTheDocument();
    expect(screen.getByTestId("mod-dir-badge")).toHaveTextContent("本地目录预览");
    expect(screen.getByTestId("mod-dir-badge")).toHaveTextContent("目录服务未接入");
    expect(screen.getByTestId("mod-dir-installed-empty")).toBeInTheDocument();
  });

  it("renders local directory mods with name type abilities version", () => {
    render(<ModDirectory />);
    const item = screen.getByTestId("mod-dir-item-midnight-theme");
    expect(item).toHaveTextContent("Midnight 主题");
    expect(item).toHaveTextContent("主题");
    expect(item).toHaveTextContent("theme.apply");
    expect(item).toHaveTextContent("v1.2.0");
    expect(screen.getByTestId("mod-dir-install-midnight-theme")).toBeEnabled();
    expect(screen.getByTestId("mod-dir-item-kanban-stats")).toBeInTheDocument();
    expect(screen.getByTestId("mod-dir-item-git-guard")).toBeInTheDocument();
    expect(screen.getByTestId("mod-dir-item-i18n-pack")).toBeInTheDocument();
  });

  it("installs a directory mod into the installed list", () => {
    render(<ModDirectory />);
    fireEvent.click(screen.getByTestId("mod-dir-install-midnight-theme"));
    expect(screen.getByTestId("mod-dir-install-midnight-theme")).toBeDisabled();
    expect(screen.getByTestId("mod-dir-install-midnight-theme")).toHaveTextContent("已安装");
    const installedItem = screen.getByTestId("mod-dir-installed-item-midnight-theme");
    expect(installedItem).toHaveTextContent("Midnight 主题");
    expect(installedItem).toHaveTextContent("来自目录");
    expect(screen.getByTestId("mod-dir-install-kanban-stats")).toBeEnabled();
  });

  it("installs a valid mod from file input", async () => {
    render(<ModDirectory />);
    pickFile(JSON.stringify({ name: "测试 Mod", type: "命令", abilities: ["git.read"], version: "1.0.0" }));
    await waitFor(() => {
      const item = screen.getByTestId("mod-dir-installed-item-file-测试-mod");
      expect(item).toHaveTextContent("测试 Mod");
      expect(item).toHaveTextContent("命令");
      expect(item).toHaveTextContent("git.read");
      expect(item).toHaveTextContent("v1.0.0");
      expect(item).toHaveTextContent("来自文件");
    });
    expect(screen.queryByTestId("mod-dir-file-error")).not.toBeInTheDocument();
  });

  it("shows error when file content is not valid JSON", async () => {
    render(<ModDirectory />);
    pickFile("{not json");
    await waitFor(() => {
      expect(screen.getByTestId("mod-dir-file-error")).toHaveTextContent("解析失败");
    });
    expect(screen.getByTestId("mod-dir-installed-empty")).toBeInTheDocument();
  });

  it("shows error when file payload misses required fields", async () => {
    render(<ModDirectory />);
    pickFile(JSON.stringify({ name: "缺字段 Mod", type: "工具", version: "1.0.0" }));
    await waitFor(() => {
      expect(screen.getByTestId("mod-dir-file-error")).toHaveTextContent("解析失败");
    });
    expect(screen.getByTestId("mod-dir-installed-empty")).toBeInTheDocument();
  });

  it("creates a skeleton mod from the template form", () => {
    render(<ModDirectory />);
    fireEvent.change(screen.getByTestId("mod-dir-template-name"), { target: { value: "我的面板" } });
    fireEvent.change(screen.getByTestId("mod-dir-template-type"), { target: { value: "面板" } });
    fireEvent.click(screen.getByTestId("mod-dir-template-new"));
    const item = screen.getByTestId("mod-dir-installed-item-tpl-我的面板");
    expect(item).toHaveTextContent("我的面板");
    expect(item).toHaveTextContent("面板");
    expect(item).toHaveTextContent("v0.1.0");
    expect(item).toHaveTextContent("无声明能力");
    expect(item).toHaveTextContent("来自模板");
    expect(screen.getByTestId("mod-dir-template-name")).toHaveValue("");
  });

  it("disables template button while name is empty", () => {
    render(<ModDirectory />);
    expect(screen.getByTestId("mod-dir-template-new")).toBeDisabled();
    fireEvent.change(screen.getByTestId("mod-dir-template-name"), { target: { value: "x" } });
    expect(screen.getByTestId("mod-dir-template-new")).toBeEnabled();
  });
});
