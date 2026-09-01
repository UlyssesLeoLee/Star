/* 4 行业预设 · P7 · public · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p7'] = {
    phase: 'P7', phaseId: 'P7',
    industry: 'public', industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P7-PUB-001',
        title: '住民向け周知 (広報・HP・SNS・広報誌)',
        desc: 'システム切替に関する住民向け周知計画。広報誌・HP・SNS・テレビ・ラジオ等の複数チャネル活用。',
        priority: 'P0',
        tags: ['公共','周知','広報','住民'],
        linkedDocs: ['DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 8
      },
      {
        id: 'P7-PUB-002',
        title: '既存システム並行稼働 / 旧システム停止計画',
        desc: '旧システム並行稼働期間中のデータ同期と旧システム停止計画。住民サービスの無停止保証。',
        priority: 'P0',
        tags: ['公共','並行稼働','停止計画'],
        linkedDocs: ['DOC-17','DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 12
      },
      {
        id: 'P7-PUB-003',
        title: 'マイナンバー利用事務系への接続',
        desc: 'マイナンバー利用事務系 (情報提供等) との本番接続。LGWAN 経由の疎通・データ授受確認。',
        priority: 'P1',
        tags: ['公共','マイナンバー','LGWAN'],
        linkedDocs: ['DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 6
      }
    ]
  };
})(window);
