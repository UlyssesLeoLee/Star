/* 4 行业预设 · P1 · embedded · 任务定义
 * 出典: industry-knowledge.md
 * 主题: ISO 26262 ASIL/機能安全/HW-SW IF
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['embedded-p1'] = {
    phase: 'P1', phaseId: 'P1',
    industry: 'embedded', industryJa: '組込',
    color: '#10b981',
    tasks: [
      {
        id: 'P1-EMB-001',
        title: 'ISO 26262 ASIL 分解 + 機能安全コンセプト',
        desc: '車両機能に対するハザード分析・リスク評価 (ASIL A〜D) と、安全目標 (Safety Goal) / 安全状態 / フォールト許容時間のコンセプト化。',
        priority: 'P0',
        tags: ['組込', 'ISO 26262'],
        linkedDocs: ['DOC-01', 'DOC-02'],
        reviewPoints: ['RP-12'],
        estimate: 8
      },
      {
        id: 'P1-EMB-002',
        title: '規格調査 + 認証戦略 (IEC 61508/EN 50128/MISRA-C)',
        desc: 'IEC 61508 (機能安全)/EN 50128 (鉄道)/MISRA-C:2012 等の適用規格マッピングと、認証機関 (TÜV/SGS) との認証ルート立上げ。',
        priority: 'P0',
        tags: ['組込', '規格'],
        linkedDocs: ['DOC-02'],
        reviewPoints: ['RP-12'],
        estimate: 6
      },
      {
        id: 'P1-EMB-003',
        title: 'HW-SW インターフェース + 量産性検討',
        desc: 'HW/SW 機能分割のトレードオフ評価 (コスト/性能/安全要求) と、量産時の製造性・保守性・部品寿命 (10 年+) の事業性検討。',
        priority: 'P1',
        tags: ['組込', '量産'],
        linkedDocs: ['DOC-03'],
        reviewPoints: ['RP-12'],
        estimate: 6
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
