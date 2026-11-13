# GPUI 全量候选与 Windows 验收

用户已确认阶段 0 Windows 实机验证通过。fix1 修复范围已在 Windows 复测通过（49 项测试、Release/Clippy、四路热键和重新打包），见 [复测记录](14-windows-fix1-retest-2026-11-11.md)。W-01 与四项脚本问题已闭环；全量交互、安装与性能验收仍未通过。S5-01 在 [fix2 Windows 复测](18-windows-fix2-retest-2026-11-12.md)已闭环：52 项测试、真实剪贴板回读、指定合成画面的 GUI 复制、热键与重新打包均通过；不外推所有坐标/DPI 和取消隐私场景通过。剩余闸门见 [阶段 5 验收](15-gpui-stage5-validation.md)。原始问题证据见 [本机验收记录](12-windows-full-acceptance-2026-11-11.md)。当前根 workspace 已实现全量业务候选；阶段 0 确认不代替新增业务、最终性能和发行包验收。旧 GUI 与独立构建入口保留，正式发行切换须满足方案阶段 5。

## 工程与边界

- `crates/quickpick-core`：设置归一化、原子配置仓库、三域 AI 请求、分块响应限额、快捷键事务、八语言、截图状态机和请求代次。
- `crates/quickpick-platform`：原目录 `com.quickpick.desktop`、DPAPI 文件兼容、UIA/剪贴板取词、原生快捷键辅助、自启动事务、显示器/窗口/系统通知、xcap 区域捕获。
- `apps/quickpick-gpui`：GPUI Kit 0.9.1 / GPUI 0.3.12；设置、划词、结果、输入、实时截图遮罩、托盘与快捷键。仅选择所需图标，不注册 AllAssets。
- 单一 Tokio runtime；阻塞系统操作进入后台。隐藏/关闭清理内容并取消请求，代次拒绝旧完成。窗口状态只保存工具窗口位置与尺寸，不保存文本、截图或结果。
- 进入、框选和取消遮罩不捕获；明确点击复制/提取/翻译才提交。AI 配置先校验；隐藏窗口、DwmFlush 和有界等待后区域捕获。复制使用 BMP，AI 使用 PNG。
- 旧 Tauri 通过适配器调用同一 core/platform；保留原前端、截图链和发行命令以支持回退。候选依赖闭包没有 Tauri/WebView。

## 本机命令

先正常退出旧版和已运行的候选，安装 Rust、MSVC C++ Build Tools、含 fxc.exe 的 Windows SDK。构建、测试并启动安全候选模式：

```powershell
powershell -ExecutionPolicy Bypass -File scripts\build-gpui.ps1 -Launch
```

候选路径为 `target/gpui-candidate/release/quickpick.exe`。`--candidate` 读取现有配置/密钥，允许保存设置及密钥，但禁止更改自启动。无参数运行完整应用，会按用户设置同步自启动。不要同时运行旧版与新版。

热键自动检查使用固定合成文本，只检查路由、显示和 Esc，不上传 AI：

```powershell
powershell -ExecutionPolicy Bypass -File scripts\verify-hotkeys.ps1
```

原 Tauri 验证改用 `scripts/verify-hotkeys.ps1 -Legacy`。自动热键测试不能代替真实跨应用取词和焦点验收。

安装 NSIS 后生成便携 ZIP、安装包、SHA-256 和第三方许可正文：

```powershell
powershell -ExecutionPolicy Bypass -File scripts\build-gpui.ps1 -Package
```

产物在 `target/gpui-candidate/artifacts`。打包缺许可正文即失败；附带不可变上游来源及文件哈希。CI 构建通过不证明桌面实机行为通过。

## 全量验收矩阵

仅使用合成测试文本、测试图片和测试服务；报告不得含真实密钥、截图内容或完整 AI 响应。

| 范围 | 必须确认 |
| --- | --- |
| 配置兼容 | 旧版八语言/主题/透明度/材质/热键/三域供应商、模型、URL 保持；损坏配置不覆盖；旧版能回读 |
| DPAPI | 原 text/vision/input 文件与仅文本旧密钥回退；新增、更新、删除、损坏错误；跨域不回退 |
| 设置 | 七分类、密码遮蔽、供应商默认值；热键冲突回滚；录制期间临时注销并在结束/取消恢复；隐藏后重开 420×600 |
| 取词 | 浏览器/编辑器/Office UIA、可关闭剪贴板兜底、原剪贴板恢复、空选区、修饰键释放、外部焦点恢复 |
| AI | 三域隔离、超时/网络/错误/超长分块响应、重试、语言/方向、Ctrl+Enter 与中文 IME 组合输入；成功文本原样显示 |
| 生命周期 | 失焦、固定、关闭、再次打开、连续触发、请求中隐藏、取消后旧结果到达、退出时 UIA 尚未返回 |
| 截图 | 新建/移动/八向缩放/交叉锚点/丢捕获；最小 8 物理像素；Enter 不提交；只明确按钮提交一次 |
| 隐私与取消 | 无配置 AI 动作不捕获；Esc/右键/失焦/超时/锁屏/挂起/拓扑/DPI 取消；剪贴板不含遮罩，截图不落盘 |
| 显示器 | 100/125/150/200%、负原点、竖屏、混合 DPI、覆盖任务栏、边缘工具栏翻转、不越屏 |
| 视觉 | 八语言无明显溢出、主题/材质/透明度、拖动时降级并恢复、加载/失败/禁用状态 |
| 发行 | NSIS 新装/覆盖旧版/候选回退/卸载、正在运行时拒绝安装；配置密钥保留；无 WebView2 环境与便携 ZIP |
| 基线 | 同一机器同条件记录旧/新冷启动、空闲/交互内存和 CPU、请求/截图响应、ZIP/安装包/落盘体积；不得编造数值 |

## 当前证据

本轮修复后 Linux 原生测试通过 38 项 core 单元测试、5 项 HTTP 集成测试与 4 项平台测试；Windows GNU 全目标 Clippy（拒绝警告）与保留的 Tauri 适配器检查通过。Windows 特有测试仅交叉编译，未在此环境执行。NSIS 在 Linux 用合成占位文件完成语法检查，不是有效发行包。

MSVC 目标依赖闭包为 519 个包，单一 GPUI 后端，无活动 Tauri/wry/Fast/syntect/tree-sitter；锁文件 868 个包的审计未发现已知漏洞，存在 8 项上游停止维护提示（Windows 活动链 4 项）。第三方正文收集和哈希不是完整法律审查结论。

用户 fix1 复测确认 MSVC Release、49 项 Windows 测试、Clippy、四路热键及实际 ZIP/NSIS 重新生成通过。该版本修复范围已闭环；其余人工矩阵、安装升级/回退和受控性能仍待验收。用户旧安装基线为 0.1.3。当前执行器是 Linux，用户第 18 号报告确认 fix2 修复范围、MSVC release 和实际包静态检查通过；本地仅保存报告并同步状态，不重建或修改已验证应用。fix1 的资源/基本输入观察仍按其版本和范围引用。全量验收前保留旧 GUI，不宣称已完成正式发行切换。
