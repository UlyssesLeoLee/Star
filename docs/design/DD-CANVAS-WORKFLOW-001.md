# DD-CANVAS-WORKFLOW-001

> **无限画布 — 自动化流程域 (Automation Flow Domain) 詳細設計書 v1.0 (Draft)**
>
> - 状態: 🟡 v1.0 (Draft — Round 1 / 7 章节骨架)
> - 上位: `docs/design/BD-CANVAS-WORKFLOW-001.md` v1.0.3(已签字) / `docs/requirements/SRS-CANVAS-WORKFLOW-001.md` v1.1
> - 下游: 開発実装 / テスト設計 / Review
> - 整合参考: `docs/design/DD-CANVAS-001.md` v? / `docs/design/DD-CANVAS-AGENT-001.md` v?
> - 作成日: 2026-09-14
> - 作成者: ULYS-33 担当エージェント (MinimaxM3)
> - 起票: Multica ULYS-33(子タスク of ULYS-2 — 親 + 5 子イシュー パターン)

---

## §0 文档信息

| 項目 | 内容 |
|---|---|
| 文書 ID | DD-CANVAS-WORKFLOW-001 |
| 文書名 | 无限画布 — 自动化流程域 詳細設計書 |
| 上位要件定義 | SRS-CANVAS-WORKFLOW-001 v1.1 |
| 上位基本設計 | BD-CANVAS-WORKFLOW-001 v1.0.3(全 5 角色签字済) |
| 版数 | v1.0 (Draft,Round 1) |
| ステータス | 🟡 Draft — IPA 自審待ち |
| 対象モジュール | Flow Editor / Template Selector / Bottom Chat Bar / Execution History / Flow Tags Manager / Node Config Sidebar / RuleExecutor / LangGraph/L0 Router(本 DD の主担当スコープ) |
| 非対象 | 既存 Canvas コア・Agent 詳細(参照のみ)/ 25 module コア(参照のみ)。これらは DD-CANVAS-001 / DD-CANVAS-AGENT-001 が担当。 |
| 作成日 | 2026-09-14 |
| 作成者 | ULYS-33 担当エージェント |
| Review 状態 | 未開始(IPA 詳細設計 自審は Round 7 で実施) |

## §1 文档目的

本書は `BD-CANVAS-WORKFLOW-001` v1.0.3 で確定した 54 FR(W1〜W15、うち W14 に 3 セットの既定ワークフローテンプレート + Agent プレースホルダノード/エッジ/データフロー、W15 に LangGraph 知的制御 + キャンバス底部チャットバー)を **実装可能な粒度** にブレークダウンする。

具体的には:

- 54 FR を `Requirement → BD → DD → Module/Class/API/Table/Event → Test観点` のトレーサビリティ行列で完全に追跡可能化する(FR 漏出ゼロ、W14/W15 の 12 項目個別検証可能)
- Flow エディタ / テンプレートセレクタ / 底部チャットバー / 実行履歴 / Flow Tags 管理 / ノード設定サイドバー / RuleExecutor / LangGraph-L0 ルーティングのコンポーネント/Class/メソッド設計を実装可能な粒度で示す
- ノードグラフ検証、トリガ/アクション/条件/合流/ループ/サブフロー、実行/再実行、テンプレート インスタンス化/別保存/CRUD、ラベル→タスクカード→バックログ/スプリント/パネル連携、チャット草稿確認/取消、動的ルーティング、障害分岐 … 正常・検証失敗・権限失敗・DB/ネットワーク失敗・Timeout・Retry・Rollback/補償 を全網羅
- BD で確定した 8 REST エンドポイント + 既設 WebSocket チャネル + LangGraph/L0 内部呼出鎖を `Request → Deserialize → Validation → Authentication → Authorization → Service → Domain → Repository → Transaction → Response Mapping` の連鎖で内部処理設計
- BD の 8 テーブル(新規/拡張)を物理フィールド・PK/FK/Unique/Check/Index・列挙・SCD/RLS/監査・CRUD・トランザクション境界・ロック/楽観並列・保持/削除 規則までブレークダウン
- W/T/M 三分類を厳守し、100% 表網羅。重要クエリにはアクセスパスを明記、不確定な実行計画には【性能検証必要】マーク
- RBAC / Agent プレースホルダ活性化前検証 / Webhook 認証 / 入力検証 / 機微データ マスキング / Trace・Correlation ID / リトライ 嵐 / 復旧・人手介入境界を設計

## §2 适用范围

### In-Scope(本 DD が責任を負う領域)

