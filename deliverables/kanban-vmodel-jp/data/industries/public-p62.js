/* 4 行业预设 · P6.2 · public · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p62'] = {
    phase: 'P6.2', phaseId: 'P62',
    industry: 'public', industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P62-PUB-001',
        title: 'LGWAN 接続試験',
        desc: '総合行政ネットワーク (LGWAN) 経由の外部システム接続試験。',
        priority: 'P0',
        tags: ['公共','LGWAN','外部IF'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 6
      },
      {
        id: 'P62-PUB-002',
        title: 'マイナンバー API 連携試験 (情報提供ネットワーク)',
        desc: 'マイナンバー制度に基づく情報提供等記録開示システム / マイナポータル API 連携。',
        priority: 'P0',
        tags: ['公共','マイナンバー','API'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 8
      },
      {
        id: 'P62-PUB-003',
        title: '政府共通プラットフォーム / ガバメントクラウド API 試験',
        desc: '政府共通プラットフォーム・ガバメントクラウド上の共通基盤 API (認証/通知/ファイル) との結合。',
        priority: 'P1',
        tags: ['公共','共通基盤','API'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 6
      }
    ]
  };
})(window);
