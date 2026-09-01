/* 4 行业预设 · P4 · embedded · 任务定义
 * 出典: industry-knowledge.md
 * 主题: MISRA-C 準拠詳細/メモリ管理詳細/ISR 詳細/ブートローダ詳細
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p4'] = {
    phase: 'P4', phaseId: 'P4',
    industry: 'embedded', industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P4-EMB-001',
        title: 'MISRA-C:2012 (Amendment 1) 準拠実装詳細',
        desc: 'MISRA-C:2012 Amendment 1 の各 Rule 実装方針。必須/必須ではない Rule の区分・deviation 記録 (理由/影響範囲/代替策) ・静的解析 (Coverity / C-STAT / Polyspace) 設定・コーディング規約の派生ルール・トレーサビリティ (Rule → 該当コード) の詳細仕様。',
        priority: 'P0',
        tags: ['組込', 'MISRA-C'],
        linkedDocs: ['DOC-06'],
        reviewPoints: ['RP-03'],
        estimate: 8
      },
      {
        id: 'P4-EMB-002',
        title: 'メモリ管理詳細 (MPU / プール / ASIL 別制約)',
        desc: 'メモリ管理詳細。静的メモリプール (固定サイズ) ・動的メモリプール (TLSF / dlmalloc) ・フラグメンテーション対策・MPU 領域分割 (カーネル/タスク/共有) ・ASIL レベル別メモリ制約 (B: ECC 必須, D: 二重化) ・リーク検出 (W-MISRAC / Valgrind) の実装仕様。',
        priority: 'P0',
        tags: ['組込', 'メモリ'],
        linkedDocs: ['DOC-06'],
        reviewPoints: ['RP-03'],
        estimate: 6
      },
      {
        id: 'P4-EMB-003',
        title: 'ISR 詳細 (割込み優先度 / ネスト / 同期)',
        desc: 'ISR (Interrupt Service Routine) 詳細。割込み優先度 (NVIC グループ) ・ネスト制御・クリティカルセクション長 (max 50 命令) ・割込みレイテンシ (最悪値) ・ISR-タスク同期 (イベントフラグ / セマフォ / メールボックス) ・優先度逆転 (Priority Inheritance) の実装仕様。',
        priority: 'P1',
        tags: ['組込', 'ISR'],
        linkedDocs: ['DOC-06'],
        reviewPoints: ['RP-03'],
        estimate: 6
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
