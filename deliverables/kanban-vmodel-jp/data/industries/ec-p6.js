/* 4 行业预设 · P6 · ec · 任务定义
 * 出典: industry-knowledge.md (per 2026-09-01 21:25 JST 業界知識)
 * 適用: E コマース・マーケットプレイス・サブスク 向け P6 テスト工程 (主 phase) 跨子 phase 工程管理級タスク
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p6'] = {
    phase: 'P6',
    phaseId: 'P6',
    industry: 'ec',
    industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P6-EC-001',
        title: '決済試験 (3D セキュア / リダイレクト) 統合管理',
        desc: 'EMV 3DS 2.x 認証フロー (リスクベース認証・免責 / 不免責パターン) とトークン化リダイレクト決済を P6.2 PSP 結合 / P6.3 大量決済 / P6.4 業務シナリオ で横断検証。カード番号非保持と与信・売上確定の整合を RP-06/07/08 で確認。',
        priority: 'P0',
        tags: ['EC', '決済', '3DS', 'リダイレクト'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-06', 'RP-07', 'RP-08'],
        estimate: 10
      },
      {
        id: 'P6-EC-002',
        title: '性能試験 (キャンペーン) / 不正検知試験 / 在庫整合性 統合管理',
        desc: 'キャンペーン時の想定ピーク RPS 負荷試験を P6.3 で実施し、在庫引当・戻入の整合性テストを P6.1 単体 (境界値) / P6.3 大量決済時 / P6.4 ピーク日シミュレーション で検証。不正検知ルール (デバイス指紋・行動分析) のテストを RP-04/07/08 ゲートと連動。',
        priority: 'P0',
        tags: ['EC', '性能', '不正検知', '在庫'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04', 'RP-07', 'RP-08'],
        estimate: 12
      }
    ]
  };
})(window);
