/* 4 行业预设 · P8 · ec · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p8'] = {
    phase: 'P8', phaseId: 'P8',
    industry: 'ec', industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P8-EC-001',
        title: '売上 / 在庫 / 顧客行動の常時監視',
        desc: 'リアルタイム売上ダッシュボード / 在庫切れ予兆検知 / 顧客行動 (離脱・カート落ち) 監視。',
        priority: 'P0',
        tags: ['EC','売上','在庫','監視'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 6
      },
      {
        id: 'P8-EC-002',
        title: '不正検知 (チャージバック / 不審注文)',
        desc: '機械学習ベースの不正注文検知。チャージバック率監視。3D セキュア認証成功率追跡。',
        priority: 'P0',
        tags: ['EC','不正','チャージバック','3DS'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 6
      },
      {
        id: 'P8-EC-003',
        title: 'CS 品質管理 / VOC 分析',
        desc: 'CS 応対品質 (CSAT/NPS) 監視。VOC (顧客の声) テキストマイニング → 改善バックログ反映。',
        priority: 'P1',
        tags: ['EC','CS','VOC','NPS'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 4
      }
    ]
  };
})(window);
