/* 4 行业预设 · P2 · embedded · 任务定义
 * 出典: industry-knowledge.md
 * 主题: 安全要件 (ASIL)/リアルタイム性能/MISRA-C 準拠/HW 制約
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p2'] = {
    phase: 'P2', phaseId: 'P2',
    industry: 'embedded', industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P2-EMB-001',
        title: '安全要件 (ISO 26262 ASIL 分解)',
        desc: '車両機能に対するハザード分析とリスク評価 (ASIL A〜D) を行い、安全目標・安全状態・フォールト応答時間の要件化。',
        priority: 'P0',
        tags: ['組込', 'ISO 26262'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 8
      },
      {
        id: 'P2-EMB-002',
        title: 'リアルタイム性能 + RTOS 制約要件',
        desc: '制御周期・割込みレイテンシ・ジッタ許容値・最悪実行時間 (WCET) と RTOS タスク優先度・排他制御の要件化。',
        priority: 'P0',
        tags: ['組込', 'RTOS'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 6
      },
      {
        id: 'P2-EMB-003',
        title: 'MISRA-C 準拠 + HW 制約要件',
        desc: 'MISRA-C:2012 準拠ルールセットの取捨選択と、ROM/RAM/消費電力/ピン数の HW 制約下でのソフト配置の要件化。',
        priority: 'P1',
        tags: ['組込', 'MISRA-C'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 6
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
