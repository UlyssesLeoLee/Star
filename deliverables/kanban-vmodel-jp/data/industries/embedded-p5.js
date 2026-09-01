/* 4 行业预设 · P5 · embedded · 任务定义
 * 出典: industry-knowledge.md · Mavis
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p5'] = {
    phase: 'P5', phaseId: 'P5',
    industry: 'embedded', industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P5-EMB-001',
        title: 'MISRA-C 静的解析 (QAC/Coverity/C-STAT) 組込',
        desc: 'MISRA-C:2012 (Rule/Advisory) を含む静的解析を CI パイプラインに組込み、Deviation 登録された項目のみ許容。CI ゲートで重大違反 0 を保証。',
        priority: 'P0',
        tags: ['組込', 'MISRA-C', '静的解析', 'CI'],
        linkedDocs: ['DOC-07'],
        reviewPoints: ['RP-04', 'RP-05'],
        estimate: 8
      },
      {
        id: 'P5-EMB-002',
        title: '単体テスト (VectorCAST/Ceedling) 実装',
        desc: 'ISO 26262 / IEC 61508 の要求に応じ、モジュール単体・統合単体・ISR 単体を VectorCAST または Ceedling で実装。分岐/Modified Condition/Decision カバレッジを測定し目標達成。',
        priority: 'P0',
        tags: ['組込', '単体テスト', 'VectorCAST', 'カバレッジ', 'ISO 26262'],
        linkedDocs: ['DOC-14'],
        reviewPoints: ['RP-04'],
        estimate: 12
      },
      {
        id: 'P5-EMB-003',
        title: '性能プロファイリング/トレース (Tracealyzer/perf)',
        desc: 'RTOS (FreeRTOS/AUTOSAR) 上で Tracealyzer または perf でタスク実行時間/優先度逆転/スタック使用量を計測し、リアルタイム性能要件に対する余裕度を数値化。プロファイリングログは成果物として保管。',
        priority: 'P1',
        tags: ['組込', '性能', 'プロファイリング', 'RTOS', 'Tracealyzer'],
        linkedDocs: ['DOC-19'],
        reviewPoints: ['RP-04', 'RP-05'],
        estimate: 8
      }
    ]
  };
})(window);
