# Brief: STABLE-HARDENING-CANVAS-OCI-20261031

**Agent**: worker
**Phase**: STABLE-HARDENING
**Created**: 2026-10-31 08:18:58

---

读取仓库 AGENTS.md 与本 brief。只处理本 lane manifest：.github/workflows/publish-star-canvas-services.yml、crates/canvas-engine/Dockerfile、crates/domain-canvas/Dockerfile、crates/canvas-realtime/Dockerfile、deploy/star-canvas-services-k3s.yaml、新建/修改 .dockerignore，以及本 lane 报告/dispatcher 文件。基线 dev@04d8b7a84514dcb454874eb852651778e1452dea。发行范围只有三份 Star Canvas service images；构建输入和缓存里都不能出现 star-game、游戏遗留或其他产品域。必须先推导 Cargo workspace/service 依赖闭包；不能仅靠忽略文件导致 cargo 解析失败，不能把整个仓库塞进 builder 再声称排除成功。部署 manifest 应固定到 image digest，不能保留 dev/latest mutable tag。只做静态/可执行的本地验收；若 Docker/OCI runtime不可用就如实报告，不运行发布、push、tag 或部署。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
