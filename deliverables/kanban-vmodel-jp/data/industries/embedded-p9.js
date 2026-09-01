/* 4 行业预设 · P9 · embedded · 任务定义
 * 出典: industry-knowledge.md · Mavis 接手 (per AGENTS.md §1 19:39 JST 代签)
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p9'] = {
    phase: 'P9', phaseId: 'P9',
    industry: 'embedded', industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P9-EMB-001',
        title: '量産移管 (試作 → 製造ライン)',
        desc: '試作から量産ラインへの設計移管 / 製造 BOM 確定 / 初期流動管理 (SPC) / 製造認証 (IATF 16949 等) の立会確認。',
        priority: 'P0',
        tags: ['組込', '量産移管', 'SPC'],
        linkedDocs: [],
        reviewPoints: ['RP-12'],
        estimate: 8
      },
      {
        id: 'P9-EMB-002',
        title: '設計資産アーカイブ・長期保守契約締結',
        desc: '回路図 / ソース / ビルド環境 / キャリブレーションデータの長期アーカイブ (10 年+) / 保守契約 (SLA / 部品寿命) 締結。',
        priority: 'P1',
        tags: ['組込', '保守契約', 'アーカイブ'],
        linkedDocs: ['DOC-19'],
        reviewPoints: ['RP-12'],
        estimate: 6
      }
    ]
  };
})(window);
