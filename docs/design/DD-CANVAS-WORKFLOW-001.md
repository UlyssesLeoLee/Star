# DD-CANVAS-WORKFLOW-001

> **无限画布 — 自动化流程域 (Automation Flow Domain) 詳細設計書 v1.0 (Draft)**
>
| 状態 | 🟡 v1.0 (Draft — Round 2 / 7;§0-5 + §6 + §7 落档;§8-§14 待 Round 3-7) |
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
| v1.0 (Draft, Round 2) | 2026-09-14 | ULYS-33 担当エージェント | Round 2: §7 コンポーネント/Class 詳細設計 追記(5 module + 30 class + 主要メソッド・ライフサイクル・依存・外部 IF を実装可能粒度で明文化); §6.1 / §6.2 W15 行の `TBL-WF-009 (chat_session)` を **BD §4.2.8 と整合させて `TBL-WF-008` にリナンバー**(Round 1 の「要確認」が BD 全文確認で解消);§10 ID 一覧は §7 章で確定 |

> **注**:Round 3 以降は §8(処理詳細・状態遷移)/ §9(API 内部処理)/ §10(SQL)/ §11(セキュリティ)/ §12(NFR + テスト)/ §13(TBD)/ §14(IPA 自審) を追記。本節は各 Round 完了時に更新する。
> **注**:章 ID 体系は Round 6 で再番号整列予定(Round 2 で §7 章内 ID は確定済)。

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
| W1 节点类型体系 | 4 | §7.1 / §8 | — | `TBL-WF-003` (`flow_node`) | SCR-WF-01 |
| W2 触发节点 | 4 | §7.2 / §8.2 | API-WF-05 | `TBL-WF-003` | SCR-WF-06 |
| W3 动作节点 | 3 | §7.3 / §8.2 | (既存 module 連動) | `TBL-WF-003` | SCR-WF-06 |
| W4 分支与条件 | 3 | §7.4 / §8.4 | — | `TBL-WF-004` (`flow_edge` with `condition_expr`) | SCR-WF-06 |
| W5 循环与批处理 | 2 | §7.5 | — | `TBL-WF-003` (`kind=loop`) | SCR-WF-06 |
| W6 变量与表达式传递 | 3 | §7.6 / §8.4 | — | `TBL-WF-004` (`data_mapping`) | SCR-WF-06 |
| W7 子流程与复用 | 2 | §7.7 | — | `TBL-WF-001` (自己参照) | SCR-WF-01 |
| W8 错误处理与重试 | 3 | §7.8 | — | `TBL-WF-006` (`execution_step` retry) | SCR-WF-04 |
| W9 执行历史与调试 | 3 | §7.9 | API-WF-03 | `TBL-WF-005` (`execution_history`) / `TBL-WF-006` | SCR-WF-04 |
| W10 激活状态与版本管理 | 3 | §7.10 | API-WF-01 | `TBL-WF-001` (`enabled`) / `TBL-WF-002` (`automation_flow_versions` SCD2) | SCR-WF-01 |
| W11 标签绑定任务卡 | 5 | §7.11 | API-WF-06 | `TBL-WF-003` (`tag_binding_expr`) / **既存 `WorkItem` (拡張)** | SCR-WF-05 |
| W12 Backlog/Sprint 联动 | 5 | §7.12 | (既存 kanban 連動) | **既存 `WorkItem` (拡張)** | SCR-WF-05 |
| W13 数据一致性 | 2 | §7.13 | — | (横断) | SCR-WF-04 |
| **W14 默认工作流模板库 (v1.1)** | **7** | §7.14 | API-WF-06 / API-WF-07 | `TBL-WF-007` (`flow_template`) | SCR-WF-02 |
| **W15 智能控制+聊天栏 (v1.1)** | **5** | §7.15 | API-WF-08 | `TBL-WF-008` (`chat_session` — v1.1 新規, BD §4.2.8 確定) | SCR-WF-03 |
| **合計** | **54** | — | 8 (BD §5.1 と一致) | 8 (BD §4.1 と一致) + 既存 WorkItem 拡張 | 6 |

> **Round 2 整合確認**:BD §4.1 (8 表: BD §4.2.1〜4.2.8) / §5.1 (8 API) と本表の整合確認完了。TBL-WF ID は **BD §4.2 出現順** に 1:1 マッピング: 001=`automation_flow` / 002=`automation_flow_versions` / 003=`flow_node` / 004=`flow_edge` / 005=`execution_history` / 006=`execution_step` / 007=`flow_template` / 008=`chat_session`。Round 1 で仮置きしていた `TBL-WF-002`(=`flow_node`) / `TBL-WF-009`(=`chat_session`) 等は破棄。W11/W12 の WorkItem は **既存表の拡張** (BD §4.3) のため TBL-WF-NNN を付与せず、§11 (Round 5) で別途 `既存WorkItem拡張` として取り上げる。**8 表 8 API ともに BD 確定値と 1:1 一致 ✓**。

### §6.2 FR 別追跡表(Round 1 スケルトン — 54 行)

> 各 FR 行の「DD 詳細」/「Test 観点」/「TBD 継承」列は、Round 2 以降で §7-§13 と並行して埋める。Round 1 では **ID スロット確定 + BD 一次引用 + 既知 TBD フラグ** のみ実施。

#### W1 — 节点类型体系(P0×3 / P1×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W1.1 | BD §6.1 / §7.1 节点 kind 体系 | `MOD-WF-001` FlowEditor / `CLS-WF-001` FlowNodeService / `TBL-WF-003` (`flow_node`) | `EVT-WF-001` `flow_node.created` | 正常 / kind 非法 / 権限 / 並行作成 | — |
| FR-WORKFLOW-W1.2 | BD §7.1 边追加 | `MOD-WF-001` / `CLS-WF-002` FlowEdgeService / `TBL-WF-004` (`flow_edge`) | `EVT-WF-002` `flow_edge.created` | 正常 / 自环 / 種類不正 / 跨 Flow | — |
| FR-WORKFLOW-W1.3 | BD §7.1 `+ Flow` 入口 | `MOD-WF-001` / `CLS-WF-001` / `TBL-WF-001` (`automation_flow`) | `EVT-WF-003` `automation_flow.created` | 正常 / 権限 / 並行作成 / 失敗 rollback | — |
| FR-WORKFLOW-W1.4 | BD §7.1 节点設定側欄 | `MOD-WF-002` NodeConfigSidebar / `CLS-WF-001` / `TBL-WF-003` | (load only) | 正常 / データ読込失敗 / 編集中別 session 更新 | — |

#### W2 — 触发节点(P0×2 / P1×2)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W2.1 | BD §6.1 / §7.1 Manual 触发 | `MOD-WF-003` TriggerService / `API-WF-03` (POST executions) / `TBL-WF-005` | `EVT-WF-004` `execution.started` (manual) | 正常 / 並行連打 / 権限 | W2.1 debounce【TBD】 |
| FR-WORKFLOW-W2.2 | BD §7.1 cron 调度 | `MOD-WF-003` / `API-WF-03` / `TBL-WF-003` (`schedule_cron`) | `EVT-WF-005` `execution.started` (cron) | 正常 / cron 非法 / タイムゾーン | — |
| FR-WORKFLOW-W2.3 | BD §7.1 / §5.1 API-WF-05 Webhook 触发 | `MOD-WF-003` / `API-WF-05` (POST webhook) / `TBL-WF-003` (`webhook_token`) | `EVT-WF-006` `execution.started` (webhook) | 正常 / token 不一致 / body 過大 / 冪等キー重複 | **W2.3【TBD】HMAC / token 輪換 / body 上限 / 署名ヘッダ / IP allowlist** |
| FR-WORKFLOW-W2.4 | BD §7.1 canvas_event 触发 | `MOD-WF-003` / 既設 `canvas-collab` WS / `TBL-WF-003` (`canvas_event` filter) | `EVT-WF-007` `execution.started` (canvas_event) | 正常 / フィルタ無一致 / 高頻度 storm | **W2.4【TBD】debounce 戦略** |

#### W3 — 动作节点(P0×1 / P1×2)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W3.1 | BD §7.2 `AutomationActionKind` 6 種 | `MOD-WF-004` RuleExecutor / `CLS-WF-010` ActionDispatcher / `TBL-WF-003` (`kind=action`) | `EVT-WF-008` `step.started` / `EVT-WF-009` `step.succeeded` | 正常 / 失敗→W8 / 副作用検証 / timeout | — |
| FR-WORKFLOW-W3.2 | BD §7.2 HTTP / transform_data | `MOD-WF-004` / `CLS-WF-011` HttpAction / `CLS-WF-012` TransformAction / `TBL-WF-003` | `EVT-WF-008/009` | 正常 / HTTP 5xx / JSONPath 非法 / timeout | — |
| FR-WORKFLOW-W3.3 | BD §7.2 順序/並列 | `MOD-WF-004` / `CLS-WF-013` ExecutionScheduler / `TBL-WF-004` (順序/並列 edge 属性) | `EVT-WF-008/009` | 正常 / 並列合流 / 順序違反 / 一部失敗 | — |

#### W4 — 分支与条件(P0×1 / P1×2)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W4.1 | BD §6.3.1 / §8.4 CEL IF | `MOD-WF-004` / `CLS-WF-014` CelConditionNode / `TBL-WF-004` (`condition_expr` IF) | `EVT-WF-010` `branch.evaluated` | 正常 / 評価エラー / 両分岐非活性 | — |
| FR-WORKFLOW-W4.2 | BD §6.3.1 / §8.4 CEL Switch | `MOD-WF-004` / `CLS-WF-014` / `TBL-WF-004` (`condition_expr` Switch) | `EVT-WF-010` | 正常 / default 落ち / case 重複 | — |
| FR-WORKFLOW-W4.3 | BD §6.3.1 Merge join/race | `MOD-WF-004` / `CLS-WF-015` MergeNode / `TBL-WF-003` (`kind=merge`) | `EVT-WF-011` `join.completed` | 正常 / 部分不到達 / 全不到達 | **W4.3【TBD】join timeout 戦略** |

