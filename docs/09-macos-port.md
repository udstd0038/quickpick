# QuickPick macOS 移植说明

## 状态

macOS 移植的目标开发分支为 `feature/macos-port`，开发版本为 `0.1.4`。第一、二、三阶段的原提交均已取得各自 CI 结果；阶段整合候选在 `codex/macos-integration-20261021` 按 PR #2 → #3 → #4 顺序保留祖先关系整合，取得新 CI 后再推进目标开发分支。本工具补验候选包含第四阶段 PR #5 的工具实现，并接入上述阶段整合祖先；PR #5 仍为草稿，待本次最终提交的 CI 及 Windows 11 交互验收，尚未合入目标开发分支。实体 Mac 验收继续延期：

- Windows 保留默认快捷键与 DPAPI/NSIS 流程，第一阶段 Windows 回归已通过。
- macOS 使用 Tauri 2、React、xcap 和 Rust 原生/官方插件能力。
- 已发布 `v0.1.3-macos` universal DMG 预发布包；该包未签名、未公证，不能代表最新分支代码。
- 实体 Mac 曾发现快捷键无法录入及默认快捷键不触发，后续修复仍需使用对应新提交的产物复验；完整功能验收尚未完成。
- 第一阶段验收提交为 [`2ab5a52`](https://github.com/udstd0038/quickpick/commit/2ab5a52)，[CI run `38381632543`](https://github.com/udstd0038/quickpick/actions/runs/38381632543) 已通过 Windows 回归、macOS 双架构检查、原生 Rust 测试、universal DMG 构建及产物验证。
- 该 PR 构建实际 checkout 的合并提交为 `bd1689bed6f1e8789de76a82bcd10f3eca711430`，验收产物为 `quickpick-macos-universal-bd1689bed6f1e8789de76a82bcd10f3eca711430`。源码提交与产物合并提交分别记录，后续更改需取得对应新提交的验证结果。
- 第二阶段源码为 [`e48f45592f40a2eb167ac4333dc5e7aff124ae84`](https://github.com/udstd0038/quickpick/commit/e48f45592f40a2eb167ac4333dc5e7aff124ae84)，[CI run `38440004266`](https://github.com/udstd0038/quickpick/actions/runs/38440004266) 通过：前端 46/46、Windows Rust 54/54、macOS Rust 70/70，Windows 回归、macOS 双架构检查与 universal DMG 构建/架构验证均完成。[安全审计 run `38440004418`](https://github.com/udstd0038/quickpick/actions/runs/38440004418) 通过。
- 第二阶段 PR 实际验证的合并提交为 `d536ff8679a1b95b225c59ce5e9bfd1d64f40b7e`；[artifact `11835018469`](https://github.com/udstd0038/quickpick/actions/runs/38440004266/artifacts/11835018469) 对应该合并提交，未签名，仅用于验收。私有剪贴板 round-trip 验证原生恢复，模拟后端验证时序与 Keychain 错误；这些结果不能代替用户桌面的完整实机验收。
- 第三阶段源码为 [`4e6d3ec46a54d61f3a0dc5f051621f45599cf61c`](https://github.com/udstd0038/quickpick/commit/4e6d3ec46a54d61f3a0dc5f051621f45599cf61c)，源码树为 `51557c4416e5fa39a548a3932446917091679032`。[CI run `38498994618`](https://github.com/udstd0038/quickpick/actions/runs/38498994618) 通过：两平台前端各 141/141、Windows Rust 66/66、macOS Rust 86/86、Windows 回归、macOS ARM/Intel check 及 universal DMG 构建/lipo 验证。[安全审计 run `38498994568`](https://github.com/udstd0038/quickpick/actions/runs/38498994568) 通过。
- 第三阶段 PR 实际验证的合并提交为 `4906125a4072e832807f130417101f9bfae9e131`；[artifact `11865053097`](https://github.com/udstd0038/quickpick/actions/runs/38498994618/artifacts/11865053097) 对应该提交，未签名，保留至 2026-10-28。代码与自动化验收通过，不计作权限、Retina、多屏、Spaces/全屏的真实设备验收通过。
- 阶段整合在补依赖前的提交为 `6414ba110b445394703836397fed9dad10e83598`，源码树与第三阶段原源码完全一致。与该阶段源码相比，本次应用锁文件精确更新 `rustls 0.23.45 → 0.23.46`、`rustls-webpki 0.103.15 → 0.103.16` 和 `webpki-root-certs 1.0.9 → 1.0.10` 的版本及校验和，其他包与全部依赖边保持不变。
- 上述更新分别处理 2026-10-14 公布的 RUSTSEC-2026-0346/0347/0348/0349，以及 10 月 19/20 日公布的证书信任公告 RUSTSEC-2026-0382/0383；新增校验和来自 crates.io 官方索引。旧审计成功不能证明当前安全审计通过，最终候选仍待独立原生 CI 和当前依赖审计。
- 用户目前没有可用的实体 Mac；第一阶段通过说明构建与自动化基线可用，完整原生行为和手动检查保持待验收。
- 当前仅推进开发链整合与工具补验，CI 产物用于验收，不自动发布 Release。本次最终整合、依赖更新和 Windows 验收 CI 尚未取得完整新结果，不把历史结果记作本次提交通过。
- 本工具候选已删除原五个 PowerShell 脚本，审计、资源生成、快捷键验收与 README 截图入口使用 Node.js/Rust；不能据此宣称目标开发分支或默认 `main` 已完成迁移。PR #5 原源码 `addbbd26ea08c903c82d1c6286fd9ecc6a2b381b` 的独立 CI 已通过资源、helper、fixture 和原生编译检查，Windows 交互动作仍为 `NOT RUN`；本次新增的默认 release/NSIS 和独立桌面验收包须取得最终提交的新运行记录。

## 平台差异

### 目标系统与快捷键

- macOS 默认设置快捷键为 `Command+Comma`，划词/截图/输入翻译为 `Command+Option+2/3/4`。
- Rust 设置归一化在 macOS 要求 Command 或 Ctrl；Option-only 及 Shift+Option-only 旧配置回退到平台默认值，兼容 macOS 15+ 的全局快捷键限制。Windows 仍要求 Alt。
- 前端录制和展示使用 Command/Option 名称，Windows 继续使用 Alt/Ctrl/Win。

### API Key

- Windows 继续使用 DPAPI 文件。
- macOS 使用 Keychain，service 为 `com.quickpick.desktop`，account 为 `text`、`vision`、`input`。
- Key 不写入仓库、日志、设置文件或 CI 产物。

### 划词

- Windows 使用 UIAutomation 优先，失败后按用户设置使用剪贴板兜底。
- macOS 使用 `axuielement` 读取 `AXSelectedText`；授权缺失时请求辅助功能权限。
- 用户开启剪贴板兜底时，macOS 才模拟 `Command+C`。第二阶段已使用 NSPasteboard 全 item/类型快照、实际复制完成判定、触发键释放和失败清理，并通过对应 CI；真实应用取词与系统权限行为保持待实机验收。
- 第二阶段以 `NSPasteboard.changeCount` 检测剪贴板变化，复制失败或超时不得把旧文本标为成功；复制相同文本也应以实际变更事件判断，不能只比较字符串。
- 恢复前发现并发修改时跳过恢复，保留新剪贴板并给出可理解提示。`NSPasteboard` 没有 compare-and-swap（CAS）恢复接口：最后一次检查 `changeCount` 与 `clear` 之间仍有竞争窗口，不能保证零竞争，也不能仅凭计数证明变化来源。自动化验证已检测并发时的降级策略，真实系统中的剩余竞争风险必须如实记录。
- 第三阶段已补充 macOS 15.4+ 的剪贴板访问保护并通过代码与 CI：可查询的访问策略为 `AlwaysDeny` 时，必须在读取快照或注入复制前安全停止；`pasteboardItems` 返回 nil 且无法可靠区分拒绝访问与空板时不能视为空快照，不能继续复制或恢复。真实访问策略仍待设备验收，不表示已在实体 Mac 上复现数据丢失。

### 截图与剪贴板

- 截图捕获继续复用 xcap，前端仍使用 WebView 框选。
- Windows 继续写 CF_DIB/CF_BITMAP；macOS 将 RGBA 图片交给剪贴板插件写入系统剪贴板。
- macOS 截图前检查屏幕录制权限；第三阶段已接入明确的授权操作、拒绝提示与重试，不在设置页挂载时自动弹授权。
- 第三阶段优先选择鼠标所在显示器，让遮罩预览与最终裁剪复用同一次捕获的 RGBA 会话。选区坐标相对图像像素，不能混入桌面原点或缩放后的逻辑坐标；macOS 显示器 bounds 为点，Windows bounds 为物理像素，转换以图像和实际显示器尺寸为依据。
- 第三阶段在取消、失败、再次捕获和会话过期时清理图像；已经失效的后台任务晚完成不能重新填回旧图像。预览、复制与 AI 裁剪不得重新截取已经包含遮罩的桌面，不写截图文件。单屏 Retina、外屏、负坐标和混合缩放保持待实机验收。

### 系统集成

- macOS 使用菜单栏 template 图标、`ActivationPolicy::Accessory` 和 Tauri 原生菜单。
- 设置页使用原生标题栏与交通灯，隐藏自绘窗口按钮。
- 开机自启通过 `tauri-plugin-autostart` LaunchAgent 实现。
- 深浅主题、玻璃色板和布局 token 仍由前端共享变量控制，与 Windows 页面保持一致。
- macOS 原生玻璃效果实际使用 Popover/ContentBackground；第三阶段已按平台展示实际外观名称，Windows 继续保留对应名称与效果。
- 第三阶段将 macOS 截图遮罩定位为覆盖选定显示器的普通无边框窗口，使用 `CanJoinAllSpaces`/`FullScreenAuxiliary` 行为，不以应用原生全屏制造新 Space；代码与 CI 已通过，Spaces/全屏应用中的可见性、取消及焦点恢复留待实机检查。

### 权限引导（第三阶段代码与 CI 已通过，实机待验收）

- 设置页按辅助功能、屏幕录制、输入监控三种用途展示状态和所支持的动作；首次挂载只查询，查询失败不能显示为已授权。
- 用户明确点击后才请求对应权限，请求返回后重新查询；不能把请求方法正常返回等同于授权成功。
- “打开系统设置”仅打开 Rust 端固定映射的 Privacy 页面 URL，不接受任意 URL 或 shell 参数。
- 从系统设置返回应用时刷新状态，并提供手动复查；拒绝、撤销、需要重启或暂时无法查询时给出对应可执行提示，不自动反复请求权限。
- 三种权限状态各自独立；权限引导本身不读取选区、剪贴板或屏幕，也不发起 AI 请求。

## CI

`.github/workflows/macos-build.yml` 在 push、pull request 和手动运行时提供：

- Windows：显式使用 `cmd`，执行前端测试/构建、`cargo check --locked` 和 `cargo test --locked`。
- macOS：显式使用 `bash`，执行前端测试/构建、ARM 与 Intel 的 `cargo check --locked`、Runner 原生架构的 `cargo test --locked`，以及带 `--locked` 的未签名 universal DMG 构建。原生测试既支持 ARM Runner，也支持 Intel Runner；另一架构的运行行为仍需实机验收。
- CI 显式将 `CARGO_TARGET_DIR` 固定为 `apps/desktop/src-tauri/target`，避免仓库开发用 `target-monorepo-check` 配置与产物路径不一致；打包后用 `lipo` 验证 ARM/Intel 两架构。
- pnpm 使用 `--frozen-lockfile`；Rust 与 Tauri 构建使用 `--locked`，防止 CI 静默改写依赖锁文件。
- DMG 与 `build-info.txt` 上传为 `quickpick-macos-universal-<完整 github.sha>`，保留 14 天。验收记录必须写入该提交和对应 workflow run。PR 的 `github.sha` 是 GitHub 为该 PR 生成的合并提交，与 checkout 的验证代码对应。

第四阶段新增 `.github/workflows/development-tools.yml`：Ubuntu/Bash 生成并验证临时资源、测试 helper、检查 Windows 目标 API；Windows/cmd 测试 opt-in fixture 并构建内嵌前端的 `quickpick-fixture.exe`，尝试真实注册快捷键与 owned 窗口导出。交互宿主不可用或 WebView 图片不可读时工具返回 2，Summary 记为 `NOT RUN`；此状态不能算交互验收通过。安全审计同时覆盖 `tools/native-helper/Cargo.lock`。工具命令与隔离规则见 `docs/06-dev-workflow.md` 第 9 节。

本次工具补验另增加不启用 `dev-tools` 的默认应用 Rust 测试、Windows release 与 NSIS 构建，以及记录源码身份和 SHA256 的 `build-info.json`。工作流还准备平铺的专用 fixture、原生 driver 和源码提交记录，供 Windows 11 前台终端直接执行真实快捷键及固定界面导出；不需要安装 Node.js、pnpm 或 Rust。生成与上传验收包不等于真实桌面动作通过，完整命令与验证边界见 `docs/06-dev-workflow.md`。

push 和 pull request 始终使用 `windows-latest`、`macos-latest`。手动 `workflow_dispatch` 默认也使用托管 Runner；只有明确选择 `self-hosted` 时，才分别使用同时带有 `self-hosted` + `Windows` 或 `self-hosted` + `macOS` 标签的 Runner。自托管 Windows 需要可用的 `cmd` 和 Rust/MSVC 工具链，macOS 需要可用的 `bash`、Xcode 工具链及 DMG 打包工具。Runner 还需能访问本工作流使用的 Actions 和依赖源。自托管模式仅用于手动运行，不接收 push/PR 自动任务。

GitHub 的手动触发入口要求此工作流同时存在于仓库默认分支。仅合入 macOS 功能分支时，不能保证 Actions 页面已提供该入口；需要先将工作流纳入默认分支，再手动选择待验证分支和 Runner。PR 自动验证不依赖手动入口。

历史 CI 曾在 `8db8485` 和已发布包对应的 `c6bf3b3` 上成功。快捷键修复后，日志记载当时遇到 Actions 付款/额度阻塞；本次审查时 `feature/macos-port` 的 `9354796` CI 失败且没有执行验证步骤。这些历史结果都不能作为本轮最新提交通过验证的证据，也不能证明当前失败原因仍是付款问题。

第一、二、三阶段原提交已经通过上文记录的各自完整 CI。本次阶段整合、工具补验及三项依赖更新仍需针对最终提交执行前端、Windows 回归、默认 Windows release/NSIS、macOS 双架构检查、原生 Rust 测试、开发工具编译、universal 打包和当前依赖安全审计；尚未取得本次完整新结果。受控后端与时序的自动化用例用于验证逻辑，不能代替真实 Mac 的 Accessibility、键盘事件、NSPasteboard、Keychain、显示器和窗口行为，也不能将工具编译视为交互式 Windows 桌面验收通过。详细条目见 `docs/05-testing-acceptance.md` 的分阶段清单。

## 后续分步计划与验收门槛

1. **统一开发基线与 CI（已通过）**：验收提交 `2ab5a52`，CI run `38381632543`；Windows 回归、macOS 双架构 check、原生测试、universal DMG 及架构/产物验证已完成。
2. **取词与 Keychain 正确性（代码与 CI 已通过）**：源码 `e48f45592f40a2eb167ac4333dc5e7aff124ae84`，CI run `38440004266`，安全审计 run `38440004418`。剪贴板事务、按键清理、Keychain 错误处理及设置页未知状态已取得自动化结果；真实 Mac 手动检查保留待执行，不提前标为通过。
3. **权限与多屏截图体验（代码与 CI 已通过）**：源码 `4e6d3ec46a54d61f3a0dc5f051621f45599cf61c`，CI run `38498994618`，安全审计 run `38498994568`。权限引导、同帧截图会话、取消/过期清理、macOS 窗口行为、平台外观名称及 `AlwaysDeny`/歧义 nil 防护取得独立自动化结果；Retina、外接屏、混合缩放、负坐标、TCC、Spaces/全屏继续待实机验收。
4. **移除剩余 PowerShell 工具（工具候选已实现，PR #5 仍为草稿待补验）**：快捷键验收、README 截图、图标与安装器资源生成已替换为 `pnpm verify:hotkeys`、`pnpm capture:readme`、`pnpm tools:icons`、`pnpm tools:installer-assets`，加上第一阶段 `pnpm security:audit`，本工具候选原五脚本已删除并同步现行规范，尚未合入目标开发分支。资源生成复用现有工具链；快捷键验证真实 `SendInput` 和对应窗口，README 截图使用隔离的专用 `dev-tools` fixture。`pnpm test:tools` 检查真实生成资源，现有已验收二进制资源保留；最终候选须取得完整独立 CI、默认 Windows release/NSIS 及 Windows 11 真实快捷键和固定界面采集结果。退出码 2 表示 `NOT RUN`，不能以其他构建成功替代。
5. **实体 Mac 完整验收与正式分发**：使用准确提交的产物复验四组快捷键、中文输入法、常见应用取词、截图粘贴、Keychain、菜单栏、LaunchAgent、Spaces/全屏及深浅主题，记录 Apple Silicon/Intel 与系统版本及每项结果。完整功能验收通过后配置 Developer ID 签名、公证和 staple，验证 DMG 安装、卸载、升级与登录自启，再发布正式版本。自动更新尚未实现，另立后续阶段。

用户已确定：各阶段代码与 CI 按 1 → 2 → 3 → 4 → 5 的顺序推进，每阶段自动化验证通过后再推进下一阶段；Mac 实体验收延后，不因当前缺少设备阻挡第四阶段。待验收的实机项目独立保留，不能标为通过；第五阶段正式分发仍须完成真实设备验收及签名、公证验证。

签名和公证需要配置 Apple 证书及对应 CI secrets；本轮 CI 不依赖这些凭据，也不发布 GitHub Release。

## 第五阶段执行顺序（等待真实 Mac）

1. **确定支持范围并冻结候选**：根据 Tauri/依赖能力明确最低支持 macOS 版本，再配置发布要求；准备 Apple Silicon、Intel、Retina/外屏及 macOS 15.4+ 访问策略测试条件。记录源码 SHA、PR 构建合并 SHA、workflow run、artifact、DMG SHA256、设备与系统版本。CI artifact 仅保留 14 天，过期后须为明确候选重新构建并更新记录。
2. **统一原生功能验收**：按 `docs/05-testing-acceptance.md` 第 11.5 节，用固定非敏感数据执行权限、快捷键/中文输入法、取词与完整剪贴板、Keychain、同帧多屏截图、Spaces/全屏、菜单栏/主题和 LaunchAgent。每项记录步骤、预期、实际、通过/失败/未执行及错误类型，不记录凭据或测试内容；失败回到对应阶段修复，缺设备项目不能记为通过。
3. **实现签名与公证流程**：完整功能验收通过后，准备 Developer ID Application 证书及私钥、Team ID 和公证凭据，通过 CI secrets 提供；实现并验证签名、公证与 staple，记录签名身份、公证请求 ID 及最终 DMG SHA256。当前工作流使用 `--no-sign`，配置 secrets 本身不表示此流程已经实现。
4. **验证最终安装包再分发**：在最终安装位置复验 Gatekeeper、TCC、Keychain、安装/卸载/旧版升级、登录自启与禁用后的清理，覆盖两架构和所声明的系统范围；签名身份变化后的行为必须重新验证。只有最终产物全部通过后才发布正式版本，自动更新另立后续阶段。
