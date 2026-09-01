/* 4 行业预设 · P6.4 · embedded · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p64'] = {
    phase: 'P6.4', phaseId: 'P6.4',
    industry: 'embedded', industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P64-EMB-001',
        title: '量産前試作 (PVS) フィールド試験',
        desc: '量産前試作品を実環境で運用試験。振動/温度/EMC を含む長期フィールド評価。',
        priority: 'P0',
        tags: ['組込','UAT','PVS','フィールド'],
        linkedDocs: ['DOC-15'],
        reviewPoints: ['RP-08','RP-09'],
        estimate: 12
      },
      {
        id: 'P64-EMB-002',
        title: '認証機関立会 (ISO 26262 / IEC 61508)',
        desc: '認証機関立会での機能安全認証試験。ASIL レベル判定・セーフティケース妥当性確認。',
        priority: 'P0',
        tags: ['組込','UAT','認証','機能安全'],
        linkedDocs: ['DOC-15'],
        reviewPoints: ['RP-08','RP-09','RP-10'],
        estimate: 10
      }
    ]
  };
})(window);