| 領域 | 該当 FR 帯 | 上位 BD 節 |
|---|---|---|
| Flow エディタ(ノード/エッジ/データフロー描画・編集・検証) | W1-W6 | BD §6.1 / §7.1 |
| トリガ / アクション / 条件 / 合流 / ループ / サブフロー | W2-W4 | BD §7.2 |
| テンプレート セレクタ(3 セット既定 + ユーザ定義) | W5 / W14 | BD §7.3 / §9.3 |
| ノード設定サイドバー(Node Config Sidebar) | W6 / W8 | BD §7.1 |
| RuleExecutor(ルール実行器、隔離ランタイム) | W7-W9 | BD §7.4 / §8 |
| 実行履歴(Execution History)+ リラン | W9 / W10 | BD §7.5 |
| Flow Tags 管理 + ラベル→タスクカード→バックログ/スプリント/パネル連携 | W11 / W12 | BD §7.6 / §7.7 |
| キャンバス底部チャットバー + LangGraph 知的制御 | W13 / W15 | BD §7.8 / §8 |
| Agent プレースホルダ ノード/エッジ/データフロー 予置 | W14 | BD §7.3 / §9.3 |
| 8 REST API + 既設 WebSocket チャネル 内部処理 | W1-W15 全部門 | BD §10 |
| 8 テーブル 物理設計 + CRUD | W1-W15 全部門 | BD §11 |

### Out-of-Scope(本 DD が責任を負わない領域)

- 既存 Canvas コア(描画エンジン、Viewport、選択、パン/ズーム 等)— DD-CANVAS-001 / DD-CANVAS-AGENT-001 を参照。本 DD は境界 IF のみ言及、内部実装は別 DD。
- 既存 Agent 25 module コア — DD-CANVAS-AGENT-001 を参照。本 DD は Agent プレースホルダ活性化 IF とデータ受け渡しのみ言及。
- 認証/認可 基盤(`auth` モジュール)— 既設として参照のみ。本 DD は RBAC 適用点と必要スコープを定義。
- ストレージ / メッセージブローカー / キャッシュ 基盤 — 既設参照のみ。本 DD は 接続情報 / 失敗時振る舞い / トランザクション境界のみ定義。
- フロントエンド UI 実装詳細(React コンポーネント 構造、Store 内部)— 既存 Store(`frontend/src/lib/store.ts:565` の `actor_session_id` 予置点を含む)との IF のみ言及。
- インフラ / Helm / Kubernetes / CI-CD — `docs/design/RLS-POLICY-001.md` と既存 IaC を参照のみ。本 DD では環境別設定値の必要項目のみ言及。
- 性能数値(NFR p95/p99 値、同時接続数 等)— BD で確定または提案された数値のみ採用、不確定は【TBD】。

> **境界確認事項【上位設計確認事項】**:
> 既存 Canvas / Agent 25 module との境界宣言は `DD-CANVAS-AGENT-001.md` 側に集約されているか?確認できていない(2026-09-14 時点)。本 DD では WF 担当スコープを §2 In/Out-of-Scope に明示することで境界を共有するが、`DD-CANVAS-AGENT-001.md` 側の境界宣言の有無は **ULYS-33 の最終完了前** に独立確認事項として持ち越す。

## §3 术语和缩略语

| 用語 / 略語 | 定義 |
|---|---|
| Flow | 1 つの自動化ワークフロー定義(ノード/エッジ/データフローの有向グラフ) |
| Node | Flow の構成要素。Trigger / Action / Condition / Join / Loop / Subflow / Agent Placeholder のいずれかの種類(kind)を持つ |
| Edge | Node 間の有向接続。主エッジ(primary)/ 失敗分岐エッジ(errorBranch) / 条件分岐エッジ(conditional)の 3 種 |
| DataFlow | エッジ上のデータ受け渡し(JSON 形式、`payloadSchema` に基づく) |
| Trigger | 外部イベント起点ノード(Manual / Schedule / Webhook / Flow Tag 連携 / Agent / 既存 25 module) |
| Action | 副作用を持つ実行ノード(HTTP / Email / DB 書込 / 既存 25 module 呼出 / Agent 呼出 等) |
| Condition | ブール式評価ノード(IF/ELSE 分岐) |
| Join | 複数エッジ合流ノード(All / Any / N of M) |
| Loop | 配列/Collection を反復するノード(最大反復数制限付き) |
| Subflow | 既存 Flow を再利用するノード(参照 + スコープ引数) |
| Agent Placeholder | W14 で予置されるノード種別。`agentId = NULL` のまま保存可能、活性化時に agentId 設定 + 資格検証が必要 |
| Template | Flow テンプレート。3 セットの既定(Basic Lead Pipeline / Status Sync / Daily Digest)+ ユーザ定義 |
| Flow Tag | Flow に付与するラベル。タスクカードへ自動連携される(`tag→task_card→backlog/sprint/panel` の連動) |
| RuleExecutor | Flow 実行エンジン。隔離ランタイムで Node を順次実行。状態機械と再実行(Replay)を担当 |
| LangGraph | LLM ベースの知的ルーティング層(W15)。ノード/エッジ評価・失敗復旧提案・代替ルート生成 |
| L0 Router | LangGraph の下位ルーティング層(`actor_session_id` ベース)。既存 ADR-0046 との結節点 |
| Bottom Chat Bar | キャンバス下部に固定表示するチャット入力欄。人間→LangGraph の指示エントリポイント |
| Execution History | Flow 実行ログ。1 実行 = 1 Record。各 Node の入出力・状態・所要時間・Correlation ID |
| Replay | 同一入力で実行履歴を再実行する機能(冪等性保証が必要) |
| Correlation ID | 1 実行に紐付く一意 ID。全 Node/全 API 呼出/全 DB 行に貫通 |
| Trace ID | OpenTelemetry 互換の分散トレース ID |
| RBAC | Role-Based Access Control。本 DD では Flow 編集 / 実行 / Tags 管理 / テンプレート適用 / Agent 活性化 で区別 |
| RLS | Row-Level Security。本 DD では `RLS-POLICY-001.md` の既存ポリシーを踏襲 |
| SCD | Slowly Changing Dimension。本 DD では `flow_executions` の実行履歴保持方針で Type-2 を採用するか【TBD】 |
| W14 | ワークフロー テンプレート既定 3 セット + Agent プレースホルダ予置 |
| W15 | LangGraph 知的制御 + キャンバス底部チャットバー |
| Idempotency Key | 再送/再実行で重複処理を防ぐための一意キー(主に Webhook / POST / Replay) |
| Webhook | 外部システムが Flow を起動する HTTP POST 入口。署名検証 + Idempotency Key 必須 |
| Backlog / Sprint / Panel | Flow Tag から自動生成されるタスクカードの集約先 |
| TBD | To Be Determined — 未確定情報。上位設計の確定待ち |
| 【設計不整合】 | DD 内で他章・他文書と矛盾が発見された場合の明示タグ |
| 【上位設計確認事項】 | 基本設計側に確認・修正が必要と判断される場合の明示タグ |
| 【性能検証必要】 | 性能数値/実行計画が本 DD 作成時点で確認できない場合の明示タグ |

