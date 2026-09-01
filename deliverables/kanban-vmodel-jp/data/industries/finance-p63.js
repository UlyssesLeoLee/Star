/* 4 行业预设 · P6.3 · finance · 任务定义
 * 出典: industry-knowledge.md · P6.3 システム試験 (金融)
 *       重点: 負荷試験 (同時取引数) / DR 切替 / 監査ログ完全性
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p63'] = {
    phase: 'P6.3', phaseId: 'P6.3',
    industry: 'finance', industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P63-FIN-001',
        title: '同時取引負荷試験',
        desc: '通常時・ピーク時・キャンペーン時の同時取引数 (TPS) を段階的に上げ、ボトルネックと限界点を計測。',
        priority: 'P0',
        tags: ['金融', 'ST', 'LT', 'TT-06'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 16
      },
      {
        id: 'P63-FIN-002',
        title: 'DR 切替・復旧試験',
        desc: 'FISC 安全対策基準に基づく DR サイト切替手順と RPO / RTO 目標の達成検証を本番相当環境で実施。',
        priority: 'P0',
        tags: ['金融', 'ST', 'TT-09', 'FISC'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 8
      },
      {
        id: 'P63-FIN-003',
        title: '監査ログ完全性検証',
        desc: '全取引の監査ログ欠落なし / 改ざん不可 / 法定保存期間内の取得を検証。FISC 監査証跡要件の充足確認。',
        priority: 'P0',
        tags: ['金融', 'ST', '監査', 'FISC'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-07'],
        estimate: 6
      }
    ]
  };
})(window);
