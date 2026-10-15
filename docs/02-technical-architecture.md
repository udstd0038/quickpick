# QuickPick 技术架构规范

## 0. 当前架构快照（2026-08）

本节描述现行共享窗口基线。后文保留的原生 Win32 弹窗、白窗恢复闸门、旧诊断页及仅 `main` capability 条目属于历史 Windows 实现与排查记录，不能覆盖本节已恢复的 WebView 链路，也不作为禁止 macOS 预创建窗口的通用闸门。macOS 本轮分阶段状态见 `docs/09-macos-port.md`；第三阶段代码与独立 CI 已通过，真实 Mac 验收仍待设备。

- 设置页、划词、结果、输入、截图遮罩全部使用 Tauri/WebView 预创建窗口，`App.tsx` 根据窗口 label 渲染对应前端组件。
- 截图链路只有一条：WebView 截图遮罩显示 xcap 捕获画面，前端用 `react-image-crop` 框选，Rust 通过 `capture_region_rect` 裁剪并写剪贴板或调用 AI。
- `screenshot/mod.rs` 已移除旧原生 Win32 框选代码，只保留捕获、坐标换算、裁剪、剪贴板和 PNG/BMP 编码。
- 前端设置模型唯一入口为 `lib/settingsTypes.ts`；`SettingsWindow.tsx` 不再保存重复的 `AppSettings`、默认值和归一化逻辑。
- 界面语言使用 key-based `t()`，不扫描 DOM，不依赖 MutationObserver。
- Rust 设置与 AI 常见错误返回 `i18n:<key>`，前端通过 `t()` 映射为当前语言；带状态码错误使用 `i18n:<key>:<detail>`。
- 快捷键保存会返回“已注册/已被占用”状态；未注册成功的设置热键不会进入运行状态。
- `get_main_diagnostics` 和旧诊断面板代码已移除，不再作为开发入口。
- `screenshot_preview` 窗口已冻结，不再进入窗口配置、capability 或前端路由；源文件待确认后删除。

### 0.1 第三阶段截图会话（代码与 CI 已通过，实机待验收）

- Windows/macOS 的遮罩预览与最终裁剪必须绑定同一次 xcap 捕获的 RGBA 会话；不能在点击复制或 AI 动作时重新截取已显示遮罩的桌面。
- 优先选择鼠标所在显示器，记录显示器身份、bounds 与实际图像尺寸。macOS bounds 使用逻辑点，Windows bounds 使用物理像素；相对选区坐标使用图像像素，桌面原点只用于窗口定位。
- 按图像尺寸与显示器 bounds 计算映射，不假定所有屏幕同一缩放比例；负原点、Retina、外屏和混合缩放不得把选区错移到其他显示器。
- 会话只保留在内存，有明确的当前身份及失效边界；取消、失败、过期与后续捕获必须清理。失效任务即使晚完成，也不得发布预览或复活已清理的旧会话。
- AI 配置和凭据预检查必须先于区域裁剪及上传；配置缺失或读取拒绝时只显示提示，不提取区域、不发请求。遮罩预览由用户主动触发截图生成，不等同于授权上传。
- macOS 遮罩使用绑定选定显示器的普通无边框窗口和 `CanJoinAllSpaces`/`FullScreenAuxiliary`；避免进入原生全屏新建 Space。代码验证和真实 Spaces/全屏验收分别记录。

### 0.2 第三阶段 macOS 权限与访问保护（代码与 CI 已通过，实机待验收）

- 设置页分别查询辅助功能、屏幕录制和输入监控状态；挂载与应用回到前台时只复查，只有明确点击授权动作才调用请求接口，之后重新查询。
- 请求、查询失败及未授权不能混同；系统设置链接由 Rust 按固定权限类别映射，不接受前端任意 URL 或命令。保留用户手动复查与需要重启的提示。
- macOS 15.4+ 可查询的 NSPasteboard 访问策略为 `AlwaysDeny` 时，读取快照和复制注入前停止。`pasteboardItems` 返回 nil 而无法区分拒绝与空板时，不构造空快照、不覆盖剪贴板；只有访问许可明确且能可靠确认的空板才允许按空快照处理。
- 权限查询与引导本身不读取用户文本、屏幕或剪贴板，不持久化 TCC 状态，也不调用 AI。

