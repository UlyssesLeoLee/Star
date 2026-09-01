/* 4 行业预设 · P4 · finance · 任务定义
 * 出典: industry-knowledge.md
 * 主题: PCI DSS 制御実装詳細/暗号化 API 詳細/監査ログ詳細/二要素認証
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p4'] = {
    phase: 'P4', phaseId: 'P4',
    industry: 'finance', industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P4-FIN-001',
        title: 'PCI DSS v4.0 制御実装詳細',
        desc: 'PCI DSS v4.0 の Req 3 (保存データ保護) / Req 4 (伝送暗号化) / Req 6 (セキュア開発) / Req 8 (識別認証) / Req 10 (監査ログ) の実装詳細 (鍵管理・カードデータ非保持化・PAN トークン化・鍵ローテーション・アクセス制御マトリクス)。',
        priority: 'P0',
        tags: ['金融', 'PCI DSS'],
        linkedDocs: ['DOC-06'],
        reviewPoints: ['RP-03'],
        estimate: 8
      },
      {
        id: 'P4-FIN-002',
        title: '暗号化 API 詳細仕様 (AES-256-GCM + HSM)',
        desc: 'HSM (Hardware Security Module) 連携の暗号化 API 詳細。AES-256-GCM の鍵長・IV 生成・AAD 設定・Argon2id ハッシュ・鍵ローテーション・鍵アクセス API 仕様・鍵分離 (KEK/DEK) の関数 IF・エラーコード・性能要件。',
        priority: 'P0',
        tags: ['金融', '暗号化'],
        linkedDocs: ['DOC-06'],
        reviewPoints: ['RP-03'],
        estimate: 8
      },
      {
        id: 'P4-FIN-003',
        title: '二要素認証実装詳細 (TOTP / FIDO2 WebAuthn)',
        desc: 'TOTP (RFC 6238) と FIDO2 WebAuthn の多要素認証詳細。リスクベース認証 (金額/属性/位置) ・デバイス紐付け・バックアップコード・セッション管理 (Cookie 属性 / SameSite / Secure) ・不正ロックアウトの関数 IF・状態遷移。',
        priority: 'P0',
        tags: ['金融', '認証'],
        linkedDocs: ['DOC-06'],
        reviewPoints: ['RP-03'],
        estimate: 6
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
