/* 4 行业预设 · P2 · ec · 任务定义
 * 出典: industry-knowledge.md
 * 主题: 商品/決済/物流/在庫/プロモーション要件/カード情報非保持要件
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p2'] = {
    phase: 'P2', phaseId: 'P2',
    industry: 'ec', industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P2-EC-001',
        title: '商品/決済/物流/在庫/プロモーション要件',
        desc: '商品マスタ・価格ルール・決済手段・送料/配送リードタイム・在庫引当・クーポン/ポイントのプロモーション要件化。',
        priority: 'P0',
        tags: ['EC', '要件'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 8
      },
      {
        id: 'P2-EC-002',
        title: 'カード情報非保持要件 (PCI DSS / EMV 3DS)',
        desc: '加盟店におけるカード情報非保持化と EMV 3D セキュア必須化 (2025 年 3 月以降) に伴うリダイレクト/トークン化の要件化。',
        priority: 'P0',
        tags: ['EC', 'PCI DSS'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 6
      },
      {
        id: 'P2-EC-003',
        title: '不正検知 + 3D セキュア運用要件',
        desc: '不正ログイン・チャージバック抑止のための不正検知ルール (デバイス/行動/住所) と 3DS 認証フローの運用要件化。',
        priority: 'P1',
        tags: ['EC', '3DS'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 6
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
