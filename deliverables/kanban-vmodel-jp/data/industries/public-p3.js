/* 4 行业预设 · P3 · public · 任务定义
 * 出典: industry-knowledge.md §P3 基本設計 (公共)
 * 重点: ユニバーサルデザイン / 多言語切替 / 公開鍵基盤 (LGPKI) / 共通基盤連携
 */
(function (global) {
  'use strict';
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p3'] = {
    phase: 'P3',
    phaseId: 'P3',
    industry: 'public',
    industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P3-PUB-001',
        title: 'ユニバーサルデザイン (JIS X 8341) 設計',
        desc: 'JIS X 8341-3:2016 適合等級 AA 達成方針とアクセシビリティ API・ARIA ロール・キーボード操作の画面層設計。',
        priority: 'P0',
        tags: ['公共', 'アクセシビリティ', 'JIS'],
        linkedDocs: ['DOC-05', 'DOC-09'],
        reviewPoints: ['RP-02'],
        estimate: 6
      },
      {
        id: 'P3-PUB-002',
        title: '多言語切替 + LGPKI 認証基盤設計',
        desc: 'i18n 切替フレームワーク (ja/en/zh/ko 等) + LGPKI / 公的個人認証サービスによる本人認証 IC カード連携設計。',
        priority: 'P0',
        tags: ['公共', '多言語', 'LGPKI', '認証'],
        linkedDocs: ['DOC-13'],
        reviewPoints: ['RP-02'],
        estimate: 8
      },
      {
        id: 'P3-PUB-003',
        title: '政府共通プラットフォーム / マイナンバー連携設計',
        desc: 'ガバメントクラウド共通基盤 + マイナンバー API (情報提供ネットワーク) 連携 + 既存 LGWAN システム IF 設計。',
        priority: 'P0',
        tags: ['公共', 'マイナンバー', '共通基盤'],
        linkedDocs: ['DOC-12', 'DOC-20'],
        reviewPoints: ['RP-02'],
        estimate: 8
      }
    ]
  };
})(window);
