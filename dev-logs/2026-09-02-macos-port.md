# 2026-09-02 macOS 移植日志

## 今日完成

- 在 `feature/macos-port` 建立 Windows 基线并实施首批 macOS 支持：
  - Windows-only Rust 依赖移入 `[target.'cfg(windows)'.dependencies]`。
  - 新增 macOS target 依赖：`axuielement`、`enigo`、`keyring`、`tauri-plugin-autostart`、`tauri-plugin-clipboard-manager`、`tauri-plugin-macos-permissions`、`tauri-plugin-os`。
  - 新增 `selection_windows.rs`/`selection_macos.rs` 平台分派。
  - macOS Keychain 保存文本、截图、输入三类 API Key。
  - macOS 划词直接读取 AX 选中文本，按设置启用剪贴板兜底。
  - macOS 截图复制使用 PNG RGBA，Windows 继续使用 CF_DIB/CF_BITMAP。
  - macOS 菜单栏、Accessory 激活策略、LaunchAgent、权限检查和原生标题栏适配。
  - 前端支持 mac 默认快捷键、Command/Option 录制和 `data-platform="macos"` 布局。
  - 生成 `.icns` 与多平台图标，新增 macOS 平台配置。
  - 新增 `.github/workflows/macos-build.yml`，Windows 与 macOS CI 同时验证。

## 关键决策

- `main` 继续面向 Windows；macOS 适配全部落在 `feature/macos-port`。
- Windows 继续使用 DPAPI 和现有注册表自启动；macOS 使用 Keychain 与 LaunchAgent。
- 当前不签名、不公证、不发布 GitHub Release。

## 验证结果

- Windows 前端测试 29/29 通过。
- Windows 前端构建通过。
- Windows Rust check 通过，Rust 测试 25/25 通过。
- 本地 Windows 无法完成 macOS target check，macOS 编译事实来源为 GitHub Actions。

## 待办

- 推送 `feature/macos-port` 后等待 macOS CI。
- 实体 Mac 人工验收辅助功能、屏幕录制、菜单栏、快捷键、Keychain、截图复制与深浅主题。
- 用户提供 Apple 证书/Team/App Store Connect secrets 后再配置签名公证与发布。

## 下一步建议

- 按 CI 报错继续修正 macOS target 编译问题。
- 推送前先完成 Git 提交和 Windows 最终回归。