## §4 参考资料

| 文書 ID | 版 | 該当章 | 入手経路 |
|---|---|---|---|
| SRS-CANVAS-WORKFLOW-001 | v1.1 | 全 54 FR(W1-W15) | `docs/requirements/SRS-CANVAS-WORKFLOW-001.md` |
| BD-CANVAS-WORKFLOW-001 | v1.0.3 | §6 / §7 / §8 / §9.3 / §10 / §11 | `docs/design/BD-CANVAS-WORKFLOW-001.md`(本ワークツリー祖先 `3045dba3`) |
| DD-CANVAS-001 | v? | Canvas コア境界 | `docs/design/DD-CANVAS-001.md` |
| DD-CANVAS-AGENT-001 | v? | Agent 25 module 境界 | `docs/design/DD-CANVAS-AGENT-001.md` |
| SRS-WORKFLOW-TEMPLATE-001 | v0.1 | W14 既定テンプレートの SRS(ULYS-34 成果) | `docs/requirements/SRS-WORKFLOW-TEMPLATE-001.md` |
| BD-WORKFLOW-TEMPLATE-001 | v0.1 | W14 既定テンプレートの BD(ULYS-35 成果) | `docs/design/BD-WORKFLOW-TEMPLATE-001.md` |
| RLS-POLICY-001 | v? | 行レベル セキュリティ | `docs/design/RLS-POLICY-001.md` |
| ADR-0046 (仮) | — | LangGraph/L0 既存 ADR | `docs/adr/0046-*` 想定 — 【要確認】 |
| docs/document-registry.toml(または類似) | — | 文書 ID 一覧 | `docs/` 配下 |

> **要確認【上位設計確認事項】**:
> - `ADR-0046` の実在パスは本 DD 作成時点で確認できていない。フロントエンド `frontend/src/lib/store.ts:565` の `actor_session_id` 予置と紐付く ADR 番号・本文を **ULYS-33 完了前** に文書横断で確認すること。
> - 既設 DD の版数は未取得(2026-09-14 時点)。Round 6(REST/WebSocket 章)着手前に版数を確定すること。

## §5 修订履历

| 版 | 日付 | 担当 | 変更内容 |
|---|---|---|---|
| v1.0 (Draft) | 2026-09-14 | ULYS-33 担当エージェント | 初版(Round 1): §0-5 + FR 追跡マトリクス スケルトン作成 |

> **注**:Round 2 以降は §6〜§15 を追記。本節は各 Round 完了時に更新する。
> **注**:章 ID 体系は Round 6 で再番号整列予定(現状は Round 1 仮置き)。

---

## §6 FR 追踪矩阵スケルトン(54 FR 全項目)