### 0.3 第四阶段开发工具迁移（代码已实现，独立 CI 与交互验收待执行）

- 活跃工具入口统一使用 Node.js；资源生成复用现有 Tauri CLI 与 Rust 图像依赖，不额外引入图像工具链。
- 快捷键验收使用 Windows 原生 `SendInput` 注入完整组合键，验证真实注册的动作、对应窗口与关闭/失焦行为；不能用 `WM_HOTKEY` 直接投递或“任意窗口可见”代替触发成功。
- README 截图使用带 `dev-tools` 的专用 `quickpick-fixture` binary，普通 release 不包含该工具入口。启动前核验编译标记，启动后核验隔离目录和随机 token 的 loopback 接口；fixture 使用单独的应用身份、配置、窗口状态、WebView profile 和固定测试内容，不读取真实用户设置、API Key 或桌面内容，不发起 AI 请求，不修改真实登录自启或系统剪贴板。仅终止本次工具创建的进程。
- fixture 的捕获来自固定 RGBA；README 导出只以 `PrintWindow` 读取 owned 窗口客户区，原生不透明背景及禁用玻璃避免真实桌面穿透，不使用屏幕捕获作失败兜底。
- 原生工具编译与交互桌面验收分别记录；无法使用交互桌面时返回非零“未执行”，不能报告通过。

## 1. 技术栈

首版采用：

- 桌面框架：Tauri 2。
- 系统层：Rust。
- 前端：React + TypeScript + Vite。
- 目标系统：Windows 11 x64。
- AI 接口：OpenAI 兼容 HTTP API。

默认优先选择 Tauri 官方插件和成熟 Rust crate。只有当官方插件无法满足需求时，才引入额外依赖。

## 2. 进程与窗口

应用以托盘静默常驻为核心。启动和开机自启后不主动显示主页面；用户通过设置入口显示预创建的 `main` 设置页。

当前窗口身份使用第 0 节的 `main`、`selection`、`result`、`input`、`screenshot_overlay`；设置也可由实际注册成功的设置快捷键打开。macOS 设置窗口使用原生标题栏与交通灯，关闭后隐藏并保持菜单栏常驻；Windows 保留自绘标题栏。未固定的菜单窗口失焦隐藏，固定结果窗口、截图遮罩按各自明确退出规则处理。

以下“建议窗口”、原生 Win32 及白窗条目保留作为 **Windows 历史实现与排查记录**，其中禁用 WebView 和诊断面板的要求仅适用于对应未恢复的历史路径。现行链路按第 0 节与当前代码/capability 验证；macOS 第三阶段无需重新引入旧 `settings` 或冻结的 `screenshot_preview` 窗口。

建议窗口：

- `main`：预创建但默认隐藏的设置页容器，托盘“设置”触发后显示。
- `settings`：独立设置子窗口已暂停，不作为运行入口。
- `selection_bar`：Tauri/WebView 划词横条菜单后续恢复候选；白窗问题解决前不作为运行入口。
- `screenshot_overlay`：截图遮罩窗口。
- `screenshot_preview`：截图预览窗口。
- `result`：Tauri/WebView AI 结果弹窗后续恢复候选；白窗问题解决前不作为运行入口。

窗口原则：

