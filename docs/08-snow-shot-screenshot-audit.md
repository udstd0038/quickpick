# Snow Shot 截图参考实现审计

## 1. 审计结论

本轮只做源码结构和移植边界审计，不复制 Snow Shot 源码进入 QuickPick。

结论：

- Snow Shot 的截图能力不是单个函数，而是一条跨 Rust 命令、截图 crate、窗口/WebView、前端页面和事件的完整链路。
- QuickPick 首版不适合整包照搬 Snow Shot，因为 Snow Shot 包含标注、滚动截图、窗口识别、历史、录屏、插件和复杂前端页面。
- 可优先复刻的部分是“截图捕获命令、显示器/DPI 处理、图片写剪贴板、截图完成事件流”的最小链路。
- 白窗问题解决前，`Alt+3` 只允许执行原生 Win32 区域截图，仍不得恢复 Tauri/WebView 全屏遮罩或截图预览子窗口；托盘截图入口保持撤回。

## 2. 来源与版本

主参考仓库：

- 仓库：`https://github.com/mg-chao/snow-shot`
- 分支：`main`
- 审计 commit：`c7f2d9fe3114ad0dba6e5efdfe4bd8ecbc1f1de3`
- 审计日期：`2026-07-21`

截图相关自定义依赖：

- `https://github.com/mg-chao/xcap`，分支 `custom/master`，commit `d353731a19ae41b420ac28c7929eeb384c47bbf7`
- `https://github.com/mg-chao/scap`，分支 `20250712_custom`，commit `88914f1b708bfbbe8b281f198493ee06aae8484b`
- `https://github.com/mg-chao/device_query`，分支 `custom/master`，commit `7b9a56c543e3c3b3822b209ea379a5b693020d66`

这些依赖的顶层许可证已完成初步审计，但传递依赖和实际引入版本仍需在新增依赖前再次扫描。

## 3. 许可证状态

Snow Shot 仓库当前可见：

- `LICENSE-Commercial`：GPL-3.0。
- `LICENSE-NonCommercial`：Apache-2.0。
- GitHub 仓库侧栏显示 GPL-3.0 和 Apache-2.0 两类许可证。

当前处理原则：

- 不把本轮审计视为法律结论。
- 只要目标文件许可证或复用边界不清晰，就不直接复制代码。
- 如果后续决定复制源码，必须保留来源、许可证声明和修改记录。
- 如果许可证会影响 QuickPick 后续发布方式，则改为等价重写。

## 4. Snow Shot 截图链路地图

Rust 顶层命令入口：

- `src-tauri/src/screenshot.rs`
- 作用：把 Tauri command 暴露给前端，并委托给 `snow-shot-tauri-commands-screenshot`。
- 已看到的能力：当前显示器截图、所有显示器截图、焦点窗口截图、UI 元素初始化、窗口元素读取、鼠标位置读取、创建绘制窗口、设置绘制窗口样式、全屏截图、图片写入剪贴板。

Rust 截图核心 crate：

- `src-tauri/src-crates/tauri-commands/screenshot/`
- `src-tauri/src-crates/tauri-commands/screenshot/src/lib.rs`
- 作用：截图核心实现所在 crate。
- 依赖：`image`、`tauri`、`tokio`、`xcap`、`serde`、`log`、`rayon`、`app-shared`、`app-os`、`app-utils`、`webview`、`global_state`，Windows 下额外依赖 `webview2-com`、`windows-core`、`windows`。

Rust 系统能力支撑：

- `src-tauri/src-crates/app-os/src/ui_automation/`
- 作用：窗口元素、焦点窗口、UI 自动化相关能力。
- QuickPick MVP 首轮不需要完整智能窗口识别，可暂缓移植。

Rust WebView 支撑：

- `src-tauri/src-crates/webview/`
- 作用：Snow Shot 的窗口/WebView 交互支撑。
- QuickPick 当前正卡在 Tauri 子窗口白窗和无法关闭问题，必须先审计它的窗口创建方式，不能直接套用全屏遮罩。

前端截图命令封装：

- `src/commands/screenshot.ts`
- `src/functions/screenshot.ts`
- 作用：封装截图命令调用、截图事件触发、焦点窗口截图、完成截图和释放绘制页面事件。

