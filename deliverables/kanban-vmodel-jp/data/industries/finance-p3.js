/* 4 行业预设 · P3 · finance · 任务定义
 * 出典: industry-knowledge.md §P3 基本設計 (金融)
 * 重点: FISC 準拠セキュリティ設計 / AES-256 + HSM / 監査ログ / 二重化 / DR 設計
 */
(function (global) {
  'use strict';
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p3'] = {
    phase: 'P3',
    phaseId: 'P3',
    industry: 'finance',
    industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P3-FIN-001',
        title: 'FISC 準拠セキュリティ設計',
        desc: 'FISC「金融機関等コンピュータシステムの安全対策基準」最新版への準拠方針・適用基準・例外管理プロセスを設計。',
        priority: 'P0',
        tags: ['金融', 'FISC', 'セキュリティ'],
        linkedDocs: ['DOC-13'],
        reviewPoints: ['RP-02'],
        estimate: 8
      },
      {
        id: 'P3-FIN-002',
        title: '暗号鍵管理 (HSM) 設計',
        desc: 'AES-256 鍵のライフサイクル (生成・配布・更新・廃棄) 管理と HSM による鍵保管・アクセス制御・テンポラル権限制御を設計。',
        priority: 'P0',
        tags: ['金融', '暗号化', 'HSM'],
        linkedDocs: ['DOC-13'],
        reviewPoints: ['RP-02'],
        estimate: 6
      },
      {
        id: 'P3-FIN-003',
        title: '監査ログ + 二重化 / DR 設計',
        desc: '改ざん不可監査ログ (WORM/デジタル署名) + Active-Active 二重化 + DR サイト (RPO/RTO) 構成を設計。',
        priority: 'P0',
        tags: ['金融', '監査', 'DR', '可用性'],
        linkedDocs: ['DOC-13', 'DOC-20'],
        reviewPoints: ['RP-02'],
        estimate: 6
      }
    ]
  };
})(window);
