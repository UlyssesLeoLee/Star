/* 4 行业预设 · P6.1 · public · 任务定义
 * 出典: industry-knowledge.md (P6.1 単体試験 · public 重点)
 * 重点: アクセシビリティ単体 (aria 属性) / 入力検証単体
 * 業界: 自治体・中央省庁・独法 (JIS X 8341 / デジタル手続法 / 個情法)
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p61'] = {
    phase: 'P6.1', phaseId: 'P6.1',
    industry: 'public', industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P61-PUB-001',
        title: 'aria 属性単体試験',
        desc: 'role / aria-label / aria-describedby / aria-live / aria-required 等の単体試験 (NVDA / JAWS 読み上げ互換、JIS X 8341-3 準拠)。',
        priority: 'P0',
        tags: ['公共', 'UT', 'a11y', 'JIS X 8341'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 6
      },
      {
        id: 'P61-PUB-002',
        title: '入力検証単体試験',
        desc: '必須チェック / 形式バリデーション (メール・郵便番号・マイナンバー) / 範囲 / 文字種 / 依存フィールド (確認入力) の単体試験。',
        priority: 'P0',
        tags: ['公共', 'UT', '入力検証', 'バリデーション'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 5
      },
      {
        id: 'P61-PUB-003',
        title: 'フォーカス管理単体試験',
        desc: 'Tab キー移動順 / フォーカストラップ (モーダル) / スキップリンク / フォーカス可視化 の単体試験 (キーボードのみ操作、JIS X 8341 準拠)。',
        priority: 'P1',
        tags: ['公共', 'UT', 'a11y', 'キーボード'],
        linkedDocs: ['DOC-14', 'DOC-15'],
        reviewPoints: ['RP-04'],
        estimate: 4
      }
    ]
  };
})(window);