- 按需创建或显示，失焦后隐藏。
- 菜单类窗口无边框、置顶、透明背景。
- 设置窗口可以记忆位置和尺寸。
- `main` 窗口默认 `visible=false`，不再承载诊断主页面；前端 `main` label 直接渲染设置页。
- `main` 设置窗口使用无系统标题栏的自定义玻璃标题栏，关闭按钮只隐藏窗口，不退出托盘常驻进程。
- 托盘“设置”只显示预创建的 `main` 窗口，不创建新的 `settings` WebView 子窗口。
- 当前划词操作和 AI 结果使用 Rust 原生 Win32 弹窗，不走 Tauri/WebView 子窗口。
- 原生划词菜单和原生截图横条菜单应尽量复用设置页浅色 Frosted Glass 色板：浅玻璃底、统一浅色按钮、柔和边线和圆角按钮。
- 软件字体统一参考 TieZ Frosted Glass 的字体栈：Web 侧使用 `"Segoe UI", "Inter", "Microsoft YaHei", sans-serif`；原生 Win32 自绘窗口使用 `Segoe UI`，并依赖 Windows 字体链接回退中文。
- 原生结果弹窗必须支持右上角关闭图标、点击外部销毁、右下角复制结果和右上角图钉固定置顶。
- 原生结果弹窗成功态不显示底部完成说明；只在错误、配置缺失或排障状态保留必要 detail 提示。
- 原生结果弹窗按钮状态由 Win32 消息维护 Hover、Pressed 和松开复位反馈，视觉与设置页按钮保持一致。
- 原生结果弹窗通过双缓冲绘制和局部区域刷新降低 Hover 与 Skeleton 动画过程中的整窗闪烁。
- 文本 AI 和截图 AI 点击后必须立即打开原生结果弹窗显示带扫光动效的 Skeleton，AI 返回后在同一个窗口以轻量动效更新为最终结果或错误。
- 翻译结果弹窗顶部提供源语言下拉、方向按钮和目标语言下拉；方向切换或语言变化后，原生弹窗只复用当前内存中的原文/截图字节重新发起用户触发的翻译请求。
- Loading 状态不显示底部 detail 提示，只显示 Skeleton。
- 原生结果弹窗内容滚动必须被内容框裁剪，不得绘制到底部按钮区域。
- 原生结果弹窗默认显示时应尽量不强制抢走原应用焦点，避免影响下一次划词；用户点击弹窗后仍可操作复制、固定和关闭。
- 原生划词弹窗和结果弹窗通过按钮、`Esc` 或窗口内部空白关闭时，应尽量把焦点还给打开弹窗前的前台应用，避免下一次 `Alt+2` 读到 QuickPick 自身。
- 原生划词弹窗和结果弹窗通过外部点击或失焦销毁时，不得强行恢复旧焦点；应尊重 Windows 已切换到的新前台窗口，避免覆盖用户刚点击的目标应用。
- 划词读取采用 UIAutomation 优先；直接读取失败时才允许临时发送 `Ctrl+C` 走剪贴板兜底，读取后必须尽量恢复原剪贴板格式。
- `Alt+2` 触发瞬间必须记录当前前台窗口句柄，后续取词应以该窗口为目标，避免热键释放、托盘或隐藏设置页改变焦点判断。
- 划词读取启动后必须先等待 `Alt+2` 中的 Alt 和 2 都释放，并短暂等待前台应用稳定；随后再执行 UIAutomation 读取或剪贴板兜底，避免快速松开热键时选区状态未恢复。
- 剪贴板兜底前应尝试恢复到 `Alt+2` 触发时记录的前台窗口，并清理 Alt 菜单状态，再发送 `Ctrl+C`。
- 剪贴板兜底发送 `Ctrl+C` 时不得带着 Alt 键残留，避免变成 `Ctrl+Alt+C` 导致浏览器无法复制选中文本。
- 当前区域截图使用原生 Win32 临时框选层覆盖鼠标所在显示器；Tauri/WebView 截图遮罩继续暂停。
- 如果动态子窗口出现白屏或无法关闭，必须停止新增子窗口入口，先在静态源码和隐藏设置页路径内排查。
- 旧主窗口诊断不再作为用户默认入口；相关诊断命令只能作为开发只读能力保留，不创建窗口、不读取用户内容。
- 开发诊断可展示 capability 窗口列表、构建资源状态、子窗口配置对照、窗口创建链路风险对照、前端窗口识别分支对照、视图初始化副作用对照、视图错误态兜底对照、构建产物加载关系对照、`settings` 失败撤回记录、子窗口 WebView 创建失败原因对照、配置权限与插件链路核对、capability 授权精简执行状态、Snow Shot 截图链路审计状态、Snow Shot 窗口创建与关闭链路审计、Snow Shot 非窗口截图核心候选审计、Snow Shot 截图依赖许可证审计、只读截图命令草稿实施方案、最小截图依赖引入与扫描方案、传递依赖扫描执行方案、许可证扫描工具选择与离线替代方案、最小截图依赖引入状态、原生区域截图剪贴板 MVP、已暂停入口和风险清单。
- 旧 `settings` 独立窗口已人工验证失败；当前设置入口不得创建 `settings` label 子窗口，只能复用预创建且 capability 已授权的 `main` 窗口。
- 白窗问题排查完成前，托盘菜单和快捷键都不得触发 `selection_bar`、`result`、`screenshot_overlay`、安全预览或窗口诊断等 Tauri/WebView 子窗口；`Alt+2` 只允许打开原生 Win32 划词弹窗，`Alt+3` 只允许执行原生 Win32 区域截图。
- 子窗口 WebView 创建失败原因对照只能基于源码、配置和人工验收记录生成，不得调用窗口创建函数。
- 配置权限与插件链路核对只能读取 `tauri.conf.json`、capability、插件注册和现有窗口创建残留，不得新增或调用任何窗口创建入口。
- capability 授权已临时精简为仅 `main`；后续任何子窗口恢复都必须先补回对应 capability 授权，再单独进行安全验证。
- Snow Shot 非窗口截图核心候选审计只能记录源码关系和迁移边界；原生区域截图 MVP 只在用户框选并点击“复制、提取或翻译”后读取所选区域，不保存截图文件、不写截图历史。
- Snow Shot 截图依赖许可证审计只能核对锁定 commit 的顶层许可证和必要声明，不得自动新增依赖；真正引入前还必须补做传递依赖扫描。
- 只读截图命令草稿已升级为原生区域截图复制 MVP；后续仍不得恢复 Tauri/WebView 遮罩或预览子窗口。
- 最小截图依赖引入与扫描方案只能描述候选依赖、扫描命令、版本策略和回退线，不得修改 `Cargo.toml` 或新增截图模块。
- 传递依赖扫描执行方案只能描述扫描工具、命令、记录格式和阻塞标准，不得安装工具、联网下载依赖或修改 `Cargo.toml`。
- 许可证扫描工具选择与离线替代方案只能描述 `cargo-deny`、`cargo-about` 和离线人工核对流程，不得安装工具、联网下载依赖或修改 `Cargo.toml`。
- 最小截图依赖引入状态展示 `xcap`、`image`、Windows 目标依赖树、许可证扫描结论和当前无窗口截图入口状态。
- 原生区域截图 MVP 位于 `src-tauri/src/screenshot/`，通过 `windows-sys` 创建临时框选层，通过 `xcap` 捕获所选区域；复制使用 `CF_DIB + CF_BITMAP` 写入剪贴板，提取和翻译使用临时 PNG data URL 调用 OpenAI 兼容视觉模型。
- AI 配置放在托盘触发的隐藏 `main` 设置页内；划词操作、文本 AI 结果和截图 AI 结果通过原生 Win32 弹窗显示，不打开 `settings`、`selection_bar` 或 `result` 子窗口。

