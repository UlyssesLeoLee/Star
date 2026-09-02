/**
 * /automation-debug layout (per docs/automation-design.md v0.2 §12)
 */

import { ReactNode } from "react";

export const metadata = {
  title: "Automation Debug Console",
  description: "13 份 Python 脚本 + 5 套 unittest 调试控制台",
};

export default function AutomationDebugLayout({ children }: { children: ReactNode }) {
  return children;
}
