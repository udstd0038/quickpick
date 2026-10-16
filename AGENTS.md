# QuickPick 工作说明

本文件是 QuickPick 项目的开发入口和协作规则。每次开发前必须先阅读本文件，再阅读与任务相关的 `docs/` 规范、当前有效决策和 Contract。

## 项目定位与当前进度

QuickPick 是轻量划词、区域截图和输入翻译 AI 工具，使用 Tauri 2、Rust、React、TypeScript 和 Vite。

- 当前检出的代码面向 Windows 11 x64，版本为 `0.1.4`；Windows 正式发行提供 NSIS 安装包和便携 ZIP。
- macOS `v0.1.3-macos` universal 预发布属于独立的 `feature/macos-port` 分支，尚未签名、公证或完成实体 Mac 人工验收，不能据此认定当前检出代码已支持 macOS。
- 当前工作以已发布版本的维护、问题修复、体验验证和打包验收为主，不再按空项目骨架阶段安排任务。路线图用于理解功能范围，不能把其中的阶段目标直接当作当前待办。
- 划词支持复制、翻译、总结和搜索；区域截图支持框选、复制、文字提取和图片翻译；输入翻译支持语言、方向切换及 `Ctrl+Enter` 提交。
- 划词模型、截图模型和输入模型各自保存供应商、Base URL、模型和 API Key，请求不跨配置回退。
- 设置持久化、Windows DPAPI 密钥保存、托盘、开机自启、快捷键映射、浅深主题、窗口效果、不透明度和八种界面语言均有实现。功能已实现不等于所有运行场景已验收。
- 界面使用 Tauri/WebView，活动窗口 label 为 `main`（设置）、`selection`（划词）、`result`（结果）、`input`（输入翻译）和 `screenshot_overlay`（截图）。独立 `screenshot_preview` 不在当前活动窗口和前端路由中。
- 截图链路由 Rust `xcap` 捕获、前端 `react-image-crop` 框选和 Rust `capture_region_rect` 处理组成。截图读取和上传的预期边界仍须遵守有效隐私约定，不能仅凭链路现状放宽。
- 设置窗口默认及最小逻辑尺寸为 `420×600`；隐藏或最小化后重新打开时重置默认尺寸，`main` 不恢复窗口状态文件中的尺寸。不同 DPI 下的尺寸与最小拖动限制仍有人工验收待办。
- 仓库提供前端测试、Rust 单元测试和 Windows 热键事件链验证脚本；现有 GitHub Actions 运行依赖安全审计，不能当作构建、功能测试或实机验收通过的证据。

进度核对入口：包清单与 `apps/desktop/src-tauri/tauri.conf.json`、`apps/desktop/src/App.tsx`、`apps/desktop/src-tauri/src/`，以及最新相关开发日志。日志中的验证结果只覆盖其记录的版本、平台和场景；单次任务“无待办”不能证明其他人工验收已经完成。

## 标准文件与代码路径

开发前按任务类型阅读：

- 产品需求：`docs/00-product-requirements.md`
- 开发路线：`docs/01-development-roadmap.md`
- 技术架构：`docs/02-technical-architecture.md`
- UI 设计：`docs/03-ui-design-spec.md`
- 安全隐私：`docs/04-security-privacy-spec.md`
- 测试验收：`docs/05-testing-acceptance.md`
- 开发工作流：`docs/06-dev-workflow.md`
- 参考代码移植：`docs/07-reference-migration-spec.md`
- Snow Shot 截图审计：`docs/08-snow-shot-screenshot-audit.md`
- 贡献流程与漏洞报告：`CONTRIBUTING.md`、`SECURITY.md`
- 开发日志：`dev-logs/YYYY-MM-DD.md`

主要代码位置：

- 前端：`apps/desktop/src/`，窗口入口为 `App.tsx`，设置入口为 `windows/settings/SettingsWindow.tsx`。
- 前端设置类型、默认值和归一化：`apps/desktop/src/lib/settingsTypes.ts`；Rust 对应 `apps/desktop/src-tauri/src/app_settings.rs`。
- 系统集成、窗口与命令：`apps/desktop/src-tauri/src/lib.rs`。
- AI、密钥、取词和截图：同目录的 `ai.rs`、`security.rs`、`selection.rs`、`screenshot/mod.rs`。
- Tauri 配置与权限：`apps/desktop/src-tauri/tauri.conf.json`、`apps/desktop/src-tauri/capabilities/`。
- 验证脚本：`scripts/`；安装配置：`apps/desktop/src-tauri/installer/`。

