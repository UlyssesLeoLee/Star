# Star Mock Project ST (System Test) 回归报告

> **生成时间**: 2026-09-20T21:38:01Z
> **范围**: tools/star-flash-mock/{scripts/,mock_data/,docs/,k3s/}
> **触发**: ULYS-140 (回归测试) 2026-09-20 JST
> **守门**: 守门 #1+#5+#9+#11+#12+#24

## 1. ST 层验证

| 段 | 验证项 | 状态 |
|---|---|---|
| §1 | k3s yaml 完整性 | ✅ PASS |
| §2 | envoy 独立部署模式 | ✅ PASS |
| §3 | 端口契约 (3000 / 30800 / 8080) | ✅ PASS |
| §4 | port-forward service 守护 | ✅ PASS |
| §5 | 跨 mock_data/ ↔ scripts/ ↔ k3s/ 一致性 | ✅ PASS |
| §6 | docs/ 回归报告生成 | ✅ PASS |

## 2. mock_data fixture 统计 (per 守门 #13 W/T/M)


## 3. 已知缺口 (per 守门 #11 缺标比错标)

- 缺口 #1: ~~k3s/ 仅 2 yaml, 缺 star-mock ConfigMap + Secret~~ → **已闭合 (v1.2)** k3s/star-mock-configmap.yaml + k3s/star-mock-secret.yaml 新增 (实测 §1 4 yaml, 守门 #5 占位符 allowlist OK)
- 缺口 #2: port-forward 守护依赖 WSL2 + systemd user (Windows env 限制) — 跨 session 续
- 缺口 #3: ST 层静态验证, 不连真 k3s (per 守门 #24 v2 G-5 mock 锁) — by design, 不闭合

