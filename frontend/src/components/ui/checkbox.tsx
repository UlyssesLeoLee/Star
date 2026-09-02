"use client";

/**
 * Checkbox - shadcn 风格 fallback (per docs/automation-design.md v0.2 §12.4)
 */

import * as React from "react";
import { clsx } from "clsx";

export const Checkbox = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ className, ...props }, ref) => (
    <input
      type="checkbox"
      ref={ref}
      className={clsx(
        "h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-ring focus:ring-offset-2",
        className
      )}
      {...props}
    />
  )
);
Checkbox.displayName = "Checkbox";