前端截图页面：

- `src/pages/draw/`
- `src/pages/fullScreenDraw/`
- 作用：截图绘制页和全屏绘制页。
- 包含大量绘图、工具栏、标注和状态逻辑；QuickPick MVP 不整包移植。

## 5. QuickPick 可复刻范围

优先复刻：

- 显示器枚举和截图捕获流程。
- 多显示器和 DPI 坐标处理思路。
- 截图完成后的图片数据返回方式。
- 图片写入系统剪贴板的 Rust 侧处理方式。
- 前端通过事件触发截图、完成截图、释放截图页的流程。

谨慎参考：

- 创建截图绘制窗口的参数。
- WebView 与窗口透明、置顶、无边框、全屏之间的组合。
- HDR 颜色修正和多显示器合并图片。

首版暂缓：

- 智能窗口识别。
- UI Automation 元素缓存。
- 滚动截图。
- 标注工具。
- 贴图。
- 历史记录。
- 录屏。
- 插件化截图能力。
- Excalidraw 相关能力。

## 6. QuickPick 移植顺序

1. 继续保持区域遮罩和截图预览暂停。
2. 审计 Snow Shot 的 `create_draw_window`、`set_draw_window_style`、WebView 创建参数和关闭事件。
3. 在 QuickPick 主窗口诊断中新增“Snow Shot 截图链路审计状态”分组，只展示静态信息。
4. 先实现 Rust 侧只读截图能力草稿，不创建任何新窗口。
5. 完成非窗口单元验证后，再尝试普通非透明、非置顶、非全屏测试窗口。
6. 普通窗口可渲染并可关闭后，才进入小尺寸截图预览窗口。
7. 小尺寸窗口通过后，才进入非热键截图遮罩验证。
8. 最后才允许把 `Alt+3` 从原生 Win32 区域截图切换为 Tauri/WebView 区域遮罩。

## 7. 白窗风险提醒

QuickPick 已人工验证失败的入口：

- 截图遮罩。
- 截图安全预览。
- 静态窗口诊断。
- `settings` 普通窗口。

因此后续不能因为 Snow Shot 可用就直接创建同类窗口。必须先确认：

- capability 授权是否补齐。
- 子窗口 URL 是否加载正确。
- WebView 初始化是否失败。
- window-state 插件是否影响子窗口关闭。
- 标题栏关闭、任务栏关闭、`Esc`、托盘退出是否至少有一条稳定路径。

## 8. 窗口创建与关闭链路审计

审计来源：

- `src-tauri/src-crates/tauri-commands/screenshot/src/lib.rs:608-666`
- `src-tauri/src-crates/app-os/src/utils/windows.rs:55-65`
- `src/routes/_noLayout/draw.lazy.tsx`
- `src/pages/draw/page.tsx`
- `src/functions/screenshot.ts`
- `src/commands/core.ts`

静态结论：

- Snow Shot 使用运行时 `WebviewWindowBuilder` 创建 `draw-*` 窗口，窗口 URL 是 `/draw`，不是 `tauri.conf` 预创建窗口。
- `draw` 窗口创建参数包含无边框、透明、跳过任务栏、不可调整、默认不可见、默认不聚焦和 `1x1` 初始尺寸。
- Windows 下 `set_draw_window_style` 当前是占位函数，主要窗口行为来自创建参数和后续前端事件流。
- 创建后会禁用 DWM 转场并主动隐藏窗口，因此正常显示依赖 `/draw` 前端页面完成初始化。
- `/draw` 页面监听 `execute-screenshot`、`finish-screenshot` 和 `release-draw-page`；释放事件最终调用当前窗口关闭。
- 释放页面时还会创建新的绘制窗口并安排延迟关闭，属于 Snow Shot 的截图循环机制，QuickPick 当前不能直接复用。

QuickPick 风险判断：

- 如果 WebView 未加载到前端事件监听阶段，`release-draw-page` 的关闭兜底就不可用，风险形态接近 QuickPick 已遇到的白窗无法关闭。
- Snow Shot 的窗口参数比 QuickPick 已失败的普通 `settings` 窗口更激进，因此不能作为下一步直接恢复遮罩的依据。
- 后续应先做非窗口截图核心验证，或只做普通、非透明、非置顶、标题栏可关闭的小测试窗口。