参考实现原则：

- 截图模块放弃继续自研核心实现，优先参考 Snow Shot 已验证截图链路。
- 复制或移植前必须按 `docs/07-reference-migration-spec.md` 完成许可证、来源 commit、文件清单和回退方案记录。
- 许可证边界不清楚时，只学习架构和行为，不直接复制源码。
- 任何移植代码不得绕过当前白窗安全闸门；不得直接恢复全屏遮罩或截图预览子窗口。

## 3. Rust 侧职责

Rust 侧负责贴近系统的能力：

- 全局快捷键注册和更新。
- 托盘创建和菜单事件。
- 开机自启。
- 读取当前选中文本。
- 剪贴板读写。
- 默认浏览器搜索。
- 截图捕获。
- 本地设置读写。
- API Key 加密保存。
- AI 请求封装，或提供安全代理命令。

Rust 命令应保持小而清晰，不把大量 UI 状态逻辑塞进 Rust。

## 4. 前端职责

前端负责用户交互和视图状态：

- 划词横条菜单。
- 截图遮罩交互。
- 截图预览窗口。
- AI 结果弹窗。
- 设置页表单。
- 加载、错误、空状态展示。

前端不应保存敏感数据到普通 localStorage。

当前截图预览嵌在 `screenshot_overlay` 中，前述独立预览窗口仅为历史建议，不恢复冻结的 `screenshot_preview`。第三阶段权限状态、明确操作和错误提示由设置页承载，系统查询、授权请求及固定 URL 映射在 Rust 侧封装。

