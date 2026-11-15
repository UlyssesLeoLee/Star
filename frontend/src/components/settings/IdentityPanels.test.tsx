/*
@cypher schema=1 source_sha256=7f9116e7887ed44feae466a1ec55b5e7fd68c9468dd91b7ac83d30deeb687e97
MERGE (self:File {path:"frontend/src/components/settings/IdentityPanels.test.tsx"})
MERGE (suite:Symbol {id:"frontend/src/components/settings/IdentityPanels.test.tsx::IdentityPanels",kind:"suite"})
MERGE (panels:File {path:"frontend/src/components/settings/IdentityPanels.tsx"})
MERGE (self)-[:IMPORTS]->(panels)
MERGE (self)-[:DEFINES]->(suite)
MERGE (suite)-[:TESTS]->(panels)
@endcypher
*/
// =====================================================================
// IdentityPanels 4 面板渲染测试 (per 2026-11-15)
// 断言全部基于 store 真实 seed 集合, 数量用 seed 派生而非硬编码。
// =====================================================================
import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import * as seed from "@/lib/seed";
import { useStore } from "@/lib/store";
import { IdentityPanel, ProfilePanel, PermissionPanel, TenantPanel } from "./IdentityPanels";

beforeEach(() => {
  // 固定为 seed 真实集合, 隔离其他测试写入 localStorage 的持久化状态
  useStore.setState({
    identities: seed.identities,
    tenants: seed.tenants,
    projects: seed.projects,
    permissionRules: seed.permissionRules,
    permissionSchemes: seed.permissionSchemes,
    auditEvents: seed.auditEvents,
    notifications: seed.notifications,
    localRuntimes: seed.localRuntimes,
    savedSearches: seed.savedSearches,
  });
});

