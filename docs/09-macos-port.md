# QuickPick macOS 移植说明

## 状态

macOS 移植在 `feature/macos-port` 分支开发，目标是：

- Windows 行为不回归，`main` 仍保留 Windows 默认快捷键与 DPAPI/NSIS 流程。
- macOS 使用 Tauri 2、React、xcap 和 Rust 原生/官方插件能力。
- 当前阶段只做代码、CI 和可编译验证；不创建未签名 Release。
- 真正的辅助功能、屏幕录制、菜单栏、快捷键、Keychain、截图复制和深浅主题验收仍需要实体 macOS 人工完成。

## 平台差异

### 目标系统与快捷键

- macOS 默认设置快捷键为 `Command+Comma`，划词/截图/输入翻译为 `Command+Option+2/3/4`。
- Rust 设置归一化在 macOS 接受 Command 或 Option；Windows 仍强制 Alt。
- 前端录制和展示使用 Command/Option 名称，Windows 继续使用 Alt/Ctrl/Win。

### API Key

- Windows 继续使用 DPAPI 文件。
- macOS 使用 Keychain，service 为 `com.quickpick.desktop`，account 为 `text`、`vision`、`input`。
- Key 不写入仓库、日志、设置文件或 CI 产物。

### 划词

- Windows 使用 UIAutomation 优先，失败后按用户设置使用剪贴板兜底。
- macOS 使用 `axuielement` 读取 `AXSelectedText`；授权缺失时请求辅助功能权限。
- 用户开启剪贴板兜底时，macOS 才模拟 `Command+C`，并通过 `tauri-plugin-clipboard-manager` 读取和恢复文本。

### 截图与剪贴板

- 截图捕获继续复用 xcap，前端仍使用 WebView 框选。
- Windows 继续写 CF_DIB/CF_BITMAP；macOS 使用 PNG RGBA 写入系统剪贴板。
- macOS 截图前检查并请求屏幕录制权限。

### 系统集成

- macOS 使用菜单栏 template 图标、`ActivationPolicy::Accessory` 和 Tauri 原生菜单。
- 设置页使用原生标题栏与交通灯，隐藏自绘窗口按钮。
- 开机自启通过 `tauri-plugin-autostart` LaunchAgent 实现。
- 深浅主题、玻璃色板和布局 token 仍由前端共享变量控制，与 Windows 页面保持一致。

## CI

`.github/workflows/macos-build.yml` 提供：

- Windows：前端测试/构建、Rust check/test。
- macOS：前端测试/构建、`aarch64-apple-darwin` 与 `x86_64-apple-darwin` check/test、universal build。

签名、公证、App Store 发布和 GitHub Release 需要用户提供 Apple 证书与 secrets 后再单独执行。
