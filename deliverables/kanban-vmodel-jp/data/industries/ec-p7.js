/* 4 行业预设 · P7 · ec · 任务定义
 * 出典: industry-knowledge.md
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p7'] = {
    phase: 'P7', phaseId: 'P7',
    industry: 'ec', industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P7-EC-001',
        title: '本番リリース (キャンペーン同時)',
        desc: '新システムの本番リリース。大規模キャンペーン (周年/季節セール) 同時の高負荷タイミングを避ける、または逆手に取る判断。',
        priority: 'P0',
        tags: ['EC','リリース','キャンペーン','本番'],
        linkedDocs: ['DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 8
      },
      {
        id: 'P7-EC-002',
        title: 'PiP (Peak in Production) 検証',
        desc: '本番環境でピーク時間帯 (例: 0 時の日付変更 / ランチタイム / 通勤後) の負荷・レスポンス検証。',
        priority: 'P0',
        tags: ['EC','PiP','負荷','ピーク'],
        linkedDocs: ['DOC-15','DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 6
      },
      {
        id: 'P7-EC-003',
        title: 'ロールバック手順 / 在庫整合性検証',
        desc: '障害発生時のロールバック手順。決済・在庫・ポイントの整合性を保ったまま戻す手順書。',
        priority: 'P1',
        tags: ['EC','ロールバック','在庫整合性'],
        linkedDocs: ['DOC-18'],
        reviewPoints: ['RP-09'],
        estimate: 4
      }
    ]
  };
})(window);
