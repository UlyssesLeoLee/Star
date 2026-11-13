# Brief: STAR-STABLE-K3S-CONTEXT-INVENTORY-20261107

**Agent**: worker
**Phase**: STAR-STABLE-W7
**Created**: 2026-11-07 14:10:20

---

基线 dev@6c6dfa9d79acb4198d4f1071c68503363fe10439。只做 K3s-local 已生成构建上下文的证据整理，不改产品源代码或 WBS 主文件。

现有生成物：C:/Users/leo19/AppData/Local/Temp/star-k3s-context-inventory-dev-6c6dfa9d-20261107；生成器输出 334 files、digest e894ebd1e5cd4dcc76a44bc80d32e269827e93dd30254f693eb38fd8ca020100。协调者另扫到 334 个文件路径中没有 game/legacy 命名；这是路径证据，不表示最终镜像已验证。

请以该临时生成物为输入，整理排序后的 per-file relative path、size、SHA-256 为 docs/reports/data/STAR-K3S-CONTEXT-INVENTORY-20261107.jsonl；核对 Cargo workspace closure、Cargo.lock package names/dependencies、生成后的 deploy/k3s-local/Dockerfile 每个显式 crate COPY，拒绝 broad COPY crates ./crates、game/legacy 路径或 package。不要把源代码注释中的 legacy 一词误判成旧业务 payload。记录辅助代码如何验证、命令输出、上下文摘要及边界。

在登记清单范围内新增阶段报告 docs/reports/PHASE-STAR-STABLE-K3S-CONTEXT-INVENTORY-20261107.md，并用 dispatcher brief API 留下本 brief 和指定 dispatch JSONL。报告必须清楚写：本次只证明过滤后的 build context，不构成 Docker build、最终镜像层、SBOM/provenance、签名、K3s 部署或回滚验收；这些缺口继续 OPEN。最终 WBS 主文件与保留的 Rust candidate 有同路径改动，禁止修改它，禁止触碰 main、远端、其他 worktree 或 D:\Star 用户脏文件。只改已登记的四个路径，并按仓库约定提交。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
