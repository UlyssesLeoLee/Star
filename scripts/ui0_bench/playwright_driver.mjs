// @cypher schema=1 source_sha256=a4fb6df5659c3da812e2e9cce36e0e1118ad8f17457738dce584e97873492b71
// MERGE (self:File {path:"scripts/ui0_bench/playwright_driver.mjs"})
// MERGE (playwright:ExternalService {name:"@playwright/test", version:"1.63.0"})
// MERGE (parse_args:Symbol {id:"scripts/ui0_bench/playwright_driver.mjs::parseArgs", kind:"function"})
// MERGE (capture_snapshot:Symbol {id:"scripts/ui0_bench/playwright_driver.mjs::captureSnapshot", kind:"function"})
// MERGE (make_samples:Symbol {id:"scripts/ui0_bench/playwright_driver.mjs::makeSamples", kind:"function"})
// MERGE (measure_route:Symbol {id:"scripts/ui0_bench/playwright_driver.mjs::measureRoute", kind:"function"})
// MERGE (write_samples:Symbol {id:"scripts/ui0_bench/playwright_driver.mjs::writeSamples", kind:"function"})
// MERGE (main:Symbol {id:"scripts/ui0_bench/playwright_driver.mjs::main", kind:"function"})
// MERGE (self)-[:DEPENDS_ON]->(playwright)
// MERGE (self)-[:DEFINES]->(parse_args)
// MERGE (self)-[:DEFINES]->(capture_snapshot)
// MERGE (self)-[:DEFINES]->(make_samples)
// MERGE (self)-[:DEFINES]->(measure_route)
// MERGE (self)-[:DEFINES]->(write_samples)
// MERGE (self)-[:DEFINES]->(main)
// MERGE (measure_route)-[:CALLS]->(capture_snapshot)
// MERGE (measure_route)-[:CALLS]->(make_samples)
// MERGE (main)-[:CALLS]->(parse_args)
// MERGE (main)-[:CALLS]->(measure_route)
// MERGE (main)-[:CALLS]->(write_samples)
// @endcypher

