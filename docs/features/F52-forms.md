# F52 表单（公开表单与提交）

> 状态摘要: 表单功能目前只有 `crates/domain-form` 的实体、字段校验与 slug 领域代码（11 个单元测试，无生产调用方）；表单持久化、REST 路由、管理页与公开填写页均未实现，页面与 UI 状态为 ⬜。本功能为新增模块，尚无页面。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

- 项目成员定义表单：字段、条件逻辑、字段校验、访问控制、提交动作。
- 表单发布后获得公开 URL `{base}/forms/{slug}`（`crates/domain-form/src/lib.rs:307-310`）；匿名或受限用户按访问控制提交。
- 提交可触发创建工作项、触发自动化、发送邮件、调用 Webhook（类型定义见 `lib.rs:203-224`，执行器未实现）。
- 提交须经访问控制、速率限制与 Workflow Guard 校验（依据 `docs/specs/domain-form-spec.md` 的 INV-FORM-02、INV-FORM-04 与 RISK-FORM-01、RISK-FORM-03）。

### 1.2 范围

范围内：

- 表单定义与字段（类型、必填、默认值、选项、字段校验、条件规则）。
- 公开 URL slug、匿名提交、邮箱白名单、访问 token、每小时速率限制。
- 提交记录与提交动作的触发。
- 表单管理页 `/forms` 与公开填写页 `/forms/[slug]`。

范围外：

- WorkItem 聚合与工单生命周期，属 `domain-work-item`；表单只经端口创建工单（spec §1）。
- 通知投递，属 `domain-notification`（spec §1）。
- Workflow Guard 的实现，属 `domain-workflow`（REQ-WF-003，`docs/basic-design.md:628`）。
- Canvas 内的表单小部件：`docs/requirements/SRS-CANVAS-001.md:145` 标为 P3+，`docs/requirements/SRS-CANVAS-AGENT-001.md:136` 归数据与工作项 App 承担。

## 2. 用户与场景

| 角色 | 主要场景 |
|---|---|
| 表单设计者（项目成员，具备表单管理权限） | 创建表单；添加字段并设置校验与条件；设置访问控制与提交动作；预览；发布；查看提交 |
| 表单管理员 | 发布与归档表单；查看提交记录；管理白名单与 token |
| 提交者（匿名，或限定邮箱/token） | 打开公开链接；填写按条件显示的字段；提交并看到确认；收到字段校验错误 |
| 系统（提交后动作） | 创建工作项；触发自动化；发送邮件；调用 Webhook；记录审计 |

核心故事：

- S1：设计者发布"Bug 上报"表单，生成公开链接，匿名用户提交后自动创建工作项。
- S2：设计者设置"选择某选项时才显示字段 Y，且 Y 必填"，服务端强制校验。
- S3：管理员限定仅白名单邮箱可提交，并在超出每小时次数时拒绝。

## 3. 需求

优先级由本文档依据依赖关系赋值（源 spec 未标优先级）。状态口径见 §5 首段。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-52-01 | 表单定义包含名称、描述、字段列表、提交动作、访问控制与速率限制 | P0 | 🟡 | 字段 `crates/domain-form/src/lib.rs:232-253`；构造 `lib.rs:281-295`；无持久化与编辑入口 |
| FR-52-02 | 字段类型：spec 列 12 种，代码实际 18 种（含 UserPicker、MultiUser、Attachment、RichText、Cascader） | P0 | ✅ | `lib.rs:20-57`；`all()` `lib.rs:61-82`；`name()` `lib.rs:85-106`；测试 `lib.rs:480-489` |
| FR-52-03 | 字段 key 在表单内唯一 | P0 | ✅ | `lib.rs:297-305`；测试 `lib.rs:517-532` |
| FR-52-04 | 必填校验：缺失或为 null 时拒绝 | P0 | 🟡 | `lib.rs:385-395`；空字符串 `""` 被视为已填（仅判断 `!v.is_null()`，`lib.rs:390`） |
| FR-52-05 | 字段校验：长度（min_length/max_length）与 pattern 正则 | P0 | 🟡 | 已实现 `lib.rs:432-450`，测试 `lib.rs:600-635`；数值 min/max（定义于 `lib.rs:147-149`）与按字段类型的格式校验未执行 |
| FR-52-06 | 条件规则 show/hide/require/optional，比较运算符集合 | P1 | 🟡 | 模型 `lib.rs:127`、`lib.rs:158-167`、`lib.rs:169-201`；`validate_submission` 不求值（`lib.rs:379-421`） |
| FR-52-07 | 提交动作：创建工作项、触发自动化、发邮件、调 Webhook | P1 | 🟡 | 类型 `lib.rs:203-224`；无执行器；`Cargo.toml:18-24` 无 work-item、notification、webhook、automation 依赖 |
| FR-52-08 | 公开 slug：由名称生成并拼接 `{base}/forms/{slug}`，slug 全局唯一 | P0 | 🟡 | 生成与拼接已实现 `lib.rs:308-310`、`lib.rs:423-429`，测试 `lib.rs:562-577`；唯一性未检查，`SlugConflict` 定义于 `lib.rs:362` 无调用方 |
| FR-52-09 | 公开表单匿名提交，不要求登录 | P0 | 🟡 | 领域层：`public` 为真时跳过访问检查（`lib.rs:408`）；无 REST 端点（`crates/star-api-rest/src/lib.rs:135-138` 未挂载 forms） |
| FR-52-10 | 邮箱白名单 | P1 | 🟡 | 检查 `lib.rs:412-418`；仅在提交携带 `submitter_email` 时执行 |
| FR-52-11 | 访问 token 校验 | P1 | 🟡 | 字段 `lib.rs:264-265` 存在但从不比较；`lib.rs:409-411` 仅检查 email 是否存在 |
| FR-52-12 | 每小时速率限制（默认 100，超限拒绝） | P1 | 🟡 | 字段 `lib.rs:247-248`，默认值 `lib.rs:291`，错误 `lib.rs:351-353`；无执行逻辑 |
| FR-52-13 | 发布后 schema 锁定，变更须新建 version（INV-FORM-01） | P1 | ⬜ | `Form` 无 version 与 status 字段（`lib.rs:232-253`） |
| FR-52-14 | 提交记录保存字段值、提交时间、提交者邮箱与 IP | P0 | 🟡 | 类型 `lib.rs:315-328`；无存储实现 |
| FR-52-15 | 表单与提交归属租户和项目（spec 的 tenant_id、project_id） | P0 | ⬜ | `lib.rs:232-253`、`lib.rs:315-328` 无 tenant_id、project_id 字段；crate 内无此类引用 |
| FR-52-16 | 表单 CRUD、发布、归档的命令端口与查询端口（spec §4） | P0 | ⬜ | crate 无 `pub trait`（`docs/reports/DDD-DOMAIN-MODEL-COMPLETENESS-AUDIT.md:104`） |
| FR-52-17 | 提交触发经 Workflow Guard 校验，不可绕过（INV-FORM-04，REQ-WF-003） | P1 | ⬜ | `docs/ac-test-matrix.csv:8` 标为 gap；代码无对应实现 |
| FR-52-18 | 表单操作与提交写入审计（spec §5 AuditRecorder） | P1 | ⬜ | crate 无 domain-audit 依赖（`crates/domain-form/Cargo.toml:18-24`） |
| FR-52-19 | 管理页 `/forms`：列表、创建、字段编辑、条件、访问控制、预览、发布、归档、查看提交 | P0 | ⬜ | `frontend/src/app/(app)/` 下无 forms 目录；`frontend/src/lib/nav/registry.ts` 无 forms 模块 |
| FR-52-20 | 公开填写页 `/forms/[slug]`：渲染、校验、提交、确认 | P0 | ⬜ | 无页面文件；根布局对所有路由挂载导航与引导（`frontend/src/app/layout.tsx:77-79`、`:99`、`:107`） |
| FR-52-21 | 导航入口：侧栏分组与 Branch 标签分组 | P2 | ⬜ | `registry.ts:684-714` 的 `NAV_GROUPS` 无 tasks 键；`registry.ts:755-762` 的 tasks 标签不含 forms |
| FR-52-22 | 条件规则循环引用检测（INV-FORM-03） | P2 | ⬜ | 条件仅为单字段规则（`lib.rs:158-167`），无图结构与检测代码 |
| FR-52-23 | 公开提交的限流与 captcha（RISK-FORM-01） | P1 | ⬜ | `crates/star-api-rest/src/middleware/rate_limit.rs:21` 为 pass-through stub；无 captcha 代码 |

