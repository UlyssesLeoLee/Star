/* 4 行业预设 · P2 · public · 任务定义
 * 出典: industry-knowledge.md
 * 主题: アクセシビリティ要件 (JIS X 8341-3)/多言語要件/個人情報保護要件/法令遵守要件
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p2'] = {
    phase: 'P2', phaseId: 'P2',
    industry: 'public', industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P2-PUB-001',
        title: 'アクセシビリティ要件 (JIS X 8341-3 準拠)',
        desc: 'JIS X 8341-3:2016 適合レベル AA を満たす WCAG 2.1 達成基準の要件化。対象画面・支援技術・検証方法を明確化。',
        priority: 'P0',
        tags: ['公共', 'JIS'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 8
      },
      {
        id: 'P2-PUB-002',
        title: '多言語要件 (やさしい日本語/英語/中国語等)',
        desc: '住民向け画面の多言語対応範囲、翻訳更新フロー、文化依存表現・通貨・日付フォーマット等のローカライゼーション要件化。',
        priority: 'P1',
        tags: ['公共', '多言語'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 6
      },
      {
        id: 'P2-PUB-003',
        title: '個人情報保護 + 法令遵守要件 (個情法/デジタル手続法)',
        desc: '個人情報保護法・行政手続オンライン化法・番号利用法等に基づく取得項目・保管期間・PIA・第三者提供の要件化。',
        priority: 'P0',
        tags: ['公共', '法令'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 6
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
