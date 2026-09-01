/* 4 行业预设 · P6 · embedded · 任务定义
 * 出典: industry-knowledge.md (per 2026-09-01 21:25 JST 業界知識)
 * 適用: 車載/IoT/医療機器/産業機器 向け P6 テスト工程 (主 phase) 跨子 phase 工程管理級タスク
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p6'] = {
    phase: 'P6',
    phaseId: 'P6',
    industry: 'embedded',
    industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P6-EMB-001',
        title: 'HIL テスト / EMC 試験 統合管理',
        desc: 'Hardware-in-the-Loop による HW-SW 結合試験を P6.2 で実施し、EMC (電磁両立性) 試験 (放射エミッション / イミュニティ) を P6.3 システム試験で実機検証。車載ネットワーク (CAN/LIN) 通信負荷下での振る舞いを RP-06/07 ゲートと連動。',
        priority: 'P0',
        tags: ['組込', 'HIL', 'EMC', '車載'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-06', 'RP-07'],
        estimate: 10
      },
      {
        id: 'P6-EMB-002',
        title: '安全解析レビュー / 性能ベンチ 統合管理',
        desc: 'ISO 26262 ASIL に対応する FTA (故障の木解析) / FMEA (故障モード影響解析) レビューを P6.1 モジュール単体段階から P6.4 量産前試作まで継続。リアルタイム性能ベンチ (割込応答 / タスクジッタ) を P6.3 で計測し、安全目標逸脱を RP-04/07/08 で検出。',
        priority: 'P0',
        tags: ['組込', 'ISO 26262', 'FTA/FMEA', '性能'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04', 'RP-07', 'RP-08'],
        estimate: 12
      }
    ]
  };
})(window);