## 9. 非窗口截图核心候选审计

审计来源：

- `src-tauri/src/screenshot.rs`
- `src-tauri/src-crates/tauri-commands/screenshot/src/lib.rs:16-46`
- `src-tauri/src-crates/tauri-commands/screenshot/src/lib.rs:48-136`
- `src-tauri/src-crates/tauri-commands/screenshot/src/lib.rs:138-178`
- `src-tauri/src-crates/tauri-commands/screenshot/src/lib.rs:676-802`
- `src-tauri/src-crates/app-utils/src/lib.rs:49-75`
- `src-tauri/src-crates/app-utils/src/lib.rs:200-245`
- `src-tauri/src-crates/app-utils/src/lib.rs:281-343`
- `src-tauri/src-crates/app-utils/src/lib.rs:545-585`
- `src-tauri/src-crates/app-utils/src/lib.rs:650-781`

静态结论：

- `capture_current_monitor` 会根据鼠标位置定位当前显示器，调用 `capture_target_monitor` 捕获图片，再编码为 PNG 或 WebP 字节返回。
- `capture_all_monitors` 会通过 `get_capture_monitor_list` 选择单屏或多屏，在 Windows 分支使用 `Rgba8`，并可走 SharedBuffer 优化。
- `capture_full_screen` 会裁剪当前活动显示器图片，但同时保存文件和截图历史图片，不符合 QuickPick 首版“不保存截图历史”的默认要求。
- `save_and_copy_image` 会并行执行保存文件和写入剪贴板，QuickPick 后续不能直接复用保存文件部分。
- `encode_image` 支持 PNG、WebP、JPEG 和 AVIF；QuickPick MVP 可先只保留 PNG，降低跨应用粘贴和预览风险。
- `write_bitmap_image_to_clipboard` 在 Windows 下把 PNG 解码成 RGBA，再写入 CF_DIB；QuickPick 可参考行为，但优先使用 Tauri 图片剪贴板或等价重写。

QuickPick 迁移边界：

- 可优先做一个只读 Rust 命令草稿：捕获当前显示器并返回 PNG 字节，不创建窗口、不保存文件、不复制剪贴板、不上传。
- `xcap`、`scap`、`device_query` 自定义分支顶层许可证已完成初步审计；传递依赖未扫描前不直接引入。
- 截图内容不得写入开发日志；测试时只记录是否成功、尺寸和错误类型。

## 10. 截图依赖许可证审计

审计来源：

- `xcap`：`https://github.com/mg-chao/xcap`，commit `d353731a19ae41b420ac28c7929eeb384c47bbf7`
- `scap`：`https://github.com/mg-chao/scap`，commit `88914f1b708bfbbe8b281f198493ee06aae8484b`
- `device_query`：`https://github.com/mg-chao/device_query`，commit `7b9a56c543e3c3b3822b209ea379a5b693020d66`

静态结论：

- `xcap` 的 `Cargo.toml` 标识 `license = "Apache-2.0"`，根目录 `LICENSE` 为 Apache License 2.0；未发现 `NOTICE`、`COPYRIGHT` 或 `AUTHORS` 文件。
- `scap` 的 `Cargo.toml` 标识 `license = "MIT"`，根目录 `LICENSE` 为 MIT License；未发现 `NOTICE`、`COPYRIGHT` 或 `AUTHORS` 文件。
- `device_query` 的 `Cargo.toml` 标识 `license = "MIT"`，根目录 `LICENSE` 为 MIT License；未发现 `NOTICE`、`COPYRIGHT` 或 `AUTHORS` 文件。
- 三者是 Snow Shot 使用的自定义分支或自定义 commit，后续不能直接假定与 crates.io 发布包完全一致。

QuickPick 处理原则：

- 顶层许可证未阻塞继续静态参考和后续小范围等价重写。
- 真正新增依赖前，仍要固定 commit 或版本、保留许可证文本，并补做传递依赖 license scan。
- 本轮不修改 `Cargo.toml`，不新增依赖，不复制源码，不恢复截图入口。

## 11. 只读截图命令草稿实施方案

目标：