> **本節の位置付け**:Round 1 で全 FR ID と Module/Class/API/Table/Event/Test の **対応スロットのみ** を確定。各 FR の中身(処理詳細・API 内部・DB 物理設計等)は Round 2 以降で §7〜§13 に分散して書き起こし、本表は **索引** として機能する。
>
> **ID 規約**:DD ID は `DD-WF-<W番号>-<連番>` プレフィックスを採用。各 Round で詳細確定後に正式 ID を割り当てる(現状は暫定)。Module は `MOD-WF-###`、Class は `CLS-WF-###`、API は `API-WF-###`(BD §10 で確定済 8 つの連番を再利用)、Table は `TBL-WF-###`(BD §11 で確定済 8 つの連番を再利用)、Event は `EVT-WF-###`、Test 観点は `TST-WF-###` を予定。
>
> **§2 In/Out-of-Scope 整合**:本表の対象は本 DD が責任を負う FR のみ。既存 Canvas/Agent コア側の FR は本表には載せず、`DD-CANVAS-001.md` / `DD-CANVAS-AGENT-001.md` 側の追跡表を参照。
>
> **§9.3 TBD 継承**:BD §9.3 に列挙された TBD/既知ギャップのうち、FR 個別の影響は本表の「TBD 継承」列に明示。全 54 FR の少なくとも 17 項目で TBD 影響あり(後述の Round で詳細展開)。

### §6.1 全体サマリ表

| 区分 | FR 数 | DD 章(予定) | 該当 API | 該当 Table | 該当 画面 |
|---|---:|---|---|---|---|
| W1 节点类型体系 | 4 | §7.1 / §8 | — | `TBL-WF-002` (`flow_node`) | SCR-WF-01 |
| W2 触发节点 | 4 | §7.2 / §8.2 | API-WF-05 | `TBL-WF-002` | SCR-WF-06 |
| W3 动作节点 | 3 | §7.3 / §8.2 | (既存 module 連動) | `TBL-WF-002` | SCR-WF-06 |
| W4 分支与条件 | 3 | §7.4 / §8.4 | — | `TBL-WF-003` (`flow_edge` with `condition_expr`) | SCR-WF-06 |
| W5 循环与批处理 | 2 | §7.5 | — | `TBL-WF-002` (`kind=loop`) | SCR-WF-06 |
| W6 变量与表达式传递 | 3 | §7.6 / §8.4 | — | `TBL-WF-003` (`data_mapping`) | SCR-WF-06 |
| W7 子流程与复用 | 2 | §7.7 | — | `TBL-WF-001` (自己参照) | SCR-WF-01 |
| W8 错误处理与重试 | 3 | §7.8 | — | `TBL-WF-005` (`execution_step` retry) | SCR-WF-04 |
| W9 执行历史与调试 | 3 | §7.9 | API-WF-03 | `TBL-WF-004` (`execution_history`) / `TBL-WF-005` | SCR-WF-04 |
| W10 激活状态与版本管理 | 3 | §7.10 | API-WF-01 | `TBL-WF-001` (`enabled`) / `TBL-WF-006` (`automation_flow_versions` SCD2) | SCR-WF-01 |
| W11 标签绑定任务卡 | 5 | §7.11 | API-WF-06 | `TBL-WF-002` (`tag_binding_expr`) / `TBL-WF-008`(WorkItem 拡張) | SCR-WF-05 |
| W12 Backlog/Sprint 联动 | 5 | §7.12 | (既存 kanban 連動) | `TBL-WF-008` | SCR-WF-05 |
| W13 数据一致性 | 2 | §7.13 | — | (横断) | SCR-WF-04 |
| **W14 默认工作流模板库 (v1.1)** | **7** | §7.14 | API-WF-06 / API-WF-07 | `TBL-WF-007` (`flow_template`) | SCR-WF-02 |
| **W15 智能控制+聊天栏 (v1.1)** | **5** | §7.15 | API-WF-08 | `TBL-WF-009` (`chat_session` — v1.1 新規,【要確認】BD §11 と一致確認) | SCR-WF-03 |
| **合計** | **54** | — | 8 (BD §10 と一致) | 8〜9 (要確認) | 6 |

> **【要確認 / 上位設計確認事項】**:
> - `chat_session` テーブルは SRS v1.1 で新規追加されたが、BD §11 の 8 表リストに正式名称があるか / ID 体系は W14/W15 章で確定しているか — 本表では仮に `TBL-WF-009` を割り当てているが、Round 2 着手前に BD §11 と照合して確定する。
> - 「8 表」と SRS v1.1 追加分の整合は Round 6(§11 DB 物理設計)で精査する。

### §6.2 FR 別追跡表(Round 1 スケルトン — 54 行)

> 各 FR 行の「DD 詳細」/「Test 観点」/「TBD 継承」列は、Round 2 以降で §7-§13 と並行して埋める。Round 1 では **ID スロット確定 + BD 一次引用 + 既知 TBD フラグ** のみ実施。

