/* 4 行业预设 · P6.3 · ec · 任务定义
 * 出典: industry-knowledge.md · P6.3 システム試験 (EC)
 *       重点: ピーク負荷 (キャンペーン) / カード大量決済 / 不正攻撃シミュレーション
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p63'] = {
    phase: 'P6.3', phaseId: 'P6.3',
    industry: 'ec', industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P63-EC-001',
        title: 'キャンペーン ピーク負荷試験',
        desc: 'タイムセール / 限定販売 / TV 放映などの瞬間ピーク PV と購入完了率を段階負荷で計測。',
        priority: 'P0',
        tags: ['EC', 'ST', 'LT', 'TT-06'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 16
      },
      {
        id: 'P63-EC-002',
        title: 'カード大量決済試験',
        desc: '3D セキュア (EMV 3DS) + トークン決済 + 与信応答遅延下での決済スループットと整合性を検証。',
        priority: 'P0',
        tags: ['EC', 'ST', '決済', '3DS'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 12
      },
      {
        id: 'P63-EC-003',
        title: '不正攻撃シミュレーション',
        desc: 'リスト型アカウント攻撃 / カード不正 / 在庫スクレイピング等を模擬し、不正検知ルールと WAF の有効性を検証。',
        priority: 'P0',
        tags: ['EC', 'ST', '不正', 'TT-07'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 8
      }
    ]
  };
})(window);
