/* 4 行业预设 · P6.1 · embedded · 任务定义
 * 出典: industry-knowledge.md (P6.1 単体試験 · embedded 重点)
 * 重点: モジュール単体 (VectorCAST) / ISR 単体 / メモリ境界
 * 業界: 車載 / IoT / 医療機器 / 産業機器 (ISO 26262 ASIL / MISRA-C / RTOS)
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p61'] = {
    phase: 'P6.1', phaseId: 'P6.1',
    industry: 'embedded', industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P61-EMB-001',
        title: 'モジュール単体試験 (VectorCAST)',
        desc: '機能モジュール (タスク / HAL / ドライバ) の単体試験を VectorCAST で実施。分岐網羅 (C1) / 条件網羅 (C2) 基準を満たす。',
        priority: 'P0',
        tags: ['組込', 'UT', 'VectorCAST', 'ISO 26262'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 8
      },
      {
        id: 'P61-EMB-002',
        title: 'ISR (割込処理) 単体試験',
        desc: '割込み優先度 / ネスト割込 / クリティカルセクション / ジッタ / ディスパッチ遅延 の単体試験 (リアルタイム性検証)。',
        priority: 'P0',
        tags: ['組込', 'UT', 'ISR', 'RTOS'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 6
      },
      {
        id: 'P61-EMB-003',
        title: 'メモリ境界単体試験',
        desc: 'バッファオーバーラン / アンダーラン / メモリプール枯渇 / スタック溢れ / ヒープ断片化 の単体試験 (MISRA-C 準拠、Valgrind / Cppcheck 併用)。',
        priority: 'P0',
        tags: ['組込', 'UT', 'メモリ', 'MISRA-C'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 6
      }
    ]
  };
})(window);