#### W1 — 节点类型体系(P0×3 / P1×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W1.1 | BD §6.1 / §7.1 节点 kind 体系 | `MOD-WF-001` FlowEditor / `CLS-WF-001` FlowNodeService / `TBL-WF-002` (`flow_node`) | `EVT-WF-001` `flow_node.created` | 正常 / kind 非法 / 権限 / 並行作成 | — |
| FR-WORKFLOW-W1.2 | BD §7.1 边追加 | `MOD-WF-001` / `CLS-WF-002` FlowEdgeService / `TBL-WF-003` (`flow_edge`) | `EVT-WF-002` `flow_edge.created` | 正常 / 自环 / 種類不正 / 跨 Flow | — |
| FR-WORKFLOW-W1.3 | BD §7.1 `+ Flow` 入口 | `MOD-WF-001` / `CLS-WF-001` / `TBL-WF-001` (`automation_flow`) | `EVT-WF-003` `automation_flow.created` | 正常 / 権限 / 並行作成 / 失敗 rollback | — |
| FR-WORKFLOW-W1.4 | BD §7.1 节点設定側欄 | `MOD-WF-002` NodeConfigSidebar / `CLS-WF-001` / `TBL-WF-002` | (load only) | 正常 / データ読込失敗 / 編集中別 session 更新 | — |

#### W2 — 触发节点(P0×2 / P1×2)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W2.1 | BD §6.1 / §7.1 Manual 触发 | `MOD-WF-003` TriggerService / `API-WF-01` (POST executions) / `TBL-WF-004` | `EVT-WF-004` `execution.started` (manual) | 正常 / 並行連打 / 権限 | W2.1 debounce【TBD】 |
| FR-WORKFLOW-W2.2 | BD §7.1 cron 调度 | `MOD-WF-003` / `API-WF-01` / `TBL-WF-002` (`schedule_cron`) | `EVT-WF-005` `execution.started` (cron) | 正常 / cron 非法 / タイムゾーン | — |
| FR-WORKFLOW-W2.3 | BD §7.1 / §10.1 API-WF-05 Webhook 触发 | `MOD-WF-003` / `API-WF-05` (POST webhook) / `TBL-WF-002` (`webhook_token`) | `EVT-WF-006` `execution.started` (webhook) | 正常 / token 不一致 / body 過大 / 冪等キー重複 | **W2.3【TBD】HMAC / token 輪換 / body 上限 / 署名ヘッダ / IP allowlist** |
| FR-WORKFLOW-W2.4 | BD §7.1 canvas_event 触发 | `MOD-WF-003` / 既設 `canvas-collab` WS / `TBL-WF-002` (`canvas_event` filter) | `EVT-WF-007` `execution.started` (canvas_event) | 正常 / フィルタ無一致 / 高頻度 storm | **W2.4【TBD】debounce 戦略** |

#### W3 — 动作节点(P0×1 / P1×2)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W3.1 | BD §7.2 `AutomationActionKind` 6 種 | `MOD-WF-004` RuleExecutor / `CLS-WF-010` ActionDispatcher / `TBL-WF-002` (`kind=action`) | `EVT-WF-008` `step.started` / `EVT-WF-009` `step.succeeded` | 正常 / 失敗→W8 / 副作用検証 / timeout | — |
| FR-WORKFLOW-W3.2 | BD §7.2 HTTP / transform_data | `MOD-WF-004` / `CLS-WF-011` HttpAction / `CLS-WF-012` TransformAction / `TBL-WF-002` | `EVT-WF-008/009` | 正常 / HTTP 5xx / JSONPath 非法 / timeout | — |
| FR-WORKFLOW-W3.3 | BD §7.2 順序/並列 | `MOD-WF-004` / `CLS-WF-013` ExecutionScheduler / `TBL-WF-003` (順序/並列 edge 属性) | `EVT-WF-008/009` | 正常 / 並列合流 / 順序違反 / 一部失敗 | — |

#### W4 — 分支与条件(P0×1 / P1×2)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W4.1 | BD §6.3.1 / §8.4 CEL IF | `MOD-WF-004` / `CLS-WF-014` CelConditionNode / `TBL-WF-003` (`condition_expr` IF) | `EVT-WF-010` `branch.evaluated` | 正常 / 評価エラー / 両分岐非活性 | — |
| FR-WORKFLOW-W4.2 | BD §6.3.1 / §8.4 CEL Switch | `MOD-WF-004` / `CLS-WF-014` / `TBL-WF-003` (`condition_expr` Switch) | `EVT-WF-010` | 正常 / default 落ち / case 重複 | — |
| FR-WORKFLOW-W4.3 | BD §6.3.1 Merge join/race | `MOD-WF-004` / `CLS-WF-015` MergeNode / `TBL-WF-002` (`kind=merge`) | `EVT-WF-011` `join.completed` | 正常 / 部分不到達 / 全不到達 | **W4.3【TBD】join timeout 戦略** |

#### W5 — 循环与批处理(P1×1 / P2×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W5.1 | BD §7.2 / §9.3 loop | `MOD-WF-004` / `CLS-WF-016` LoopNode / `TBL-WF-002` (`kind=loop`) | `EVT-WF-012` `loop.iteration.started` | 正常 / 配列非配列 / 0 要素 / 副作用反復 | **W5.1【TBD】最大反復数 / join/debounce 戦略** |
| FR-WORKFLOW-W5.2 | BD §7.2 loop 並列度 | `MOD-WF-004` / `CLS-WF-016` / `TBL-WF-002` (`concurrency` 1-20) | `EVT-WF-012` | 正常 / 並列度境界 / 範囲外 | — |