## 4. 设计

### 4.1 概念模型

- **Form**（聚合根）：表单 ID、名称、描述、字段列表、提交动作、公开 slug、访问控制、速率限制、创建与更新时间（`lib.rs:230-253`）。
- **FormField**：字段 key、标签、字段类型、必填、默认值、选项、字段校验、条件规则（`lib.rs:109-128`）。
- **FieldValidation**：min_length、max_length、min、max、pattern、自定义提示（`lib.rs:139-154`）。
- **ConditionalRule**：动作（Show、Hide、Require、Optional）、被控字段 key、比较运算符、比较值（`lib.rs:156-201`）。
- **AccessControl**：public、邮箱白名单、require_token、token（`lib.rs:255-277`）。
- **SubmitAction**：动作类型与配置 JSON（`lib.rs:203-210`）。
- **FormSubmission**：提交 ID、表单 ID、`(key, value)` 列表、提交时间、提交者邮箱、提交者 IP（`lib.rs:313-328`）。
- **FormService**：无状态服务，提供提交校验与 slug 生成（`lib.rs:369-430`）。

与 spec 的差异：spec 的 `FormDefinition` 含 tenant_id、project_id、version、triggers、title，代码中的 `Form` 不含这些字段，名称为 name，触发器为 `submit_actions`。本节按代码描述当前形态；spec 的目标字段在 §4.2 中标为 ⬜。

### 4.2 数据模型

| 实体.字段 | 代码位置 | spec 对应 | 状态 |
|---|---|---|---|
| Form.id | `lib.rs:234` | form_id | ✅ |
| Form.name | `lib.rs:236` | title | ✅ |
| Form.description | `lib.rs:238` | — | ✅ |
| Form.fields | `lib.rs:240` | fields[] | ✅ |
| Form.submit_actions | `lib.rs:242` | triggers[] | 🟡（类型存在，无执行） |
| Form.public_url_slug | `lib.rs:244` | public_url_slug | 🟡（字段存在，唯一性未检查） |
| Form.access_control | `lib.rs:246` | — | ✅ |
| Form.rate_limit_per_hour | `lib.rs:248` | — | 🟡（无执行） |
| Form.created_at、updated_at | `lib.rs:250-252` | — | ✅ |
| Form.tenant_id、project_id | 无 | tenant_id、project_id | ⬜ |
| Form.version、status | 无 | version（INV-FORM-01） | ⬜ |
| FormField.conditional | `lib.rs:127` | FormCondition | 🟡（无求值） |
| FieldValidation.min、max | `lib.rs:147-149` | — | 🟡（无执行） |
| AccessControl.token | `lib.rs:264-265` | — | 🟡（不比较） |
| FormSubmission.id、form_id | `lib.rs:317`、`lib.rs:319` | submission_id、form_id | ✅ |
| FormSubmission.values | `lib.rs:321` | field_values{} | 🟡（列表形式） |
| FormSubmission.submitted_at | `lib.rs:323` | submitted_at | ✅ |
| FormSubmission.submitter_email、submitter_ip | `lib.rs:325`、`lib.rs:327` | submitted_by | 🟡（字段名与 spec 不同） |
| FormSubmission.tenant_id | 无 | tenant_id | ⬜ |

