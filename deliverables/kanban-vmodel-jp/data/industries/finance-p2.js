/* 4 行业预设 · P2 · finance · 任务定义
 * 出典: industry-knowledge.md
 * 主题: 法令要件收集 (金商法/割賦販売法)/リスクアセスメント/KYC/監査ログ要件
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p2'] = {
    phase: 'P2', phaseId: 'P2',
    industry: 'finance', industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P2-FIN-001',
        title: '法令・規制要件收集 (金商法/割賦販売法/犯収法)',
        desc: '金融商品取引法・割賦販売法・犯罪収益移転防止法などの適用法令・監督指針を收集し、要件項目として体系化。',
        priority: 'P0',
        tags: ['金融', '法令'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 8
      },
      {
        id: 'P2-FIN-002',
        title: 'リスクアセスメント + KYC/AML 要件定義',
        desc: '商品特性・取引パターンに基づく AML リスク評価と、顧客確認 (KYC) レベル・スコアリング・疑わしい取引届出の業務要件化。',
        priority: 'P0',
        tags: ['金融', 'KYC/AML'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 8
      },
      {
        id: 'P2-FIN-003',
        title: '監査ログ要件 (FISC 安全対策基準準拠)',
        desc: 'FISC 安全対策基準に基づく監査ログの取得項目・保管期間・完全性保証・不正改ざん防止の NFR 化。',
        priority: 'P0',
        tags: ['金融', 'FISC'],
        linkedDocs: ['DOC-04'],
        reviewPoints: ['RP-01'],
        estimate: 6
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
