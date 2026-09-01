/* 4 行业预设 · P3 · embedded · 任务定义
 * 出典: industry-knowledge.md §P3 基本設計 (組込)
 * 重点: RTOS 選定 / HAL 抽象化 / メモリマップ / タスク優先度 / 車載ネットワーク (CAN/LIN)
 */
(function (global) {
  'use strict';
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p3'] = {
    phase: 'P3',
    phaseId: 'P3',
    industry: 'embedded',
    industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P3-EMB-001',
        title: 'RTOS 選定 + タスク優先度設計',
        desc: 'FreeRTOS / AUTOSAR OS / TOPPERS 比較選定 + Rate Monotonic による優先度割当 + デッドラインモノトニック sched 設計。',
        priority: 'P0',
        tags: ['組込', 'RTOS', 'リアルタイム'],
        linkedDocs: ['DOC-05'],
        reviewPoints: ['RP-02'],
        estimate: 8
      },
      {
        id: 'P3-EMB-002',
        title: 'HAL 抽象化 + メモリマップ設計',
        desc: 'MCU 抽象化層 (HAL) + フラッシュ / RAM / 周辺領域のメモリマップ + MPU メモリ保護単位の割当設計。',
        priority: 'P0',
        tags: ['組込', 'HAL', 'メモリ'],
        linkedDocs: ['DOC-05', 'DOC-20'],
        reviewPoints: ['RP-02'],
        estimate: 6
      },
      {
        id: 'P3-EMB-003',
        title: '車載ネットワーク (CAN / LIN) 設計',
        desc: 'CAN FD バス構成 (ID 割当・メッセージ周期) + LIN スレーブタスク + OSEK / AUTOSAR ネットワークマネジメント設計。',
        priority: 'P1',
        tags: ['組込', '車載', 'CAN', 'LIN'],
        linkedDocs: ['DOC-12'],
        reviewPoints: ['RP-02'],
        estimate: 6
      }
    ]
  };
})(window);