持久化：`db/migrations/` 下无 form 相关表（`db/` 与 `docs/data-design/` 检索 `form_definition`、`form_submission`、`FormDefinition` 无命中）。`docs/data-design/p3-d-classification-w-t-m.md:39` 与 `:568-583` 列出 domain-form 的 14 张 T 类表，但 pin 处无对应迁移，本文档不采纳该数量（见 §8）。

### 4.3 接口

**(a) 现有 Rust 公开 API（crate 内）**

| 函数 | 位置 | 用途 | 状态 |
|---|---|---|---|
| `Form::new(name, public_url_slug)` | `lib.rs:281` | 构造表单，默认非公开、速率 100 | ✅ |
| `Form::add_field(&mut self, FormField)` | `lib.rs:298` | 追加字段，key 重复返回 `DuplicateFieldKey` | ✅ |
| `Form::public_url(&self, base)` | `lib.rs:308` | 生成 `{base}/forms/{slug}` | ✅ |
| `FieldType::all()` / `name()` | `lib.rs:61`、`lib.rs:85` | 枚举与展示名 | ✅ |
| `FormService::validate_submission(&Form, &FormSubmission)` | `lib.rs:379` | 必填、字段校验、访问控制 | 🟡（仅测试调用） |
| `FormService::generate_slug(&str)` | `lib.rs:424` | 名称转 slug | ✅ |

**(b) 端口（spec §4，均为 ⬜）**

- `FormDefinitionCommandPort`：create、update、publish（锁定 schema）、archive。
- `FormDefinitionQueryPort`：get、list-by-project、get-by-public-slug。
- `FormSubmissionPort`：submit（含条件执行与触发动作）、list-by-form。

**(c) REST 端点（提议，挂载于 `crates/star-api-rest/src/lib.rs:135-138` 的 `/api/v1` 之下，全部 ⬜）**

| 方法 | 路径 | 用途 | 鉴权 |
|---|---|---|---|
| GET | `/api/v1/forms?project_id=` | 列出项目表单 | 项目成员 |
| POST | `/api/v1/forms` | 创建 draft | 设计者 |
| GET | `/api/v1/forms/{id}` | 读取定义 | 项目成员 |
| PATCH | `/api/v1/forms/{id}` | 修改 draft；已发布后拒绝 | 设计者 |
| POST | `/api/v1/forms/{id}/publish` | 锁定 schema，确认 slug | 设计者 |
| POST | `/api/v1/forms/{id}/archive` | 停止接收提交 | 管理员 |
| GET | `/api/v1/forms/{id}/submissions` | 查看提交记录 | 管理员 |
| GET | `/api/v1/public/forms/{slug}` | 匿名读取已发布定义，不含 tenant、project、token、白名单 | 匿名 |
| POST | `/api/v1/public/forms/{slug}/submissions` | 匿名提交 | 匿名，经限流与 captcha |

公开端点使用 `/api/v1/public/` 前缀，与前端公开页 `/forms/[slug]` 区分，两者都是提议。

**(d) 中间件**

- 鉴权：`crates/star-api-rest/src/middleware/auth.rs:23-31` 为 pass-through，中间件层不拒绝匿名请求，`/api/v1` 路由当前无鉴权拦截。
- 限流：`crates/star-api-rest/src/middleware/rate_limit.rs:21` 为 pass-through。

### 4.4 状态机与关键流程

**表单生命周期（提议，⬜）**

| 状态 | 进入条件 | 允许操作 | 现状 |
|---|---|---|---|
| draft | 创建 | 修改字段、条件、访问控制、提交动作；预览 | ⬜（Form 无 status 字段，`lib.rs:232-253`） |
| published | 发布，slug 确认 | 接收提交；查看提交；只读定义 | ⬜ |
| archived | 归档 | 只读；拒绝提交 | ⬜ |

**提交流程（当前代码覆盖的步骤与提议步骤）**

| 序号 | 步骤 | 现状 | 证据 |
|---|---|---|---|
| 1 | 接收请求，解析表单与 slug | ⬜ | 无端点 |
| 2 | 速率检查（每小时计数） | ⬜ | `lib.rs:247-248` 无执行 |
| 3 | 必填检查 | 🟡 | `lib.rs:385-395` |
| 4 | 字段校验（长度、pattern） | 🟡 | `lib.rs:397-406`、`lib.rs:432-450` |
| 5 | 访问控制（public、白名单、token） | 🟡 | `lib.rs:407-419` |
| 6 | 条件规则求值 | ⬜ | `lib.rs:379-421` 不含求值 |
| 7 | 持久化提交记录 | ⬜ | 无存储 |
| 8 | Workflow Guard 校验 | ⬜ | REQ-WF-003 为 gap（`docs/ac-test-matrix.csv:8`） |
| 9 | 执行提交动作（工单、自动化、邮件、Webhook） | ⬜ | 类型存在（`lib.rs:203-224`），无执行器 |
| 10 | 返回确认 | ⬜ | 无端点 |

当前校验顺序为必填、字段校验、访问控制（`lib.rs:384-419`）。访问控制放在最后，匿名提交者会先收到字段级错误，字段定义因此会暴露给未授权者。

### 4.5 权限与安全

