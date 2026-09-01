/* 4 行业预设 · P7 · embedded · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p7'] = {
    phase: 'P7', phaseId: 'P7',
    industry: 'embedded', industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P7-EMB-001',
        title: 'OTA アップデート / 差分配信',
        desc: 'OTA (Over-The-Air) による差分ソフトウェア配信。A/B パーティション切替で rollback 可能に。',
        priority: 'P0',
        tags: ['組込','OTA','差分','配信'],
        linkedDocs: ['DOC-17','DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 8
      },
      {
        id: 'P7-EMB-002',
        title: 'フィールド展開 / パイロット運用',
        desc: '少数の実機 / 車両 / 機器でのパイロット展開。運用条件下での信頼性・性能・EMC 確認。',
        priority: 'P0',
        tags: ['組込','フィールド','パイロット'],
        linkedDocs: ['DOC-15','DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 12
      },
      {
        id: 'P7-EMB-003',
        title: '製造ライン組込 / 量産フラッシュ',
        desc: '製造ラインでの量産フラッシュ工程。製造側との生産性 (秒/台) すり合わせ。',
        priority: 'P1',
        tags: ['組込','量産','フラッシュ','製造'],
        linkedDocs: ['DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 6
      }
    ]
  };
})(window);
