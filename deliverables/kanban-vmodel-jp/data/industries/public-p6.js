/* 4 行业预设 · P6 · public · 任务定义
 * 出典: industry-knowledge.md (per 2026-09-01 21:25 JST 業界知識)
 * 適用: 自治体・中央省庁・独立行政法人 向け P6 テスト工程 (主 phase) 跨子 phase 工程管理級タスク
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p6'] = {
    phase: 'P6',
    phaseId: 'P6',
    industry: 'public',
    industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P6-PUB-001',
        title: 'アクセシビリティ試験 (JIS X 8341-3) 統合管理',
        desc: 'JIS X 8341-3:2016 適合レベル AA を P6.1 単体 (aria 属性) / P6.3 システム (NVDA・JAWS 読上げ) / P6.4 受入 (当事者参加) の 3 段階で横断検証。自動検査 (axe-core 等) と手動検査を組合せ、適合等級宣言の証跡を RP-04/07/08 ゲートで承認。',
        priority: 'P0',
        tags: ['公共', 'アクセシビリティ', 'JIS X 8341'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04', 'RP-07', 'RP-08'],
        estimate: 10
      },
      {
        id: 'P6-PUB-002',
        title: '政府共通プラットフォーム試験 / ユーザビリティ試験 統合管理',
        desc: '政府共通プラットフォーム (認証・API・LGWAN) 接続試験を P6.2 結合で検証し、ユーザビリティ試験 (住民視点タスク分析) を P6.3/P6.4 で実施。共通基盤リファレンス実装との互換性確認と住民モニターによる満足度を RP-06/07/08 ゲートで評価。',
        priority: 'P1',
        tags: ['公共', '共通基盤', 'ユーザビリティ'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-06', 'RP-07', 'RP-08'],
        estimate: 8
      }
    ]
  };
})(window);