| 项 | 当前行为 | 证据 | 状态 |
|---|---|---|---|
| 公开标志 | `public` 为真时跳过所有访问检查，包括白名单与 token | `lib.rs:408` | 🟡 |
| 白名单绕过 | 提交不携带 `submitter_email` 时白名单不生效，匿名提交可通过 | `lib.rs:412-418` | 🟡（缺陷） |
| token 校验 | `require_token` 仅检查 email 是否存在；`token` 字段不比较，错误或缺失的 token 均可通过 | `lib.rs:409-411`、`lib.rs:264-265` | 🟡（缺陷） |
| 必填空串 | `""` 满足必填 | `lib.rs:388-390` | 🟡（缺陷） |
| 数值与格式 | 非字符串值被转为 `""`，min、max 与字段类型格式均不校验 | `lib.rs:433` | 🟡（缺陷） |
| 非法正则 | 非法 pattern 被静默跳过，校验通过 | `lib.rs:445-449`；测试 `lib.rs:632-634` | 🟡（缺陷） |
| 表单管理权限 | 无 form 资源；RBAC 只定义 5 个业务域 | `crates/star-api-rest/src/rbac.rs:26-50` | ⬜ |
| 认证 | pass-through，中间件层不拒绝匿名请求 | `middleware/auth.rs:23-31` | 🟡 |
| 速率限制 | pass-through，公开提交无防护 | `middleware/rate_limit.rs:21` | 🟡 |
| captcha | 无 | — | ⬜ |
| 个人信息 | 提交者邮箱与 IP 为明文字段，无保留期与脱敏规则 | `lib.rs:325-327` | ⬜ |
| 审计 | 无 | — | ⬜ |
| slug 唯一与保留 | 生成结果可能为空（如 `"!!!"` 生成 `""`），无校验 | `lib.rs:424-429` | 🟡（缺陷） |

### 4.6 依赖关系

- **当前依赖**：仅 serde、serde_json、thiserror、uuid、chrono、regex（`crates/domain-form/Cargo.toml:18-24`）；无 star-* 或 domain-* 依赖。
- **当前消费方**：无。工作区 `Cargo.toml:42` 只注册了成员，其他 `Cargo.toml` 不依赖 `domain-form`。`crates/star-dispatcher/src/sa_real_impls.rs:253` 仅在字符串列表中出现该名称，不是依赖。
- **spec 计划的依赖**（均为 ⬜）：work-item（Customer-Supplier 端口，创建工单）、identity（UserId 共享内核）、audit（AuditRecorder，追加写）、notification（异步发送）、workflow（Guard），以及 star-webhook 出站调用。
- **已存在的集成目标**：工单创建 `crates/star-api-rest/src/group_api/work_items.rs:274`；通知路由 `crates/star-api-rest/src/lib.rs:131`；Webhook 管理路由文件 `crates/star-api-rest/src/routes/webhooks.rs:2-6`。三者均未与表单连接。
- **基本设计**：`docs/basic-design.md:407` 将 domain-form 列为"表单（WorkItem 表单 / 表单模板）"。

### 4.7 页面设计

#### 4.7.1 /forms

**文件**：`frontend/src/app/(app)/forms/page.tsx`（不存在，⬜）。builder 为该页的右侧抽屉，由 `?id=` 查询参数打开，不占用 `/forms/...` 路径，避免与公开 slug 冲突。

**布局**：

| 区域 | 提供方 | 位置 | 现状 |
|---|---|---|---|
| 顶栏 | AppHeader（由 AppShell 挂载） | `frontend/src/components/AppShell.tsx:20`、`:48-50` | ✅ 复用 |
| 侧栏 | Sidebar（由根布局挂载） | `frontend/src/app/layout.tsx:77-79`；分组渲染 `frontend/src/components/Sidebar.tsx:200` | ✅ 复用，分组 ⬜ |
| 主区 | main（由 AppShell 挂载） | `AppShell.tsx:61-73`；`/forms` 不在 FULL_HEIGHT、WIDE 列表（`frontend/src/app/(app)/layout.tsx:28-41`），采用默认 `max-w-[1440px]` 容器（`AppShell.tsx:69`） | ✅ 复用 |
| 右侧栏 | RightChatRail | `AppShell.tsx:77`；定义 `frontend/src/components/chat/RightChatRail.tsx:1-20` | ✅ 复用 |
| 底栏 | MobileBottomNav（仅 <768px） | `frontend/src/app/layout.tsx:99` | ✅ 复用，桌面端无底栏 |

**组件树（提议，⬜）**

- FormsPage
  - FormsToolbar（搜索、新建）
  - FormsListTable（名称、slug、状态、提交数、更新时间）
  - FormBuilderDrawer（`?id=`）
    - FieldPalette（18 种字段类型，源 `lib.rs:20-57`）
    - FieldEditor（标签、必填、默认值、选项、字段校验）
    - ConditionEditor（ConditionalRule）
    - AccessControlPanel（public、白名单、token）
    - SubmitActionEditor（SubmitActionType）
    - PublishBar（校验、发布、归档）
    - FormPreview
    - SubmissionsTab（提交记录）

**状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 数据返回且有表单 | 列表与可打开的抽屉 | ⬜ |
| 加载 | 请求进行中 | 列表骨架屏 | ⬜ |
| 空 | 项目无表单 | 空状态与"新建表单"按钮 | ⬜ |
| 错误 | API 失败或校验失败 | 错误条与重试 | ⬜ |
| 未授权 | 无表单管理权限或 401 | 403 提示，不显示表单内容 | ⬜（认证为 pass-through，`middleware/auth.rs:23-31`） |
| 未接入 | REST 端点未挂载（当前状态） | 显示"未接入"，不以示例表单冒充（`AGENTS.md:22` 规则） | ⬜ |
| 禁用 | 表单已发布（字段不可改）或已归档（只读） | 字段与发布按钮禁用，并说明原因 | ⬜ |

