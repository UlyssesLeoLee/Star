/* 4 行业预设 · P6.2 · embedded · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p62'] = {
    phase: 'P6.2', phaseId: 'P62',
    industry: 'embedded', industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P62-EMB-001',
        title: 'HW-SW 結合試験 (マイコン + ペリフェラル)',
        desc: 'マイコンと周辺機器 (GPIO/UART/I2C/SPI/ADC) との結合動作検証。',
        priority: 'P0',
        tags: ['組込','HW-SW','ペリフェラル'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 8
      },
      {
        id: 'P62-EMB-002',
        title: '車載ネットワーク (CAN/CAN-FD/LIN) 結合試験',
        desc: 'CAN/CAN-FD/LIN バス経由の ECU 間通信結合試験、ISO 14229 (UDS) 診断サービス含む。',
        priority: 'P0',
        tags: ['組込','車載','CAN'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 6
      },
      {
        id: 'P62-EMB-003',
        title: 'リアルタイム性 / 割り込みレイテンシ 試験',
        desc: 'RTOS 上のタスク応答性 / 割り込みレイテンシ / ジッタ測定。ASIL 要件との適合確認。',
        priority: 'P0',
        tags: ['組込','RTOS','ISO26262'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 6
      }
    ]
  };
})(window);
