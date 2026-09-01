/* 4 行业预设 · P6.4 · public · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p64'] = {
    phase: 'P6.4', phaseId: 'P6.4',
    industry: 'public', industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P64-PUB-001',
        title: '自治体職員 UAT (業務シナリオ実走)',
        desc: '実際の自治体職員による業務シナリオ実走。住民異動・税・福祉など主要業務で確認。',
        priority: 'P0',
        tags: ['公共','UAT','自治体','業務'],
        linkedDocs: ['DOC-15'],
        reviewPoints: ['RP-08','RP-09','RP-10'],
        estimate: 12
      },
      {
        id: 'P64-PUB-002',
        title: '住民モニター試験 (アクセシビリティ当事者参加)',
        desc: '視覚・聴覚障害当事者を含む住民モニターによる実利用試験。支援技術 (NVDA/JAWS/音声ブラウザ) 併用。',
        priority: 'P0',
        tags: ['公共','UAT','アクセシビリティ','当事者'],
        linkedDocs: ['DOC-15'],
        reviewPoints: ['RP-08'],
        estimate: 8
      },
      {
        id: 'P64-PUB-003',
        title: '政府共通プラットフォーム / LGWAN 接続検証',
        desc: '政府共通プラットフォーム・LGWAN との接続検証。マイナンバー利用事務系の疎通・データ授受確認。',
        priority: 'P1',
        tags: ['公共','UAT','ガバメント','LGWAN'],
        linkedDocs: ['DOC-15'],
        reviewPoints: ['RP-08'],
        estimate: 6
      }
    ]
  };
})(window);