**交互**

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 新建表单 | 项目设计者 | 创建 draft，打开抽屉 | POST `/api/v1/forms` | ⬜ |
| 添加字段 | draft | 追加字段，key 唯一 | 客户端镜像 `Form::add_field`（`lib.rs:298`），服务端校验 ⬜ | ⬜ |
| 设置条件 | draft | 保存条件规则 | 需要求值器（当前无，`lib.rs:379-421`） | ⬜ |
| 设置访问控制 | draft | 更新 public、白名单、token | PATCH `/api/v1/forms/{id}` | ⬜ |
| 发布 | 必填校验通过，slug 唯一 | 锁定 schema，生成公开链接 | POST `/api/v1/forms/{id}/publish` | ⬜ |
| 归档 | published | 停止接收提交 | POST `/api/v1/forms/{id}/archive` | ⬜ |
| 复制公开链接 | published | 复制 `{base}/forms/{slug}` | `Form::public_url`（`lib.rs:308`） | ⬜ |
| 查看提交 | 管理员 | 列出提交记录 | GET `/api/v1/forms/{id}/submissions` | ⬜ |

**数据**：当前无数据源（REST 未挂载，MSW handlers 未注册 forms，`frontend/src/mocks/handlers/index.ts:1-40`）。数据契约取自 crate：Form（`lib.rs:232-253`）、FormField（`lib.rs:110-128`）、AccessControl（`lib.rs:257-266`）、FormSubmission（`lib.rs:315-328`）。字段均经 serde 序列化（`lib.rs` 中相应 derive）。

**与文档设计的差距**

- FR-52-19：页面与组件均未实现。
- FR-52-13：发布后锁定 schema 未实现，编辑态未区分。
- FR-52-15：列表与抽屉无租户与项目过滤。
- FR-52-16：无查询与命令端口，页面无数据来源。
- FR-52-21：侧栏无 forms 入口；`docs/frontend/design/ui-3pane-arch.md` §1.2.1 路由映射表无 `/forms` 行，`ui-3pane-arch.md:62` 仅将 Forms 列为主视图族之一。
- AGENTS.md:22 的未接入规则：当前页面不存在，接入前须显示未接入状态。

#### 4.7.2 /forms/[slug]

**文件**：`frontend/src/app/forms/[slug]/page.tsx`（不存在，⬜）。该路由位于 `(app)` 之外，并需要独立布局（见下文）。

**布局**：

| 区域 | 目标 | 现状 |
|---|---|---|
| 顶栏、侧栏、右侧栏 | 不显示 | 根布局对所有路由挂载 `Sidebar`（`frontend/src/app/layout.tsx:77-79`），无按路径的条件渲染，⬜ |
| 底栏 | 不显示 | 根布局挂载 `MobileBottomNav`（`layout.tsx:99`），⬜ |
| 引导 | 不显示 | 根布局挂载 `OnboardingGuard`（`layout.tsx:107`）；它在存在密钥时弹出引导（`frontend/src/components/OnboardingGuard.tsx:75-87`），并请求 `/api/cli-profiles`（`OnboardingGuard.tsx:58`）；公开页同样会挂载并运行该组件，⬜ |
| 主区 | 表单与确认 | 无，⬜ |
| Toast | 全局 | `layout.tsx:113` 为全局挂载，可保留 |

需要改动根布局，或将导航与引导挂入 `(app)` 布局，使公开路由只渲染表单内容。这属于 §6 步骤 8 的决策项。

**组件树（提议，⬜）**

- PublicFormPage
  - FormHeader（标题、描述）
  - FormRenderer
    - FieldInput（按 FieldType 渲染，18 种）
    - ConditionalVisibility（条件显示与必填）
  - SubmitBar（captcha 占位，⬜）
  - ConfirmPanel（提交确认）

**状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 表单已发布，可提交 | 渲染字段，提交按钮可用 | ⬜ |
| 加载 | 获取定义中 | 骨架屏 | ⬜ |
| 空 | 表单无字段 | 说明文字，不提供提交 | ⬜ |
| 错误 | 字段校验失败 | 字段级错误（对应 `FormError::Validation`，`lib.rs:340-347`） | ⬜ |
| 未授权 | 白名单或 token 不通过 | "需要 token" 或"邮箱不在白名单"（`FormError::TokenRequired`、`EmailNotWhitelisted`，`lib.rs:354-359`） | ⬜ |
| 未接入 | REST 端点未挂载（当前状态） | 显示未接入状态，不渲染示例表单 | ⬜ |
| 禁用 | 表单未发布或已归档 | 表单只读并说明不可提交 | ⬜ |
| 限流 | 超过每小时次数 | "提交过于频繁"（`FormError::RateLimited`，`lib.rs:351-353`） | ⬜ |

**交互**

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 填写字段 | 表单正常 | 按条件显示或隐藏字段 | 客户端求值（当前无，`lib.rs:379-421`） | ⬜ |
| 提交 | 校验通过，未超限，captcha 通过 | 显示确认页 | POST `/api/v1/public/forms/{slug}/submissions` | ⬜ |
| 查看确认 | 提交成功 | 显示确认信息，不回显内部字段 | 响应体 | ⬜ |

**数据**：GET `/api/v1/public/forms/{slug}`（提议）。返回字段须排除 tenant_id、project_id、token、白名单与内部审计字段。当前无数据源。

**与文档设计的差距**

- FR-52-09：领域层的公开提交语义存在，无端点与页面。
- FR-52-20：公开页未实现，且被根布局挂载的导航与引导阻断（见上）。
- FR-52-23：限流与 captcha 均未实现（RISK-FORM-01）。
- INV-FORM-02：领域层 `public` 为真时不要求登录（`lib.rs:408`），但访问控制存在缺陷（§4.5）。
- FR-52-04、FR-52-05：客户端与服务端的校验规则需统一，当前服务端规则存在缺陷。

