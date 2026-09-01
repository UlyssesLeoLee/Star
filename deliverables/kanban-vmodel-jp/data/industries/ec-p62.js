/* 4 行业预设 · P6.2 · ec · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p62'] = {
    phase: 'P6.2', phaseId: 'P62',
    industry: 'ec', industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P62-EC-001',
        title: 'PSP 接続試験 (3Dセキュア EMV 3DS 含む)',
        desc: 'クレジットカード決済 PSP (GMO/Veritrans/Stripe 等) との接続試験、EMV 3DS 認証フロー含む。',
        priority: 'P0',
        tags: ['EC','決済','PSP'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 8
      },
      {
        id: 'P62-EC-002',
        title: '物流 API 接続試験 (ヤマト運輸/佐川急便等)',
        desc: '主要配送業者の送り状発行・追跡 API との結合試験。',
        priority: 'P1',
        tags: ['EC','物流','外部IF'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 4
      },
      {
        id: 'P62-EC-003',
        title: '在庫同期試験 (OMS/WMS/店舗在庫)',
        desc: 'OMS / WMS / 実店舗在庫とのリアルタイム同期 API 結合試験、競合制御含む。',
        priority: 'P0',
        tags: ['EC','在庫','API'],
        linkedDocs: ['DOC-14','DOC-15'],
        reviewPoints: ['RP-06'],
        estimate: 6
      }
    ]
  };
})(window);
