# 2026-09-02 macOS 快捷键验收修复

## 验收问题

- 实体 macOS 上设置页快捷键无法录入。
- 已配置或默认快捷键按下后没有反应。

## 代码检查结论

macOS 的全局快捷键链路存在三个高风险点：

- `global-shortcut` 插件在 `setup` 内动态注册，而不是随 Tauri Builder 链在标准时机初始化。
- macOS 15+ 不允许仅使用 Option 或 Shift+Option 的 `RegisterEventHotKey`；旧设置文件若遗留这类快捷键，会注册失败。
- 快捷键录制依赖单个按钮的 React keydown；WKWebView 在 macOS 上可能丢失 DOM 焦点，导致按键事件无法到达按钮。

## 修复

- 将 `tauri-plugin-global-shortcut` 改为 Builder 链标准注册。
- 快捷键处理从 `Released` 改为 `Pressed`，减少依赖 macOS key-up 事件。
- macOS 设置归一化要求 Command 或 Ctrl；Option-only 旧值在读取时回退到平台默认值。
- 快捷键录制增加 `window` 级 capture keydown，并在开始录制时主动 `setFocus()`。
- 快捷键占位符按 macOS/Windows 分别显示默认值。

## 验证

- Windows `cargo test`：25/25 通过。
- Windows `cargo check`：通过。
- 前端测试：29/29 通过。
- 前端生产构建：通过。

## 待办

- 推送后等待 macOS GitHub Actions 编译验证。
- 需要在实体 macOS 上重新人工验收快捷键录入与四个默认快捷键触发。
