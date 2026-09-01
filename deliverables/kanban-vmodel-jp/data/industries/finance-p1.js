/* 4 行业预设 · P1 · finance · 任务定义
 * 出典: industry-knowledge.md
 * 主题: 金融商品取引法/FISC/犯罪収益移転防止法/KYC/AML
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p1'] = {
    phase: 'P1', phaseId: 'P1',
    industry: 'finance', industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P1-FIN-001',
        title: '金融商品取引法適用範囲評価 + 監督指針整理',
        desc: '対象業務が金商法の適用を受けるか (第一種/第二種/投資助言/運用等) を判定し、監督局指針・自主規制団体 (日証協/全銀協等) のルールを整理。',
        priority: 'P0',
        tags: ['金融', '金商法'],
        linkedDocs: ['DOC-01'],
        reviewPoints: ['RP-12'],
        estimate: 8
      },
      {
        id: 'P1-FIN-002',
        title: 'FISC 安全対策基準ギャップ分析 + KYC/AML リスク評価',
        desc: 'FISC「金融機関等コンピュータシステムの安全対策基準」最新版とのギャップ分析と、商品/顧客/取引パターン別の AML リスク評価フレームを立上げ。',
        priority: 'P0',
        tags: ['金融', 'FISC'],
        linkedDocs: ['DOC-02'],
        reviewPoints: ['RP-12'],
        estimate: 8
      },
      {
        id: 'P1-FIN-003',
        title: '監督当局報告体制 + 法定帳簿・帳票要件整理',
        desc: '犯罪収益移転防止法に基づく疑わしい取引届出手続と、法定帳簿 (仕訳帳/総勘定元帳等) の保管期間・電子的保存要件の企画化。',
        priority: 'P1',
        tags: ['金融', 'AML'],
        linkedDocs: ['DOC-03'],
        reviewPoints: ['RP-12'],
        estimate: 6
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