## 硬规则：文档与代码注释只描述当前状态

- 所有文档和代码注释均描述当前状态，不得将尚未确认的实现偏差写成新的业务规则。当前状态包括已核实的实际行为、当前有效的预期行为和明确标注的未决冲突，不得混为一谈。
- 文档正文只保留当前有效需求、决策、Contract 和验证结论，不保留已被替代的决策作为现行说明，也不堆叠开发过程中的临时方案或历史演变。
- 例如开发中临时规划过 Phase 5 字段，更新文档时只需依据当前代码、项目现状和有效决策描述实际字段、用途与约束，不保留临时阶段规划，不把未实现字段写成现有能力。
- 例如决策 B 已明确获准替代决策 A，文档只记录当前决策 B；删除被替代的 A 和 A 到 B 的过程叙述，不追加一份与旧决策并存的说明。未经授权的实现差异不能被当作决策 B。
- 实际行为与预期不一致时，按下述冲突处理规则判定和处理；不得通过改写文档掩盖回归、错误测试或证据缺口。
- 验证结论必须说明实际覆盖的版本、平台和场景。尚未运行的检查、尚未完成的人工验收只记录为待验证事项，不写成已通过。

## 冲突处理

本节中的 Contract 指当前有效的行为约定，Claim 指需要证据支撑的断言或验收结论；不以是否存在同名文件为前提。

- 代码、测试和必要的运行验证用于核实当前实际行为；当前明确的用户要求、有效决策与 Contract 用于确定预期行为。测试通过不能单独证明实现符合有效决策。
- 发现不一致时，先识别描述过期、实现回归、测试错误或已获批准的行为变更。描述过期则修订文档；实现回归则修复代码并补足必要验证；测试错误则修正测试；已获批准的行为变更则按授权同步实现、测试和受影响的规范。
- 不得仅因当前实现如此或测试通过，就覆盖有效决策、弱化 Claim 或降低证据强度。
- 已有明确授权的变更按授权执行，不重复请求确认；依据不足或涉及尚未确认的行为取舍时，反馈具体冲突与影响，只对存在分歧的部分请求判断。可独立完成且无争议的工作继续执行。

确定预期行为时，先核实要求或决策是否仍有效、适用于当前任务，以及是否已有明确授权的替代决策。在均有效且适用的前提下，按以下顺序处理：

1. 用户当前明确要求和已授权的有效决策。
2. 当前有效 Contract；修改其约束需要有对应授权。
3. 本 `AGENTS.md` 工作规则。
4. `docs/00-product-requirements.md` 与 `docs/04-security-privacy-spec.md`。
5. 其他适用的 `docs/` 规范。

安全和隐私要求优先于开发便利。同一层级的有效要求仍有冲突时，明确列出分歧，不以文件位置、更新时间、现有代码或测试结果代替行为取舍。

## 每次开发前

- 阅读本文件、相关规范与最新相关开发日志，核对当前版本、代码入口、有效决策和未完成验收。
- 检查工作区已有修改，先了解现状再修改。
- 明确本次目标和验收点；范围较大时拆成可验证步骤，按用户已授权范围完成。
- 发现旧阶段描述与当前决策或实现不一致时，先按冲突处理规则分类，不直接沿用旧计划，也不自动认定实现正确。

## 每次开发中

