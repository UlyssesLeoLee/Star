/* 4 行业预设 · P1 · public · 任务定义
 * 出典: industry-knowledge.md
 * 主题: 政策立案/予算要求/PIA/アクセシビリティ方針
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p1'] = {
    phase: 'P1', phaseId: 'P1',
    industry: 'public', industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P1-PUB-001',
        title: '政策立案支援 + 関連省庁協議',
        desc: '自治体/省庁の政策課題からシステム化企画を起案し、主管省庁・デジタル庁・総務省等との事前協議・規制確認を実施。',
        priority: 'P0',
        tags: ['公共', '政策'],
        linkedDocs: ['DOC-01'],
        reviewPoints: ['RP-12'],
        estimate: 8
      },
      {
        id: 'P1-PUB-002',
        title: '予算要求 + 個人情報保護影響評価 (PIA)',
        desc: 'デジタル田園都市国家構想交付金等の財源を見据えた予算要求資料の作成と、個人情報保護法に基づく PIA (影響評価) の初期実施。',
        priority: 'P0',
        tags: ['公共', 'PIA'],
        linkedDocs: ['DOC-02', 'DOC-03'],
        reviewPoints: ['RP-12'],
        estimate: 8
      },
      {
        id: 'P1-PUB-003',
        title: 'アクセシビリティ方針 (JIS X 8341 適合等級決定)',
        desc: 'JIS X 8341-3:2016 適合レベル (AA/AAA) のターゲット決定と、誰一人取り残さないデジタル社会実現に向けた対応範囲の方針化。',
        priority: 'P1',
        tags: ['公共', 'JIS'],
        linkedDocs: ['DOC-03'],
        reviewPoints: ['RP-12'],
        estimate: 4
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
