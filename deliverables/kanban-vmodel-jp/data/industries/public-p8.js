/* 4 行业预设 · P8 · public · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p8'] = {
    phase: 'P8', phaseId: 'P8',
    industry: 'public', industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P8-PUB-001',
        title: '自治体職員向けヘルプデスク / FAQ',
        desc: '自治体職員からの問い合わせ対応。FAQ 整備・一次回答・エスカレーション。業務時間外対応も含む。',
        priority: 'P0',
        tags: ['公共','ヘルプデスク','自治体','FAQ'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 8
      },
      {
        id: 'P8-PUB-002',
        title: '自治体職員研修 / ヘルプ動画',
        desc: '操作研修・ヘルプ動画整備。業務継続性 (BCP) 観点の研修計画。',
        priority: 'P1',
        tags: ['公共','研修','BCP','動画'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 4
      },
      {
        id: 'P8-PUB-003',
        title: '事業評価 / 効果測定 (KPI 報告)',
        desc: '事業評価・KPI (住民満足度・処理時間短縮等) 測定。国庫補助金報告向けの効果測定資料。',
        priority: 'P1',
        tags: ['公共','評価','KPI','効果測定'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 4
      }
    ]
  };
})(window);
