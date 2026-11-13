#!/usr/bin/env node
/*
@cypher schema=1 source_sha256=c2e46f17adb4f170736fce831fd81d4e20e7a69f23414a004f7d952384d8123a
MERGE (self:File {path:"scripts/ci/validate_sbom.js"})
MERGE (fn:Symbol {id:"scripts/ci/validate_sbom.js::validateSbom",kind:"function"})
MERGE (equiv:Symbol {id:"scripts/ci/validate_sbom.js::EQUIV_DESCRIBES",kind:"function"})
MERGE (selftest:Symbol {id:"scripts/ci/validate_sbom.js::selfTest",kind:"function"})
MERGE (self)-[:DEFINES]->(fn)
MERGE (self)-[:DEFINES]->(equiv)
MERGE (self)-[:DEFINES]->(selftest)
@endcypher
*/
// =====================================================================
// validate_sbom.js — 桌面发布 SPDX 2.3 SBOM 校验器 (per WBS 10.3.4)
//
// 背景 (docs/ci-cd.md:163): cargo-sbom 0.10.0 对 star-desktop 生成的 SPDX 是
//   DOCUMENT DESCRIBES SPDXRef-File-star-desktop, 且该 File GENERATED_FROM
//   根 Package; 而 npm sbom 输出是 DOCUMENT 直接 DESCRIBES 根 Package。
//   原 workflow 内联 JS 只接受后者 → Rust SBOM 校验全红。
//
// 修法 (本脚本): 两种形态都接受 —
//   A. DOCUMENT --DESCRIBES--> 根 Package        (npm sbom 形态)
//   B. DOCUMENT --DESCRIBES--> File F 且
//      F --GENERATED_FROM--> 根 Package           (cargo-sbom 形态)
// 内置 self-test (变异拒绝) 保持 fail-closed。
//
// 用法: node scripts/ci/validate_sbom.js <rust_sbom.json> <frontend_sbom.json>
// =====================================================================

"use strict";

const fs = require("node:fs");

/** 形态 A + B 的 DESCRIBES 等价判定。返回 true 或抛 Error。 */
function hasValidDescribes(doc, rootPkg) {
  const describesTargets = doc.relationships.filter(
    (x) =>
      x &&
      x.spdxElementId === "SPDXRef-DOCUMENT" &&
      x.relationshipType === "DESCRIBES",
  );
  if (describesTargets.length === 0) throw new Error("DESCRIBES");
  for (const rel of describesTargets) {
    if (rel.relatedSpdxElement === rootPkg.SPDXID) return true; // 形态 A
    // 形态 B: File F GENERATED_FROM 根 Package
    const generatedFromRoot = doc.relationships.some(
      (y) =>
        y &&
        y.spdxElementId === rel.relatedSpdxElement &&
        y.relationshipType === "GENERATED_FROM" &&
        y.relatedSpdxElement === rootPkg.SPDXID,
    );
    if (generatedFromRoot) return true;
  }
  throw new Error("DESCRIBES");
}

function validateSbom(sbom, expectedName, minPackages) {
  if (!sbom || sbom.spdxVersion !== "SPDX-2.3") throw new Error("SPDX-2.3");
  if (!Array.isArray(sbom.packages) || !sbom.packages.length) throw new Error("packages");
  if (!Array.isArray(sbom.relationships) || !sbom.relationships.length) throw new Error("relationships");
  if (sbom.packages.length < minPackages) throw new Error("minimum");
  const root = sbom.packages.find((p) => p && p.name === expectedName && p.SPDXID);
  if (!root) throw new Error("root");
  hasValidDescribes(sbom, root);
}

/** 变异 self-test: 每类错误都必须被拒绝 (fail-closed)。 */
function selfTest() {
  const base = {
    spdxVersion: "SPDX-2.3",
    packages: [{ name: "star-desktop", SPDXID: "SPDXRef-Package-star-desktop" }],
    relationships: [
      { spdxElementId: "SPDXRef-DOCUMENT", relationshipType: "DESCRIBES", relatedSpdxElement: "SPDXRef-Package-star-desktop" },
    ],
  };
  const reject = (sbom, name, min, expected) => {
    try {
      validateSbom(sbom, name, min);
    } catch (e) {
      if (e.message === expected) return;
      throw e;
    }
    throw new Error("self-test " + expected);
  };
  reject({ ...base, packages: [] }, "star-desktop", 1, "packages");
  reject({ ...base, relationships: [] }, "star-desktop", 1, "relationships");
  reject({ ...base, spdxVersion: "SPDX-2.2" }, "star-desktop", 1, "SPDX-2.3");
  // cargo-sbom 形态 B 的合法样本 (DOCUMENT DESCRIBES File → GENERATED_FROM Package)
  const cargoStyle = {
    spdxVersion: "SPDX-2.3",
    packages: [{ name: "star-desktop", SPDXID: "SPDXRef-Package-star-desktop" }],
    files: [{ fileName: "./star-desktop", SPDXID: "SPDXRef-File-star-desktop" }],
    relationships: [
      { spdxElementId: "SPDXRef-DOCUMENT", relationshipType: "DESCRIBES", relatedSpdxElement: "SPDXRef-File-star-desktop" },
      { spdxElementId: "SPDXRef-File-star-desktop", relationshipType: "GENERATED_FROM", relatedSpdxElement: "SPDXRef-Package-star-desktop" },
    ],
  };
  validateSbom(cargoStyle, "star-desktop", 1); // 形态 B 必须通过
  // 形态 B 缺 GENERATED_FROM 链 → 拒绝
  reject(
    { ...cargoStyle, relationships: cargoStyle.relationships.slice(0, 1) },
    "star-desktop",
    1,
    "DESCRIBES",
  );
  // 形态 A 直接 DESCRIBES Package 保持通过
  validateSbom(base, "star-desktop", 1);
  console.log("SELF-TEST PASS");
}

function main() {
  selfTest();
  const args = process.argv.slice(2);
  if (args.length !== 2) throw new Error("expected Rust and frontend SBOM paths");
  const rust = JSON.parse(fs.readFileSync(args[0], "utf8"));
  const frontend = JSON.parse(fs.readFileSync(args[1], "utf8"));
  validateSbom(rust, "star-desktop", 1);
  validateSbom(frontend, "star-desktop-frontend", 225);
  for (const sbom of [rust, frontend]) {
    const text = JSON.stringify(sbom);
    if (/star[-_]game/i.test(text)) {
      throw new Error("Unexpected retired game package in Desktop SBOM");
    }
  }
  console.log("PASS: SBOM validation (SPDX-2.3, DESCRIBES A/B, minimums, retired-game scan)");
}

main();
