# Industry Knowledge — 4 行业预设知识库

> 13 子代理 worktree 共享此文档。每 worker 按 phase 维度和行业产出 data/industries/<industry>-<phase>.js。

## 行业定义

| code | ja | color | 適用業務 |
|---|---|---|---|
| `finance` | 金融 | `#dc2626` | 銀行・証券・保険・決済。FISC/PCI DSS/金融商品取引法/個人情報保護法/犯罪収益移転防止法 |
| `public` | 公共 | `#0ea5e9` | 自治体・中央省庁・独立行政法人。JIS X 8341 アクセシビリティ/デジタル手続法/個人情報保護/会計法 |
| `ec` | EC | `#f59e0b` | E コマース・マーケットプレイス・サブスク。3Dセキュア/不正検知/在庫/配送/売上 |
| `embedded` | 組込 | `#10b981` | 車載/IoT/医療機器/産業機器。ISO 26262 (ASIL)/MISRA-C/RTOS/リアルタイム性 |

## Phase 別業界典型タスク要約

### P1 超上流工程
- **finance**: 金融商品取引法対応/FISC 安全対策基準/犯罪収益移転防止法/KYC 業務要件/AML リスク評価
- **public**: 政策立案支援/関連省庁協議/予算要求/個人情報保護影響評価 (PIA)/アクセシビリティ方針
- **ec**: 事業計画/KPI 設定 (GMV/転換率)/法務確認 (特定商取引法/景品表示法)/競合分析
- **embedded**: ISO 26262 ASIL 分解/機能安全コンセプト/規格調査 (IEC 61508/EN 50128)/HW/SW インターフェース

### P2 要件定義
- **finance**: 法令要件收集 (金商法/割賦販売法)/リスクアセスメント/顧客 KYC 要件/監査ログ要件
- **public**: アクセシビリティ要件 (JIS X 8341-3)/多言語要件/個人情報保護要件/法令遵守要件
- **ec**: 商品/決済/物流/在庫/プロモーション要件/カード情報非保持要件
- **embedded**: 安全要件 (ASIL)/リアルタイム性能要件/MISRA-C 準拠要件/HW 制約要件

### P3 基本設計
- **finance**: FISC 準拠セキュリティ設計/AES-256 + HSM/監査ログ設計/二重化/DR 設計
- **public**: ユニバーサルデザイン設計/多言語切替/公開鍵基盤 (LGPKI)/共通基盤連携
- **ec**: 決済連携 (PSP) 設計/在庫同期/配送 API/カード非保持化 (リダイレクト/トークン化)
- **embedded**: RTOS 選定/HAL 抽象化/メモリマップ/タスク優先度/車載ネットワーク (CAN/LIN)

### P4 詳細設計
- **finance**: PCI DSS 制御実装詳細/暗号化 API 詳細/監査ログ詳細/二要素認証
- **public**: WCAG 2.1 AA 詳細/支援技術対応 (NVDA/JAWS)/JIS X 8341 テスト詳細
- **ec**: 3Dセキュア (EMV 3DS) 詳細/不正検知ルール/在庫引当ロジック/配送料計算
- **embedded**: MISRA-C 準拠詳細/メモリ管理詳細/ISR 詳細/ブートローダ詳細

### P5 実装
- **finance**: セキュアコーディング (OWASP)/コード署名/SAST/秘密情報管理 (Vault)
- **public**: JIS X 8341 準拠実装/自動試験 (a11y ツール)/アクセシブル UI コンポーネント
- **ec**: 決済 API 実装/不正検知実装/在庫引当実装/在庫切れ処理
- **embedded**: MISRA-C 静的解析/単体テスト (VectorCAST 等)/トレース/性能プロファイリング

### P6 テスト工程 (主 phase: テスト戦略/管理)
- **finance**: PCI ASV 脆弱性診断/ペネトレーション/FISC 監査対応テスト/ログ監査テスト
- **public**: アクセシビリティ試験/ユーザビリティ試験/画面読み上げ試験/政府共通プラットフォーム試験
- **ec**: 決済試験 (3DS/リダイレクト)/性能試験 (キャンペーン)/不正検知試験/在庫整合性
- **embedded**: HIL テスト/性能ベンチ/EMC 試験/安全解析レビュー

