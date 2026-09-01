/* 4 行业预设 · P9 · ec · 任务定义
 * 出典: industry-knowledge.md · Mavis 接手 (per AGENTS.md §1 19:39 JST 代签)
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p9'] = {
    phase: 'P9', phaseId: 'P9',
    industry: 'ec', industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P9-EC-001',
        title: 'セール分析・売上総括 (KPI / キャンペーン効果)',
        desc: 'GMV / 転換率 / リピート率 / カゴ落ち率など KPI の総括分析 / 主要キャンペーンの効果測定レポート。',
        priority: 'P0',
        tags: ['EC', '売上分析', 'KPI'],
        linkedDocs: [],
        reviewPoints: ['RP-12'],
        estimate: 4
      },
      {
        id: 'P9-EC-002',
        title: '契約更新・顧客フィードバック反映計画',
        desc: 'PSP / 物流パートナー / モール出店契約の更新手続き / 顧客レビュー・VOC の次回開発 backlog への反映計画策定。',
        priority: 'P1',
        tags: ['EC', '契約更新', 'フィードバック'],
        linkedDocs: ['DOC-19'],
        reviewPoints: ['RP-12'],
        estimate: 4
      }
    ]
  };
})(window);