#### W5 — 循环与批处理(P1×1 / P2×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W5.1 | BD §7.2 / §9.3 loop | `MOD-WF-004` / `CLS-WF-016` LoopNode / `TBL-WF-003` (`kind=loop`) | `EVT-WF-012` `loop.iteration.started` | 正常 / 配列非配列 / 0 要素 / 副作用反復 | **W5.1【TBD】最大反復数 / join/debounce 戦略** |
| FR-WORKFLOW-W5.2 | BD §7.2 loop 並列度 | `MOD-WF-004` / `CLS-WF-016` / `TBL-WF-003` (`concurrency` 1-20) | `EVT-WF-012` | 正常 / 並列度境界 / 範囲外 | — |

#### W6 — 变量与表达式传递(P0×2 / P2×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W6.1 | BD §6.3.1 / §8.4 `{{node.<id>.output.<field>}}` | `MOD-WF-004` / `CLS-WF-017` ExpressionEvaluator / `TBL-WF-004` (`data_mapping`) | `EVT-WF-013` `expression.evaluated` | 正常 / node_id 不存在 / field 不存在 / 型不一致 | — |
| FR-WORKFLOW-W6.2 | BD §6.3.1 / §8.4 `{{flow.variables.<key>}}` | `MOD-WF-004` / `CLS-WF-017` / `TBL-WF-001` (`variables`) | `EVT-WF-013` | 正常 / key 不存在 / 過去 Execution 不変 | — |
| FR-WORKFLOW-W6.3 | BD §8.4 CEL 共通 | `MOD-WF-004` / `CLS-WF-017` / 既設 CEL parser | `EVT-WF-013` | 正常 / 構文不正 / `/automation` 同一性 | — |

#### W7 — 子流程与复用(P1×1 / P2×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W7.1 | BD §6.3.1 / §7.2 Subflow | `MOD-WF-004` / `CLS-WF-018` SubflowNode / `TBL-WF-001` (自己参照) / `TBL-WF-005` (`parent_execution_id`) | `EVT-WF-014` `subflow.invoked` | 正常 / 循環 / 深さ >5 / 孤立 | **W7.1【TBD】循環検出アルゴリズム詳細** |
| FR-WORKFLOW-W7.2 | BD §7.3 / W14 テンプレート選択入口 | `MOD-WF-005` TemplateSelector / `TBL-WF-007` (`flow_template`) | (load only) | 正常 / 0 件 / 失敗 → 手動作成経路確保 | — |

#### W8 — 错误处理与重试(P0×2 / P1×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W8.1 | BD §6.3.2 / §7.4 `retry_policy` | `MOD-WF-004` / `CLS-WF-019` RetryPolicyExecutor / `TBL-WF-006` (`retry_policy`) | `EVT-WF-015` `step.retry_scheduled` / `EVT-WF-016` `step.retry_executed` | 正常 / 一時的失敗回復 / 永久失敗 / 並行 retry | **W8.1【TBD】最大回数 / backoff 既定値** |
| FR-WORKFLOW-W8.2 | BD §6.3.2 `on_error` 边 | `MOD-WF-004` / `CLS-WF-020` OnErrorRouter / `TBL-WF-004` (`on_error` edge) | `EVT-WF-017` `error_branch.activated` | 正常 / `on_error` 不存在 → Execution failed / 多段 `on_error` | — |
| FR-WORKFLOW-W8.3 | BD §7.5 通知 domain | `MOD-WF-004` / 既設 notification / `TBL-WF-005` (`notified_at`) | `EVT-WF-018` `execution.failed_notified` | 正常 / 通知失敗 / 30s 遅延検証 | — |

#### W9 — 执行历史与调试(P0×2 / P1×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W9.1 | BD §7.5 履歴 append-only | `MOD-WF-004` / `CLS-WF-021` ExecutionRecorder / `TBL-WF-005` / `TBL-WF-006` | `EVT-WF-019` `execution_step.appended` | 正常 / 書込失敗 retry / 容量超過 | **W9.1【TBD】at-least-once 保証方式 / 容量・保存期間** |
| FR-WORKFLOW-W9.2 | BD §7.5 失敗再実行 | `MOD-WF-004` / `CLS-WF-022` ReplayService / `TBL-WF-005` (`resumed_from_execution_id`) | `EVT-WF-020` `execution.replayed` | 正常 / 非冪等 action 再実行 / 孤立 replay | **W9.2【TBD】冪等性標記メカニズム** |
| FR-WORKFLOW-W9.3 | BD §7.5 ステップ詳細表示 | `MOD-WF-002` / `CLS-WF-021` / `TBL-WF-006` | (read only) | 正常 / JSON 過大 / 権限 | **W9.3【TBD】JSON 分頁/截断** |

