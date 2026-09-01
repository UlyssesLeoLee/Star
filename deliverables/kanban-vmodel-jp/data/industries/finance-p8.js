/* 4 行业预设 · P8 · finance · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p8'] = {
    phase: 'P8', phaseId: 'P8',
    industry: 'finance', industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P8-FIN-001',
        title: 'インシデント対応 (FISC 報告)',
        desc: 'FISC 安全対策基準に準拠したインシデント検知・対応。重要インシデントは所管監督当局へ報告。',
        priority: 'P0',
        tags: ['金融','インシデント','FISC','監督当局'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 8
      },
      {
        id: 'P8-FIN-002',
        title: '暗号鍵ローテーション / HSM 運用',
        desc: 'AES-256 データ暗号化鍵 / HSM 鍵の定期ローテーション。FISC 制御 + NIST SP 800-57 準拠。',
        priority: 'P0',
        tags: ['金融','暗号','HSM','ローテーション'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 4
      },
      {
        id: 'P8-FIN-003',
        title: '監査ログ完全性 / 法定保存',
        desc: '監査ログ (改ざん検知機能付き) の保管と法定保存期間 (7-10 年) 管理。タイムスタンプ + WORM ストレージ。',
        priority: 'P0',
        tags: ['金融','監査ログ','WORM','法定保存'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 6
      }
    ]
  };
})(window);
