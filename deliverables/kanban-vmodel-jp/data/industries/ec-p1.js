/* 4 行业预设 · P1 · ec · 任务定义
 * 出典: industry-knowledge.md
 * 主题: 事業計画/KPI/特定商取引法/競合分析
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p1'] = {
    phase: 'P1', phaseId: 'P1',
    industry: 'ec', industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P1-EC-001',
        title: '事業計画 + KPI 設計 (GMV/転換率/CAC/LTV)',
        desc: '取扱高 (GMV)/転換率/CAC/LTV/リピート率等の事業 KPI と、5W1H に基づく事業計画書 (収益モデル・コスト構造) の策定。',
        priority: 'P0',
        tags: ['EC', '事業計画'],
        linkedDocs: ['DOC-01', 'DOC-03'],
        reviewPoints: ['RP-12'],
        estimate: 8
      },
      {
        id: 'P1-EC-002',
        title: '法務確認 (特定商取引法/景品表示法/個人情報保護)',
        desc: '特定商取引法・景品表示法 (ステレスマ広告/二重価格)・個人情報保護法のコンプライアンス要件と、表示事項 (事業者名/連絡先/返品ポリシー) の整理。',
        priority: 'P0',
        tags: ['EC', '法令'],
        linkedDocs: ['DOC-02'],
        reviewPoints: ['RP-12'],
        estimate: 6
      },
      {
        id: 'P1-EC-003',
        title: '競合分析 + ポジショニング戦略',
        desc: 'Amazon/楽天/Yahoo!ショッピング/メルカリ等の競合マッピング (価格/配送/UI/手数料) と、差別化ポイント・ポジショニング戦略の立案。',
        priority: 'P1',
        tags: ['EC', '競合'],
        linkedDocs: ['DOC-02'],
        reviewPoints: ['RP-12'],
        estimate: 6
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
