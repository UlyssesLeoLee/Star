/* 4 行业预设 · P3 · ec · 任务定义
 * 出典: industry-knowledge.md §P3 基本設計 (EC)
 * 重点: 決済連携 (PSP) 設計 / 在庫同期 / 配送 API / カード非保持化
 */
(function (global) {
  'use strict';
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p3'] = {
    phase: 'P3',
    phaseId: 'P3',
    industry: 'ec',
    industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P3-EC-001',
        title: 'PSP 決済連携 (3D セキュア) 設計',
        desc: 'PSP (Stripe / Veritrans / PAY.JP 等) との API 連携方式 + EMV 3DS 2.0 認証フロー + 与信・売上確定の二段階設計。',
        priority: 'P0',
        tags: ['EC', 'PSP', '3DS', '決済'],
        linkedDocs: ['DOC-12', 'DOC-13'],
        reviewPoints: ['RP-02'],
        estimate: 8
      },
      {
        id: 'P3-EC-002',
        title: 'カード非保持化 (トークン化) 設計',
        desc: 'リダイレクト / トークン方式による PCI DSS 対象外化 + 自社 DB・ログへの PAN 非保持ポリシーとマスキング設計。',
        priority: 'P0',
        tags: ['EC', 'PCI DSS', '非保持化', 'セキュリティ'],
        linkedDocs: ['DOC-13'],
        reviewPoints: ['RP-02'],
        estimate: 6
      },
      {
        id: 'P3-EC-003',
        title: '在庫同期 + 配送 API 設計',
        desc: 'WMS / 倉庫システムとの在庫同期 (楽観ロック / イベント駆動) + 配送キャリア API (ヤマト/佐川) 連携と発送ステータス設計。',
        priority: 'P1',
        tags: ['EC', '在庫', '物流', 'API'],
        linkedDocs: ['DOC-12'],
        reviewPoints: ['RP-02'],
        estimate: 6
      }
    ]
  };
})(window);