## 5. 当前实现

判定口径：✅ 指 crate 公开 API 实现完整且被单元测试覆盖。`crates/domain-form` 当前无生产调用方（§4.6），因此多数行为的可用性仅在 crate 内成立。

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| domain-form 实体、校验、slug | `crates/domain-form/src/lib.rs` | 🟡 | 实体 `lib.rs:20-328`；校验 `lib.rs:379-452`；slug `lib.rs:423-429` |
| crate 清单与依赖 | `crates/domain-form/Cargo.toml` | ✅ | 工作区成员 `Cargo.toml:42`；依赖 `crates/domain-form/Cargo.toml:18-24` |
| 单元测试（11 个） | `crates/domain-form/src/lib.rs:475-635` | ✅ | 测试名见 §7.1 |
| 集成测试 | `crates/domain-form/tests/` | ⬜ | 目录不存在 |
| 生产调用方 | 无 | ⬜ | §4.6 |
| REST 路由 | `crates/star-api-rest/src/lib.rs:115-138`；`crates/star-api-rest/src/routes/` 无 forms 文件 | ⬜ | 路由表无 forms 挂载 |
| 数据库表 | `db/migrations/` | ⬜ | 无 form 相关表（§4.2） |
| 管理页 | `frontend/src/app/(app)/forms/` | ⬜ | 目录不存在 |
| 公开页 | `frontend/src/app/forms/` | ⬜ | 目录不存在 |
| 导航注册 | `frontend/src/lib/nav/registry.ts:684-714`、`:755-762` | ⬜ | 无 forms 模块与 forms 标签 |
| MSW mock | `frontend/src/mocks/handlers/index.ts:1-40` | ⬜ | 无 forms handler |
| 鉴权中间件 | `crates/star-api-rest/src/middleware/auth.rs:23-31` | 🟡 | pass-through stub |
| 限流中间件 | `crates/star-api-rest/src/middleware/rate_limit.rs:21` | 🟡 | pass-through stub |
| 同类 CRUD 页的参照（非本功能） | `frontend/src/app/(app)/settings/developer/page.tsx:63`、`:75`、`:176` | ✅ | 页面内 state 管理数据，tab 切换；可作为布局与交互参照 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 无持久化与仓储（FR-52-14、FR-52-16） | 表单与提交无法保存，无法查询 | 定义 FormRepository 与提交仓储端口；按守门 #13 分类：Form 为 M（SCD Type 2 版本化），FormSubmission 为 T（追加写），速率计数为 W（短 TTL） |
| 无租户与项目归属（FR-52-15） | 无法隔离数据，匿名端点无法限定范围 | 在 Form 与 FormSubmission 增加 tenant_id、project_id，并按 RLS 13 类规则设计 |
| 无版本与发布锁定（FR-52-13） | 已发布表单可被修改，提交数据无法对应 schema 版本 | 增加 version 与 status，发布时冻结 schema（INV-FORM-01） |
| 访问控制缺陷（§4.5：白名单绕过、token 不比较、空串必填、数值与格式不校验、非法正则静默通过） | 未授权提交可通过，校验形同虚设 | 修正 `validate_submission` 与 `validate_field`，并补充测试（§7.2 AC-52-01 至 AC-52-05） |
| 条件规则不求值（FR-52-06） | 条件字段无法强制必填或隐藏 | 实现求值器（纯函数）；先决定 spec 的 JSON Logic 与代码的运算符枚举的取舍（§8.2） |
| 提交动作无执行器（FR-52-07） | 提交不会创建工单、发通知或调用 Webhook | 按 spec §6 RISK-FORM-03 设计 Saga，依次接入 work-item、notification、webhook 端口 |
| 速率限制无执行（FR-52-12、FR-52-23） | 公开提交无防护 | 实现每表单每小时计数（W 类存储），并在公开提交端点执行；captcha 作为独立能力接入 |
| 无 REST 端点（FR-52-09、FR-52-16） | 前端无 API 可调用 | 新增 `crates/star-api-rest/src/routes/forms.rs` 并在 `build_router` 中挂载（`lib.rs:115-138`） |
| 无页面（FR-52-19、FR-52-20） | 功能无法被用户使用 | 先实现未接入状态的页面与 MSW mock，再接入 REST |
| 公开页被根布局阻断（§4.7.2） | 匿名访问者看到项目导航与引导，并触发 `/api/cli-profiles` 请求 | 决定方案：将导航与引导移入 `(app)` 布局，或在根布局按路径条件渲染 |
| slug 唯一性与保留（FR-52-08） | 两个表单可能共用公开地址；空 slug 被接受 | 在发布时检查唯一性（使用 `SlugConflict`，`lib.rs:362`），拒绝空 slug |
| 导航位置未定（FR-52-21） | 表单无法从侧栏进入 | 决定分组（§8.2），更新 `registry.ts` 与 `docs/frontend/design/ui-3pane-arch.md` §1.2.1 |
| 审计缺失（FR-52-18） | 表单操作与提交无可追溯记录 | 接入 domain-audit 的 AuditRecorder（追加写） |
| Workflow Guard 未实现（FR-52-17） | 提交可绕过状态与审批约束 | 依赖 REQ-WF-003 的实现（`docs/ac-test-matrix.csv:8` 为 gap） |

**实施顺序**

