# Windows visual-fix2 本机复测（2026-11-13）

对象：`quickpick-gpui-visual-fix2-2026-11-13.zip`，独立目录 `gpui-visual-fix2/snapshot`，GPUI 0.1.4。已完成本机本轮可执行复测、退出恢复及重新打包；整体完整验收仍未通过，不能将下列限定通过推广到未测矩阵。

## 结论

- 设置窗品牌/标题移动，浅深主题默认/手调尺寸的四边四角缩放、最小限制和最大化还原，在本轮已测实屏场景正常。深色点击下拉框和保存按钮不移动窗口。输入/结果窗标题移动、固定/取消固定及关闭重开清空，经用户确认正常。划词窗口本轮未复测，V-04 仅部分范围闭环。
- Acrylic 冷启动及 Mica→Acrylic、两材质×浅深主题×30/70/100% 静止效果和控件可读性经用户确认正常。保存深色 Acrylic/70% 后，预览30%不保存，关闭重开恢复70%。预览/保存/回滚阶段本地 AI 请求保持0。V-06 的静止不透明在已测组合未复现。
- 用户澄清按住边缘/拖动时是模糊消失，非透明度提高，松手约2秒后恢复。该观感差异保留为反馈，不将当前实现改写成产品要求。另有两项未实现外观反馈：opacity 黑/白滑条颜色突兀；重置按钮应放在 Save Settings 右侧。
- 手动 Alt+4、中文 IME 候选 Enter 只上屏不提交、Ctrl+Enter 合成返回正常。自动输入未唤出窗口的尝试不认定为产品热键故障。Alt+0、Alt+3在本轮路径可用；Alt+2及完整失焦矩阵仍待本轮补验。
- Chrome 合成页面实时计数/橙条、框选移动和八方向缩放、Enter 保持遮罩及 Copy 经用户确认正常；实际剪贴板四色像素检查通过。
- 原设置、三个 DPAPI 文件和原窗口状态共五文件均恢复，哈希和时间戳一致；本轮私有临时备份已清理。候选、桌面夹具和本地服务均已退出，未覆盖正式安装。

## 来源和代码保留

输入 ZIP：4,995,298字节，SHA-256 `4E996D7E4BC27C481A96E8A1738B8120538FE0F3A882760BC9D57105C2084188`。ZIP共163项；schema2清单的162项长度/哈希均在解包时校验通过，使用 entries 字段。第22号报告及35张上版截图保留于 `../baseline-visual-fix1/`。

本轮未修改应用源码。结束时重新校验 apps/crates/scripts/Cargo 共109个清单文件全部匹配，见 `source-preservation.json`。原 `full_acceptance.rs` SHA-256 `45BD3B0B5C451D1DC11F80A26CACC114B696A119C36FE331D3B3E8AF5F0B1D3D`，Cargo.lock与visual-fix1相同。本轮增加报告、日志、证据及 target 下测试启动夹具副本。

## Windows 检查

格式、全目标 Clippy（-D warnings）、MSVC release通过。66项测试：49 core、原full_acceptance.rs 4项、full_acceptance_regression.rs 5项、platform 8项。新增八方向命中和遮罩排除隐藏窗口测试已运行，但不能替代真实鼠标/DPI矩阵。PowerShell工具夹具20项通过，合成剪贴板example九次CF_DIB/CF_BITMAP通过。

正常退出后的首轮打包在受限沙箱中遇到 loopback 10013 与临时配置写入失败，日志 `package-build.txt` 如实保留；改用本机权限运行原脚本，66项测试和构建/打包全部通过，日志 `package-build-windows.txt`。未修改测试绕过失败。

## 实屏和图像证据

十张当前设置截图位于 `screenshots/visual-fix2/`，哈希索引 `screenshot-index.json`。包含冷启动、品牌拖动、Mica→Acrylic、浅深及透明度预览、保存回滚和一次深色还原。捕获图不代替用户实际材质观感。

品牌拖动实测原点(860,449)→(960,531)，宽高保持422×601。自动客户区角落尝试仅移动位置，未计为缩放通过；八方向通过依据是用户实际操作。一次深色默认尺寸最大化→还原首次捕获完整，用户另确认手调尺寸还原正常；未借用上版20轮结果。

用户明确在Chrome本地四色页面复制后回读：624×165，102,960像素。红20,955、绿33,000、蓝33,000、白16,005，other=0，overlay_border_blue=0；中行顺序E02020→208020→2040E0→FFFFFF。见 `gui-clipboard-validation.json`。未保存剪贴板图像或读取文字，仅证明该合成矩形复制，不证明所有坐标/DPI/提交动作。

最后用户按指定顺序框选→Enter→Esc并正常托盘退出。约96秒的人工操作区间剪贴板序号93→93，未读内容，见 `cancel-sequence-validation.json`。这不是紧密观测的遮罩区间，也不能证明零像素读取；精确取消隐私闸门继续待验。

本地假AI最终统计见 `mock-service-final-status.json`；IME回归后曾观测到两次stage5-input完成，不能将区间描述成一次请求或推断第二次操作来源。无真实外部服务。

## 退出恢复和交付

夹具正常退出后自动恢复 settings.json、text/input/vision_api_key.dpapi 和 native-window-state.json。独立核对五文件存在性、SHA及LastWriteTimeUtc全部一致，仅布尔结果保存在 `restoration-validation.json`。备份精确路径清理完成，候选/夹具进程与2529监听数均为0。

最终EXE 14,152,192字节，SHA-256 `BC0A0DF17DCE9382DAA721578BDDA9E660AE9C7511BA2081E80EC4666228CF4A`，与GUI实测相同。

产物位于 `target/gpui-candidate/artifacts/`：

| 产物 | 字节 | SHA-256 |
| --- | ---: | --- |
| QuickPick_0.1.4_gpui_candidate_x64.zip | 7,650,768 | C138CBA56E43A71AB55EE2833D176E8D6F09DA0EF3365CDE5E7103333F37CB8F |
| QuickPick_0.1.4_gpui_candidate_x64_setup.exe | 4,284,888 | F14BFC87B48AA166D3CDDDC92CFD9AE16A3A4A2AD65F915AD14387C6DC45CA6F |

`package-validation.json`：ZIP937文件、解压18,965,122字节、516个第三方包，EXE哈希/字节与x64 GUI头及许可文件存在性通过。NSIS仅检查哈希，未执行安装、升级、回滚或卸载；许可存在性不替代许可证审查。

## 仍待验

125/150/200% DPI及多屏负坐标/混合DPI/竖屏，八语言完整矩阵，本轮划词窗移动和Alt+2、完整固定/失焦矩阵、V-01全页与V-02结果旧几何首次显示的重复矩阵、精确取消零捕获/零写入证据、截图提取/翻译本轮回归，以及隔离安装升级/卸载与无WebView2环境。无副屏且无隔离安装环境的条件不以测试通过替代。旧GUI与原安装继续保留。
