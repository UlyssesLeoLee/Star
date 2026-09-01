/* 4 行业预设 · P6.2 · finance · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p62'] = {
    phase: 'P6.2', phaseId: 'P62',
    industry: 'finance', industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P62-FIN-001',
        title: 'FISC ネットワーク経由 API 結合試験',
        desc: 'FISC 安全対策基準に準拠した対外接続ネットワーク (クローズド・IP-VPN 等) 経由の REST/メッセージ API 結合試験。',
        priority: 'P0',
        tags: ['金融','FISC','API'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 8
      },
      {
        id: 'P62-FIN-002',
        title: '与信接続試験 (信用情報機関 CIC/JICC)',
        desc: ' CIC・JICC・KSC 等の信用情報機関との与信照会/登録 API 結合。',
        priority: 'P0',
        tags: ['金融','与信','外部IF'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 6
      },
      {
        id: 'P62-FIN-003',
        title: '口座振替接続試験 (全銀システム/内国為替)',
        desc: '全銀システム (ZENGIN) / 内国為替制度に基づく口座振替 API 結合。',
        priority: 'P0',
        tags: ['金融','決済','外部IF'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 8
      }
    ]
  };
})(window);