- 只规划下一轮最小 Rust 截图命令草稿，不在本阶段新增依赖或实现截图。
- 后续命令候选名：`capture_current_monitor_png_draft`。
- 命令目标是捕获鼠标所在显示器并返回 PNG 字节、图片宽高和错误类型。

实施边界：

- 依赖策略：优先评估 `xcap` 和 `image` 的最小组合；真正新增前先做传递依赖 license scan。
- 权限边界：当前已接入 `Alt+3` 原生区域截图；托盘截图和主窗口截图按钮入口已撤回，仍不接入截图遮罩或截图预览窗口。
- 数据边界：返回 PNG 字节和宽高；测试日志只记录成功状态、尺寸、字节长度和错误类型。
- 隐私边界：内存处理，不保存文件，不写剪贴板，不上传 AI，不保存截图历史。
- 回退边界：如果编译、权限或人工测试失败，立即撤回命令入口并保留主窗口静态诊断。

验证顺序：

1. 先只新增依赖和未暴露的 Rust 模块草稿。
2. 通过 `cargo fmt`、`cargo check` 和 `npm run build`。
3. 再加主窗口内显式测试按钮，按钮文案必须说明会读取当前屏幕。
4. 人工测试只记录尺寸和状态，不记录图片内容。
5. 通过后才考虑下一步图片复制或预览。

## 12. 最小截图依赖引入与扫描方案

目标：

- 只规划后续依赖引入顺序，不在本阶段修改 `Cargo.toml`。
- 第一候选组合为 `xcap + image`，用于后续当前显示器 PNG 截图草稿。
- 暂不引入 `scap`、`device_query`、SharedBuffer、录屏或窗口识别相关能力。

扫描要求：

- 新增依赖后先运行 `cargo tree`，记录新增传递依赖数量和关键包。
- 使用可用的许可证扫描工具检查传递依赖；如果本地没有工具，先记录阻塞原因，不继续暴露截图命令。
- 对 Apache-2.0、MIT、BSD 等常见宽松许可证保留来源记录；遇到 GPL、AGPL、未知或自定义许可证时暂停。
- 如果使用 git commit 而非 crates.io 版本，必须记录仓库、commit、许可证文件和回退方案。

回退线：

- 如果依赖导致编译失败、体积增长不可接受、许可证边界不清或 Windows 行为异常，撤回依赖。
- 撤回后保留主窗口静态诊断；只允许保留无窗口截图复制，不恢复任何子窗口。

## 13. 传递依赖扫描执行方案

工具现状：

- 已确认 `cargo tree` 可用。
- `cargo-deny` 当前未安装。
- `cargo-about` 当前未安装。

执行顺序：

1. 新增依赖前记录当前 `Cargo.toml` 和 `Cargo.lock` 状态。
2. 新增候选依赖后先跑 `cargo tree --locked --prefix depth`。
3. 优先使用 `cargo-deny check licenses` 或 `cargo-about generate` 做 license scan；工具缺失时停止，不暴露截图命令。
4. 记录包名、版本、来源、许可证、是否新增、处理结论、阻塞原因。
5. 遇到 GPL、AGPL、未知、自定义、无 LICENSE 或来源无法确认，暂停并撤回依赖。

边界：

- 本阶段不安装扫描工具，不联网下载依赖，不修改 `Cargo.toml`，不注册截图命令，不读取屏幕。

## 14. 许可证扫描工具选择与离线替代方案

工具选择：

- 首选 `cargo-deny check licenses`，用于形成可复现的阻塞判断。
- `cargo-about generate` 可作为许可证报告补充，但不是继续新增依赖的唯一前置条件。
- 在本机尚未安装上述工具时，允许先采用离线人工兜底方案。

离线兜底：

1. 运行 `cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-pc-windows-msvc`，确认 Windows 目标依赖元数据可在本地解析。
2. 运行 `cargo tree --locked --prefix depth`，记录新增依赖树。
3. 对照 `Cargo.lock` 和本地 Cargo registry 源码目录，核对新增包的 `license` 字段、`license-file` 字段和 LICENSE 文件。
4. 记录包名、版本、来源、许可证、是否新增、处理结论和阻塞原因。
5. 如果本地源码缺失、许可证字段不一致、LICENSE 文件缺失或无法确认来源，暂停新增依赖。