### P6.1 単体試験
- **finance**: 暗号化関数の境界値/認証フロー分岐/KYC スコアリング単体
- **public**: アクセシビリティ単体 (aria 属性)/入力検証単体
- **ec**: 決済金額計算/送料計算/在庫引当単体
- **embedded**: モジュール単体 (VectorCAST)/ISR 単体/メモリ境界

### P6.2 結合試験
- **finance**: API 結合 (FISC ネットワーク)/与信接続/口座振替接続
- **public**: 外部システム結合 (LGWAN/マイナンバー)/共通基盤 API
- **ec**: PSP 接続/物流 API 接続/在庫同期
- **embedded**: HW-SW 結合/車載ネットワーク (CAN) 結合

### P6.3 システム試験
- **finance**: 負荷試験 (同時取引数)/DR 切替/監査ログ完全性
- **public**: 全シナリオ試験/アクセシビリティ試験/負荷試験
- **ec**: ピーク負荷 (キャンペーン)/カード大量決済/不正攻撃シミュレーション
- **embedded**: HILS/性能/EMC/振動/温度

### P6.4 受入試験
- **finance**: 業務部門 UAT (与信/振替)/コンプライアンス部門承認/監査法人立会
- **public**: 自治体職員 UAT/住民モニター試験/アクセシビリティ検証 (当事者参加)
- **ec**: 業務シナリオ UAT/ピーク日シミュレーション/障害対応訓練
- **embedded**: 量産前試作/フィールド試験/認証機関立会

### P7 移行・リリース
- **finance**: カットオーバー (土日夜間)/並行稼働/法定帳簿同期/監督当局報告
- **public**: 住民向け周知/既存システム並行稼働/旧システム停止計画
- **ec**: 本番リリース/キャンペーン同時/PiP 検証
- **embedded**: OTA アップデート/フィールド展開/製造ライン組込

### P8 運用・保守
- **finance**: インシデント対応 (FISC 報告)/監督当局報告/監査ログ保管/暗号鍵ローテーション
- **public**: ヘルプデスク/自治体職員研修/システム改善/事業評価
- **ec**: 売上監視/在庫補充/不正監視/顧客対応/CS 品質管理
- **embedded**: 故障監視/リコール対応/ソフトウェア更新/長期保守契約

### P9 終結
- **finance**: 監査対応/法定保存 (7-10 年)/監督当局完了報告
- **public**: 事業評価/効果測定/公開データセット整備/後継システム引継ぎ
- **ec**: セール分析/フィードバック反映/契約更新
- **embedded**: 量産移管/保守契約/設計資産アーカイブ

## 任务数据 schema

```js
{
  id: 'P1-FIN-001',           // 格式: <phaseId>-<industryCode3>-<NNN>
  title: '...',                 // 日本語タイトル
  desc: '...',                  // 1-2 行の説明
  priority: 'P0' | 'P1' | 'P2' | 'P3',
  tags: ['金融','法令'],        // 2-4 个标签
  linkedDocs: ['DOC-01'],       // 关联到主 data.js 的 DOC 编号
  reviewPoints: ['RP-12'],      // 关联 review point
  estimate: 8                   // 人时
}
```

## 出力ファイル形式

`deliverables/kanban-vmodel-jp/data/industries/<industry>-<phase>.js`:

```js
/* 4 行业预设 · <phase> · <industry> · 任务定义
 * 出典: industry-knowledge.md · Mavis
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['<industry>-<phase>'] = {
    phase: '<P1|P2|...|P6.1|...>',
    phaseId: 'P1',              // 数字 part
    industry: '<finance|public|ec|embedded>',
    industryJa: '<金融|公共|EC|組込>',
    color: '#dc2626',
    tasks: [ /* 2-3 任务 */ ]
  };
})(window);
```

## 任务数量要求

每 wt 4 行业 × 2-3 任务 = **8-12 任务总计**。
P6 测试工程主 phase 因为没有直接子任务，任务应放在 4 子 phase wt 里做（P6 主 wt 可加 1-2 个跨子 phase 管理任务，每行业）。

## 守门

- 禁"per 历史形态" 等回溯叙事
- 每文件必须 git commit 实证
- 不跨界修改其他 phase 的 data/industries/* 文件
- 命名严格按 schema
- 完成后输出 `git log -1 --stat` 实证
