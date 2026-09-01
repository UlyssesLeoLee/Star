/* 4 行业预设 · P4 · ec · 任务定义
 * 出典: industry-knowledge.md
 * 主题: 3Dセキュア (EMV 3DS) 詳細/不正検知ルール/在庫引当ロジック/配送料計算
 */
(function (global) {
  global.VMODEL_INDUSTRIES = global.VMODEL_INDUSTRIES || {};
  global.VMODEL_INDUSTRIES['ec-p4'] = {
    phase: 'P4', phaseId: 'P4',
    industry: 'ec', industryJa: 'EC',
    color: '#f59e0b',
    tasks: [
      {
        id: 'P4-EC-001',
        title: '3D セキュア (EMV 3DS 2.x) 詳細仕様',
        desc: 'EMV 3DS 2.x メッセージフロー詳細。AReq / ARes / CReq / CRes / PReq / PRes のスキーマ・リスクベース認証 (RBA) ・フリクションレスフロー・チャレンジフロー切替条件・3DS サーバ (MPI) と DS 連携・加盟店免責条件の実装仕様。',
        priority: 'P0',
        tags: ['EC', '3DS'],
        linkedDocs: ['DOC-06'],
        reviewPoints: ['RP-03'],
        estimate: 8
      },
      {
        id: 'P4-EC-002',
        title: '不正検知ルール詳細 (デバイス / 行動 / 住所)',
        desc: '不正検知ルールエンジン詳細。デバイスフィンガープリント・IP リスク評価・行動分析 (ログイン / 購入速度) ・住所突合 (AVS) ・スコアリング閾値・ブラックリスト/ホワイトリスト・3DS 動的切替・チャージバックアラートの IF・状態遷移。',
        priority: 'P0',
        tags: ['EC', '不正検知'],
        linkedDocs: ['DOC-06'],
        reviewPoints: ['RP-03'],
        estimate: 6
      },
      {
        id: 'P4-EC-003',
        title: '在庫引当ロジック詳細 (即時/決済後/複数倉庫)',
        desc: '在庫引当の詳細ロジック。即時引当 (注文時点) / 決済後引当 / 予約引当の 3 方式・引当タイムアウト (15-30 分) ・自動戻し・複数倉庫引当優先度・競合制御 (楽観ロック / 排他) ・在庫切れ時の代替倉庫検索・引当履歴監査の実装仕様。',
        priority: 'P1',
        tags: ['EC', '在庫'],
        linkedDocs: ['DOC-06', 'DOC-08'],
        reviewPoints: ['RP-03'],
        estimate: 6
      }
    ]
  };
})(typeof window !== 'undefined' ? window : globalThis);