## 5. 建议模块划分

Rust 侧建议模块：

- `app_state`：全局状态和配置缓存。
- `settings`：设置模型、加载、保存。
- `tray`：系统托盘。
- `hotkeys`：全局快捷键。
- `selection`：选区文本读取。
- `clipboard`：剪贴板文本和图片处理。
- `screenshot`：区域截图能力。
- `ai`：OpenAI 兼容请求。
- `security`：API Key 加密解密。

前端建议模块：

- `components`：通用 UI 组件。
- `windows`：不同窗口入口。
- `services`：Tauri invoke 封装。
- `state`：轻量状态管理。
- `styles`：全局样式、主题、窗口效果辅助样式。

## 6. 数据模型

首版只保存设置，不保存内容历史。

建议设置字段：

- `text_ai_provider`：`openai_compatible`、`deepseek`、`xiaomi_mimo`、`kimi`、`glm`、`minimax`、`qwen`。
- `text_ai_base_url`。
- `text_ai_model`。
- `vision_ai_provider`：`openai_compatible`、`deepseek`、`xiaomi_mimo`、`kimi`、`glm`、`minimax`、`qwen`。
- `vision_ai_base_url`。
- `vision_ai_model`。
- `translation_target_language`：默认 `zh-Hans`，可切换简体中文、英文、日文、韩文、法文、德文、西班牙文。
- `theme_mode`：`system`、`light`、`dark`；旧配置中的 `workbench` 加载时自动迁移为 `dark`。
- `selection_hotkey`：默认 `Alt+2`，设置页可修改；首版要求组合中包含 `Alt`。
- `screenshot_hotkey`：默认 `Alt+3`，设置页可修改；首版要求组合中包含 `Alt`。
- `autostart_enabled`：默认 `true`，设置页可关闭或重新启用。
- 设置页快捷键修改使用前端捕获控件，不提供手动文本输入；捕获期间 Rust 侧开启快捷键录制模式，临时忽略当前全局快捷键触发，避免录制旧快捷键时误触发划词或截图。
- `search_engine`：默认系统浏览器打开搜索 URL。
- `ai_timeout_seconds`。
- `window_effect`：固定为 `mica`；旧配置中的其他值加载时自动归一为 `mica`。

API Key 单独加密保存，不以明文出现在普通配置文件中。

## 7. AI 调用规范

AI 接口按 OpenAI 兼容格式设计。

供应商适配：

- 通用 OpenAI 兼容：用户自定义 Base URL、文本模型和视觉模型，适合 OpenAI、OpenRouter、阿里百炼兼容模式等。
- DeepSeek：默认 Base URL 为 `https://api.deepseek.com`，默认文本模型为 `deepseek-v4-flash`，默认视觉模型为 `deepseek-v4-flash-vision-exp`；请求走 `/chat/completions`，文本请求附加关闭思考模式的 `thinking` 参数。
- 小米 MiMo：默认 Base URL 为 `https://api.xiaomimimo.com/v1`，`tp-` Key 且未手动填写 Base URL 时自动切换到 `https://token-plan-cn.xiaomimimo.com/v1`，默认文本和视觉模型为 `mimo-v2.5`。
- Kimi：默认 Base URL 为 `https://api.moonshot.cn/v1`，默认文本和视觉模型为 `kimi-k2.6`。
- GLM：默认 Base URL 为 `https://open.bigmodel.cn/api/paas/v4`，默认文本模型为 `glm-5.2`，默认视觉模型为 `glm-4.5v`。
- MiniMax：默认 Base URL 为 `https://api.minimaxi.com/v1`，默认文本模型为 `MiniMax-M2.7`，默认视觉模型为 `MiniMax-VL-01`。
- Qwen：默认 Base URL 为 `https://dashscope.aliyuncs.com/compatible-mode/v1`，默认文本模型为 `qwen-plus`，默认视觉模型为 `qwen3-vl-plus`。
- 文本模型和视觉模型各自独立保存供应商、Base URL、模型名和 API Key；供应商适配层只决定默认地址、默认模型、请求体差异和视觉能力边界，用户仍可手动覆盖 Base URL 和模型名。

文本翻译：