1. 决策：确认路由（管理 `/forms`、公开 `/forms/[slug]`、API `/api/v1/forms` 与 `/api/v1/public/forms`），确认侧栏分组，确认条件规则采用代码的运算符枚举还是 spec 的 JSON Logic，并同步 `docs/frontend/design/ui-3pane-arch.md` §1.2.1 与 spec §4。
2. 领域修正：修复 §4.5 的访问控制、必填、数值与格式、正则缺陷，增加 tenant_id、project_id、version、status，并让 slug 唯一性通过端口检查。每项修复配单元测试。
3. 纯函数：条件求值器与提交校验器，覆盖 show、hide、require、optional 四种动作。
4. 持久化：按 M、T、W 分类实现 in-memory 仓储，再实现 Postgres 迁移。
5. REST：新增 `routes/forms.rs`，在 `build_router` 中挂载；公开端点使用限流；接入审计。
6. 提交动作：实现 Saga 执行器，依次接入工单创建、通知、Webhook、自动化。
7. 管理页：先实现 MSW mock 与未接入状态，再接入 REST，实现抽屉式 builder。
8. 公开页：解决根布局阻断，实现 `/forms/[slug]`、captcha 占位、各错误与限流状态。
9. 导航：更新 `registry.ts` 的 NAV_GROUPS 与 MODULE_GROUPS，补充测试。
10. E2E：新增 `frontend/e2e/` 用例，覆盖发布、匿名提交、工单生成。

## 7. 验收与测试

### 7.1 现有测试

位置：`crates/domain-form/src/lib.rs:475-635`，共 11 个单元测试，可用 `cargo test -p domain-form` 运行。

| 测试 | 行号 | 验证内容 |
|---|---|---|
| test_field_type_all_count | `lib.rs:480` | 字段类型数量不少于 12 |
| test_field_type_name | `lib.rs:486` | 字段类型展示名 |
| test_form_new | `lib.rs:492` | 表单构造与默认非公开 |
| test_form_add_field | `lib.rs:500` | 追加字段 |
| test_form_duplicate_field_key | `lib.rs:517` | 字段 key 重复拒绝 |
| test_validate_required_field_missing | `lib.rs:535` | 必填字段缺失拒绝 |
| test_generate_slug | `lib.rs:562` | slug 生成 |
| test_public_url | `lib.rs:571` | 公开 URL 拼接 |
| test_regex_lite_real_impl | `lib.rs:582` | 正则编译与匹配 |
| test_regex_lite_invalid_pattern | `lib.rs:595` | 非法正则返回错误 |
| test_validate_field_uses_real_regex | `lib.rs:602` | 字段 pattern 校验，含非法 pattern 时校验通过 |

**已验证**：字段类型、key 唯一、必填缺失、slug 生成、公开 URL 拼接、pattern 匹配。

**未验证**：条件求值、提交动作、速率限制、token 比较、白名单绕过场景、数值 min/max、字段类型格式、空串必填、持久化、REST、页面。

### 7.2 验收条件（可测试，当前均未满足）

| 编号 | 条件 | 当前行为 | 测试位置 |
|---|---|---|---|
| AC-52-01 | 必填字段值为 `""` 时，`validate_submission` 返回 `RequiredFieldMissing` | 返回 Ok（`lib.rs:390`） | crate 单元测试 |
| AC-52-02 | 非公开且 `require_token` 为真时，缺失或不匹配的 token 被拒绝 | 仅检查 email（`lib.rs:409-411`） | crate 单元测试 |
| AC-52-03 | 白名单非空、非公开时，无 `submitter_email` 的提交被拒绝 | 放行（`lib.rs:412`） | crate 单元测试 |
| AC-52-04 | Number 字段的 min、max 生效；Email 字段格式非法的值被拒绝 | 不校验（`lib.rs:432-450`） | crate 单元测试 |
| AC-52-05 | 非法 pattern 在发布时被拒绝，不静默通过 | 静默通过（`lib.rs:445-449`，测试 `lib.rs:632-634`） | crate 单元测试 |
| AC-52-06 | 同一表单每小时超过 `rate_limit_per_hour` 次提交返回 `RateLimited` | 无执行 | crate 单元测试与端点测试 |
| AC-52-07 | 条件 require 在触发条件成立且字段缺失时拒绝提交 | 不求值 | crate 单元测试 |
| AC-52-08 | 已发布表单修改字段返回错误，并需新建 version（INV-FORM-01） | 无 status 与 version | crate 单元测试 |
| AC-52-09 | 名称生成空 slug（如 `"!!!"`）时发布被拒绝 | 生成空字符串，无校验（`lib.rs:424-429`） | crate 单元测试 |
| AC-52-10 | 第二个表单使用已占用 slug 时发布返回 `SlugConflict` | 无检查，`SlugConflict` 未使用 | crate 单元测试 |
| AC-52-11 | 公开页匿名提交后，提交记录可见，配置的 CreateWorkItem 动作生成工作项 | 无端点与页面 | E2E（frontend/e2e） |
| AC-52-12 | REST 未挂载时，`/forms` 显示"未接入"且不展示示例表单 | 页面不存在 | 前端组件测试 |
| AC-52-13 | 公开页不渲染 Sidebar、MobileBottomNav、OnboardingGuard | 根布局全部挂载（`layout.tsx:77-79`、`:99`、`:107`） | 前端组件测试 |
| AC-52-14 | 公开端点响应不含 tenant_id、project_id、token、白名单 | 无端点 | API 测试 |

## 8. 来源与裁决

