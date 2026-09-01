/* 4 行业预设 · P6.3 · embedded · 任务定义
 * 出典: industry-knowledge.md · P6.3 システム試験 (組込)
 *       重点: HILS / 性能 / EMC / 振動 / 温度
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p63'] = {
    phase: 'P6.3', phaseId: 'P6.3',
    industry: 'embedded', industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P63-EMB-001',
        title: 'HILS システム試験',
        desc: '実機 ECU + 車載ネットワーク (CAN / LIN) を HILS に接続し、走行シナリオでの振る舞いを統合検証。',
        priority: 'P0',
        tags: ['組込', 'ST', 'HILS', 'ASIL'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 16
      },
      {
        id: 'P63-EMB-002',
        title: 'EMC / 振動 / 温度 環境試験',
        desc: '車載 / 機器搭載条件を想定した EMC (放射 / 伝導) / ランダム振動 / 温度サイクル (-40〜+85℃) 下での動作継続性を検証。',
        priority: 'P0',
        tags: ['組込', 'ST', 'EMC', '環境'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 12
      },
      {
        id: 'P63-EMB-003',
        title: 'リアルタイム性能試験',
        desc: 'RTOS タスク優先度下での応答時間 / ジッタ / CPU / メモリ使用率をプロファイルし、ASIL 要件の時間余裕を検証。',
        priority: 'P1',
        tags: ['組込', 'ST', 'PT', 'RTOS'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 8
      }
    ]
  };
})(window);
