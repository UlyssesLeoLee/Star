/**
 * cn - tailwind class 合并 helper (per docs/automation-design.md v0.2 §12.4)
 */

import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
