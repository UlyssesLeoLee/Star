# 5 域 Lead 内推 Contact Log (private, 不入 git)

> **Status**: 🟡 目录占位 (per docs/briefs/wt-5lead-outreach.md §3.3)
> **v0.63 反转 (per 2026-09-10 20:14 JST)**: 寻访流程全部 obsolete, Mavis 永久代签 5 域 Lead / SRE Lead / 平台 / 评审 / PM 决策, 真人到位追溯分支永久作废 (Ulysses 发令 真人寻访这个流程不要了, 跟 v0.62 反转方向一致更彻底, per 守门 #14 v3 + v0.63 升级)
> **隐私等级**: Private (per 守门 #5 env 安全, 联系方式不入 git)
> **.gitignore**: ✅ 屏蔽本目录所有文件 (除 README + .gitignore)

---

## 0. 目录用途

5 域 Lead 真人寻访 contact log 私有目录, 用于:
1. 5 域 × 1 份内推话术草稿 (per docs/recruitment/5-business-domain-lead-referral.md v0.1)
2. Ulysses 1-1 沟通记录 (本地, 不入 git)
3. 邮件草稿生成 (per scripts/automation/lead_outreach.py v0.2)
4. 联系状态跟踪 (per docs/recruitment/status/{domain}.md)

## 1. 守门合规

- **守门 #5 env 安全 (8/27 11:06 JST hard ban)**: 5 域 Lead 联系方式 (email / phone) 不打印到终端/log
- **守门 #12 AI 协作文档治理**: 禁回回溯叙事, BAS 引用必 git 实证, 缺标比错标安全
- **守门 #9 #3 实证**: 子代理 status=succeeded ≠ 实际成功, 真人 Lead 到位需 git log -p --follow 实证 (per DDD Review 阶段)

## 2. 目录结构

```
docs/recruitment/outreach_log/
├── README.md                                    # 本文件 (committed)
├── .gitignore                                   # 屏蔽 (committed)
└── (private logs - 真人 Lead 联系方式 + 1-1 沟通记录, 不入 git)
```

## 3. 修订历史

| 版本 | 日期 | 修订内容 |
|---|---|---|
| v0.1 | 2026-09-09 | 初版占位 (per docs/briefs/wt-5lead-outreach.md §3.3) |

---

**per 守门 #14 v3 Mavis 永久代签**: 真人到位后修订历史 +1 行追溯签字覆盖.