/* 4 行业预设 · P8 · embedded · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p8'] = {
    phase: 'P8', phaseId: 'P8',
    industry: 'embedded', industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P8-EMB-001',
        title: 'フィールド故障監視 / 予兆検知',
        desc: 'OTA 経由の故障ログ集約。故障率トレンド監視・予兆検知アルゴリズム。プロアクティブ保証。',
        priority: 'P0',
        tags: ['組込','故障監視','予兆','OTA'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 6
      },
      {
        id: 'P8-EMB-002',
        title: 'リコール対応 / OTA 緊急パッチ',
        desc: '市場不具合発生時のリコール判断・OTA 緊急パッチ配信。A/B ロールバック可能設計の活用。',
        priority: 'P0',
        tags: ['組込','リコール','OTA','緊急パッチ'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 4
      },
      {
        id: 'P8-EMB-003',
        title: '長期保守契約 / 部品供給保証',
        desc: '車載・医療・産業機器で必要な 10-15 年長期保守。部品供給保証・セキュリティパッチ長期提供計画。',
        priority: 'P1',
        tags: ['組込','長期保守','部品供給','セキュリティ'],
        linkedDocs: ['DOC-19'],
        reviewPoints: [],
        estimate: 4
      }
    ]
  };
})(window);
