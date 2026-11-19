/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/components/context-menu/clipboard.ts"})
MERGE (copyText:Symbol {id:"frontend/src/components/context-menu/clipboard.ts::copyText",kind:"function"})
MERGE (self)-[:DEFINES]->(copyText)
@endcypher
*/

/**
 * 复制文本到剪贴板 — 共享工具。
 * 优先 navigator.clipboard (https 上下文); 不可用时回落 execCommand("copy")
 * (http / 旧浏览器)。返回 boolean 供调用方给诚实 toast。
 */
export async function copyText(text: string): Promise<boolean> {
  try {
    if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
    if (typeof document === "undefined") return false;
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.setAttribute("readonly", "");
    ta.style.position = "fixed";
    ta.style.top = "-9999px";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    let ok = false;
    try {
      ok = document.execCommand("copy");
    } catch {
      ok = false;
    }
    document.body.removeChild(ta);
    return ok;
  } catch {
    return false;
  }
}
