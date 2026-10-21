# QuickPick macOS 移植说明

## 状态

macOS 移植在 `feature/macos-port` 分支开发。本轮在 `codex/macos-baseline` 同步 Windows 主线修复，将开发版本统一到 `0.1.4`；第一阶段已通过 CI 验收，当前进入第二阶段的取词与 Keychain 正确性开发：

- Windows 保留默认快捷键与 DPAPI/NSIS 流程，第一阶段 Windows 回归已通过。
- macOS 使用 Tauri 2、React、xcap 和 Rust 原生/官方插件能力。
- 已发布 `v0.1.3-macos` universal DMG 预发布包；该包未签名、未公证，不能代表最新分支代码。
- 实体 Mac 曾发现快捷键无法录入及默认快捷键不触发，后续修复仍需使用对应新提交的产物复验；完整功能验收尚未完成。
- 第一阶段验收提交为 [`2ab5a52`](https://github.com/udstd0038/quickpick/commit/2ab5a52)，[CI run `38381632543`](https://github.com/udstd0038/quickpick/actions/runs/38381632543) 已通过 Windows 回归、macOS 双架构检查、原生 Rust 测试、universal DMG 构建及产物验证。
- 该 PR 构建实际 checkout 的合并提交为 `bd1689bed6f1e8789de76a82bcd10f3eca711430`，验收产物为 `quickpick-macos-universal-bd1689bed6f1e8789de76a82bcd10f3eca711430`。源码提交与产物合并提交分别记录，后续更改需取得对应新提交的验证结果。
- 用户目前没有可用的实体 Mac；第一阶段通过说明构建与自动化基线可用，完整原生行为和手动检查保持待验收。
- 当前阶段生成供验收的 CI 产物，不自动发布 Release。第二阶段用例尚未验收，不将第一阶段成功计为第二阶段通过。

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
- 用户开启剪贴板兜底时，macOS 才模拟 `Command+C`。第一阶段基线仅通过 `tauri-plugin-clipboard-manager` 读取和恢复文本；第二阶段正在完善复制完成判定、触发键释放和原剪贴板全格式恢复，具体通过状态见测试验收记录。
- 第二阶段以 `NSPasteboard.changeCount` 检测剪贴板变化，复制失败或超时不得把旧文本标为成功；复制相同文本也应以实际变更事件判断，不能只比较字符串。
- 恢复前发现并发修改时跳过恢复，保留新剪贴板并给出可理解提示。`NSPasteboard` 没有 compare-and-swap（CAS）恢复接口：最后一次检查 `changeCount` 与 `clear` 之间仍有竞争窗口，不能保证零竞争，也不能仅凭计数证明变化来源。自动化验证已检测并发时的降级策略，真实系统中的剩余竞争风险必须如实记录。

### 截图与剪贴板

- 截图捕获继续复用 xcap，前端仍使用 WebView 框选。
- Windows 继续写 CF_DIB/CF_BITMAP；macOS 将 RGBA 图片交给剪贴板插件写入系统剪贴板。
- macOS 截图前检查并请求屏幕录制权限。
- 当前捕获优先选择主显示器；Retina 像素/逻辑坐标、多屏定位和遮罩绑定仍需完善及实机验证。

### 系统集成

- macOS 使用菜单栏 template 图标、`ActivationPolicy::Accessory` 和 Tauri 原生菜单。
- 设置页使用原生标题栏与交通灯，隐藏自绘窗口按钮。
- 开机自启通过 `tauri-plugin-autostart` LaunchAgent 实现。
- 深浅主题、玻璃色板和布局 token 仍由前端共享变量控制，与 Windows 页面保持一致。
- macOS 原生玻璃效果实际使用 Popover/ContentBackground；设置页暂沿用 Acrylic/Mica 名称，后续需要调整平台展示。

## CI

`.github/workflows/macos-build.yml` 在 push、pull request 和手动运行时提供：

- Windows：显式使用 `cmd`，执行前端测试/构建、`cargo check --locked` 和 `cargo test --locked`。
- macOS：显式使用 `bash`，执行前端测试/构建、ARM 与 Intel 的 `cargo check --locked`、Runner 原生架构的 `cargo test --locked`，以及带 `--locked` 的未签名 universal DMG 构建。原生测试既支持 ARM Runner，也支持 Intel Runner；另一架构的运行行为仍需实机验收。
- CI 显式将 `CARGO_TARGET_DIR` 固定为 `apps/desktop/src-tauri/target`，避免仓库开发用 `target-monorepo-check` 配置与产物路径不一致；打包后用 `lipo` 验证 ARM/Intel 两架构。
- pnpm 使用 `--frozen-lockfile`；Rust 与 Tauri 构建使用 `--locked`，防止 CI 静默改写依赖锁文件。
- DMG 与 `build-info.txt` 上传为 `quickpick-macos-universal-<完整 github.sha>`，保留 14 天。验收记录必须写入该提交和对应 workflow run。PR 的 `github.sha` 是 GitHub 为该 PR 生成的合并提交，与 checkout 的验证代码对应。

push 和 pull request 始终使用 `windows-latest`、`macos-latest`。手动 `workflow_dispatch` 默认也使用托管 Runner；只有明确选择 `self-hosted` 时，才分别使用同时带有 `self-hosted` + `Windows` 或 `self-hosted` + `macOS` 标签的 Runner。自托管 Windows 需要可用的 `cmd` 和 Rust/MSVC 工具链，macOS 需要可用的 `bash`、Xcode 工具链及 DMG 打包工具。Runner 还需能访问本工作流使用的 Actions 和依赖源。自托管模式仅用于手动运行，不接收 push/PR 自动任务。

GitHub 的手动触发入口要求此工作流同时存在于仓库默认分支。仅合入 macOS 功能分支时，不能保证 Actions 页面已提供该入口；需要先将工作流纳入默认分支，再手动选择待验证分支和 Runner。PR 自动验证不依赖手动入口。

历史 CI 曾在 `8db8485` 和已发布包对应的 `c6bf3b3` 上成功。快捷键修复后，日志记载当时遇到 Actions 付款/额度阻塞；本次审查时 `feature/macos-port` 的 `9354796` CI 失败且没有执行验证步骤。这些历史结果都不能作为本轮最新提交通过验证的证据，也不能证明当前失败原因仍是付款问题。

第一阶段已经通过上文记录的完整 CI。第二阶段仍需针对其最新提交执行前端、Windows 回归、macOS 双架构检查、原生 Rust 测试和 universal 打包；受控后端与时序的自动化用例用于验证逻辑，不能代替真实 Mac 的 Accessibility、键盘事件、NSPasteboard 和 Keychain 行为。详细条目见 `docs/05-testing-acceptance.md` 的 macOS 第二阶段清单。

## 后续分步计划与验收门槛

1. **统一开发基线与 CI（已通过）**：验收提交 `2ab5a52`，CI run `38381632543`；Windows 回归、macOS 双架构 check、原生测试、universal DMG 及架构/产物验证已完成。
2. **取词与 Keychain 正确性（当前阶段，待验收）**：完善剪贴板变化判定、超时等待、热键释放、Command 清理和原剪贴板恢复；区分 Keychain 的不存在、锁定和访问拒绝，避免虚假的未配置或清除成功。自动化覆盖复制失败、慢复制、相同内容、全部格式/空板、并发修改、键释放/权限拒绝及 Keychain 读写删除错误。真实 Mac 手动检查保留待执行，不提前标为通过。
3. **权限与多屏截图体验**：完成拒绝授权、进入系统设置、授权后重试/重启的闭环；统一显示器、逻辑坐标、像素坐标与缩放，绑定截图遮罩，调整 macOS 外观名称。完成本阶段代码及 CI 验证后推进第四阶段；单屏 Retina、外接屏、混合缩放及负坐标布局的真实设备检查保留到有 Mac 时执行。
4. **移除剩余 PowerShell 工具**：替换快捷键验收、README 截图、图标与安装器资源生成四个脚本，同步现行规范；确认活跃构建、审计、验收入口无 PowerShell 依赖，原生验收和资源外观覆盖保留。
5. **实体 Mac 完整验收与正式分发**：使用准确提交的产物复验四组快捷键、中文输入法、常见应用取词、截图粘贴、Keychain、菜单栏、LaunchAgent、Spaces/全屏及深浅主题，记录 Apple Silicon/Intel 与系统版本及每项结果。完整功能验收通过后配置 Developer ID 签名、公证和 staple，验证 DMG 安装、卸载、升级与登录自启，再发布正式版本。自动更新尚未实现，另立后续阶段。

用户已确定：各阶段代码与 CI 按 1 → 2 → 3 → 4 → 5 的顺序推进，每阶段自动化验证通过后再推进下一阶段；Mac 实体验收延后，不因当前缺少设备阻挡第四阶段。待验收的实机项目独立保留，不能标为通过；第五阶段正式分发仍须完成真实设备验收及签名、公证验证。

签名和公证需要配置 Apple 证书及对应 CI secrets；本轮 CI 不依赖这些凭据，也不发布 GitHub Release。
