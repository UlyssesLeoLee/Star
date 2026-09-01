/* 4 行业预设 · P7 · finance · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p7'] = {
    phase: 'P7', phaseId: 'P7',
    industry: 'finance', industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P7-FIN-001',
        title: 'カットオーバー (土日夜間一括切替)',
        desc: '金融勘定系特有の週末夜間カットオーバー。勘定締切 → 移行 → 起動を 6-12 時間以内で完了。',
        priority: 'P0',
        tags: ['金融','移行','カットオーバー','勘定系'],
        linkedDocs: ['DOC-17','DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 12
      },
      {
        id: 'P7-FIN-002',
        title: '並行稼働 / 勘定突合',
        desc: '旧新システム並行稼働期間 (2-4 週間) での勘定突合。日次・週次差異確認と修正。',
        priority: 'P0',
        tags: ['金融','並行稼働','勘定突合'],
        linkedDocs: ['DOC-15','DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 16
      },
      {
        id: 'P7-FIN-003',
        title: '監督当局完了報告 / 届出',
        desc: '金融庁等監督当局への新システム稼働届出 / 報告書作成。FISC 監査対応含む。',
        priority: 'P1',
        tags: ['金融','監督当局','届出','FISC'],
        linkedDocs: ['DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 6
      }
    ]
  };
})(window);