安全边界：

- 离线兜底只读本地文件，不联网、不安装扫描工具、不读取屏幕、不注册截图命令。
- 不带 `--filter-platform x86_64-pc-windows-msvc` 的完整 metadata 可能触发其他平台未缓存包的离线解析失败，不能作为 QuickPick Windows 审计命令。
- 后续若安装 `cargo-deny` 或 `cargo-about`，必须先更新本审计记录，再以自动扫描结果替换人工兜底结论。

## 15. 最小截图依赖只读引入状态

本阶段只新增依赖，不新增截图模块、不注册命令、不读取屏幕。

已引入依赖：

- `xcap = { version = "0.9.7", default-features = false }`
- `image = { version = "0.25", default-features = false, features = ["png"] }`

来源与许可证：

- `cargo info xcap` 显示 `xcap 0.9.7` 许可证为 Apache-2.0，仓库为 `https://github.com/nashaofu/xcap.git`。
- 本地 Cargo 缓存中的 `xcap-0.9.7` 声明 `license = "Apache-2.0"`，并包含 `LICENSE` 文件。
- `image 0.25.10` 在 metadata 中声明 `MIT OR Apache-2.0`。

Windows 目标审计：

- `cargo tree --locked --target x86_64-pc-windows-msvc -p xcap --prefix depth` 成功。
- Windows 目标下 `xcap` 传递树共 39 个包。
- 通过 `cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-pc-windows-msvc` 抽取的 Windows 目标包许可证未发现 GPL、AGPL 或未知许可证。

已知边界：

- `Cargo.lock` 会锁定 `xcap` 的跨平台解析项，包含 Linux/macOS 等非 Windows 目标依赖；QuickPick 当前审计和构建以 Windows 11 x64 为目标。
- 初次离线解析 `xcap` 时，因为本地索引缺少 `libwayshot-xcap`，`cargo check --offline` 会失败；联网补齐 crates.io 索引和缓存后，普通 `cargo check` 已通过。
- 该历史阶段没有新增 `src-tauri/src/screenshot/` 模块，没有注册 Tauri `invoke`，没有恢复 `Alt+3` 或托盘截图。

## 16. 未暴露 Rust 截图模块骨架

本阶段只新增 Rust 模块边界，不新增真实截图调用。

已新增文件：

- `src-tauri/src/screenshot/mod.rs`

模块内容：

- `CaptureRegion`：区域截图坐标和尺寸结构。
- `CaptureTarget`：当前显示器或指定区域的目标枚举。
- `CaptureDraftRequest`：后续截图草稿请求结构。
- `CaptureDraftResult`：后续截图草稿结果结构。
- `ScreenshotScaffoldStatus`：主窗口诊断使用的静态状态。
- `disabled_capture_result`：当前只返回禁用状态和空 PNG 字节。

安全边界：

- 未调用 `xcap` 捕获 API。
- 未注册 Tauri `invoke` 命令。
- 未读取屏幕。
- 未写剪贴板。
- 未创建 overlay 或 preview 子窗口。
- 该历史阶段未恢复 `Alt+3` 或托盘截图入口。

验证重点：

- `screenshot` 模块可以随 Rust crate 编译。
- 单元测试只验证禁用状态和空结果，不读取屏幕。
- 主窗口诊断只读取 `scaffold_status()` 和禁用结果。

## 17. 原生区域截图剪贴板 MVP

本阶段从当前显示器复制推进到区域框选截图，但仍绕开所有已验证失败的 Tauri/WebView 子窗口路径。

实现范围：

- 保留 `capture_current_monitor_to_clipboard` 作为历史命令边界。
- 新增并注册 `capture_region_to_clipboard` Rust 命令。
- `Alt+3` 全局热键调用原生 Win32 区域框选。
- 托盘新增“区域截图”入口。
- 主窗口新增“区域截图”按钮和截图状态提示。
- 框选后显示“复制、提取、翻译”横条菜单。
- “复制”写入剪贴板；“提取”和“翻译”调用 OpenAI 兼容视觉模型，并在独立原生 Win32 结果弹窗显示结果。
- 主窗口新增安全 AI 配置区，避免打开已暂停的 `settings` 子窗口。
- 主窗口诊断新增“原生区域截图剪贴板 MVP”分组。

