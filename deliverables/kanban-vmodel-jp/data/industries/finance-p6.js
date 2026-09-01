/* 4 行业预设 · P6 · finance · 任务定义
 * 出典: industry-knowledge.md (per 2026-09-01 21:25 JST 業界知識)
 * 適用: 銀行・証券・保険・決済 向け P6 テスト工程 (主 phase) 跨子 phase 工程管理級タスク
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p6'] = {
    phase: 'P6',
    phaseId: 'P6',
    industry: 'finance',
    industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P6-FIN-001',
        title: 'PCI ASV 脆弱性診断 / ペネトレーションテスト 統合管理',
        desc: 'PCI DSS Req.11 準拠の ASV 脆弱性診断 (外部) と内部ペネトレーションテストを P6.3 システム試験段階で計画・実施・証跡保管。発見脆弱性の CVSS 評価・修正検証・再診断を ST ゲート承認 (RP-07) と連動。',
        priority: 'P0',
        tags: ['金融', 'セキュリティ', 'PCI DSS', 'ペネトレーション'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07', 'RP-08'],
        estimate: 8
      },
      {
        id: 'P6-FIN-002',
        title: 'FISC 監査対応テスト / ログ監査テスト 統合管理',
        desc: 'FISC 安全対策基準 (実務指針) 第 9 章「監査対応」要件に対するログ監査テストを P6.1-P6.4 全段で横断実施。監査ログ完全性・改ざん検知・保存期間 (7 年) ・証跡突合を RP-04/06/07/08 ゲート全てで検証。',
        priority: 'P0',
        tags: ['金融', 'FISC', '監査', 'ログ'],
        linkedDocs: ['DOC-14', 'DOC-15', 'DOC-20'],
        reviewPoints: ['RP-04', 'RP-06', 'RP-07', 'RP-08'],
        estimate: 10
      }
    ]
  };
})(window);
