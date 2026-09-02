"use client";

/**
 * Label - shadcn 风格 fallback (per docs/automation-design.md v0.2 §12.4)
 */

import * as React from "react";
import { clsx } from "clsx";

export const Label = React.forwardRef<HTMLLabelElement, React.LabelHTMLAttributes<HTMLLabelElement>>(
  ({ className, ...props }, ref) => (
    <label
      ref={ref}
      className={clsx("text-sm font-medium leading-none", className)}
      {...props}
    />
  )
);
Label.displayName = "Label";