- 不做与当前任务无关的重构，不加入未经授权的扩展功能。滚动截图、标注、贴图、内容历史、云同步和插件系统不属于当前维护任务的默认范围。
- 默认不复制参考项目代码；截图模块按 `docs/07-reference-migration-spec.md` 的许可和复用边界执行。引入第三方截图代码前记录来源仓库、commit 或版本、文件清单、许可证和回退方案。
- 不把 API Key、用户文本、截图内容、AI 请求 payload 或完整响应写入仓库、日志、Issue 或 CI 产物。
- 不保存文本、截图和 AI 结果历史；AI 上传必须由用户主动操作触发，遵守有效隐私 Contract。
- API Key 在 Windows 使用 DPAPI 保存，不明文写入普通配置文件；外部 AI 服务使用 HTTPS，HTTP 仅允许本地或私有网络地址。
- 划词优先使用 UI Automation，剪贴板兜底默认开启且可关闭；兜底时尽量恢复原剪贴板，不自动用 AI 结果覆盖剪贴板。
- 弹窗隐藏或关闭时清空选区、截图、输入和结果等敏感状态；按有效窗口生命周期约定保留可复用窗口。
- 涉及系统能力时优先使用 Tauri 官方插件或 Windows 标准能力；涉及窗口、权限、焦点和快捷键时核对实际调用链，不凭函数名称判断是否仍为原生窗口。
- UI 保持轻量、克制，兼顾浅深主题、加载态、错误态、窗口关闭路径和 DPI。界面文案使用现有 `t()` 多语言机制。

## 验证与构建

按改动范围选择必要检查，不把历史检查结果当成本次验证：

| 范围 | 命令与环境 |
| --- | --- |
| 前端测试 | 根目录执行 `pnpm test` |
| TypeScript 与前端构建 | 根目录执行 `pnpm build` |
| Rust 检查与测试 | 在 Windows 执行 `cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml`、`cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml` |
| 桌面开发 | 根目录执行 `pnpm desktop:dev` |
| Windows release 可执行文件 | `pnpm --filter quickpick-desktop exec tauri build --no-bundle --ci` |
| Windows NSIS 安装包 | `pnpm --filter quickpick-desktop exec tauri build --bundles nsis --ci --no-sign` |
| Windows 热键事件链 | 在已运行 QuickPick 的 Windows 环境执行 `powershell -ExecutionPolicy Bypass -File scripts\verify-hotkeys.ps1` |
| 发布前依赖安全审计 | Windows 执行 `powershell -ExecutionPolicy Bypass -File scripts\security-audit.ps1` |

- Node.js 要求 22+；pnpm 版本以根目录 `package.json` 的 `packageManager` 为准；Rust 与平台依赖按 Tauri 2 要求配置。
- 热键脚本覆盖事件投递、窗口显示和失焦等检查，不能替代真实按键、跨应用取词、剪贴板恢复、截图坐标和 DPI 人工验收。
- 纯文档变更核对内容、引用、链接和差异；无需为此构建或启动应用。
- 在非 Windows 环境不得声称已完成 Windows Rust、桌面运行或打包验收；不能执行的检查说明具体原因和待验证范围。

## 每次开发结束前

- 执行适合本次改动的验证，记录命令、结果和覆盖边界；不能执行时写明原因。
- 修改应用代码、配置、资源或发行产物时，在可运行的 Windows 环境完成验证、覆盖默认 release，并自动启动 `apps/desktop/src-tauri/target/release/quickpick.exe` 供用户验收。环境不支持时明确记录，不能将未运行写成通过。
- 更新当天 `dev-logs/YYYY-MM-DD.md`，包含本次完成、当前待办、当前有效决策、遇到的问题、验证结果和下次建议；同一事项的决策被授权替代时更新其现行说明，不追加相互冲突的版本。
- 需求、架构、UI、安全或测试规则变化时，同步受影响的规范，遵守硬规则和冲突处理规则。
- 收尾说明实际改动、验证结果和剩余验收项。

## 默认配置

- 项目名：QuickPick；当前检出代码目标：Windows 11 x64。
- 设置快捷键：`Alt+0`；划词：`Alt+2`；截图：`Alt+3`；输入翻译：`Alt+4`。
- 界面语言与主题默认跟随系统；翻译目标语言按当前语言设置归一化。
- 默认窗口效果：Acrylic；默认不透明度：100%；开机自启默认开启。
- 默认隐私策略：不保存文本、截图或 AI 结果历史。
- 开发日志由 Agent 在每次开发结束时维护，不配置 Windows 定时任务。
