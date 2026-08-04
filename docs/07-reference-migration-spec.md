# QuickPick 参考代码移植规范

## 1. 适用范围

本规范用于后续从参考项目复刻或移植可用实现，当前只允许用于截图模块。

当前截图方向：

- 放弃继续自研截图核心实现。
- 优先参考 Snow Shot 中已经可用的截图、预览、OCR/AI 串联思路。
- 首版只复刻 QuickPick MVP 需要的区域截图、预览和复制图片链路。

不进入首版：

- 滚动截图。
- 截图标注。
- 贴图。
- 历史记录。
- 录屏。
- 插件系统。

## 2. 已知参考来源

主要参考项目：

- Snow Shot：`https://github.com/mg-chao/snow-shot`
- 当前截图审计记录：`docs/08-snow-shot-screenshot-audit.md`

截至 2026-07-21 已查到的许可证信息：

- 仓库页面显示存在 GPL-3.0 和 Apache-2.0 两类许可证。
- `LICENSE-Commercial` 内容为 GPL-3.0。
- `LICENSE-NonCommercial` 内容为 Apache-2.0。
- `xcap` 锁定 commit 顶层许可证为 Apache-2.0。
- `scap` 锁定 commit 顶层许可证为 MIT。
- `device_query` 锁定 commit 顶层许可证为 MIT。

后续复制或移植任何代码前，必须再次以目标 commit 的仓库文件为准确认许可证，不能只依赖旧记录。

## 3. 引入前必须完成

每次引入参考代码前，必须先记录：

- 来源仓库 URL。
- 来源 commit、tag 或 release 版本。
- 计划复用的源文件路径。
- 计划复用的函数、模块或行为边界。
- 对应许可证和需要保留的版权声明。
- QuickPick 内的目标文件路径。
- 与原实现的差异。
- 回退方案。

如果许可证边界不清楚，只允许学习实现思路，不直接复制代码。

## 4. 移植方式

优先级：

1. 许可证明确兼容且范围清楚时，做小范围移植，并保留来源记录和必要声明。
2. 许可证不清楚或会影响 QuickPick 发布方式时，只做等价重写。
3. 只移植截图 MVP 所需路径，不连带引入标注、历史、插件、录屏等大模块。

移植后的代码必须隔离在清晰模块内，避免散落到无关逻辑中。

Rust 侧建议边界：

- `src-tauri/src/screenshot/`
- 截图捕获。
- 区域选择数据结构。
- 图片复制到剪贴板。

最小依赖引入原则：

- 第一候选只评估 `xcap` 与 `image` 的组合。
- 暂不引入 `scap`、`device_query`、SharedBuffer 或录屏相关能力。
- 新增依赖后必须记录 `cargo tree` 和传递依赖许可证扫描结果。
- 本机当前可用 `cargo tree`；`cargo-deny` 和 `cargo-about` 未安装时，必须完成离线许可证兜底记录后才可暴露截图命令。
- 传递依赖记录至少包含包名、版本、来源、许可证、是否新增、处理结论和阻塞原因。
- 首选 `cargo-deny check licenses` 做阻塞判断，`cargo-about generate` 只作为许可证报告补充。
- 自动扫描工具缺失时，只允许采用 `cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-pc-windows-msvc`、`cargo tree --locked --prefix depth` 和本地 Cargo registry 源码 LICENSE 文件做离线人工兜底。
- 不带 `--filter-platform x86_64-pc-windows-msvc` 的完整 metadata 可能触发非 Windows 目标依赖的离线缺包问题，不作为 QuickPick Windows 审计命令。
- 离线人工兜底发现本地源码缺失、许可证字段不一致、LICENSE 文件缺失或来源无法确认时，必须暂停新增依赖。
- 如果许可证、体积、编译或 Windows 行为不可接受，必须立即撤回依赖。
- 截至 2026-07-22，QuickPick 已显式引入 `xcap = { version = "0.9.7", default-features = false }` 和 `image = { version = "0.25", default-features = false, features = ["png"] }`。
- `xcap 0.9.7` 来源为 `https://github.com/nashaofu/xcap.git`，crates.io 和本地缓存均声明 Apache-2.0，本地包包含 `LICENSE` 文件。
- Windows 目标下 `xcap` 传递树共 39 个包，metadata 未发现 GPL、AGPL 或未知许可证。
- 当前已完成依赖引入和离线许可证兜底，可暴露原生区域截图复制命令。
- 截至 2026-07-22，QuickPick 已在 `src-tauri/src/screenshot/mod.rs` 中实现原生区域截图剪贴板 MVP。
- 该 MVP 只允许在用户触发 `Alt+3` 时进入原生 Win32 框选流程；托盘“区域截图”和主窗口截图按钮入口已撤回。
- 只有用户完成框选并点击横条菜单“复制”后，才读取所选区域并把图片以 `CF_DIB + CF_BITMAP` 写入剪贴板。
- 该 MVP 不保存截图文件、不上传 AI、不创建 Tauri/WebView overlay 或 preview 子窗口。

前端侧建议边界：

- `src/windows/screenshot_overlay`
- `src/windows/screenshot_preview`
- 当前白窗阶段不恢复上述 Tauri/WebView 窗口；截图 AI 结果走独立原生 Win32 结果弹窗，AI 配置仍放在主窗口内。
- 后续恢复预览窗口时，仅保留框选、取消、预览、复制、提取和翻译按钮。

## 5. 安全闸门

白窗和无法关闭问题完全解决前：

- `Alt+3` 只允许执行原生 Win32 区域截图，不允许创建 Tauri/WebView 截图遮罩；托盘截图入口保持撤回。
- 原生 Win32 临时框选层必须具备 `Esc`、右键和超时退出路径。
- 不打开新的截图预览子窗口。

恢复验证顺序：

1. 主窗口内静态审计通过。
2. 普通非透明、非置顶、非全屏测试窗口可渲染并可关闭。
3. 小尺寸截图预览窗口可渲染并可关闭。
4. 非热键截图入口可取消、可关闭。
5. 最后才允许把 `Alt+3` 从原生 Win32 区域截图切换为 Tauri/WebView 区域遮罩。

任何一步失败，必须撤回入口并记录日志。

## 6. 验收要求

引入参考代码后必须验证：

- `cargo fmt` 成功。
- `cargo check` 成功。
- `npm run build` 成功。
- 原生 Win32 区域截图不创建 Tauri/WebView 子窗口，构建验证必须通过。
- 后续恢复任何截图遮罩或预览窗口前，`Esc`、窗口关闭按钮、任务栏关闭或托盘退出至少有一条可靠退出路径。
- 不保存截图历史。
- 不自动上传截图。
- 不把截图内容写入开发日志。

## 7. 日志要求

每天开发日志必须记录：

- 是否引入了参考代码。
- 引入来源和 commit。
- 实际改动文件。
- 许可证处理结果。
- 验证结果。
- 是否保持截图遮罩和预览子窗口暂停。
