"use client";

/**
 * Badge - shadcn 风格 fallback (per docs/automation-design.md v0.2 §12.4)
 * 不依赖 cva, 用 clsx 直接拼
 */

import * as React from "react";
import { clsx } from "clsx";

const variantClasses: Record<string, string> = {
  default: "border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
  secondary: "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
  destructive: "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
  outline: "text-foreground",
};

const baseClasses =
  "inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2";

export interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: keyof typeof variantClasses;
}

export function Badge({ className, variant = "default", ...props }: BadgeProps) {
  return <div className={clsx(baseClasses, variantClasses[variant], className)} {...props} />;
}
