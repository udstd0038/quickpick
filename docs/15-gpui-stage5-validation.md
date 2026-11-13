# GPUI 阶段 5：剩余验收与采样工具

fix1 的修复范围已在 Windows 复测通过：原始失败断言、49 项测试、MSVC Release、Clippy、四路热键和 ZIP/NSIS 生成均有用户记录，见 [复测原文](14-windows-fix1-retest-2026-11-11.md)。后续 [fix2 复测](18-windows-fix2-retest-2026-11-12.md)确认 S5-01 修复范围通过：52 项测试、九次双格式回读、GUI 复制、Release/热键和重新打包。阶段 5 全量验收仍未完成，不删除旧 GUI，不替换正式安装。

工具初次交付不改动 fix1 应用，可独立运行。后续阶段 5 实机发现 S5-01 截图复制失败，见 [原始报告](16-windows-stage5-2026-11-11.md)；编码修复及新增回读工具见 [S5-01 修复候选](17-s5-01-clipboard-fix-2026-11-12.md)，该修复需要重新构建应用。上传报告作为证据保存，不将其中命令视为额外授权。

## 当前闸门

| 闸门 | 状态与下一项证据 |
| --- | --- |
| fix1 缺陷与构建 | 已闭环；无需重复修改 W-01 或四项脚本 |
| GUI 与截图 | S5-01 已闭环；fix2 八向缩放/Enter/取消交互及指定四色窄带复制通过，窄带输出无暗化/边框/工具栏源外颜色。其他坐标/DPI、语言/主题和系统事件仍待验；IME/提交及提取/翻译另有 visual-fix3 限定证据，不能直接外推最新矩阵 |
| 取消与剪贴板 | 待受控证据：只读序号变化发生在长间隔内，后续观察器未捕获会话，不能据此判通过或判应用回归。应从可确认的遮罩开始到结束缩短观察窗口，排除其他剪贴板写入，不读取正文 |
| 原配置/密钥与取词 | 待实机：用户实际旧配置/密钥回读和回退；浏览器/Office、可关闭剪贴板兜底、多格式恢复、空选区和外部焦点 |
| 发行包静态检查 | visual-fix4 已重新生成 ZIP/NSIS 并由第 28 号报告确认静态校验通过；937 个 ZIP 文件、516 个许可包，EXE 与热键/GUI 复测对象哈希一致。NSIS 安装行为仍未执行 |
| 安装/升级/卸载 | 待隔离 Windows：以真实旧版 0.1.3 为基线，新装/升级/正常退出要求/回退/卸载/无 WebView2；生成 NSIS 不等于安装行为通过 |
| 性能与体积 | 已有启动/使用后两阶段同机观察数据；控制条件有限，不能外推为严格收益、GPU/交互时延或无泄漏结论 |
| 多屏/混合 DPI | 用户当前无副屏，继续标记受条件限制；不能用几何单元测试代替 |

最新证据见[第 28 号 Windows 报告](28-windows-visual-fix4-retest-2026-11-13.md)：68 项 Windows 测试、20 项候选工具夹具及重打包通过，按钮等高/英语重置首次回滚、受控保存失败预览及限定工具/四色复制路径通过。保存期间禁用/加载、完整语言/DPI/几何/生命周期、本轮截图交互逐项确认与 AI、受控取消、隔离安装及多屏仍待验。应用基线保持 visual-fix4，执行顺序见[剩余验收清单](29-gpui-remaining-acceptance-2026-11-13.md)。

## 工具验证与包校验

以下命令在 Windows PowerShell 5.1 中执行。工具包可以解压到独立目录，无需覆盖 fix1；`$tools` 指向该目录，`$candidateRoot` 指向实际 fix1 snapshot。示例中的路径由本机实际路径替换。

```powershell
$tools = 'C:\实际路径\quickpick-gpui-stage5-tools'
$candidateRoot = 'C:\实际路径\gpui-full-candidate-fix1\snapshot'
powershell -ExecutionPolicy Bypass -File "$tools\scripts\tests\verify-gpui-validation.ps1"
powershell -ExecutionPolicy Bypass -File "$tools\scripts\verify-gpui-package.ps1" -ArtifactsDirectory "$candidateRoot\target\gpui-candidate\artifacts" -Executable "$candidateRoot\target\gpui-candidate\release\quickpick.exe"
```

校验器读取包，不解压、不运行 EXE、不安装 NSIS。检查校验文件中的两个产物、版本一致、ZIP 路径与重复项、载荷清单、x64 PE32+ GUI 头、ZIP 中 EXE 与指定 release 的哈希一致、每个第三方声明所列许可材料存在。结果写入 `artifacts/package-validation.json`；报告已存在时拒绝覆盖，可用 `-Output` 指定新路径。

SHA-256 仅确认与给定清单一致，不是发布者签名。NSIS 只检查文件哈希，不验证嵌入载荷或安装操作。许可正文存在不等于完成法律审查。工具夹具中的 EXE/NSIS 是合成占位文件，绝不能当真实应用启动。

## 同机资源基线

