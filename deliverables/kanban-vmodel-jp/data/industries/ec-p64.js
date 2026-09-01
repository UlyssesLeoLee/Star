/* 4 行业预设 · P6.4 · ec · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p64'] = {
    phase: 'P6.4', phaseId: 'P6.4',
    industry: 'ec', industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P64-EC-001',
        title: '業務シナリオ UAT (購入〜配送〜返品)',
        desc: '顧客視点での業務シナリオ UAT。商品検索・購入・決済・配送・返品の一連フローを業務部門で確認。',
        priority: 'P0',
        tags: ['EC','UAT','業務シナリオ','CX'],
        linkedDocs: ['DOC-15'],
        reviewPoints: ['RP-08','RP-09','RP-10'],
        estimate: 10
      },
      {
        id: 'P64-EC-002',
        title: 'ピーク日シミュレーション (セール/キャンペーン)',
        desc: 'ブラックフライデー・サイバーマンデー等のピーク日想定負荷で UAT。決済渋滞・カート落ち検証。',
        priority: 'P0',
        tags: ['EC','UAT','ピーク','キャンペーン'],
        linkedDocs: ['DOC-15'],
        reviewPoints: ['RP-08','RP-09'],
        estimate: 8
      },
      {
        id: 'P64-EC-003',
        title: '障害対応訓練 / CS エスカレーション実走',
        desc: '決済障害・配送遅延時の CS エスカレーション・代替フロー実走訓練。インシデント対応手順の有効性検証。',
        priority: 'P1',
        tags: ['EC','UAT','インシデント','CS'],
        linkedDocs: ['DOC-15'],
        reviewPoints: ['RP-08'],
        estimate: 4
      }
    ]
  };
})(window);
