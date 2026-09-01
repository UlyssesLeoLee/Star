/* 4 行业预设 · P6.3 · public · 任务定义
 * 出典: industry-knowledge.md · P6.3 システム試験 (公共)
 *       重点: 全シナリオ試験 / アクセシビリティ試験 / 負荷試験
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p63'] = {
    phase: 'P6.3', phaseId: 'P6.3',
    industry: 'public', industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P63-PUB-001',
        title: '全業務シナリオ試験',
        desc: '全業務フロー / 例外シナリオ / 季節性シナリオ (年度末・申請集中期) を網羅した E2E 検証。',
        priority: 'P0',
        tags: ['公共', 'ST', 'シナリオ', 'TT-03'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 12
      },
      {
        id: 'P63-PUB-002',
        title: 'アクセシビリティ試験 (JIS X 8341)',
        desc: 'JIS X 8341-3 準拠レベル AA の達成検証。NVDA / JAWS / VoiceOver 等支援技術での読み上げ・操作確認。',
        priority: 'P0',
        tags: ['公共', 'ST', 'a11y', 'JIS'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 10
      },
      {
        id: 'P63-PUB-003',
        title: '負荷試験 (申請集中期)',
        desc: '年度末・税制改正・災害時申請など申請集中期の想定同時アクセス数での性能維持とボトルネック計測。',
        priority: 'P1',
        tags: ['公共', 'ST', 'LT', 'TT-06'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 8
      }
    ]
  };
})(window);