| 源 | 吸收的内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/specs/domain-form-spec.md`（v0.1，2026-09-01） | 职责、实体、INV-FORM-01 至 04、端口、跨域接触面、风险 | FormDefinition 字段（tenant_id、project_id、version、triggers）代码无；12 字段类型 vs 代码 18 种；JSON Logic vs `ConditionalRule` 运算符枚举；`FormSubmission.submitted_by` vs 代码 email 与 IP；spec 无 UI 章节 | 实体与不变量作为目标设计；字段类型采用代码的 18 种（`lib.rs:20-57`，测试注释 `lib.rs:481` 亦为"12 核心 + 6 扩展"）；条件表达式采用代码的运算符枚举，JSON Logic 留待决策（§8.2）；§4.7 的 UI 由 ui-3pane 与代码推导 |
| `docs/frontend/design/ui-3pane-arch.md`（v0.2） | 第 62 行的 Forms 主视图族；第 334 行 `feat/w13-forms` 计划（拖拽表单与前端构建器） | §0.1 的 5 个 tab 不含表单（第 22-36 行）；§1.2.1 路由映射表无 `/forms`；第 15 行指向 lane-workspace WBS 作为现行布局 | 表单作为独立路由 `/forms`，保留 Forms 视图族的概念；路由映射表须补充 `/forms` |
| `docs/architecture/2026-11-15-lane-workspace/WBS-LANE-WORKSPACE-001.md`（现行布局设计） | 无表单相关条目（关键词 form、表单 检索无命中） | — | 不改变 Lane 级 App 页签列表（`AGENTS.md:22`）；表单是否进入 Lane 页签未决（§8.2） |
| `docs/basic-design.md:407`、`:628` | domain-form 归类；Workflow Guard 依赖 REQ-WF-003 | 无 | 采纳 |
| `docs/ac-test-matrix.csv:8`、`:10` | REQ-WF-003、REQ-DSG-002 状态为 gap | 无 | 采纳为 FR-52-17 的状态依据 |
| `docs/reports/DDD-DOMAIN-MODEL-COMPLETENESS-AUDIT.md:104` | 0 个 pub trait、无事件、`rate_limit_per_hour` 无执行（`lib.rs:248`） | 无 | 与代码一致，用于 FR-52-12、FR-52-16 |
| `docs/reports/PHASE-W11-W16-IMPL-REPORT.md:31`、`:42`、`:89`、`:99` | 合并记录；18 字段类型；测试期望由 12 改为不少于 12 | 历史记录 | 作为历史背景，以代码为准 |
| `docs/reports/PHASE-P3-A19-IMPL-REPORT.md:53` | domain-form 8 个测试 | 当前为 11 个（§7.1） | 以代码为准 |
| `docs/data-design/p3-d-classification-w-t-m.md:39`、`:568-583`、`:89-90` | domain-form 14 张 T 类表，CW-02 违反 | pin 处无对应迁移或表 | 不采纳，列为未决（§8.2） |
| `docs/requirements/SRS-CANVAS-001.md:145`、`SRS-CANVAS-AGENT-001.md:136` | Canvas 中的表单小部件与表单编辑器 | 属 Canvas 或数据 App 范围 | 作为范围外边界，不作为本模块设计 |
| `docs/requirements.md:2795`、`docs/frontend/design/settings-capabilities-design.md:39` | 高级设置中的 Hook 表单、MCP 添加表单 | 与表单模块无关 | 排除 |
| `scripts/gen_supporting_specs.py:69-97` | 生成 domain-form spec 的文本 | 生成器不是实现证据 | 仅作为 spec 的来源，不作为现状证据 |
| `crates/star-api-rest/src/routes/submissions.rs:1-10` | 名为 submission 的验证路由 | 与表单提交无关 | 排除 |
| `crates/star-dispatcher/src/sa_real_impls.rs:253` | 字符串列表中的 domain-form | 不是依赖 | 不视为消费方 |
| `docs/briefs/next-session/OPT-NEXT-06-code-stub.md:31`，`docs/briefs/OPT-A1-code-todo-scan.output.md:121-122`、`:151` | 正则与 stub 的历史记录 | 当前为真实正则（`lib.rs:458-467`） | 以代码为准，不再视为 stub |
| `AGENTS.md:22`（pin 处） | 未接通授权数据须显示未接入状态 | 无 | 作为 §4.7 状态矩阵的依据 |
| `docs/specs/domain-integration-spec.md` | spec §6 RISK-FORM-01 引用其 §8 Rate Limit | 未核对该章节内容 | 未核实（§8.2） |
| `frontend/e2e/` | 现有 Playwright 用例 | 5 个文件的内容检索命中 `form`，未逐条确认是否与表单相关 | 未核实（§8.2） |

### 8.1 已作出的设计决定

- 路由：管理页 `/forms`，builder 为 `?id=` 抽屉，公开页 `/forms/[slug]`，公开 API `/api/v1/public/forms/{slug}`。理由：`Form::public_url` 的前端地址格式已固定（`lib.rs:308-310`），管理页不占用 `/forms/...` 子路径，可避免与 slug 冲突。
- 侧栏分组：提议放入 `project` 组（`registry.ts:691`），与工作流类模块同组。`tasks` 是 Branch 标签分组（`registry.ts:755-762`），不是侧栏分组；是否将 forms 加入其标签见 §8.2。
- 持久化分类：按守门 #13 的 W、T、M 规则（Form 为 M，FormSubmission 为 T，速率计数为 W）。

### 8.2 未决事项

- 公开页壳层方案：将导航与引导移入 `(app)` 布局，还是在根布局按路径条件渲染（§4.7.2）。
- 侧栏分组：采用 `project` 组，或新增 `tasks` 侧栏组；是否将 forms 加入 Branch 的 `tasks` 标签（`registry.ts:755-762`）。
- 表单是否进入 Lane 级 App 页签（`AGENTS.md:22` 所列页签），还是保持全局路由。
- 条件表达式：spec 的 JSON Logic 与代码的运算符枚举取哪一种。
- `docs/specs/domain-integration-spec.md` 中 Rate Limit 章节的内容未核对。
- `docs/data-design/p3-d-classification-w-t-m.md` 的 14 张 T 类表在 pin 处无迁移，来源未确认。
- `frontend/e2e/` 中命中 `form` 的 5 个文件未逐条确认是否含表单用例。
- REQ-WF-003 的实现状态依赖 domain-workflow，本文档未核对其代码。