describe("IdentityPanels", () => {
  describe("IdentityPanel 成员与身份", () => {
    it("渲染面板容器 + seed 全部成员行", () => {
      render(<IdentityPanel />);
      expect(screen.getByTestId("settings-identity-panel")).toBeTruthy();
      for (const m of seed.identities) {
        expect(screen.getByTestId(`settings-identity-row-${m.id}`)).toBeTruthy();
      }
      expect(screen.getAllByTestId(/^settings-identity-row-/).length).toBe(seed.identities.length);
    });

    it("角色徽标: usr-001=owner (本地默认), usr-007=developer (审计 role.grant), 其余未标注", () => {
      render(<IdentityPanel />);
      expect(screen.getByTestId("settings-identity-role-usr-001").textContent).toBe("owner");
      expect(screen.getByTestId("settings-identity-role-usr-007").textContent).toBe("developer");
      expect(screen.getByTestId("settings-identity-role-usr-002").textContent).toBe("未标注");
      expect(screen.getByTestId("settings-identity-role-usr-003").textContent).toBe("未标注");
    });

    it("状态徽标: 活跃 / 已邀请 / 已禁用 按 seed.status 展示", () => {
      render(<IdentityPanel />);
      expect(screen.getByTestId("settings-identity-status-usr-001").textContent).toContain("活跃");
      expect(screen.getByTestId("settings-identity-status-usr-007").textContent).toContain("已邀请");
      expect(screen.getByTestId("settings-identity-status-usr-009").textContent).toContain("已禁用");
    });

    it("移除按钮全部禁用并标注未接入原因", () => {
      render(<IdentityPanel />);
      for (const m of seed.identities) {
        const btn = screen.getByTestId(`settings-identity-remove-${m.id}`) as HTMLButtonElement;
        expect(btn.disabled).toBe(true);
        expect(btn.getAttribute("title") ?? "").toContain("未接入");
      }
      expect(screen.getByTestId("settings-identity-disabled-reason").textContent ?? "").toContain("未接入");
    });
  });

  describe("ProfilePanel 个人资料", () => {
    it("渲染当前用户 Ulysses: 头像首字母 / 姓名 / 邮箱 (store.identities usr-001)", () => {
      render(<ProfilePanel />);
      expect(screen.getByTestId("settings-profile-panel")).toBeTruthy();
      expect(screen.getByTestId("settings-profile-name").textContent).toBe("Ulysses");
      expect(screen.getByTestId("settings-profile-email").textContent).toBe("user1@acme.studio");
      expect(screen.getByTestId("settings-profile-avatar").textContent).toBe("U");
    });

    it("关联数据来自 store 真实集合 (绑定设备 / 已保存搜索 / 收件箱通知)", () => {
      render(<ProfilePanel />);
      const expectedDevices = seed.localRuntimes.filter((r) => r.bound_user_id === "usr-001");
      const expectedSearches = seed.savedSearches.filter((s) => s.created_by === "usr-001");
      const expectedInbox = seed.notifications.filter((n) => n.recipient_id === "usr-001");
      expect(expectedDevices.length).toBeGreaterThan(0);
      expect(screen.getByTestId("settings-profile-devices").textContent).toContain("Ulysses-MBP");
      expect(screen.getByTestId("settings-profile-saved-searches").textContent).toBe(
        String(expectedSearches.length),
      );
      expect(screen.getByTestId("settings-profile-inbox-count").textContent).toBe(
        String(expectedInbox.length),
      );
    });

    it("偏好只读 + 后端未接入标注", () => {
      render(<ProfilePanel />);
      expect(screen.getByText("偏好 (本地默认, 只读)")).toBeTruthy();
      expect(screen.getByTestId("settings-profile-backend-note").textContent ?? "").toContain("未接入");
    });
  });

  describe("PermissionPanel 角色与权限", () => {
    it("渲染 owner/admin/member 角色列表 (映射真实系统角色)", () => {
      render(<PermissionPanel />);
      expect(screen.getByTestId("settings-permission-panel")).toBeTruthy();
      expect(screen.getByTestId("settings-permission-role-owner").textContent).toContain("tenant_admin");
      expect(screen.getByTestId("settings-permission-role-admin").textContent).toContain("project_admin");
      expect(screen.getByTestId("settings-permission-role-member").textContent).toContain("developer");
      expect(screen.getByTestId("settings-permission-role-member").textContent).toContain("viewer");
    });

    it("权限矩阵由 store.permissionRules 派生: owner.admin=允许, admin.write=部分, member.read=允许, owner.read=—", () => {
      render(<PermissionPanel />);
      expect(screen.getByTestId("settings-permission-cell-owner-admin").textContent).toBe("允许");
      expect(screen.getByTestId("settings-permission-cell-admin-write").textContent).toBe("部分");
      expect(screen.getByTestId("settings-permission-cell-member-read").textContent).toBe("允许");
      expect(screen.getByTestId("settings-permission-cell-member-admin").textContent).toBe("允许");
      expect(screen.getByTestId("settings-permission-cell-owner-read").textContent).toBe("—");
      expect(screen.getByTestId("settings-permission-cell-member-delete").textContent).toBe("—");
    });

    it("权限矩阵只读: 面板内无 button / input 控件", () => {
      render(<PermissionPanel />);
      const panel = screen.getByTestId("settings-permission-panel");
      expect(panel.querySelectorAll("button, input").length).toBe(0);
    });

    it("权限方案数量来自 store.permissionSchemes", () => {
      render(<PermissionPanel />);
      expect(screen.getByTestId("settings-permission-scheme-count").textContent).toContain(
        String(seed.permissionSchemes.length),
      );
    });
  });

  describe("TenantPanel 租户", () => {
    it("统计卡显示真实 store 数据 (租户名 / 项目数 / 成员数 / 席位)", () => {
      render(<TenantPanel />);
      expect(screen.getByTestId("settings-tenant-panel")).toBeTruthy();
      expect(screen.getByTestId("settings-tenant-name").textContent).toBe("ACME Studio");
      expect(screen.getByTestId("settings-tenant-projects-num").textContent).toBe(
        String(seed.projects.length),
      );
      expect(screen.getByTestId("settings-tenant-members-num").textContent).toBe(
        String(seed.identities.length),
      );
      expect(screen.getByTestId("settings-tenant-seats-num").textContent).toBe(
        `${seed.identities.length} / ${seed.tenants[0].seat_limit}`,
      );
    });

    it("邀请成员: 输入框可本地输入, 发送按钮禁用并标注未接入原因", () => {
      render(<TenantPanel />);
      const input = screen.getByTestId("settings-tenant-invite-input") as HTMLInputElement;
      fireEvent.change(input, { target: { value: "nova@acme.studio" } });
      expect(input.value).toBe("nova@acme.studio");
      const submit = screen.getByTestId("settings-tenant-invite-submit") as HTMLButtonElement;
      expect(submit.disabled).toBe(true);
      expect(screen.getByTestId("settings-tenant-invite-reason").textContent ?? "").toContain("未接入");
    });
  });
});
