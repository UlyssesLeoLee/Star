/* 4 行业预设 · P5 · ec · 任务定义
 * 出典: industry-knowledge.md · Mavis
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p5'] = {
    phase: 'P5', phaseId: 'P5',
    industry: 'ec', industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P5-EC-001',
        title: '決済 API 実装 (PSP/EMV 3DS/トークン化)',
        desc: 'PSP (Stripe/PAY.JP 等) へのリダイレクト型/トークン型決済、EMV 3-D Secure 2.0 による本人認証、カード情報の非保持化を実装。PCI DSS スコープ外に保つ構成。',
        priority: 'P0',
        tags: ['EC', '決済', 'PSP', '3DS', 'PCI DSS'],
        linkedDocs: ['DOC-07', 'DOC-11', 'DOC-13'],
        reviewPoints: ['RP-04', 'RP-05'],
        estimate: 16
      },
      {
        id: 'P5-EC-002',
        title: '不正検知ルール/スコアリング 実装',
        desc: 'デバイス指紋・IP リスク・行動パターン・配送先不一致のスコアリングで与信前ブロック/手動審査キューを分岐。ルール DSL を分離して A/B 検証可能にする。',
        priority: 'P0',
        tags: ['EC', '不正検知', 'スコアリング', 'ルール'],
        linkedDocs: ['DOC-07', 'DOC-11'],
        reviewPoints: ['RP-04', 'RP-05'],
        estimate: 12
      },
      {
        id: 'P5-EC-003',
        title: '在庫引当/在庫切れ/予約処理 実装',
        desc: '注文確定時の楽観ロックによる在庫引当、決済失敗時の自動解放タイム、在庫切れ時の代替提示/入荷通知登録までを API として実装。在庫整合性は夜間バッチと差分同期。',
        priority: 'P0',
        tags: ['EC', '在庫', '引当', 'トランザクション'],
        linkedDocs: ['DOC-07', 'DOC-08'],
        reviewPoints: ['RP-04', 'RP-05'],
        estimate: 8
      }
    ]
  };
})(window);
