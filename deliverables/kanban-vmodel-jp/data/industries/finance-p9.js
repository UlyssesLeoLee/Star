/* 4 行业预设 · P9 · finance · 任务定义
 * 出典: industry-knowledge.md · Mavis 接手 (per AGENTS.md §1 19:39 JST 代签)
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p9'] = {
    phase: 'P9', phaseId: 'P9',
    industry: 'finance', industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P9-FIN-001',
        title: '監査法人完了報告・FISC 監査対応',
        desc: '監査法人への最終報告 / FISC 安全対策基準への準拠証明 / 監督当局 (金融庁) への完了届出。',
        priority: 'P0',
        tags: ['金融', '監査', 'FISC'],
        linkedDocs: ['DOC-19'],
        reviewPoints: ['RP-12'],
        estimate: 6
      },
      {
        id: 'P9-FIN-002',
        title: '法定文書保管 (7-10 年) アーカイブ',
        desc: '金商法 / 割賦販売法 / 犯罪収益移転防止法 法定保存期間 (7-10 年) の取引履歴・監査ログ・契約書を改ざん不可ストレージにアーカイブ。',
        priority: 'P0',
        tags: ['金融', '法定保存', 'アーカイブ'],
        linkedDocs: [],
        reviewPoints: ['RP-12'],
        estimate: 8
      }
    ]
  };
})(window);