- 输入：选中文本、源语言、目标翻译语言；源语言可为自动检测。
- 输出：译文。
- 失败：显示错误，不覆盖剪贴板。

文本总结：

- 输入：选中文本。
- 输出：简体中文摘要。
- 失败：显示错误。

图片 OCR：

- 输入：用户点击“提取”后临时生成的截图 PNG data URL。
- 输出：识别到的原文，显示在独立原生 Win32 结果弹窗。
- 失败：显示错误。

图片翻译：

- 输入：用户点击“翻译”后临时生成的截图 PNG data URL、源语言、目标翻译语言；源语言可为自动检测。
- 输出：目标语言译文，显示在独立原生 Win32 结果弹窗；不显示原文对照。
- 失败：显示错误。

所有 AI 请求必须设置超时。请求期间 UI 必须可取消或可关闭。

## 8. 系统集成规范

以下 Windows API、注册表、Mica 与原生 Win32 项仅描述 Windows 能力及历史 UI 路径。macOS 使用 AX、Keychain、LaunchAgent、菜单栏 template 图标及 Tauri 原生菜单；外观展示需对应 Popover/ContentBackground 的实际语义，不能把 Windows 名称当作 macOS 原生效果。窗口选择以第 0 节现行共享 WebView 基线为准。

优先使用：

- `tauri-plugin-global-shortcut` 管理快捷键；启动时按设置注册，设置页保存后先校验并立即重新注册。
- 开机自启当前使用 Windows 用户级 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` 启动项做最小实现，设置页保存时按 `autostart_enabled` 写入或删除启动项；后续可替换为 `tauri-plugin-autostart`。
- Tauri tray API 管理托盘图标；托盘右键菜单使用原生 Win32 自绘小弹窗，避免 WebView 白窗风险。
- 托盘左键双击打开预创建的 `main` 设置窗口；右键菜单只提供“设置、自启动、退出”入口，按软件主题显示浅色或深色紧凑样式。
- 托盘运行时使用独立小尺寸纯灰图标 `src-tauri/icons/tray-icon.png`；不再保留彩色托盘图标分支；打包和窗口图标继续使用 `src-tauri/icons/icon.ico` 与 `src-tauri/icons/icon.png`。
- `src-tauri/build.rs` 必须监听 `icons/icon.ico` 和 `icons/icon.png`，确保图标变更后 release `quickpick.exe` 会重新嵌入主图标。
- 托盘“自启动”使用勾选菜单项，点击后立即保存设置并同步 Windows 用户级启动项；菜单文字不显示开启/关闭状态，只通过前置勾选标记表达状态；设置页保存开关后也要同步托盘勾选状态。
- Tauri window effects 或 Windows 相关能力实现 Mica。
- 设置页保存外观配置后，Rust 侧对预创建的 `main` 设置窗口调用 Tauri `set_theme` 和 `set_effects`；Mica 按深浅主题优先使用 `MicaDark` 或 `MicaLight`；失败时按普通透明/半透明背景降级，不创建任何新窗口。
- 前端样式层负责 Frosted Glass 观感：透明窗口、半透明壳层、磨砂面板、细边线和柔和阴影；不得直接复制 GPL 参考项目源码。
- Tauri opener 或系统 shell 打开默认浏览器搜索。

如果需要 Windows API：

- 仅在 Rust 侧封装。
- 控制 unsafe 范围。
- 给复杂 Win32 调用添加简短注释。

## 9. 性能要求

- 常驻状态不做轮询式高频扫描。
- 截图和 AI 请求只在用户触发时运行。
- 当前隐藏 `main` 窗口只承载设置页；划词菜单与文本/截图 AI 结果复用第 0 节的预创建窗口，避免在每次快捷键触发时反复创建 WebView。冻结的独立截图预览不恢复。
- 大图片处理必须避免阻塞 UI 线程。
- 不引入明显超出 MVP 需求的大型依赖。

## 10. 错误处理

必须覆盖：

- 快捷键被占用。
- 未选中文本。
- 选区读取失败。
- 剪贴板访问失败。
- 截图失败。
- API Key 缺失或错误。
- 网络请求失败。
- 模型不支持视觉输入。
- 请求超时。

错误提示要说清楚用户下一步能做什么。