import { createRequire } from "node:module";
import { writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const require = createRequire(resolve(scriptDirectory, "../../frontend/package.json"));
const { chromium } = require("@playwright/test");

const ROUTES = [
  { path: "/projects", scenario: "projects_route" },
  { path: "/worktree", scenario: "worktree_route" },
];
const DEFAULT_WARM_REPETITIONS = 5;

function parseArgs(argv) {
  const options = {
    baseUrl: "http://127.0.0.1:3000",
    outputPath: null,
    warmRepetitions: DEFAULT_WARM_REPETITIONS,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    const value = argv[index + 1];
    if (argument === "--base-url" && value) {
      options.baseUrl = value;
      index += 1;
    } else if (argument === "--output" && value) {
      options.outputPath = resolve(value);
      index += 1;
    } else if (argument === "--warm-repetitions" && value) {
      options.warmRepetitions = Number(value);
      index += 1;
    } else {
      throw new Error(`unsupported or incomplete argument: ${argument}`);
    }
  }

  if (!options.outputPath) {
    throw new Error("--output is required");
  }
  if (!Number.isInteger(options.warmRepetitions) || options.warmRepetitions < 1 || options.warmRepetitions > 10) {
    throw new Error("--warm-repetitions must be an integer from 1 to 10");
  }

  const url = new URL(options.baseUrl);
  if (!new Set(["localhost", "127.0.0.1", "[::1]"]).has(url.hostname) || !["http:", "https:"].includes(url.protocol)) {
    throw new Error("--base-url must use localhost, 127.0.0.1, or ::1");
  }
  options.baseUrl = url.origin;
  return options;
}

async function captureSnapshot(page, cdpSession) {
  const browserMetrics = await cdpSession.send("Performance.getMetrics");
  const cdpValues = new Map(browserMetrics.metrics.map(({ name, value }) => [name, value]));
  const pageValues = await page.evaluate(() => {
    const navigation = performance.getEntriesByType("navigation")[0];
    const paint = performance.getEntriesByType("paint").find(({ name }) => name === "first-contentful-paint");
    return {
      navigationLoadMs: navigation?.loadEventEnd ?? null,
      domContentLoadedMs: navigation?.domContentLoadedEventEnd ?? null,
      firstContentfulPaintMs: paint?.startTime ?? null,
      domElementCount: document.getElementsByTagName("*").length,
    };
  });

  return {
    navigation_load_ms: pageValues.navigationLoadMs,
    dom_content_loaded_ms: pageValues.domContentLoadedMs,
    first_contentful_paint_ms: pageValues.firstContentfulPaintMs,
    dom_element_count: pageValues.domElementCount,
    javascript_heap_used_bytes: cdpValues.get("JSHeapUsedSize") ?? null,
  };
}

function makeSamples(scenario, condition, snapshot) {
  const units = {
    navigation_load_ms: "ms",
    dom_content_loaded_ms: "ms",
    first_contentful_paint_ms: "ms",
    dom_element_count: "count",
    javascript_heap_used_bytes: "bytes",
  };
  return Object.entries(snapshot)
    .filter(([, value]) => typeof value === "number" && Number.isFinite(value) && value >= 0)
    .map(([metric, value]) => ({ kind: "sample", scenario, condition, metric, value, unit: units[metric] }));
}

async function measureRoute(browser, baseUrl, route, warmRepetitions) {
  const samples = [];

  const coldContext = await browser.newContext({ serviceWorkers: "block" });
  try {
    const coldPage = await coldContext.newPage();
    const coldSession = await coldContext.newCDPSession(coldPage);
    await coldSession.send("Performance.enable");
    const coldResponse = await coldPage.goto(`${baseUrl}${route.path}`, { waitUntil: "load", timeout: 30_000 });
    if (!coldResponse?.ok()) {
      throw new Error(`${route.path} returned an unsuccessful HTTP response`);
    }
    samples.push(...makeSamples(route.scenario, "cold", await captureSnapshot(coldPage, coldSession)));
    await coldSession.detach();
  } finally {
    await coldContext.close();
  }

  const warmContext = await browser.newContext({ serviceWorkers: "block" });
  try {
    const warmupPage = await warmContext.newPage();
    const warmupResponse = await warmupPage.goto(`${baseUrl}${route.path}`, { waitUntil: "load", timeout: 30_000 });
    if (!warmupResponse?.ok()) {
      throw new Error(`${route.path} warmup returned an unsuccessful HTTP response`);
    }
    await warmupPage.close();

    for (let repetition = 0; repetition < warmRepetitions; repetition += 1) {
      const warmPage = await warmContext.newPage();
      const warmSession = await warmContext.newCDPSession(warmPage);
      try {
        await warmSession.send("Performance.enable");
        const response = await warmPage.goto(`${baseUrl}${route.path}`, { waitUntil: "load", timeout: 30_000 });
        if (!response?.ok()) {
          throw new Error(`${route.path} warm navigation returned an unsuccessful HTTP response`);
        }
        samples.push(...makeSamples(route.scenario, "warm", await captureSnapshot(warmPage, warmSession)));
      } finally {
        await warmSession.detach();
        await warmPage.close();
      }
    }
  } finally {
    await warmContext.close();
  }

  return samples;
}

async function writeSamples(outputPath, samples) {
  const jsonl = `${samples.map((sample) => JSON.stringify(sample)).join("\n")}\n`;
  if (Buffer.byteLength(jsonl, "utf8") > 8 * 1024 * 1024) {
    throw new Error("driver output exceeds the 8 MiB sample input limit");
  }
  await writeFile(outputPath, jsonl, { encoding: "utf8", flag: "wx" });
}

async function main(argv) {
  const options = parseArgs(argv);
  const browser = await chromium.launch({ headless: true });
  try {
    const samples = [];
    for (const route of ROUTES) {
      samples.push(...await measureRoute(browser, options.baseUrl, route, options.warmRepetitions));
    }
    await writeSamples(options.outputPath, samples);
    process.stdout.write(`${JSON.stringify({ status: "available", samples: samples.length, routes: ROUTES.length, warmRepetitions: options.warmRepetitions, rendererVersion: browser.version() })}\n`);
  } finally {
    await browser.close();
  }
}

main(process.argv.slice(2)).catch((error) => {
  process.stderr.write(`ui0 Playwright driver failed: ${error.message}\n`);
  process.exitCode = 1;
});
