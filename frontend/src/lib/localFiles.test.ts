/*
@cypher schema=1 source_sha256=4619df946091f7f067c31f01c7c96c60f88326bd198db53f2755306e9af9b0cf
MERGE (self:File {path:"frontend/src/lib/localFiles.test.ts"})
MERGE (localMode:Symbol {id:"frontend/src/lib/localFiles.test.ts::isLocalMode_classifies_private_hosts",kind:"test"})
MERGE (ssr:Symbol {id:"frontend/src/lib/localFiles.test.ts::isLocalMode_false_without_window",kind:"test"})
MERGE (self)-[:DEFINES]->(localMode)
MERGE (self)-[:DEFINES]->(ssr)
@endcypher
*/
import { describe, it, expect, afterEach, vi } from "vitest";
import { isLocalMode } from "./localFiles";

const ORIGINAL_LOCATION = window.location;

afterEach(() => {
  vi.unstubAllGlobals();
  Object.defineProperty(window, "location", { value: ORIGINAL_LOCATION, writable: true });
});

function mockHostname(hostname: string) {
  Object.defineProperty(window, "location", {
    value: { hostname },
    writable: true,
  });
}

describe("本地模式判定 isLocalMode", () => {
  it("localhost / 127.0.0.1 / 私有网段为本地模式", () => {
    for (const host of ["localhost", "127.0.0.1", "192.168.1.5", "10.0.0.2", "172.16.0.1", "172.31.255.255"]) {
      mockHostname(host);
      expect(isLocalMode(), host).toBe(true);
    }
  });

  it("公网域名 / 公网 IP 不是本地模式", () => {
    for (const host of ["starcanvas.vercel.app", "172.32.0.1", "8.8.8.8"]) {
      mockHostname(host);
      expect(isLocalMode(), host).toBe(false);
    }
  });

  it("无 window (SSR) 返回 false", () => {
    vi.stubGlobal("window", undefined);
    expect(isLocalMode()).toBe(false);
  });
});