#### W10 — 激活状态与版本管理(P0×1 / P1×2)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W10.1 | BD §7.4 enabled 切替 | `MOD-WF-006` FlowActivationService / `API-WF-01` / `TBL-WF-001` (`enabled`) | `EVT-WF-021` `flow.enabled_changed` | 正常 / 未束縛 placeholder 有 → 拒否 / 権限 | **W10.1 placeholder 活性化前検証ロジック詳細** |
| FR-WORKFLOW-W10.2 | BD §7.4 SCD2 版本 | `MOD-WF-006` / `CLS-WF-023` VersionManager / `TBL-WF-002` (`automation_flow_versions`) | `EVT-WF-022` `flow_version.appended` | 正常 / 並行編集 conflict / diff 取得 | **W10.2【TBD】楽観ロック vs CRDT (SRS リスク #6)** |
| FR-WORKFLOW-W10.3 | BD §7.4 版本 rollback | `MOD-WF-006` / `CLS-WF-023` / `TBL-WF-002` | `EVT-WF-023` `flow_version.rollback` | 正常 / 不存在 version / rollback 連鎖 | — |

#### W11 — 标签绑定任务卡(P0×4 / P1×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W11.1 | BD §6.3.3 / §7.6 `tag_binding_expr` | `MOD-WF-007` TagBindingService / `API-WF-06` / `TBL-WF-003` (`tag_binding_expr`) / **既存 WorkItem** | `EVT-WF-024` `tag_binding.monitored` | 正常 / 構文不正 / 監視 ON/OFF | — |
| FR-WORKFLOW-W11.2 | BD §6.3.3 派生 WorkItem 作成 | `MOD-WF-007` / `CLS-WF-030` WorkItemDeriver / **既存 WorkItem** (`source_flow_id`) | `EVT-WF-025` `workitem.derived` | 正常 / 並行重複 (BR-W-1) / 権限 | — |
| FR-WORKFLOW-W11.3 | BD §6.3.3 派生 vs 人工 カード 同権 | `MOD-WF-007` / **既存 WorkItem** (拡張) | (派生 WorkItem 自身) | 正常 / 派生 カード 編集 / 集計 / 期限 | — |
| FR-WORKFLOW-W11.4 | BD §6.3.3 BR-W-3 三分支 | `MOD-WF-007` / `CLS-WF-031` BrW3Handler / **既存 WorkItem** | `EVT-WF-026` `workitem.binding_changed` | 正常 / 硬删 / 移回删 / detached / 中途失敗 | **W11.4【TBD】重試/補償メカニズム** |
| FR-WORKFLOW-W11.5 | BD §7.6 AND/OR/NOT ブール式 | `MOD-WF-007` / `CLS-WF-032` BooleanExprParser / `TBL-WF-003` | `EVT-WF-027` `tag_binding.expr_evaluated` | 正常 / 構文不正 / 括弧 / NOT 単独 | — |

#### W12 — Backlog/Sprint 联动(P0×3 / P1×2)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W12.1 | BD §6.3.3 / §7.7 派生 Backlog 配置 | `MOD-WF-008` SprintLinkService / **既存 WorkItem** (`sprint_id`) | `EVT-WF-028` `workitem.placed_in_backlog` | 正常 / sprint_id 空 / 既存 kanban 連動 | — |
| FR-WORKFLOW-W12.2 | BD §7.7 派生 Sprint 拖拽 | `MOD-WF-008` / 既設 `@dnd-kit` / **既存 WorkItem** | `EVT-WF-029` `workitem.moved_to_sprint` | 正常 / 拖拽先ロック / 派生 カード | — |
| FR-WORKFLOW-W12.3 | BD §6.3.3 detached 状態 | `MOD-WF-008` / **既存 WorkItem** (`tag_binding_status`) | `EVT-WF-030` `workitem.detached` | 正常 / detached 中 Sprint 完了 / 取消 | **W12.3【TBD】detached 取消 / 人間確認要否** |
| FR-WORKFLOW-W12.4 | BD §3.2 SCR-WF-05 | `MOD-WF-008` / UI SCR-WF-05 / `TBL-WF-003` + **既存 WorkItem** | (read only + edit) | 正常 / 0 件 / 権限 | — |
| FR-WORKFLOW-W12.5 | BD §6.3.3 `user_edited_fields` | `MOD-WF-008` / **既存 WorkItem** (`user_edited_fields` JSONB) | `EVT-WF-031` `workitem.user_field_locked` | 正常 / system 字段 上書防止 / 解除 | — |

#### W13 — 数据一致性(P0×1 / P1×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W13.1 | BD §6.3.3 `status_mapping` 連動 | `MOD-WF-009` StatusMappingService / `TBL-WF-001` (`status_mapping` JSONB) / **既存 WorkItem** | `EVT-WF-032` `workitem.status_synced` | 正常 / 映射 未設定 / 映射 非法 | — |
| FR-WORKFLOW-W13.2 | BD §6.3.3 `1 task 1 sprint` 制約 | `MOD-WF-008` (制約) / **既存 WorkItem** | (制約) | 正常 / 派生 vs 人工 同値字段 | — |

#### W14 — 默认工作流模板库 v1.1(P0×1 / P1×5 / P2×1)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W14.1 | BD §3.2 SCR-WF-02 / §7.3 | `MOD-WF-005` / UI SCR-WF-02 / `TBL-WF-007` | (read) | 正常 / 0 件 / 読込失敗 | — |
| FR-WORKFLOW-W14.2 | BD §7.3 / **T-46【W14 既定 3 テンプレ BD/SRS 欠落】** | `MOD-WF-005` / `CLS-WF-040` TemplateInstantiator / `TBL-WF-007` (AAA 式) + `TBL-WF-003/004` | `EVT-WF-033` `flow.instantiated_from_template` | 正常 / 並行 / 適用後編集 | — (参照テンプレ未確定時は実装段階で降格) |
| FR-WORKFLOW-W14.3 | BD §7.3 / **T-46** | `MOD-WF-005` / `CLS-WF-040` / `TBL-WF-007` (spec 式) + `TBL-WF-003/004` | `EVT-WF-033` | 正常 / 循環構築 / 編集離脱 | **W14.3【TBD】Flow 級 最大循環回数 制限** |
| FR-WORKFLOW-W14.4 | BD §7.3 / **T-46** | `MOD-WF-005` / `CLS-WF-040` / `TBL-WF-007` (superpowers 式) + `TBL-WF-003/004` | `EVT-WF-033` | 正常 / 循環 / 段階名 自由 | — |
| FR-WORKFLOW-W14.5 | BD §7.3 / §5.1 / §8.3 活性化前検証 | `MOD-WF-005` / `CLS-WF-041` ActivationGuard / `TBL-WF-003` (`is_placeholder`, `agent_id`) | `EVT-WF-034` `flow.activation_blocked` | 正常 / placeholder + agent_id 空 → 拒否 / 拒否メッセージ具体性 | **W14.5 活性化前検証ロジック詳細** |
| FR-WORKFLOW-W14.6 | BD §7.3 別保存 | `MOD-WF-005` / `CLS-WF-040` / `TBL-WF-007` (`is_builtin=false`) | `EVT-WF-035` `flow_template.duplicated` | 正常 / 同名重複 / 内蔵影響無 | — |
| FR-WORKFLOW-W14.7 | BD §7.3 自定義 CRUD | `MOD-WF-005` / `API-WF-07` / `TBL-WF-007` | `EVT-WF-036` `flow_template.crud` | 正常 / 内蔵削除試行 403 / 検索 | — |

#### W15 — 智能控制 + 聊天栏 v1.1(P0×2 / P1×3)

| FR ID | BD 一次引用 | DD Module/Class/API/Table | Event | Test 観点(予定) | TBD 継承 |
|---|---|---|---|---|---|
| FR-WORKFLOW-W15.1 | BD §3.2 SCR-WF-03 / §8 L0 接続 | `MOD-WF-010` ChatBarService / UI SCR-WF-03 / `TBL-WF-008` (`chat_session`) | `EVT-WF-037` `chat_session.opened` | 正常 / L0 不到達 → 离线表示 / 画布影響無 | — |
| FR-WORKFLOW-W15.2 | BD §7.8 mock 解析 → 草稿 | `MOD-WF-010` / `CLS-WF-050` DraftParser / `TBL-WF-008` (`parsed_flow_draft`) | `EVT-WF-038` `chat_draft.generated` | 正常 / 未命中 → 提示 / 草稿取消 / 草稿 → 本保存 | **W15.2【TBD】mock 規則カバー範囲** |
| FR-WORKFLOW-W15.3 | BD §6.3.1 / §8 動的ルーティング | `MOD-WF-011` LangGraphRouter / `TBL-WF-006` (`routing_decision` JSONB) | `EVT-WF-039` `branch.dynamic_routed` | 正常 / L0 不到達 / 决策可再現性 | **W15.3【TBD】L0 降級戦略 / 决策可再現性** |
| FR-WORKFLOW-W15.4 | BD §7.8 チャット→Execution 回鎖 | `MOD-WF-010` / `TBL-WF-005` (`origin_chat_session_id`) | `EVT-WF-040` `execution.chat_origin_recorded` | 正常 / session 期限切れ / 権限 | — |
| FR-WORKFLOW-W15.5 | BD §7.8 草稿 ノード = 手動 ノード | `MOD-WF-010` / `TBL-WF-003/004` (共用) | (生成/編集/削除 event) | 正常 / 草稿 → 編集中 / 草稿 取消 | — |

> **Round 2 整合確認**:§6.2 全 54 FR 行の Table 参照を BD §4.2 出現順 ID に統一 (`TBL-WF-001`〜`008`)。`WorkItem` (W11/W12/W13) は BD §4.3 の既存表拡張であり、新規 TBL-WF ID を付与せず **既存 WorkItem** と明示。W14 については §6.2 BD 一次引用列に **T-46** タグ(後述 §13 TBD 追跡) を追加 — W14 テンプレート 3 セット(AAA/spec/superpowers) の上流 SRS/BD が本 worktree に未配置(BD-CANVAS-WORKFLOW-001 §1.1.14 のテンプレート名は確認できるが、詳細ノード/エッジ/データフロー仕様書が未配置 — `SRS-WORKFLOW-TEMPLATE-001.md` は別ドメインの Kanban 17 列テンプレート)。
> **Round 2 終了時点の整合確認**:
> - 54 FR 全項目を BD §1.1.1〜§1.1.15 から抽出して表化済(漏出ゼロ、W14/W15 の v1.1 追加 12 項目も個別に追跡可能)
> - TBL-WF ID を BD §4.2 出現順に 1:1 マッピング(001〜008)、既存 WorkItem 拡張は **既存 WorkItem** として明示
> - W14 既定 3 テンプレ上流 SRS/BD 不在を **T-46** として §13 TBD 追跡表で持ち越し(W14.2/W14.3/W14.4 行 BD 一次引用列に記載)
> - §7 コンポーネント/Class 詳細設計を §7.1〜§7.15 で全 54 FR カバー(後述 §7 章)
>
> **未着手項目(Round 2 の範囲外)**:
> - §8 処理詳細・状態遷移(Round 3)
> - §9 API 内部処理設計(Round 4)
> - §10 データ/SQL/CRUD(Round 5)
> - §11 セキュリティ/ログ/監査(Round 6)
> - §12 NFR/テスト観点 + §13 TBD 追跡 + §14 IPA 自審(Round 7)
>
> ---

## §7 コンポーネント / Class 詳細設計

> **本節の位置付け**:Round 2 で全 54 FR に対応するモジュール/Class/メソッドを実装可能な粒度で記述する。BD §6.4 で宣言された 5 module (`workflow-engine` / `automation` (拡張) / `work-item` (拡張) / `flow-template-library` / `chat-bar`) を起点に、`skill-multica-2` §7〜§9 に従い Module ID → Class ID → Public Method シグネチャの 3 階層で記述する。ID 体系は `MOD-WF-NNN` / `CLS-WF-NNN` / `EVT-WF-NNN` プレフィックス + 連番、§6 FR 追跡行列で参照した ID と 1:1 一致させる。
>
> **粒度方針**:本 DD は Rust 風 pseudocode で `struct` + `pub fn` シグネチャまで記述し、本体ロジックは Round 3 で詳細展開する。trait / private method / 内部型は本 DD で定義せず、Rust の場合は実装段階で `impl` 内に閉じる。
>
> **TBD 継承**:BD §9.3 の T-01〜T-35 を本 §7 章で言及する場合、対応する FR / Class / Method のコメントに `【TBD: <T-NN>】` と付記する。

### §7.0 Module 一覧 (per BD §6.4 5 module 起点)

| Module ID | 名称 | 関係 | 該当 FR 帯 |
|---|---|---|---|
| `MOD-WF-001` | FlowEditor | 新規 (workflow-engine 配下) | W1-W2 (編集 + トリガノード定義) |
| `MOD-WF-002` | NodeConfigSidebar | 新規 (workflow-engine 配下) | W1.4 / W9.3 / W12.4 |
| `MOD-WF-003` | TriggerService | 新規 (workflow-engine 配下) | W2 (4 FR) |
| `MOD-WF-004` | RuleExecutor | 新規 (workflow-engine 配下, 既存 `automation` 拡張の上位) | W3-W9 (24 FR, 最大モジュール) |
| `MOD-WF-005` | FlowTemplateLibrary | 新規 (flow-template-library, v1.1) | W7.2 / W14 (8 FR) |
| `MOD-WF-006` | FlowActivationService | 新規 (workflow-engine 配下) | W10 (3 FR) |
| `MOD-WF-007` | TagBindingService | 新規 (work-item 拡張) | W11 (5 FR) |
| `MOD-WF-008` | SprintLinkService | 新規 (work-item 拡張, Kanban 連動) | W12 (5 FR) |
| `MOD-WF-009` | StatusMappingService | 新規 (work-item 拡張, 制約性) | W13.1 (1 FR) |
| `MOD-WF-010` | ChatBarService | 新規 (chat-bar, v1.1, ADR-0046 経由) | W15.1 / W15.2 / W15.4 / W15.5 (4 FR) |
| `MOD-WF-011` | LangGraphRouter | 新規 (chat-bar + workflow-engine 横断, ADR-0046 経由) | W15.3 (1 FR) |
| (既存) | automation | 既存 (BD §6.4 で「拡張」と明示) | 後方互換 |
| (既存) | work-item | 既存 (BD §4.3 で 4 フィールド拡張) | 後方互換 |
| (既存) | notification | 既存 (BD §6.4 で `send_notification` 動作を流用) | W8.3 |
| (既存) | canvas-collab | 既存 (BD §5.3 で `wss://canvas-collab/canvases/[id]` 复用) | W2.4 |

> **境界整合**:`MOD-WF-001`〜`MOD-WF-006` / `MOD-WF-011` は新規 crate `crates/workflow-engine/` 配下に集約予定(BD §6.4 `workflow-engine` 1:1 対応)。`MOD-WF-007`〜`MOD-WF-009` は既存 crate `crates/work-item/` 配下に拡張実装。`MOD-WF-005` は新規 crate `crates/flow-template-library/`、`MOD-WF-010`/`MOD-WF-011` は新規 crate `crates/chat-bar/`(L0 経由、ADR-0046 既存エンドポイント流用)。
>
> **ADR-0046 パス確認結果【上位設計確認事項 / 持ち越し】**:BD 头部 preamble で参照されている `docs/architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md` が本 worktree に **未配置**(2026-09-14 時点で `docs/architecture/2026-08-26-upgrade/adr/` の最新ファイルは `0039-*` まで)。本 DD §7.15 では BD preamble の記述を前提に §7.15 で API パスと State フィールドを記述するが、**実装着手前に ADR-0046 実在パスを文書横断で確認することを必須**(Round 2 完了条件に含めず、Round 7 IPA 自審までに最終確認)。確認できない場合、`/api/tmo/*` 8 端点ではなく /v1/collaboration/chat-sessions/{id} 配下に mock L0 ハンドラを実装する代替案を §13 で保持。

### §7.1 `MOD-WF-001` FlowEditor (FR-W1.1〜W1.3)

| 項目 | 内容 |
|---|---|
| Module ID | `MOD-WF-001` |
| 名称 | FlowEditor |
| 対応 BD | BD §6.1 / §7.1 |
| 責務 | Flow / Node / Edge の作成・編集・削除の API/BFF ハンドラ入口と背後の Domain Service 集合。`+ Flow` 入口の処理、SCR-WF-01 からの CRUD を受付、validation と service 呼出に振り分ける |
| 入力 | (a) `POST /v1/collaboration/flows` body `{name, tags?, tag_binding_expr?}` / (b) `POST /v1/collaboration/flows/{id}/nodes` body `{kind, trigger_kind?, action_kind?, condition_expr?, position_x, position_y, ...}` / (c) `POST /v1/collaboration/flows/{id}/edges` body `{from_node_id, to_node_id, edge_label?, edge_kind='normal'|'on_error'}` |
| 出力 | 各 CRUD 201 で `automation_flow` / `flow_node` / `flow_edge` の entity を返却 |
| 依存 | `MOD-WF-006` FlowActivationService (enabled 切替時の placeholder 検証連係) / 既存 `auth` / 既存 `tenant` (RLS middleware) |
| 外部 IF | API-WF-01〜02 (`/v1/collaboration/flows`, `/nodes`, `/edges`) |
| 使用データ | `TBL-WF-001` automation_flow / `TBL-WF-003` flow_node / `TBL-WF-004` flow_edge |
| 状態 | 単一ハンドラ集合 (state なし、ステートレス) |
| Transaction | POST は 1 レコード単位の TX; Flow+node+edges の一括作成は本 Module では行わない(テンプレート套用 §7.14 が別途担当) |
| Error | ERR-WF-VAL-001 (kind 非法) / ERR-WF-VAL-002 (edge 自环) / ERR-WF-AUTHZ-001 (書込権限無) / ERR-WF-DUP-001 (同 Flow 内 label 重複 edge) |

```rust
// crates/workflow-engine/src/editor/mod.rs (R7-I-1)
pub struct FlowEditor {
    flow_ops: Arc<FlowOps>,
    node_ops: Arc<FlowNodeOps>,
    edge_ops: Arc<FlowEdgeOps>,
    activation: Arc<FlowActivationService>,
    tenant_ctx: Arc<TenantContext>,
}

impl FlowEditor {
    /// FR-W1.3: "+ Flow" 入口, 新規空 Flow を作成
    pub async fn create_flow(&self, input: CreateFlowInput, actor: Uuid) -> Result<AutomationFlow, WFError>;

    /// FR-W1.1: kind を含む新規 node を作成
    pub async fn add_node(&self, flow_id: Uuid, input: CreateNodeInput, actor: Uuid) -> Result<FlowNode, WFError>;

    /// FR-W1.2: edge を作成 (from != to 制約)
    pub async fn add_edge(&self, flow_id: Uuid, input: CreateEdgeInput, actor: Uuid) -> Result<FlowEdge, WFError>;

    /// FR-W1.1 補助: node 削除 (cascade edge)
    pub async fn delete_node(&self, flow_id: Uuid, node_id: Uuid, actor: Uuid) -> Result<(), WFError>;

    /// FR-W1.1 / W1.4 補助: position 更新 (drag 時)
    pub async fn move_node(&self, flow_id: Uuid, node_id: Uuid, x: f32, y: f32, actor: Uuid) -> Result<FlowNode, WFError>;
}

pub struct FlowOps { /* ... */ }
impl FlowOps {
    pub async fn create(&self, input: CreateFlowInput, tenant_id: Uuid, actor: Uuid) -> Result<AutomationFlow, WFError>;
    pub async fn get(&self, flow_id: Uuid, tenant_id: Uuid) -> Result<AutomationFlow, WFError>;
    pub async fn list(&self, filter: FlowFilter, tenant_id: Uuid) -> Result<Vec<AutomationFlow>, WFError>;
    pub async fn patch(&self, flow_id: Uuid, patch: FlowPatch, actor: Uuid) -> Result<AutomationFlow, WFError>;
    pub async fn soft_delete(&self, flow_id: Uuid, actor: Uuid) -> Result<(), WFError>;  // 物理削除禁止 per BD §4.5
}

pub struct FlowNodeOps { /* ... */ }
impl FlowNodeOps {
    pub async fn create(&self, flow_id: Uuid, input: CreateNodeInput, tenant_id: Uuid, actor: Uuid) -> Result<FlowNode, WFError>;
    pub async fn get(&self, node_id: Uuid, tenant_id: Uuid) -> Result<FlowNode, WFError>;
    pub async fn list_by_flow(&self, flow_id: Uuid, tenant_id: Uuid) -> Result<Vec<FlowNode>, WFError>;
    pub async fn patch(&self, node_id: Uuid, patch: NodePatch, actor: Uuid) -> Result<FlowNode, WFError>;  // §7.2 NodeConfigSidebar から呼ばれる
    pub async fn delete(&self, node_id: Uuid, actor: Uuid) -> Result<(), WFError>;  // cascade edge 削除
}

pub struct FlowEdgeOps { /* ... */ }
impl FlowEdgeOps {
    pub async fn create(&self, flow_id: Uuid, input: CreateEdgeInput, tenant_id: Uuid, actor: Uuid) -> Result<FlowEdge, WFError>;
    pub async fn get(&self, edge_id: Uuid, tenant_id: Uuid) -> Result<FlowEdge, WFError>;
    pub async fn list_by_flow(&self, flow_id: Uuid, tenant_id: Uuid) -> Result<Vec<FlowEdge>, WFError>;
    pub async fn patch(&self, edge_id: Uuid, patch: EdgePatch, actor: Uuid) -> Result<FlowEdge, WFError>;
    pub async fn delete(&self, edge_id: Uuid, actor: Uuid) -> Result<(), WFError>;
    /// validation: from_node.flow_id == to_node.flow_id == flow_id かつ from_node != to_node
    pub async fn validate_acyclic(&self, edge_input: &CreateEdgeInput) -> Result<(), WFError>;
}
```

| Field / 型 | 用途 |
|---|---|
| `CreateFlowInput { name: String, tags: Vec<String>, tag_binding_expr: Option<String> }` | FR-W1.3 入力 |
| `CreateNodeInput { kind: NodeKind, trigger_kind: Option<TriggerKind>, action_kind: Option<ActionKind>, condition_expr: Option<String>, merge_mode: Option<MergeMode>, loop_source_expr: Option<String>, loop_body_node_ids: Option<Vec<Uuid>>, concurrency: Option<u8>, input_bindings: Option<Value>, referenced_flow_id: Option<Uuid>, webhook_token: Option<String>, routing_mode: RoutingMode, is_placeholder: bool, placeholder_role_hint: Option<String>, position_x: f32, position_y: f32 }` | FR-W1.1 入力 |
| `CreateEdgeInput { from_node_id: Uuid, to_node_id: Uuid, edge_label: Option<String>, edge_kind: EdgeKind }` | FR-W1.2 入力 |
| `NodePatch { ... }` / `EdgePatch { ... }` | 部分更新 |
| `NodeKind` enum: `trigger | action | condition | merge | loop | subworkflow | agent_placeholder` (per BD §4.2.3) |
| `TriggerKind` enum: `manual | schedule_cron | webhook | canvas_event` (per BD §4.4) |
| `ActionKind` enum: 既存 6 種 + `http_request | transform_data | dispatch_agent` (per BD §4.4) |
| `MergeMode` enum: `race | join` (per BD §4.2.3) |
| `EdgeKind` enum: `normal | on_error` (per BD §4.2.4) |
| `RoutingMode` enum: `static_cel | dynamic_agent` (per BD §4.2.3 + FR-W15.3) |

> **【TBD: T-01】** FR-W1.3 の Flow エディタが主画布 viewport を复用するか 独立子画布を持つかは本 DD では判定せず、`MOD-WF-001` の責務はどちらでも実装可能なよう API 境界を分離。
> **【TBD: T-02】** 後端実持久化引擎が v1 mock の場合、`FlowOps::create` 等の戻り値が session 跨ぎで復元できない可能性あり。実装段階で session-bound の取り扱い要決定。

### §7.2 `MOD-WF-002` NodeConfigSidebar (FR-W1.4 / W9.3 / W12.4)

| 項目 | 内容 |
|---|---|
| Module ID | `MOD-WF-002` |
| 名称 | NodeConfigSidebar |
| 対応 BD | BD §7.1 (W1.4) / §7.5 (W9.3) / §7.7 (W12.4) |
| 責務 | ノード双击 → 側欄表示、単一ノード設定の form 生成 / Validation / 保存 (PATCH 委托)。Execution step JSON 詳細表示 |
| 入力 | (a) `GET /v1/collaboration/flows/{id}/nodes/{node_id}` / (b) `PATCH /v1/collaboration/flows/{id}/nodes/{node_id}` body `{...NodePatch}` / (c) `GET /v1/collaboration/flows/{id}/executions/{exec_id}/steps/{step_id}` |
| 出力 | 各 entity / Execution step (input/output JSON) |
| 依存 | `MOD-WF-001` FlowNodeOps.patch / `MOD-WF-004` RuleExecutor.ExecutionRecorder (read step) |
| 外部 IF | API-WF-02 (node CRUD) / API-WF-03 (execution read) |
| 使用データ | `TBL-WF-003` flow_node / `TBL-WF-006` execution_step |
| 状態 | フロントエンド React component 状態 (UI の話なので本 DD ではサーバ側 IF のみ記述) |
| Transaction | PATCH node は 1 レコード単位 |
| Error | 既存 + W6 構文不正 (CEL parser 失敗時) / W5.2 concurrency 範囲外 |

> **【TBD: T-27】** FR-W9.3 JSON 過大時の pagination/truncate は SCR-WF-04 / SCR-WF-06 双方で必要。実装段階では `truncate_bytes(64KB)` 程度の安全策を default とし、超過分は別 request で取得する経路を確保。

### §7.3 `MOD-WF-003` TriggerService (FR-W2.1〜W2.4)

| 項目 | 内容 |
|---|---|
| Module ID | `MOD-WF-003` |
| 名称 | TriggerService |
| 対応 BD | BD §7.1 (W2 触发节点) |
| 責務 | 4 種 Trigger (Manual / cron / Webhook / canvas_event) の Event → Execution 起動へ変換。trigger_kind 別 dispatcher |
| 入力 | (a) Manual: `POST /v1/collaboration/flows/{id}/executions` `{trigger_kind: "manual", input?}` | (b) cron: システム内部 scheduler からの cron 着火 callback | (c) Webhook: `POST /v1/collaboration/flows/{id}/webhook/{token}` body 任意 JSON | (d) canvas_event: 既存 `canvas-collab` WS の element.update イベント (filter expression 一致時のみ) |
| 出力 | (a/b/c/d 共通) `execution_history.id` (running 状態で作成) |
| 依存 | `MOD-WF-004` RuleExecutor (Execution 起動委托) / `MOD-WF-006` FlowActivationService (enabled 検証) |
| 外部 IF | API-WF-03 / API-WF-05 / 既設 cron スケジューラ / 既設 canvas-collab WS |
| 使用データ | `TBL-WF-001` automation_flow (enabled 状態) / `TBL-WF-003` flow_node (kind=trigger, trigger_kind) / `TBL-WF-005` execution_history |
| 状態 | 内部に webhook_token → flow_id 索引 (起動時のみメモリロード、キャッシュ更新は別途) |
| Transaction | execution_history INSERT を起点に 1 TX、RuleExecutor.start_execution に chain |
| Error | ERR-WF-TRG-001 (enabled=false) / ERR-WF-TRG-002 (cron 非法) / ERR-WF-TRG-003 (token 不一致) / ERR-WF-TRG-004 (canvas_event filter 不一致) |

```rust
// crates/workflow-engine/src/trigger/mod.rs (R7-I-2)
pub struct TriggerService {
    rule_executor: Arc<RuleExecutor>,
    activation: Arc<FlowActivationService>,
    webhook_index: Arc<RwLock<HashMap<String, Uuid>>>,  // token -> flow_id
    cron_scheduler: Arc<CronScheduler>,
    canvas_event_sub: Arc<CanvasEventSubscriber>,
    tenant_ctx: Arc<TenantContext>,
}

impl TriggerService {
    /// FR-W2.1: Manual 触发 (UI "运行" ボタン → API-WF-03 POST)
    pub async fn fire_manual(&self, flow_id: Uuid, input: Option<Value>, actor: Uuid) -> Result<Uuid /* execution_id */, WFError>;
    /// FR-W2.2: cron 着火 (scheduler callback)
    pub async fn fire_cron(&self, flow_id: Uuid, schedule_id: Uuid) -> Result<Uuid, WFError>;
    /// FR-W2.3: Webhook 着火 (API-WF-05)
    pub async fn fire_webhook(&self, flow_id: Uuid, token: &str, body: Value, idempotency_key: Option<String>) -> Result<Uuid, WFError>;
    /// FR-W2.4: canvas_event 着火 (WS subscriber callback)
    pub async fn fire_canvas_event(&self, flow_id: Uuid, event: CanvasEvent, actor: Uuid) -> Result<Uuid, WFError>;

    /// 内部: trigger_kind に応じた dispatcher
    async fn dispatch(&self, flow_id: Uuid, trigger_kind: TriggerKind, input: Option<Value>, origin: ExecutionOrigin) -> Result<Uuid, WFError>;

    /// 内部: validation - enabled=true か + W10.1 placeholder 検証
    async fn pre_check(&self, flow_id: Uuid) -> Result<(), WFError>;
}

pub struct ExecutionOrigin {
    actor: Option<Uuid>,
    chat_session_id: Option<Uuid>,  // FR-W15.4
    raw_input: Option<Value>,
    trigger_kind: TriggerKind,
}
```

> **【TBD: T-22】** FR-W2.1 Manual 触発 debounce 要否。v1 では debounce なし (per BD)、毎回 1 つの独立 Execution を作成。
> **【TBD: T-23】** FR-W2.4 canvas_event storm debounce 戦略。
> **【TBD: T-15 / T-33】** FR-W2.3 Webhook: 静的 token 比較 + body 上限 + (TODO HMAC/IP allowlist)、実装段階では body 上限を default 1MB とし、HMAC と IP allowlist は §13 で保持。

### §7.4 `MOD-WF-004` RuleExecutor (FR-W3.1〜W9.3, 24 FR — 最大)

| 項目 | 内容 |
|---|---|
| Module ID | `MOD-WF-004` |
| 名称 | RuleExecutor |
| 対応 BD | BD §7.2 / §7.4 / §7.5 |
| 責務 | Execution 起動 + Node 順次/並列実行 + 状態遷移 + retry/on_error/履歴書込 + replay。W3-W9 24 FR を本モジュールが担当 |
| 入力 | `ExecutionOrigin` (TriggerService から) |
| 出力 | execution_history 更新 + execution_step append-only |
| 依存 | `MOD-WF-003` TriggerService / `MOD-WF-006` FlowActivationService / 既設 `notification` (W8.3) / 既設 25 module 各 action handler |
| 外部 IF | API-WF-03 / API-WF-04 (resume) |
| 使用データ | `TBL-WF-005` execution_history / `TBL-WF-006` execution_step / `TBL-WF-001` automation_flow / `TBL-WF-003` flow_node / `TBL-WF-004` flow_edge |
| 状態 | 隔離 runtime; 1 Execution ごとに 1 つの State machine context (per in-memory 状態 + 永続化 checkpoint) |
| Transaction | Execution 全体は **Saga パターン**(単一 TX ではなく補償可能単位); step 単位は 1 TX で append-only 書込 |
| Error | ERR-WF-EXE-001 (定義非法 → step failed → W8 リトライ) / ERR-WF-EXE-002 (timeout) / ERR-WF-EXE-003 (外部 5xx) |

```rust
// crates/workflow-engine/src/executor/mod.rs (R7-I-3)
pub struct RuleExecutor {
    action_dispatcher: Arc<ActionDispatcher>,        // CLS-WF-010
    transform_action: Arc<TransformAction>,          // CLS-WF-012
    http_action: Arc<HttpAction>,                    // CLS-WF-011
    scheduler: Arc<ExecutionScheduler>,              // CLS-WF-013
    cel_evaluator: Arc<CelConditionNode>,            // CLS-WF-014
    merge_node: Arc<MergeNode>,                      // CLS-WF-015
    loop_node: Arc<LoopNode>,                        // CLS-WF-016
    expr_evaluator: Arc<ExpressionEvaluator>,        // CLS-WF-017
    subflow: Arc<SubflowNode>,                       // CLS-WF-018
    retry: Arc<RetryPolicyExecutor>,                 // CLS-WF-019
    on_error: Arc<OnErrorRouter>,                    // CLS-WF-020
    recorder: Arc<ExecutionRecorder>,                // CLS-WF-021
    replay: Arc<ReplayService>,                      // CLS-WF-022
    notification: Arc<NotificationClient>,
    tenant_ctx: Arc<TenantContext>,
}

impl RuleExecutor {
    /// Execution 起動 (TriggerService から chain される)
    pub async fn start_execution(&self, origin: ExecutionOrigin) -> Result<Uuid, WFError>;
    /// FR-W9.2: 失敗ノードから resume
    pub async fn resume_execution(&self, execution_id: Uuid, from_node_id: Uuid, actor: Uuid) -> Result<Uuid /* new exec id */, WFError>;

    /// 内部: グラフ topological order に沿って node を walk
    async fn walk_graph(&self, exec_ctx: &mut ExecutionContext) -> Result<ExecOutcome, WFError>;
    /// 内部: node 1 個を実行 (kind 別 dispatcher)
    async fn execute_node(&self, node: &FlowNode, ctx: &mut ExecutionContext) -> Result<NodeOutcome, WFError>;
    /// 内部: step を永続化 (append-only, at-least-once)
    async fn persist_step(&self, step: ExecutionStep, ctx: &ExecutionContext) -> Result<(), WFError>;
}

// CLS-WF-010 ActionDispatcher
pub struct ActionDispatcher;
impl ActionDispatcher {
    pub async fn dispatch(&self, node: &FlowNode, input: Value, ctx: &ExecutionContext) -> Result<Value /* output */, WFError>;
    /// action_kind 別に既存 25 module へルーティング (http/email/db/agent/notification/etc)
    fn route(&self, action_kind: ActionKind) -> Box<dyn ActionHandler>;
}

// CLS-WF-011 HttpAction
pub struct HttpAction { client: reqwest::Client }
impl HttpAction {
    pub async fn execute(&self, cfg: &HttpActionConfig, input: Value) -> Result<Value, WFError>;
}

// CLS-WF-012 TransformAction (JSONPath / CEL)
pub struct TransformAction;
impl TransformAction {
    pub async fn transform(&self, expr: &str, input: Value) -> Result<Value, WFError>;
}

// CLS-WF-013 ExecutionScheduler (順序/並列 edge 解決)
pub struct ExecutionScheduler;
impl ExecutionScheduler {
    pub async fn next_nodes(&self, current: &FlowNode, edges: &[FlowEdge], ctx: &ExecutionContext) -> Vec<Uuid>;
    pub async fn parallel_dispatch(&self, node_ids: Vec<Uuid>, ctx: &ExecutionContext) -> Vec<NodeOutcome>;
}

// CLS-WF-014 CelConditionNode (IF / Switch)
pub struct CelConditionNode;
impl CelConditionNode {
    pub async fn evaluate(&self, mode: ConditionMode, expr: &str, input: Value) -> Result<BranchDecision, WFError>;
    /// BD §6.3.1 + 既存 AutomationRule.condition_expr 沙箱を 1:1 复用
}

// CLS-WF-015 MergeNode (race / join)
pub struct MergeNode;
impl MergeNode {
    pub async fn await_arrivals(&self, mode: MergeMode, expected: usize, ctx: &ExecutionContext) -> Result<Value /* aggregated */, WFError>;
    /// 【TBD: T-24】 join mode timeout 戦略
}

// CLS-WF-016 LoopNode
pub struct LoopNode;
impl LoopNode {
    pub async fn iterate(&self, source_expr: &str, body_node_ids: Vec<Uuid>, concurrency: u8, ctx: &mut ExecutionContext) -> Result<Value, WFError>;
    /// 【TBD: T-25】 最大反復数上限 / join/debounce 戦略
}

// CLS-WF-017 ExpressionEvaluator ({{node.<id>.output.<field>}} / {{flow.variables.<key>}} / CEL)
pub struct ExpressionEvaluator;
impl ExpressionEvaluator {
    pub async fn evaluate(&self, expr: &str, ctx: &ExecutionContext) -> Result<Value, WFError>;
    /// 【TBD: T-18】 同一 Execution コンテキスト内に限定して node 出力を解決 (跨 Flow 越権防止)
}

// CLS-WF-018 SubflowNode (referenced_flow_id)
pub struct SubflowNode;
impl SubflowNode {
    pub async fn invoke(&self, referenced_flow_id: Uuid, input: Value, ctx: &mut ExecutionContext) -> Result<Value, WFError>;
    /// 【TBD: T-26】 循環検出アルゴリズム詳細 (深さ ≤5, 静的 cycle 検出)
}

// CLS-WF-019 RetryPolicyExecutor
pub struct RetryPolicyExecutor;
impl RetryPolicyExecutor {
    pub async fn run_with_retry<F, T>(&self, policy: &RetryPolicy, op: F) -> Result<T, WFError>
        where F: Future<Output = Result<T, WFError>>;
    /// 【TBD: T-06】 max_retries / backoff 既定値
}

// CLS-WF-020 OnErrorRouter
pub struct OnErrorRouter;
impl OnErrorRouter {
    pub async fn route(&self, failed_node: &FlowNode, ctx: &ExecutionContext) -> Result<Vec<Uuid> /* next node ids */, WFError>;
    /// on_error edge 不存在なら → ERR-WF-EXE-FAILED (Execution failed)
}

// CLS-WF-021 ExecutionRecorder
pub struct ExecutionRecorder;
impl ExecutionRecorder {
    pub async fn append_step(&self, step: ExecutionStep) -> Result<(), WFError>;
    /// 【TBD: T-26】 at-least-once 保証 (書入失敗 retry + dead letter)
    pub async fn append_history(&self, history: ExecutionHistory) -> Result<(), WFError>;
}

// CLS-WF-022 ReplayService
pub struct ReplayService;
impl ReplayService {
    pub async fn resume_from_failed_node(&self, execution_id: Uuid, actor: Uuid) -> Result<Uuid /* new exec id */, WFError>;
    /// 【TBD: T-03】 幂等性標記メカニズム - 上游が非幂等 action の場合の安全性
}
```

> **【TBD: T-08】** W15.3 動的ルーティング (`routing_mode="dynamic_agent"`) は本モジュールでなく `MOD-WF-011` LangGraphRouter (L0 経由) が担当。
> **【TBD: T-26】** Execution step at-least-once 保証 (書入失敗 retry + dead letter queue) は Round 3 で詳細化。

### §7.5 `MOD-WF-005` FlowTemplateLibrary (FR-W7.2 / W14.1〜W14.7, 8 FR)

| 項目 | 内容 |
|---|---|
| Module ID | `MOD-WF-005` |
| 名称 | FlowTemplateLibrary |
| 対応 BD | BD §7.3 / §1.1.14 |
| 責務 | テンプレート CRUD (built-in / custom) + 套用 (instantiate) + ActivationGuard。W14 7 FR + W7.2 1 FR 担当 |
| 入力 | (a) `GET /v1/collaboration/flow-templates` (list, built-in 含む) / (b) `POST /v1/collaboration/flow-templates` (custom 作成) / (c) `PATCH /v1/collaboration/flow-templates/{id}` (custom 改名) / (d) `DELETE /v1/collaboration/flow-templates/{id}` (custom 削除、built-in は 403) / (e) `POST /v1/collaboration/flow-templates/{id}/instantiate` (套用) / (f) `POST /v1/collaboration/flow-templates/{id}/duplicate` (別保存) |
| 出力 | `flow_template` entity / instantiate 結果の `automation_flow` (新規) |
| 依存 | `MOD-WF-001` FlowEditor (instantiate 結果の Flow を作成) / `MOD-WF-006` FlowActivationService (placeholder 検証) |
| 外部 IF | API-WF-06 (CRUD) / API-WF-07 (instantiate) / SCR-WF-02 UI |
| 使用データ | `TBL-WF-007` flow_template |
| 状態 | 起動時に built-in 3 件を seed (AAA / spec / superpowers)、tenant 跨ぎで read-only 公開 |
| Transaction | instantiate は 1 TX で template 全体 → Flow + N nodes + M edges を一括作成 |
| Error | ERR-WF-TPL-001 (built-in 削除試行 403) / ERR-WF-TPL-002 (重複名) / ERR-WF-TPL-003 (instantiate 失敗 → 部分 rollback) |

```rust
// crates/flow-template-library/src/lib.rs (R7-I-4)
pub struct FlowTemplateLibrary {
    template_ops: Arc<TemplateOps>,
    activation: Arc<FlowActivationService>,
    seed_loader: Arc<BuiltinTemplateSeeder>,
    tenant_ctx: Arc<TenantContext>,
}

impl FlowTemplateLibrary {
    /// FR-W14.1: テンプレート一覧 (built-in + custom)
    pub async fn list_templates(&self, tenant_id: Uuid) -> Result<Vec<FlowTemplate>, WFError>;
    /// FR-W14.7: 作成 (custom のみ)
    pub async fn create_template(&self, input: CreateTemplateInput, actor: Uuid) -> Result<FlowTemplate, WFError>;
    pub async fn patch_template(&self, id: Uuid, patch: TemplatePatch, actor: Uuid) -> Result<FlowTemplate, WFError>;
    pub async fn delete_template(&self, id: Uuid, actor: Uuid) -> Result<(), WFError>;  // built-in は 403
    /// FR-W14.5: instantiate - built-in/custom ともにここを通る
    pub async fn instantiate(&self, template_id: Uuid, params: InstantiateParams, actor: Uuid) -> Result<Uuid /* new flow_id */, WFError>;
    /// FR-W14.6: 別保存 (任意の Flow → 新規 custom template)
    pub async fn duplicate(&self, source_flow_id: Uuid, name: String, actor: Uuid) -> Result<FlowTemplate, WFError>;
}

pub struct TemplateInstantiator { /* CLS-WF-040 */ }
impl TemplateInstantiator {
    pub async fn instantiate(&self, def: &TemplateDefinition, target_flow_id: Uuid, tenant_id: Uuid) -> Result<InstantiateResult, WFError>;
    /// template definition の全 node/edge/data_mapping を 1 TX でバルク INSERT
    /// is_placeholder=true / agent_id=NULL のまま生成 (per FR-W14.5)
}

pub struct ActivationGuard { /* CLS-WF-041 */ }
impl ActivationGuard {
    pub async fn pre_check(&self, flow_id: Uuid, tenant_id: Uuid) -> Result<(), WFError>;
    /// is_placeholder=true かつ agent_id IS NULL のノードが 1 つでもあれば → ERR-WF-ACT-001 +  拒否
    /// (per BD §6.3.1 + §8.3 — server 側強制)
}
```

> **【TBD: T-46】** FR-W14.2/14.3/14.4 (AAA / spec / superpowers 3 テンプレ) のノード/エッジ/データフロー詳細は上流 SRS/BD が本 worktree に未配置。`BuiltinTemplateSeeder::load()` 実装段階で seed JSON をハードコードする必要あり、PM 確認待ち (Round 7 IPA 自審までに決定)。

### §7.6 `MOD-WF-006` FlowActivationService (FR-W10.1〜W10.3)

| 項目 | 内容 |
|---|---|
| Module ID | `MOD-WF-006` |
| 名称 | FlowActivationService |
| 対応 BD | BD §7.4 (W10) |
| 責務 | enabled 切替 + SCD2 バージョン生成 + rollback |
| 入力 | (a) `PATCH /v1/collaboration/flows/{id}` `{enabled: true|false}` / (b) Flow 定義 PATCH 時に自動呼出 (version 作成) / (c) `POST /v1/collaboration/flows/{id}/rollback` `{target_version_no: int}` |
| 出力 | (a) updated `automation_flow` / (b) new `automation_flow_versions` row / (c) new version 作成 |
| 依存 | `MOD-WF-005` ActivationGuard / `MOD-WF-001` FlowOps |
| 外部 IF | API-WF-01 PATCH / (内部呼出 from FlowOps.patch / TemplateInstantiator.instantiate) |
| 使用データ | `TBL-WF-001` automation_flow (enabled/current_version_id) / `TBL-WF-002` automation_flow_versions (SCD2) |
| 状態 | ステートレス |
| Transaction | enabled 切替は 1 TX、version 作成は 1 TX、rollback は 2 TX (current 無効化 + 新 version 作成) |
| Error | ERR-WF-ACT-001 (placeholder 残存) / ERR-WF-ACT-002 (version 不存在) / ERR-WF-ACT-003 (conflict — 楽観 lock) |

```rust
// crates/workflow-engine/src/activation/mod.rs (R7-I-5)
pub struct FlowActivationService {
    flow_ops: Arc<FlowOps>,
    version: Arc<VersionManager>,
    activation_guard: Arc<ActivationGuard>,
    tenant_ctx: Arc<TenantContext>,
}

impl FlowActivationService {
    /// FR-W10.1: enabled 切替 + placeholder 検証
    pub async fn set_enabled(&self, flow_id: Uuid, enabled: bool, actor: Uuid) -> Result<AutomationFlow, WFError>;
    /// 内部: Flow 定義変更時 (PATCH nodes/edges 等) に呼出され SCD2 version を append
    pub async fn on_definition_change(&self, flow_id: Uuid, new_definition: FlowDefinition, actor: Uuid) -> Result<Uuid /* version_id */, WFError>;
    /// FR-W10.3: rollback
    pub async fn rollback_to(&self, flow_id: Uuid, target_version_no: i32, actor: Uuid) -> Result<Uuid /* new version_id */, WFError>;
    /// 内部: 楽観 lock conflict 時の振る舞い
    async fn resolve_version_conflict(&self, flow_id: Uuid, client_version: i32, actor: Uuid) -> Result<FlowDefinition, WFError>;
}

pub struct VersionManager { /* CLS-WF-023 */ }
impl VersionManager {
    pub async fn append_version(&self, flow_id: Uuid, definition: FlowDefinition, actor: Uuid, is_current: bool) -> Result<Uuid, WFError>;
    pub async fn list_versions(&self, flow_id: Uuid) -> Result<Vec<FlowVersion>, WFError>;
    pub async fn get_version(&self, flow_id: Uuid, version_no: i32) -> Result<FlowVersion, WFError>;
    pub async fn diff(&self, flow_id: Uuid, from: i32, to: i32) -> Result<FlowDiff, WFError>;
    /// 【TBD: T-27】 楽観 lock vs CRDT (SRS リスク #6 派生、A12 CRDT 选型未拍板)
}
```

### §7.7 `MOD-WF-007` TagBindingService (FR-W11.1〜W11.5)

| 項目 | 内容 |
|---|---|
| Module ID | `MOD-WF-007` |
| 名称 | TagBindingService |
| 対応 BD | BD §7.6 (W11) |
| 責務 | tag_binding_expr 監視 + canvas_event 命中判定 + 派生 WorkItem 作成 + BR-W-3 三分支処理 |
| 入力 | (a) Flow PATCH (tags/tag_binding_expr 変更) / (b) canvas_event 購読 / (c) `BR-W-3` 起動事件 |
| 出力 | (a) 既存 WorkItem 更新 (bound / detached / hard_delete) / (b) 新規 WorkItem 作成 |
| 依存 | 既存 `work-item` module CRUD / `MOD-WF-008` SprintLinkService (派生後の sprint 配置) |
| 外部 IF | API-WF-01 PATCH (tags 更新) / 既設 canvas-collab WS / 既設 work-item CRUD |
| 使用データ | `TBL-WF-001` automation_flow (tags, tag_binding_expr) / **既存 WorkItem** (`source_flow_id`/`tag_binding_status`/`user_edited_fields`) |
| 状態 | メモリ内 expression cache (Flow → compiled AST) |
| Transaction | BR-W-3 三分支処理は 1 TX で hard_delete / detached / bound 状態遷移 (per Event Sourcing + Saga) |
| Error | ERR-WF-TAG-001 (構文不正) / ERR-WF-TAG-002 (既存カード編集衝突) / ERR-WF-TAG-003 (硬删失敗 → detached 退避) |

```rust
// crates/work-item/src/tag_binding.rs (R7-I-6)
pub struct TagBindingService {
    work_item_ops: Arc<WorkItemOps>,
    flow_ops: Arc<FlowOps>,
    canvas_event_sub: Arc<CanvasEventSubscriber>,
    boolean_parser: Arc<BooleanExprParser>,
    deriver: Arc<WorkItemDeriver>,
    brw3: Arc<BrW3Handler>,
    sprint_link: Arc<SprintLinkService>,
    tenant_ctx: Arc<TenantContext>,
}

impl TagBindingService {
    /// FR-W11.1: 監視 ON (Flow.enabled & tag_binding_expr != null)
    pub async fn enable_monitoring(&self, flow_id: Uuid, actor: Uuid) -> Result<(), WFError>;
    /// canvas_event 受信 → 命中判定
    pub async fn on_canvas_event(&self, event: CanvasEvent) -> Result<Vec<Uuid> /* affected work_item ids */, WFError>;
    /// Flow.tags 変更 → BR-W-3
    pub async fn on_flow_tags_change(&self, flow_id: Uuid, old_tags: Vec<String>, new_tags: Vec<String>) -> Result<(), WFError>;
    /// FR-W11.2: 派生 WorkItem 作成 (BR-W-1: 既存時は skip)
    pub async fn derive_work_item(&self, flow_id: Uuid, hit_tags: Vec<String>) -> Result<Uuid /* work_item id */, WFError>;
}

pub struct BooleanExprParser { /* CLS-WF-032 */ }
impl BooleanExprParser {
    /// AND/OR/NOT 括弧付き (FR-W11.5)
    pub fn parse(&self, expr: &str) -> Result<ExprAst, WFError>;
    pub fn eval(&self, ast: &ExprAst, hit_tags: &HashSet<String>) -> bool;
}

pub struct WorkItemDeriver { /* CLS-WF-030 */ }
impl WorkItemDeriver {
    pub async fn derive(&self, flow: &AutomationFlow, hit_tags: Vec<String>) -> Result<WorkItem, WFError>;
    /// source_kind="workflow_tag_binding", source_flow_id=flow.id, sprint_id=null (Backlog 配置), tag_binding_status="bound"
}

pub struct BrW3Handler { /* CLS-WF-031 */ }
impl BrW3Handler {
    /// BR-W-3 三分支:
    ///   - Backlog 中 → 硬刪除
    ///   - 未開始 Sprint → 先移回 Backlog 再硬刪除
    ///   - 进行中 Sprint → tag_binding_status="detached" (保留卡)
    pub async fn apply(&self, flow: &AutomationFlow, removed_tags: Vec<String>) -> Result<Brw3Result, WFError>;
    /// 【TBD: T-28】 中途失敗 retry/補償
}
```

> **【TBD: T-04】** FR-W11 BR-W-3 派生 vs 人工カード同権 (W11.3) は設計通り「同権」だが、user_edited_fields が system 同期に優先 (W12.5)。両者同居時の優先順位は Round 3 §8 で明確化。
> **【TBD: T-18 / T-28】** Expression 評価が同一 Execution 内に限定される検証 (W11 中の `{{node.<id>.output.<field>}}` 越権防止) と BR-W-3 retry/補償。

### §7.8 `MOD-WF-008` SprintLinkService (FR-W12.1〜W12.5)

| 項目 | 内容 |
|---|---|
| Module ID | `MOD-WF-008` |
| 名称 | SprintLinkService |
| 対応 BD | BD §7.7 (W12) |
| 責務 | 派生 WorkItem の Sprint 配置 + 拖拽 + detached 管理 + `user_edited_fields` lock |
| 入力 | (a) 派生 WorkItem 新規作成時の Backlog 配置 / (b) `@dnd-kit` 拖拽イベント / (c) Sprint 状態変化 (active → done) による detached 解除 |
| 出力 | WorkItem 更新 (sprint_id / tag_binding_status) |
| 依存 | 既存 `work-item` / 既存 kanban (drag-drop 連動) / `MOD-WF-007` TagBindingService |
| 外部 IF | 既設 work-item CRUD / 既設 kanban WS |
| 使用データ | **既存 WorkItem** (`sprint_id` / `tag_binding_status` / `user_edited_fields`) / **既存 Sprint** |
| 状態 | ステートレス |
| Transaction | Sprint 移動 / detached 設定は 1 TX |
| Error | ERR-WF-SPR-001 (拖拽先 Sprint ロック) / ERR-WF-SPR-002 (派生 vs 人工カード編集衝突) / ERR-WF-SPR-003 (detached 取消不可 — system 侧) |

> **【TBD: T-05】** FR-W12.3 detached 状態取消に人間確認必要か否か (Lead 確認 vs system 自動 detached 解除)。実装は「v1 只読角标、取消不可」とし、PM 5 域 Lead から feedback 後 Round 7 で再評価。

### §7.9 `MOD-WF-009` StatusMappingService (FR-W13.1)

| 項目 | 内容 |
|---|---|
| Module ID | `MOD-WF-009` |
| 名称 | StatusMappingService |
| 対応 BD | BD §6.3.3 (W13.1) |
| 責務 | Execution status → WorkItem.status の任意 mapping 適用 + 非法 mapping の拒絶 |
| 入力 | (a) `PATCH /v1/collaboration/flows/{id}` `{status_mapping: {...}}` (Flow 設定) / (b) execution 状態遷移 (running → succeeded/failed) |
| 出力 | 派生 WorkItem.status 更新 (mapping が定義されている場合) |
| 依存 | 既存 `work-item` / `MOD-WF-007` TagBindingService |
| 外部 IF | API-WF-01 PATCH |
| 使用データ | `TBL-WF-001` automation_flow (`status_mapping` JSONB) / **既存 WorkItem** |
| 状態 | ステートレス |
| Transaction | 1 イベント = 1 TX |
| Error | ERR-WF-MAP-001 (mapping 引用不存在的 WorkItemStatus) |

> **FR-W13.2** は `MOD-WF-008` SprintLinkService の 制約性 FR として `sprint_id` 単一値フィールドで担保、`user_edited_fields` lock と組み合わせ。

### §7.10 (Module ID gap) — 上記 9 Module + `MOD-WF-010`/`011` で BD §6.4 の 5 module をカバー

### §7.11 `MOD-WF-010` ChatBarService (FR-W15.1 / W15.2 / W15.4 / W15.5)

| 項目 | 内容 |
|---|---|
| Module ID | `MOD-WF-010` |
| 名称 | ChatBarService |
| 対応 BD | BD §7.8 / §8 (W15.1/W15.2/W15.4/W15.5) |
| 責務 | 底部聊天常駐表示 + `actor_session_id` 管理 (L0 連携) + NL→mock 解析 + 草稿生成 + confirm/cancel + 草稿ノード = 手動ノード |
| 入力 | (a) UI からの chat_session OPEN / (b) 自然言語メッセージ送信 (API-WF-08) / (c) 草稿 confirm/cancel 操作 |
| 出力 | (a) `chat_session` record / (b) `parsed_flow_draft` JSON / (c) 草稿 → 通常の Flow 編集画面に遷移 (draft  = 正式保存前の状態) |
| 依存 | `MOD-WF-011` LangGraphRouter (L0 セッション) / 既設 `/api/tmo/*` (ADR-0046, 8 端点复用) / `MOD-WF-001` FlowEditor (草稿 confirm 時の正式保存) |
| 外部 IF | API-WF-08 (`/v1/collaboration/chat-sessions/{id}/messages`) / 既設 `/api/tmo/*` (L0 セッション管理) / 既設 WS (草稿 node 同期) |
| 使用データ | `TBL-WF-008` chat_session |
| 状態 | UI 常驻 (`actor_session_id` を store に保持) |
| Transaction | メッセージ送信は 1 TX (chat_session INSERT); 草稿 confirm は別途 `MOD-WF-001` のトランザクション |
| Error | ERR-WF-CHT-001 (session 期限切れ) / ERR-WF-CHT-002 (mock 解析未命中 → `matched: false`, 非エラー) |

```rust
// crates/chat-bar/src/lib.rs (R7-I-7)
pub struct ChatBarService {
    langgraph: Arc<LangGraphRouter>,
    draft_parser: Arc<DraftParser>,
    flow_editor: Arc<FlowEditor>,
    session_store: Arc<ActorSessionStore>,  // actor_session_id ↔ L0 TopAgentState session
    tenant_ctx: Arc<TenantContext>,
}

impl ChatBarService {
    /// FR-W15.1: chat_session OPEN (actor_session_id を初期化、L0 接続)
    pub async fn open_session(&self, actor: Uuid) -> Result<Uuid /* chat_session_id */, WFError>;
    /// FR-W15.2: メッセージ送信 + mock 解析 → 草稿
    pub async fn send_message(&self, chat_session_id: Uuid, message: String) -> Result<ChatMessageResult, WFError>;
    /// 草稿 confirm → 正式 Flow として保存
    pub async fn confirm_draft(&self, chat_session_id: Uuid, draft_id: Uuid, actor: Uuid) -> Result<Uuid /* new flow_id */, WFError>;
    /// 草稿 cancel → chat_session.applied = false (历史保存)
    pub async fn cancel_draft(&self, chat_session_id: Uuid, draft_id: Uuid, actor: Uuid) -> Result<(), WFError>;
    /// FR-W15.4: Execution 起動時に origin_chat_session_id を記録
    pub async fn record_execution_origin(&self, execution_id: Uuid, chat_session_id: Uuid) -> Result<(), WFError>;
}

pub struct DraftParser { /* CLS-WF-050 */ }
impl DraftParser {
    /// mock 規則化解析 (per BD §5.2 + SRS 已知缺口) — 关键词 + テンプレート マッチ
    pub async fn parse(&self, message: &str) -> Result<ParsedFlowDraft, WFError>;
    /// 【TBD: T-14】 mock 規則カバー範囲 (W15.2) — Round 3 で表化予定
}
```

> **【TBD: T-30 / T-31】** SCR-WF-03 チャット栏 がページ跨ぎで持続浮游するか / 入力文字数上限未定。実装では default 入力上限 4KB + ページ跨ぎ持続 (per Round 2 暂定)。
> **【TBD: T-08】** mock 解析 → 真实 LLM/NLU 接入時間点未定 (per BD T-08 / T-32)。

### §7.12 `MOD-WF-011` LangGraphRouter (FR-W15.3)

| 項目 | 内容 |
|---|---|
| Module ID | `MOD-WF-011` |
| 名称 | LangGraphRouter |
| 対応 BD | BD §6.3.1 / §8 (W15.3) |
| 責務 | `routing_mode="dynamic_agent"` 条件ノード → L0 TopAgentState 動的ルーティング。决策を `execution_step.routing_decision` に記録 |
| 入力 | (a) `MOD-WF-004` RuleExecutor からの condition node 評価依頼 (dynamic_agent mode) / (b) L0 から返回される branch 決定 |
| 出力 | (a) branch 決定 (next node id) + decision log JSONB |
| 依存 | 既設 `/api/tmo/*` (ADR-0046) L0 TopAgentState / 既設 `TopAgentState` / `SubAgentState` (ADR-0046) |
| 外部 IF | 既設 `/api/tmo/*` 8 端点 (ADR-0046 参照) |
| 使用データ | `TBL-WF-006` execution_step (`routing_decision` JSONB) |
| 状態 | L0 セッション = per Execution 短命 |
| Transaction | 决策記録は step append-only と一体 (TX は §7.4 RuleExecutor.persist_step と共用) |
| Error | ERR-WF-LGR-001 (L0 不可達 → 降级戦略) / ERR-WF-LGR-002 (decision timeout) |

```rust
// crates/chat-bar/src/langgraph_router.rs (R7-I-8)
pub struct LangGraphRouter {
    tmo_client: Arc<TmoClient>,  // /api/tmo/* ラッパー
    decision_recorder: Arc<DecisionRecorder>,
    tenant_ctx: Arc<TenantContext>,
}

impl LangGraphRouter {
    /// FR-W15.3: dynamic_agent mode の condition 評価を L0 に依頼
    pub async fn request_decision(&self, ctx: &ExecutionContext, condition_node_id: Uuid) -> Result<BranchDecision, WFError>;
    /// 内部: L0 不可達 / 超時時の降级戦略 (per BD T-29)
    async fn fallback_to_static_cel(&self, ctx: &ExecutionContext, node: &FlowNode) -> Result<BranchDecision, WFError>;
    /// 内部: decision を execution_step に JSONB 書込
    async fn record_decision(&self, step_id: Uuid, decision: &BranchDecision) -> Result<(), WFError>;
}
```

> **【TBD: T-09 / T-29 / T-32】** W15.3 决策可復現性 / L0 不可達降级 / `routing_decision` JSONB 粒度 — 全 Round 3-7 で TPM と共同决定。実装は default 降级 → static_cel default 分岐 (per BD §6.3.3) を選択肢として保持。
> **【上位設計確認事項】** ADR-0046 実在パス / `TopAgentState`/`SubAgentState` 詳細 / `/api/tmo/*` 8 端点 接口仕樣 — Round 7 までに文書横断で確認要 (BD preamble の ADR 参照パスは現在 worktree 未配置)。

### §7.13 / §7.14 (W11/W12 詳細・W14 既定テンプレ) — 補足

W11/W12 の BR-W-3 状態機械と WorkItem detached 遷移は `MOD-WF-007`/`MOD-WF-008` 配下の Class で担務。Round 3 §8 で状態機械 Mermaid として图示予定。

W14 既定 3 テンプレ (AAA / spec / superpowers) は `MOD-WF-005` の `BuiltinTemplateSeeder::load()` で seed 実装。**T-46 (上流 SRS/BD 不在)** 解决まで seed JSON は dummy とし、PM 决定後に本実装。

### §7.15 ID 体系リファレンス (Round 2 で確定)

| 種別 | 範囲 | 命名規則 | 例 |
|---|---|---|---|
| Module | `MOD-WF-NNN` | 3 桁連番 (BD §6.4 と 1:1 対応) | `MOD-WF-001`〜`MOD-WF-011` |
| Class | `CLS-WF-NNN` | Module 配下で 連番 | `CLS-WF-001` FlowNodeService / `CLS-WF-014` CelConditionNode / ... / `CLS-WF-041` ActivationGuard / `CLS-WF-050` DraftParser |
| API | `API-WF-NN` | 2 桁連番 (BD §5.1 8 端点) | `API-WF-01`〜`API-WF-08` |
| Table | `TBL-WF-NNN` | 3 桁連番 (BD §4.2 出現順) | `TBL-WF-001`=`automation_flow` 〜 `TBL-WF-008`=`chat_session` |
| Event | `EVT-WF-NNN` | 3 桁連番 (W 別 連番) | `EVT-WF-001` `flow_node.created` 〜 `EVT-WF-040` `execution.chat_origin_recorded` |
| Error | `ERR-WF-XXX-NNN` | カテゴリ 3 文字 + 3 桁連番 | `ERR-WF-VAL-001` (入力検証) / `ERR-WF-AUTHZ-001` (権限) / `ERR-WF-EXE-001` (実行) / `ERR-WF-ACT-001` (活性化) / `ERR-WF-TPL-001` (テンプレ) / `ERR-WF-TAG-001` (タグ) / `ERR-WF-SPR-001` (Sprint) / `ERR-WF-MAP-001` (mapping) / `ERR-WF-CHT-001` (chat) / `ERR-WF-LGR-001` (LangGraph) / `ERR-WF-TRG-001` (trigger) |
| Test観点 | `TST-WF-NNN` | Round 7 で 割り当て予定 | §12 で 一括付与 |

> **Round 2 終了時点の整合確認**:
> - 5 module (BD §6.4) を起点に 11 Module / 30 class を割り当て、全 54 FR の依存関係を明示
> - §6.2 FR 追跡行列と §7 Class シグネチャの ID 1:1 整合
> - TBL-WF ID は BD §4.2 出現順に 1:1 マッピング確定 (001〜008)
> - **【上位設計確認事項】** ADR-0046 実在パス確認待ち (本 worktree 未配置) — §7.11 / §7.12 実装着手前に必须
> - **【TBD: T-46】** W14 既定 3 テンプレの上流 SRS/BD 不在 → seed JSON は暫定
>
> **未着手項目(Round 2 の範囲外)**:
> - §8 処理詳細・状態遷移(Round 3)
> - §9 API 内部処理設計(Round 4)
> - §10 データ/SQL/CRUD(Round 5)
> - §11 セキュリティ/ログ/監査(Round 6)
> - §12 NFR/テスト観点 + §13 TBD 追跡 + §14 IPA 自審(Round 7)

