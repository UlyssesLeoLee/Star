/* 4 行业预设 · P4 · public · 任务定义
 * 出典: industry-knowledge.md
 * 主题: WCAG 2.1 AA 詳細/支援技術対応 (NVDA/JAWS)/JIS X 8341 テスト詳細
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p4'] = {
    phase: 'P4', phaseId: 'P4',
    industry: 'public', industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P4-PUB-001',
        title: 'WCAG 2.1 AA 達成基準詳細マッピング',
        desc: 'WCAG 2.1 AA 達成基準 (perceivable / operable / understandable / robust) を UI コンポーネント単位にマッピング。コントラスト比 4.5:1・キーボード操作・フォーカス順序・代替テキスト・フォームラベル・エラー識別・タイムアウト延長の実装仕様。',
        priority: 'P0',
        tags: ['公共', 'WCAG'],
        linkedDocs: ['DOC-06'],
        reviewPoints: ['RP-03'],
        estimate: 8
      },
      {
        id: 'P4-PUB-002',
        title: '支援技術対応詳細 (NVDA / JAWS / VoiceOver / TalkBack)',
        desc: '支援技術別の読み上げ順序・ARIA-live 領域・フォーカス移動・role/aria 属性・ショートカット・タッチターゲット 44px 以上の UI 詳細仕様。プラットフォーム (Windows / macOS / iOS / Android) 別の検証ポイント・パターンライブラリ。',
        priority: 'P0',
        tags: ['公共', '支援技術'],
        linkedDocs: ['DOC-06'],
        reviewPoints: ['RP-03'],
        estimate: 6
      },
      {
        id: 'P4-PUB-003',
        title: 'JIS X 8341-3 試験手順詳細 (適合レベル判定)',
        desc: 'JIS X 8341-3:2016 適合レベル AA 試験の詳細手順。試験環境 (支援技術バージョン / OS / ブラウザ) ・被験者選定基準・試験パターン (設計書レビュー / 自動検査 / 手動操作) ・合否判定基準・適合宣言 (等級 AA) テンプレート。',
        priority: 'P1',
        tags: ['公共', 'JIS'],
        linkedDocs: ['DOC-06'],
        reviewPoints: ['RP-03'],
        estimate: 6
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
