# QuickPick 工作说明

本文件是 QuickPick 项目的开发入口说明。每次开发前必须先阅读本文件，再阅读相关 `docs/` 标准文件。

## 项目定位

QuickPick 是运行在 Windows 11 上的轻量划词与截图 AI 工具。

首版只做稳定 MVP：

- `Alt+2` 呼出划词菜单：复制、翻译、总结、搜索。
- `Alt+3` 启动区域截图：复制、OCR 提取、OCR 后翻译。
- OpenAI 兼容 API。
- 系统托盘、开机自启、快捷键映射、暗黑模式、Win11 Mica 效果。

首版不做滚动截图、标注、贴图、历史记录、云同步、插件系统。

## 标准文件路径

开发前按任务类型阅读对应文件：

- 产品需求：`docs/00-product-requirements.md`
- 开发路线：`docs/01-development-roadmap.md`
- 技术架构：`docs/02-technical-architecture.md`
- UI 设计：`docs/03-ui-design-spec.md`
- 安全隐私：`docs/04-security-privacy-spec.md`
- 测试验收：`docs/05-testing-acceptance.md`
- 开发工作流：`docs/06-dev-workflow.md`
- 参考代码移植：`docs/07-reference-migration-spec.md`
- Snow Shot 截图审计：`docs/08-snow-shot-screenshot-audit.md`

开发日志目录：

- `dev-logs/`

每日日志格式：

- `dev-logs/YYYY-MM-DD.md`

## 每次开发前

必须完成：

- 阅读本文件。
- 阅读与当前任务相关的 `docs/` 文件。
- 查看最新开发日志，理解上次完成和待办。
- 把任务控制在一个小阶段或一个明确子任务内。

如果用户要求范围很大，先拆小步，不要一次性做完所有功能。

## 每次开发中

必须遵守：

- 先检查现有文件，再修改。
- 不做与当前任务无关的重构。
- 不加入非 MVP 功能。
- 默认不复制参考项目代码；截图模块例外，按 `docs/07-reference-migration-spec.md` 执行，可在许可证允许范围内复刻/移植 Snow Shot 可用截图链路。
- 引入任何第三方截图代码前，必须记录来源仓库、commit 或版本、文件清单、许可证和回退方案。
- 不把 API Key、用户文本、截图内容、AI 完整响应写入仓库或日志。
- 涉及系统能力时优先使用 Tauri 官方插件或 Windows 标准能力。
- UI 必须保持轻量、克制、贴近 Windows 11 工具气质。

## 每次开发结束前

必须完成：

- 执行适合本阶段的验证；如果不能执行，写明原因。
- 如果本次修改影响应用代码、配置、资源或发行产物，完成验证并覆盖默认 release 后，自动启动新版 `src-tauri/target/release/quickpick.exe` 供用户验收。
- 更新当天开发日志 `dev-logs/YYYY-MM-DD.md`。
- 日志至少包含：
  - 今日完成。
  - 今日待办。
  - 关键决策。
  - 遇到问题。
  - 验证结果。
  - 下次建议。
- 如果需求、架构、UI、安全或测试规则变化，同步更新对应 `docs/` 文件。

## 当前阶段路线

按以下阶段推进：

1. 第 0 阶段：文档与流程基建。
2. 第 1 阶段：Tauri 2 空项目骨架。
3. 第 2 阶段：全局快捷键与划词 MVP。
4. 第 3 阶段：区域截图 MVP。
5. 第 4 阶段：OpenAI 兼容 AI 能力。
6. 第 5 阶段：设置、安全与体验打磨。
7. 第 6 阶段：验收与打包。

每次只推进一个阶段中的少量可验收任务。

## 冲突处理

发生冲突时按以下顺序处理：

1. 用户最新明确要求。
2. 本 `CLAUDE.md`。
3. `docs/00-product-requirements.md`。
4. `docs/04-security-privacy-spec.md`。
5. 其他 `docs/` 文件。
6. 当前代码实现。

安全和隐私要求优先于开发便利。

## 默认配置

- 项目名：QuickPick。
- 目标系统：Windows 11 x64。
- 默认划词快捷键：`Alt+2`。
- 默认截图快捷键：`Alt+3`。
- 默认 AI 输出语言：简体中文。
- 默认隐私策略：不保存文本、截图、AI 结果历史。
- 开发日志维护方式：Agent 每次开发结束自动更新，不配置 Windows 定时任务。
