/* 4 行业预设 · P5 · finance · 任务定义
 * 出典: industry-knowledge.md · Mavis
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['finance-p5'] = {
    phase: 'P5', phaseId: 'P5',
    industry: 'finance', industryJa: '金融',
    color: '#dc2626',
    tasks: [
      {
        id: 'P5-FIN-001',
        title: 'セキュアコーディング (OWASP) 実装',
        desc: 'OWASP Top 10 / ASVS を基準に、入力検証・出力エンコード・認証・セッション管理・ログ取得をコードレベルで実装。SQLi/XSS/CSRF/SSRF 等の混入ゼロを SAST で証明する。',
        priority: 'P0',
        tags: ['金融', 'セキュア', 'OWASP', '実装'],
        linkedDocs: ['DOC-07', 'DOC-13'],
        reviewPoints: ['RP-04', 'RP-05'],
        estimate: 12
      },
      {
        id: 'P5-FIN-002',
        title: 'SAST/コード署名/シークレット管理 (Vault) 導入',
        desc: 'CI パイプラインに SAST (SonarQube/Semgrep) と コード署名 (Sigstore/HSM) を組込み、秘密情報は HashiCorp Vault の動的シークレットで一元管理。コミット時とビルド時の二段で検出。',
        priority: 'P0',
        tags: ['金融', 'SAST', '署名', 'Vault', 'DevSecOps'],
        linkedDocs: ['DOC-13'],
        reviewPoints: ['RP-04', 'RP-05'],
        estimate: 8
      },
      {
        id: 'P5-FIN-003',
        title: '暗号化 API (HSM/AES-256/TLS1.3) 実装',
        desc: 'FISC 安全対策基準に基づき、HSM 経由の鍵操作・AES-256-GCM による保存データ暗号化・TLS1.3 による通信暗号化を API レイヤで実装。鍵ローテーション API も同梱。',
        priority: 'P0',
        tags: ['金融', '暗号化', 'HSM', 'TLS1.3', 'FISC'],
        linkedDocs: ['DOC-07', 'DOC-13'],
        reviewPoints: ['RP-04', 'RP-05'],
        estimate: 16
      }
    ]
  };
})(window);