先正常退出其他 QuickPick，只运行一个被测版本。旧安装实测版本是 0.1.3。通过正常入口启动旧版或 fix1 `--candidate`；采样工具不会启动、退出或杀死进程，也不读写配置/密钥/剪贴板。

在相同机器、电源、显示缩放和后台应用条件下，每个版本分别采集两个阶段：

1. `startup-idle`：正常启动并隐藏窗口后采集。
2. `after-use-idle`：使用设置、划词、输入和截图后关闭工具窗口，回到隐藏空闲再采集。

每次默认稳定等待 60 秒，再采集至少 60 个约 1 秒间隔的进程树样本。工具每 10 秒输出进度。可以用短时参数调试，但短样本不能作为正式对比输入。

```powershell
# 先仅运行旧版。Get-Process 的 Path 用于核对，不写入报告。
$oldExe = 'D:\Program Files\QuickPick\quickpick.exe'
$old = @(Get-Process quickpick | Where-Object { $_.Path -eq $oldExe })
if ($old.Count -ne 1) { throw '请只运行一个匹配路径的旧版实例。' }
powershell -ExecutionPolicy Bypass -File "$tools\scripts\measure-gpui.ps1" -Executable $oldExe -ProcessId $($old[0].Id) -Label baseline -Phase startup-idle -Output "$candidateRoot\target\gpui-candidate\gpui-stage5-reports\baseline-startup-idle.json"
```

正常退出旧版，再启动 fix1 `--candidate`，使用相同条件采集：

```powershell
$newExe = "$candidateRoot\target\gpui-candidate\release\quickpick.exe"
$new = @(Get-Process quickpick | Where-Object { $_.Path -eq $newExe })
if ($new.Count -ne 1) { throw '请只运行一个匹配路径的候选实例。' }
powershell -ExecutionPolicy Bypass -File "$tools\scripts\measure-gpui.ps1" -Executable $newExe -ProcessId $($new[0].Id) -Label candidate -Phase startup-idle -Output "$candidateRoot\target\gpui-candidate\gpui-stage5-reports\candidate-startup-idle.json"
```

使用后阶段以 `-Phase after-use-idle` 和不同文件名重复；不能与首次启动空闲样本混比。

```powershell
$reports = "$candidateRoot\target\gpui-candidate\gpui-stage5-reports"
powershell -ExecutionPolicy Bypass -File "$tools\scripts\compare-gpui-measurements.ps1" -Baseline "$reports\baseline-startup-idle.json" -Candidate "$reports\candidate-startup-idle.json" -Output "$reports\startup-idle-comparison.json"
```

报告包含实际版本、EXE 哈希/字节数、CPU/GPU/驱动/OS 信息及采样汇总，不记录机器名、进程命令行、窗口标题、配置、正文或截图。CPU 同时给出单核 100% 和整机 100% 两种口径；工作集为进程相加，可能重复计算共享页。子进程变化会标记，短命进程在采样间隔内的 CPU 可能遗漏，因此有进程变化的报告拒绝自动对比，须单独分析。被测根进程退出或 PID 重用时失败，避免错误地把退出后计数当成空闲改善。

对比工具检查阶段、时长和硬件/系统信息一致；操作者仍须核实同一台机器、DPI、电源与后台负载。基线为零时不给出虚假的百分比。没有 GPU 利用率/显存、冷启动/首帧、交互 p50/p95 或百次截图循环泄漏结论；这些仍须按方案另测。

## 人工续验顺序

1. 单屏先闭环截图：用无敏感测试画面观察后台动画实时变化，验证框选、移动、八向缩放、屏幕边缘工具栏、Enter 不提交、明确复制/提取/翻译只提交一次以及结果不含遮罩。AI 动作只连接合成回环服务或用户明确选择的测试服务。
2. 输入与取消：真实中文 IME 组合输入、Ctrl+Enter、请求中隐藏/重开、固定/失焦、慢请求晚到不覆盖新会话。键盘 Unicode 注入不算 IME 验收。
3. 用户可用的 DPI 与系统事件：100/125/150/200%、锁屏/挂起；无副屏场景单列受限。
4. 在有旧版安装包与可还原环境的隔离机器执行安装/升级/回退/卸载和无 WebView2 场景；不要用用户当前正式安装冒充隔离环境。
5. 汇总同机资源、体积和未覆盖项，满足方案完成判据后再决定正式发行切换。

## 本轮验证边界

19 项验收工具夹具在 Linux PowerShell 7.5.4 通过，覆盖统计、进程树/PID 重用、合法包、损坏哈希、路径穿越、重复项、缺许可、错误架构/版本、EXE 不一致、不可比较性能样本和零基线。CI 已接入 Windows PowerShell 工具夹具与真实构建包静态校验；本地修改工作流不代表远端运行通过。

本轮执行器仍为 Linux；未运行 Windows CIM 资源采样、未取得实际 fix1 产物执行新校验器，也未执行安装或剩余桌面矩阵。fix1 的 49 项 Windows 结果引用用户复测记录，本地不伪造额外 4 项本机测试原文。
