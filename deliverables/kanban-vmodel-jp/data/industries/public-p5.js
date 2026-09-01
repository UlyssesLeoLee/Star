/* 4 行业预设 · P5 · public · 任务定义
 * 出典: industry-knowledge.md · Mavis
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['public-p5'] = {
    phase: 'P5', phaseId: 'P5',
    industry: 'public', industryJa: '公共',
    color: '#0ea5e9',
    tasks: [
      {
        id: 'P5-PUB-001',
        title: 'アクセシブル UI コンポーネント (JIS X 8341 準拠) 実装',
        desc: 'WCAG 2.1 AA / JIS X 8341-3:2016 適合のため、aria 属性・フォーカス順序・キーボード操作・色コントラスト 4.5:1 を満たす共通 UI コンポーネントを実装。支援技術 (NVDA/JAWS) での読み上げを確認。',
        priority: 'P0',
        tags: ['公共', 'a11y', 'JIS X 8341', 'WCAG', '実装'],
        linkedDocs: ['DOC-09', 'DOC-13'],
        reviewPoints: ['RP-04', 'RP-05'],
        estimate: 12
      },
      {
        id: 'P5-PUB-002',
        title: '自動 a11y 試験 (axe-core / pa11y) CI 組込',
        desc: 'PR 単位で axe-core (Jest) と pa11y CI を自動実行し、重大度 High の違反を 0 にゲート。スクリーンリーダー検証 (NVDA) は手動チェックリストを併用。',
        priority: 'P1',
        tags: ['公共', 'a11y', 'axe-core', 'pa11y', 'CI'],
        linkedDocs: ['DOC-14'],
        reviewPoints: ['RP-04'],
        estimate: 6
      },
      {
        id: 'P5-PUB-003',
        title: 'LGPKI/公的個人認証 (JPKI) 署名検証 実装',
        desc: 'LGPKI ブリッジ認証局の証明書チェーン検証と、公的個人認証 (JPKI) の電子署名検証を API レイヤで実装。マイナンバー利用事務に対応したアクセス制御と監査ログ付与も含む。',
        priority: 'P1',
        tags: ['公共', 'LGPKI', 'JPKI', 'マイナンバー', '署名'],
        linkedDocs: ['DOC-07', 'DOC-13'],
        reviewPoints: ['RP-04', 'RP-05'],
        estimate: 12
      }
    ]
  };
})(window);
