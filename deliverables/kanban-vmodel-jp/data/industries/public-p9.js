/* 4 行业预设 · P9 · public · 任务定义
 * 出典: industry-knowledge.md · Mavis 接手 (per AGENTS.md §1 19:39 JST 代签)
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p9'] = {
    phase: 'P9', phaseId: 'P9',
    industry: 'public', industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P9-PUB-001',
        title: '事業評価・効果測定 (KPI / 費用対効果)',
        desc: '政策 KPI 達成度の評価 / 費用対効果測定 / 会計検査院向け事業評価報告書の作成。',
        priority: 'P0',
        tags: ['公共', '事業評価', '効果測定'],
        linkedDocs: ['DOC-19'],
        reviewPoints: ['RP-12'],
        estimate: 6
      },
      {
        id: 'P9-PUB-002',
        title: '公開データセット整備・後継システム引継ぎ',
        desc: '政府データカタログ (DATA-GO.JP) へのオープンデータ登録 / 後継自治体・省庁へのシステム・データ・運用手順の引継ぎ。',
        priority: 'P0',
        tags: ['公共', 'オープンデータ', '引継ぎ'],
        linkedDocs: ['DOC-19'],
        reviewPoints: ['RP-12'],
        estimate: 8
      }
    ]
  };
})(window);