#### W6 — 变量与表达式传递(P0×2 / P2×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W6.1 | BD §6.3.1 / §8.4 `{{node.<id>.output.<field>}}` | `MOD-WF-004` / `CLS-WF-017` ExpressionEvaluator / `TBL-WF-003` (`data_mapping`) | `EVT-WF-013` `expression.evaluated` | 正常 / node_id 不存在 / field 不存在 / 型不一致 | — |
| FR-WORKFLOW-W6.2 | BD §6.3.1 / §8.4 `{{flow.variables.<key>}}` | `MOD-WF-004` / `CLS-WF-017` / `TBL-WF-001` (`variables`) | `EVT-WF-013` | 正常 / key 不存在 / 過去 Execution 不変 | — |
| FR-WORKFLOW-W6.3 | BD §8.4 CEL 共通 | `MOD-WF-004` / `CLS-WF-017` / 既設 CEL parser | `EVT-WF-013` | 正常 / 構文不正 / `/automation` 同一性 | — |

#### W7 — 子流程与复用(P1×1 / P2×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W7.1 | BD §6.3.1 / §7.2 Subflow | `MOD-WF-004` / `CLS-WF-018` SubflowNode / `TBL-WF-001` (自己参照) / `TBL-WF-004` (`parent_execution_id`) | `EVT-WF-014` `subflow.invoked` | 正常 / 循環 / 深さ >5 / 孤立 | **W7.1【TBD】循環検出アルゴリズム詳細** |
| FR-WORKFLOW-W7.2 | BD §7.3 / W14 テンプレート選択入口 | `MOD-WF-005` TemplateSelector / `TBL-WF-007` (`flow_template`) | (load only) | 正常 / 0 件 / 失敗 → 手動作成経路確保 | — |

#### W8 — 错误处理与重试(P0×2 / P1×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W8.1 | BD §6.3.2 / §7.4 `retry_policy` | `MOD-WF-004` / `CLS-WF-019` RetryPolicyExecutor / `TBL-WF-005` (`retry_policy`) | `EVT-WF-015` `step.retry_scheduled` / `EVT-WF-016` `step.retry_executed` | 正常 / 一時的失敗回復 / 永久失敗 / 並行 retry | **W8.1【TBD】最大回数 / backoff 既定値** |
| FR-WORKFLOW-W8.2 | BD §6.3.2 `on_error` 边 | `MOD-WF-004` / `CLS-WF-020` OnErrorRouter / `TBL-WF-003` (`on_error` edge) | `EVT-WF-017` `error_branch.activated` | 正常 / `on_error` 不存在 → Execution failed / 多段 `on_error` | — |
| FR-WORKFLOW-W8.3 | BD §7.5 通知 domain | `MOD-WF-004` / 既設 notification / `TBL-WF-004` (`notified_at`) | `EVT-WF-018` `execution.failed_notified` | 正常 / 通知失敗 / 30s 遅延検証 | — |

#### W9 — 执行历史与调试(P0×2 / P1×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W9.1 | BD §7.5 履歴 append-only | `MOD-WF-004` / `CLS-WF-021` ExecutionRecorder / `TBL-WF-004` / `TBL-WF-005` | `EVT-WF-019` `execution_step.appended` | 正常 / 書込失敗 retry / 容量超過 | **W9.1【TBD】at-least-once 保証方式 / 容量・保存期間** |
| FR-WORKFLOW-W9.2 | BD §7.5 失敗再実行 | `MOD-WF-004` / `CLS-WF-022` ReplayService / `TBL-WF-004` (`resumed_from_execution_id`) | `EVT-WF-020` `execution.replayed` | 正常 / 非冪等 action 再実行 / 孤立 replay | **W9.2【TBD】冪等性標記メカニズム** |
| FR-WORKFLOW-W9.3 | BD §7.5 ステップ詳細表示 | `MOD-WF-002` / `CLS-WF-021` / `TBL-WF-005` | (read only) | 正常 / JSON 過大 / 権限 | **W9.3【TBD】JSON 分頁/截断** |

