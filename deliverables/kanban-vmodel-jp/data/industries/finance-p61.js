/* 4 行业预设 · P6.1 · finance · 任务定义
 * 出典: industry-knowledge.md (P6.1 単体試験 · finance 重点)
 * 重点: 暗号化関数の境界値 / 認証フロー分岐 / KYC スコアリング単体
 * 業界: 銀行・証券・保険・決済 (FISC/PCI DSS/金商法/犯収法)
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p61'] = {
    phase: 'P6.1', phaseId: 'P6.1',
    industry: 'finance', industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P61-FIN-001',
        title: '暗号化関数の境界値試験',
        desc: 'AES-256 / HMAC-SHA256 / PBKDF2 等の暗号化関数に対し、空文字列・最大長・null・特殊文字・改行を含む境界値の単体試験 (UTP/UTS 準拠)。',
        priority: 'P0',
        tags: ['金融', 'UT', '暗号化', 'PCI DSS'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 6
      },
      {
        id: 'P61-FIN-002',
        title: '認証フロー分岐試験',
        desc: '多要素認証 (MFA) の成功 / 失敗 / アカウントロック / リトライ上限超過 / セッションタイムアウトの各分岐に対する単体試験。',
        priority: 'P0',
        tags: ['金融', 'UT', '認証', 'MFA'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 6
      },
      {
        id: 'P61-FIN-003',
        title: 'KYC スコアリング単体試験',
        desc: 'KYC リスクスコア計算ロジック (年齢・国籍・職業・取引履歴・PEP 該当) の正常系・異常系・境界値の単体試験。',
        priority: 'P1',
        tags: ['金融', 'UT', 'KYC', 'AML'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 5
      }
    ]
  };
})(window);
