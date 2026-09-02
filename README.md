# QuickPick

> macOS 移植工作在 `feature/macos-port` 分支持续验证；`main` 仍面向 Windows。

QuickPick 是一个运行在 Windows 11 上的轻量级划词与截图 AI 工具。它常驻系统托盘，通过全局快捷键在任意软件中处理选中的文字、截取屏幕区域，并调用 OpenAI 兼容 AI 模型完成翻译、总结和文字提取。

## 功能

- 划词：选中文本后按 `Alt+2` 弹出划词菜单，可复制、翻译、总结或搜索。
- 截图：按 `Alt+3` 框选屏幕区域，可复制图片、提取文字或翻译图片中的文字。
- 输入翻译：按 `Alt+4` 打开输入翻译窗口，手动输入文本并翻译。
- 设置：按 `Alt+0` 打开设置页，可配置 AI 接入、快捷键、语言、主题、窗口效果和透明度。
- 托盘：启动后常驻托盘，不主动弹出主窗口；支持开机自启和托盘菜单。
- 多语言：界面支持简体中文、繁体中文、英文、韩文、日文、法文、德文和西班牙文。

## AI 配置

QuickPick 使用 OpenAI 兼容接口，并在设置页中拆分为三个独立模型：

- 划词模型：处理划词翻译、总结等文本任务。
- 截图模型：处理截图文字提取和图片翻译等视觉任务。
- 输入模型：处理输入翻译窗口的文本任务。

API Key 使用 Windows DPAPI 加密保存。输入文本、截图内容和 AI 结果默认不写入历史记录。

## 安装

当前版本面向 Windows 11 x64。

- 下载 `QuickPick_0.1.0_x64-setup.exe` 运行安装程序。
- 默认安装位置优先选择 `D:\Program Files\QuickPick`；没有 D 盘时选择其他非系统盘，只有 C 盘时回退到 `C:\Program Files\QuickPick`。
- 也可以下载 `QuickPick_0.1.0_x64-portable.zip`，解压后直接运行。

首次使用请按 `Alt+0` 打开设置页，至少为需要使用的功能配置对应的 AI 供应商、Base URL、模型和 API Key。

## 隐私

- 不自动保存划词文本、截图、AI 结果或翻译历史。
- 不自动覆盖剪贴板；复制操作只在用户明确点击后执行。
- API Key 本地加密保存，不写入开发日志或仓库。

## 开发

项目使用 pnpm monorepo、Tauri 2、React、TypeScript 和 Rust。

```powershell
pnpm install
pnpm dev
```

构建 Windows NSIS 安装包：

```powershell
pnpm --filter quickpick-desktop exec tauri build --bundles nsis --ci --no-sign
```

## 许可

安装程序使用的许可文本位于 `apps/desktop/src-tauri/installer/LICENSE.txt`，当前为发布前占位文本，正式条款发布前会替换。