实现边界：

- 使用 `windows-sys` 创建临时 Win32 覆盖层，不使用 Tauri/WebView 子窗口。
- 使用 crates.io `xcap 0.9.7` 枚举显示器并捕获用户选中的区域图像。
- 使用 `image::RgbaImage` 在内存中转换为 BMP/DIB 数据。
- 使用 `clipboard-win` 写入 `CF_DIB + CF_BITMAP` 剪贴板格式。
- 使用 `image` PNG 编码和 data URL 传递给用户配置的视觉模型。
- 不保存截图文件。
- 只有用户点击“提取”或“翻译”时才上传 AI；配置不完整时不读取截图、不上传。
- 不记录截图内容。
- 不创建 Tauri/WebView overlay、preview、settings、result 或任何新的子窗口。

与 Snow Shot 的关系：

- 本阶段只复刻“区域框选 + 图片写剪贴板”的行为边界。
- 未复制 Snow Shot 源码。
- 未引入 Snow Shot 的 draw 窗口、标注、历史、录屏、插件或滚动截图链路。

验证结果：

- `cargo fmt` 成功。
- `cargo check` 成功。
- `cargo test` 成功，5 个截图模块单元测试通过。
- `npm run build` 成功。
- `npm run tauri:build -- --no-bundle` 成功，产物路径为 `src-tauri/target/release/quickpick.exe`。
- 人工验收通过：按 `Alt+3` 后可拖拽框选，松开后能看到横条菜单，点击“复制”后可粘贴图片。

剩余边界：

- 当前不是截图预览窗口，AI 结果显示在独立原生 Win32 结果弹窗。
- 视觉模型兼容性仍需人工使用真实 API 配置验收。
- Tauri/WebView 区域遮罩和截图预览子窗口继续暂停。
- 后续恢复 Tauri/WebView 预览窗口前，仍必须先解决 Tauri 子窗口白窗和关闭路径问题。

## 18. 下一步

主窗口内静态诊断扩展已完成并通过人工确认：

- 新增“Snow Shot 截图链路审计状态”分组。
- 新增“Snow Shot 窗口创建与关闭链路审计”分组。
- 新增“Snow Shot 非窗口截图核心候选审计”分组。
- 新增“Snow Shot 截图依赖许可证审计”分组。
- 新增“只读截图命令草稿实施方案”分组。
- 新增“最小截图依赖引入与扫描方案”分组。
- 新增“传递依赖扫描执行方案”分组。
- 新增“许可证扫描工具选择与离线替代方案”分组。
- 新增“最小截图依赖引入状态”分组。
- 新增“原生区域截图剪贴板 MVP”分组。
- 展示来源 commit、候选模块、暂缓模块、依赖审计状态和安全闸门。
- 点击“刷新诊断”不会打开任何新窗口。
- 只恢复原生 Win32 区域框选和复制，不恢复 Tauri/WebView 遮罩或预览子窗口。

人工确认：

- 已确认主窗口诊断能看到“Snow Shot 非窗口截图核心候选审计”分组。
- 已确认主窗口诊断能看到“Snow Shot 截图依赖许可证审计”分组。
- 已确认主窗口诊断能看到“只读截图命令草稿实施方案”分组。
- 已确认主窗口诊断能看到“最小截图依赖引入与扫描方案”分组。
- 已确认主窗口诊断能看到“传递依赖扫描执行方案”分组。
- 已确认主窗口诊断能看到“许可证扫描工具选择与离线替代方案”分组。
- 已确认主窗口诊断能看到“最小截图依赖只读引入状态”分组。
- 已确认主窗口诊断能看到“未暴露 Rust 截图模块骨架”分组。
- 已确认原生区域截图中 `Alt+3`、拖拽框选、横条菜单和“复制”粘贴链路可用。
- 已确认点击“刷新诊断”不会弹出任何新窗口。

后续下一步：

- 继续保持 Tauri/WebView 区域遮罩和预览子窗口暂停。
- 下一步优先把横条菜单中的“提取”和“翻译”接入 OCR/AI 结果展示。
