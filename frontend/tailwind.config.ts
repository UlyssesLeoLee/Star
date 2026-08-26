import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./src/**/*.{ts,tsx,mdx}"],
  theme: {
    extend: {
      colors: {
        bg: { DEFAULT: "#0b0d10", soft: "#11151b", card: "#161b22" },
        line: "#21262d",
        ink: { DEFAULT: "#e6edf3", dim: "#8b949e", mute: "#6e7681" },
        accent: { DEFAULT: "#2f81f7", soft: "#1f6feb" },
        ok: "#3fb950",
        warn: "#d29922",
        err: "#f85149",
        info: "#79c0ff",
      },
      fontFamily: {
        mono: ['"JetBrains Mono"', '"SF Mono"', "Menlo", "Consolas", "monospace"],
      },
    },
  },
  plugins: [],
};
export default config;
