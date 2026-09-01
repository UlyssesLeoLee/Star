/* 4 行业预设 · P6.4 · finance · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p64'] = {
    phase: 'P6.4', phaseId: 'P6.4',
    industry: 'finance', industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P64-FIN-001',
        title: '業務部門 UAT (与信/口座振替シナリオ)',
        desc: '与信審査・口座振替・キャッシュカードの業務部門による受入確認。実データでの再現試験。',
        priority: 'P0',
        tags: ['金融','UAT','与信','業務'],
        linkedDocs: ['DOC-15'],
        reviewPoints: ['RP-08','RP-09','RP-10'],
        estimate: 12
      },
      {
        id: 'P64-FIN-002',
        title: 'コンプライアンス部門承認 / 監査法人立会',
        desc: 'コンプライアンス・リスク管理部門による最終承認、および監査法人の立会検証 (内部統制評価)。',
        priority: 'P0',
        tags: ['金融','UAT','コンプラ','監査法人'],
        linkedDocs: ['DOC-15'],
        reviewPoints: ['RP-08','RP-09','RP-10'],
        estimate: 8
      },
      {
        id: 'P64-FIN-003',
        title: '監督当局向け操作ログ / 証跡レビュー',
        desc: '監督当局の求めに応じ提示可能な操作ログ・証跡の完全性レビュー。法定保存期間との整合確認。',
        priority: 'P1',
        tags: ['金融','UAT','監査','証跡'],
        linkedDocs: ['DOC-15'],
        reviewPoints: ['RP-08'],
        estimate: 6
      }
    ]
  };
})(window);
