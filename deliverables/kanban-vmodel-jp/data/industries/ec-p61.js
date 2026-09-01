/* 4 行业预设 · P6.1 · ec · 任务定义
 * 出典: industry-knowledge.md (P6.1 単体試験 · ec 重点)
 * 重点: 決済金額計算 / 送料計算 / 在庫引当単体
 * 業界: E コマース・ Marketplace・サブスク (3DS / 不正検知 / 在庫 / 配送)
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p61'] = {
    phase: 'P6.1', phaseId: 'P6.1',
    industry: 'ec', industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P61-EC-001',
        title: '決済金額計算単体試験',
        desc: '通貨換算 / 端数処理 (四捨五入・切り捨て) / 税 (内税・外税) / 割引 / クーポン / 返金額計算の単体試験 (カード非保持化前提)。',
        priority: 'P0',
        tags: ['EC', 'UT', '決済', '金額計算'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 6
      },
      {
        id: 'P61-EC-002',
        title: '送料計算単体試験',
        desc: '地域 (都道府県・離島) / 重量 / サイズ / クール便 (冷凍・冷蔵) / 代引手数料 / まとめ配送割引 の単体試験。',
        priority: 'P0',
        tags: ['EC', 'UT', '送料', '物流'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 5
      },
      {
        id: 'P61-EC-003',
        title: '在庫引当単体試験',
        desc: '在庫引当 / 解放 (注文キャンセル・タイムアウト) / 在庫切れ / 排他制御 (楽観ロック・ versioning) の単体試験。',
        priority: 'P0',
        tags: ['EC', 'UT', '在庫', '排他制御'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 6
      }
    ]
  };
})(window);