#### W10 — 激活状态与版本管理(P0×1 / P1×2)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W10.1 | BD §7.4 enabled 切替 | `MOD-WF-006` FlowActivationService / `API-WF-01` / `TBL-WF-001` (`enabled`) | `EVT-WF-021` `flow.enabled_changed` | 正常 / 未束縛 placeholder 有 → 拒否 / 権限 | **W10.1 placeholder 活性化前検証ロジック詳細** |
| FR-WORKFLOW-W10.2 | BD §7.4 SCD2 版本 | `MOD-WF-006` / `CLS-WF-023` VersionManager / `TBL-WF-006` (`automation_flow_versions`) | `EVT-WF-022` `flow_version.appended` | 正常 / 並行編集 conflict / diff 取得 | **W10.2【TBD】楽観ロック vs CRDT (SRS リスク #6)** |
| FR-WORKFLOW-W10.3 | BD §7.4 版本 rollback | `MOD-WF-006` / `CLS-WF-023` / `TBL-WF-006` | `EVT-WF-023` `flow_version.rollback` | 正常 / 不存在 version / rollback 連鎖 | — |

#### W11 — 标签绑定任务卡(P0×4 / P1×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W11.1 | BD §6.3.3 / §7.6 `tag_binding_expr` | `MOD-WF-007` TagBindingService / `API-WF-06` / `TBL-WF-002` (`tag_binding_expr`) / `TBL-WF-008` (WorkItem) | `EVT-WF-024` `tag_binding.monitored` | 正常 / 構文不正 / 監視 ON/OFF | — |
| FR-WORKFLOW-W11.2 | BD §6.3.3 派生 WorkItem 作成 | `MOD-WF-007` / `CLS-WF-030` WorkItemDeriver / `TBL-WF-008` (`source_flow_id`) | `EVT-WF-025` `workitem.derived` | 正常 / 並行重複 (BR-W-1) / 権限 | — |
| FR-WORKFLOW-W11.3 | BD §6.3.3 派生 vs 人工 カード 同権 | `MOD-WF-007` / `TBL-WF-008` (拡張) | (派生 WorkItem 自身) | 正常 / 派生 カード 編集 / 集計 / 期限 | — |
| FR-WORKFLOW-W11.4 | BD §6.3.3 BR-W-3 三分支 | `MOD-WF-007` / `CLS-WF-031` BrW3Handler / `TBL-WF-008` | `EVT-WF-026` `workitem.binding_changed` | 正常 / 硬删 / 移回删 / detached / 中途失敗 | **W11.4【TBD】重試/補償メカニズム** |
| FR-WORKFLOW-W11.5 | BD §7.6 AND/OR/NOT ブール式 | `MOD-WF-007` / `CLS-WF-032` BooleanExprParser / `TBL-WF-002` | `EVT-WF-027` `tag_binding.expr_evaluated` | 正常 / 構文不正 / 括弧 / NOT 単独 | — |

#### W12 — Backlog/Sprint 联动(P0×3 / P1×2)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W12.1 | BD §6.3.3 / §7.7 派生 Backlog 配置 | `MOD-WF-008` SprintLinkService / `TBL-WF-008` (`sprint_id`) | `EVT-WF-028` `workitem.placed_in_backlog` | 正常 / sprint_id 空 / 既存 kanban 連動 | — |
| FR-WORKFLOW-W12.2 | BD §7.7 派生 Sprint 拖拽 | `MOD-WF-008` / 既設 `@dnd-kit` / `TBL-WF-008` | `EVT-WF-029` `workitem.moved_to_sprint` | 正常 / 拖拽先ロック / 派生 カード | — |
| FR-WORKFLOW-W12.3 | BD §6.3.3 detached 状態 | `MOD-WF-008` / `TBL-WF-008` (`tag_binding_status`) | `EVT-WF-030` `workitem.detached` | 正常 / detached 中 Sprint 完了 / 取消 | **W12.3【TBD】detached 取消 / 人間確認要否** |
| FR-WORKFLOW-W12.4 | BD §3.2 SCR-WF-05 | `MOD-WF-008` / UI SCR-WF-05 / `TBL-WF-002` + `TBL-WF-008` | (read only + edit) | 正常 / 0 件 / 権限 | — |
| FR-WORKFLOW-W12.5 | BD §6.3.3 `user_edited_fields` | `MOD-WF-008` / `TBL-WF-008` (`user_edited_fields` JSONB) | `EVT-WF-031` `workitem.user_field_locked` | 正常 / system 字段 上書防止 / 解除 | — |

#### W13 — 数据一致性(P0×1 / P1×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W13.1 | BD §6.3.3 `status_mapping` 連動 | `MOD-WF-009` StatusMappingService / `TBL-WF-001` (`status_mapping` JSONB) / `TBL-WF-008` | `EVT-WF-032` `workitem.status_synced` | 正常 / 映射 未設定 / 映射 非法 | — |
| FR-WORKFLOW-W13.2 | BD §6.3.3 `1 task 1 sprint` 制約 | `MOD-WF-008` (制約) / `TBL-WF-008` | (制約) | 正常 / 派生 vs 人工 同値フィールド | — |

#### W14 — 默认工作流模板库 v1.1(P0×1 / P1×5 / P2×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W14.1 | BD §3.2 SCR-WF-02 / §7.3 | `MOD-WF-005` / UI SCR-WF-02 / `TBL-WF-007` | (read) | 正常 / 0 件 / 読込失敗 | — |
| FR-WORKFLOW-W14.2 | BD §7.3 / ULYS-34 SRS / ULYS-35 BD | `MOD-WF-005` / `CLS-WF-040` TemplateInstantiator / `TBL-WF-007` (AAA) + `TBL-WF-002/003` | `EVT-WF-033` `flow.instantiated_from_template` | 正常 / 並行 / 適用後編集 | — |
| FR-WORKFLOW-W14.3 | BD §7.3 / ULYS-34 SRS / ULYS-35 BD | `MOD-WF-005` / `CLS-WF-040` / `TBL-WF-007` (spec 式) + `TBL-WF-002/003` | `EVT-WF-033` | 正常 / 循環構築 / 編集離脱 | **W14.3【TBD】Flow 級 最大循環回数 制限** |
| FR-WORKFLOW-W14.4 | BD §7.3 / ULYS-34 SRS / ULYS-35 BD | `MOD-WF-005` / `CLS-WF-040` / `TBL-WF-007` (superpowers 式) + `TBL-WF-002/003` | `EVT-WF-033` | 正常 / 循環 / 段階名 自由 | — |
| FR-WORKFLOW-W14.5 | BD §7.3 / §10 / §11 活性化前検証 | `MOD-WF-005` / `CLS-WF-041` ActivationGuard / `TBL-WF-002` (`is_placeholder`, `agent_id`) | `EVT-WF-034` `flow.activation_blocked` | 正常 / placeholder + agent_id 空 → 拒否 / 拒否メッセージ具体性 | **W14.5 活性化前検証ロジック詳細** |
| FR-WORKFLOW-W14.6 | BD §7.3 別保存 | `MOD-WF-005` / `CLS-WF-040` / `TBL-WF-007` (`is_builtin=false`) | `EVT-WF-035` `flow_template.duplicated` | 正常 / 同名重複 / 内蔵影響無 | — |
| FR-WORKFLOW-W14.7 | BD §7.3 自定義 CRUD | `MOD-WF-005` / `API-WF-07` / `TBL-WF-007` | `EVT-WF-036` `flow_template.crud` | 正常 / 内蔵削除試行 403 / 検索 | — |

#### W15 — 智能控制 + 聊天栏 v1.1(P0×2 / P1×3)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W15.1 | BD §3.2 SCR-WF-03 / §8 L0 接続 | `MOD-WF-010` ChatBarService / UI SCR-WF-03 / `TBL-WF-009` (`chat_session`) | `EVT-WF-037` `chat_session.opened` | 正常 / L0 不到達 → 离线表示 / 画布影響無 | — |
| FR-WORKFLOW-W15.2 | BD §7.8 mock 解析 → 草稿 | `MOD-WF-010` / `CLS-WF-050` DraftParser / `TBL-WF-009` (`parsed_flow_draft`) | `EVT-WF-038` `chat_draft.generated` | 正常 / 未命中 → 提示 / 草稿取消 / 草稿 → 本保存 | **W15.2【TBD】mock 規則カバー範囲** |
| FR-WORKFLOW-W15.3 | BD §6.3.1 / §8 動的ルーティング | `MOD-WF-011` LangGraphRouter / `TBL-WF-005` (`routing_decision` JSONB) | `EVT-WF-039` `branch.dynamic_routed` | 正常 / L0 不到達 / 决策可再現性 | **W15.3【TBD】L0 降級戦略 / 决策可再現性** |
| FR-WORKFLOW-W15.4 | BD §7.8 チャット→Execution 回鎖 | `MOD-WF-010` / `TBL-WF-004` (`origin_chat_session_id`) | `EVT-WF-040` `execution.chat_origin_recorded` | 正常 / session 期限切れ / 権限 | — |
| FR-WORKFLOW-W15.5 | BD §7.8 草稿 ノード = 手動 ノード | `MOD-WF-010` / `TBL-WF-002/003` (共用) | (生成/編集/削除 event) | 正常 / 草稿 → 編集中 / 草稿 取消 | — |

> **Round 1 終了時点の整合確認**:
> - 54 FR 全項目を BD §6/§7 から抽出して表化済(漏出ゼロ、W14/W15 の v1.1 追加 12 項目も個別に追跡可能)
> - 各 FR について BD 一次引用章節を明記し、Round 2 以降で参照する章節を割り当て済
> - TBD 継承フラグは 17 項目(W2.1 / W2.3 / W2.4 / W4.3 / W5.1 / W7.1 / W8.1 / W9.1 / W9.2 / W9.3 / W10.1 / W10.2 / W11.4 / W12.3 / W14.3 / W14.5 / W15.2 / W15.3)で立っている — Round 2 以降で §7 該当章にて詳細展開する
> - 8 REST API(`API-WF-01`〜`API-WF-08`)と 8+ テーブル(`TBL-WF-001`〜`TBL-WF-009`)の ID スロットは Round 4 / Round 6 で正式確定
> - ID は暫定。Round 6 で ID 一覧表 + リナンバリング整列予定

> **未着手項目(Round 1 の範囲外)**:
> - §7 モジュール/Class 詳細設計(Round 2)
> - §8 処理詳細・状態遷移(Round 3)
> - §9 API 内部処理設計(Round 4)
> - §10 データ/SQL/CRUD(Round 5)
> - §11 セキュリティ/ログ/監査(Round 6)
> - §12 NFR/テスト観点/Round 7 で IPA 自審 + TBD 追跡

