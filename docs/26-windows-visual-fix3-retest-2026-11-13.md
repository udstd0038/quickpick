# Windows visual-fix3 本机复测（2026-11-13）

对象：`quickpick-gpui-visual-fix3-2026-11-13.zip`，GPUI 0.1.4，独立目录 `gpui-visual-fix3/snapshot`。本轮可执行检查、实屏回归、退出恢复和重新打包已完成；完整验收仍未通过。以下只描述本轮实际覆盖，旧版GUI和正式安装继续保留。

## 结论与待调整项

- 用户确认蓝色opacity滑条比原黑白配色自然；中英文×浅深×30/70/100%默认尺寸下文字和底部按钮完整。保存按钮在左占剩余宽度，重置在右，顺序符合本轮要求。
- **待调整：重置按钮需与保存按钮等高。** 当前重置图标按钮较矮，用户明确提出等高要求，本轮未改应用源码。
- **发现：重置路径的英语下拉值残留中文。** 保存en/dark/acrylic/70后点击重置，界面标签仍英语、草稿下拉值为“跟随系统”；不保存关闭重开后磁盘仍en，Theme标签仍英语，值显示“深色”而不是Dark。此时70%数值正确。后续用户切换中英文矩阵并返回时主题值已为Dark，不能据此抹去重置特定路径的失败证据，也不能将其推广为所有英语页失败。尚未定位根因或修复。
- 保存持久化、重置不保存不落盘、隐藏回滚的数值路径通过。第一次保存后磁盘为en/dark/acrylic/70；重置未保存时磁盘不变，关闭重开回到70。完整语言回滚因上述问题不判通过。
- Alt+2合成选区、划词空白移动、结果/输入标题移动，以及三种工具窗口固定后失焦保留、取消固定后失焦隐藏均在限定路径通过。Alt+4重开输入与结果为空，Copy Result禁用。当前单次结果首次显示完整；旧几何重复矩阵未完成。
- 本轮用户确认IME候选Enter只上屏、Ctrl+Enter合成返回正常。Chrome合成页实时动画、框选移动、八方向缩放、Enter保持遮罩、提取/翻译及四色复制均通过本轮限定场景。

## 来源与保留

输入ZIP 4,998,931字节，SHA-256 `F87ABCE1F58F4A6F4A23F43A35B1BBFF1BD2599F629A9D71785540603D93D29E`。165项路径/链接检查通过；schema2清单entries共164文件长度/SHA全部匹配。

对比本机fix2的应用/脚本/Cargo清单，仅 `apps/quickpick-gpui/src/app.rs` 按钮顺序和 `ui.rs` 滑条配色不同；Cargo.lock、crates、其他应用源码和脚本相同。结束时109项应用/脚本/Cargo清单文件全部匹配，本轮应用源码未修改，见 `source-preservation.json`。

私有 `full_acceptance.rs` 保留，SHA-256 `45BD3B0B5C451D1DC11F80A26CACC114B696A119C36FE331D3B3E8AF5F0B1D3D`。第24号原报告与十张fix2截图另存 `../baseline-visual-fix2/`，仅作基线，不作本轮通过证据。本轮13张纯合成画面证据位于 `screenshots/visual-fix3/`，对应哈希见 `target/gpui-candidate/visual-fix3-reports/screenshot-index.json`。

## Windows检查与范围

格式、全目标Clippy（-D warnings）、MSVC Release通过。66项测试：49 core、原full_acceptance 4、full_acceptance_regression 5、platform 8。正常退出后原打包脚本再次运行66项全部通过。日志 `build-validation.txt`、`package-build.txt`。

候选自带工具夹具20项通过，日志 `candidate-script-validation.txt`；另执行外置stage5旧工具版19项，日志 `script-validation.txt`。分别记数，不合并为39项或借用上版计数。本轮没有为两个样式改动新增镜像测试，也未重跑九次剪贴板底层example；该底层既有证据属于fix2，本轮GUI回读另计。

## 实屏证据

默认设置捕获422×601。实际操作浅色100→30→拖动69→精确70、深色70并截图；主题/透明度矩阵其余组合的实屏确认来自用户。保存左、重置右均完整可见，用户要求按钮等高。Tab可达、tooltip、悬停及保存期间禁用外观未获完整独立确认，不能列为通过。

英语重置证据：`settings-reset-draft-general.png`、`settings-reset-draft-appearance100.png`、`settings-reset-reopen-english-labels-chinese-theme.png`。后续用户矩阵返回Dark的画面为 `settings-user-matrix-return-dark70.png`。

合成桌面选区Alt+2显示QuickPick Selection，空白拖动原点(85,78)→(175,78)，尺寸466×59不变。Pin/Unpin状态和激活合成夹具后的保留/隐藏均实际观察；固定按钮点击未拖动窗口。证据 `selection-after-title-blank-drag.png`。

第二阶段合成划词Translate创建结果窗400×392，标题语言控件、正文、Copy Result首次显示完整；标题空白拖动原点(1669,780)→(1704,797)，尺寸不变，固定/失焦路径通过。证据 `result-phase2-first-display.png`。仅一次创建，不借用上版多轮结果或推断完整V-02闭环。

Alt+4输入窗按原几何1906×1016完整显示；固定合成文本Ctrl+Enter返回，标题拖动原点(111,104)→(161,126)。固定后失焦保留，取消固定失焦隐藏；Alt+4重开输入及结果均空，Copy Result禁用。相应操作期间合成请求不增加。证据 `input-phase2-synthetic-return-after-drag.png`、`input-phase2-reopened-cleared.png`。用户另确认中文IME候选真实按键正常。

用户在Chrome合成页明确复制后只回读图像计数：578×156，90,168像素；红17,472、绿31,200、蓝31,200、白10,296，other=0，overlay_border_blue=0，中行E02020→208020→2040E0→FFFFFF。见 `phase2/gui-clipboard-validation.json`。未保存剪贴板图像或读取文字，只证明这次合成矩形，不能推及全部DPI/坐标。

最后用户框选→Enter→Esc取消并正常托盘退出。约79秒人工区间剪贴板序号240→240，未读内容，见 `phase2/cancel-sequence-validation.json`。非紧密遮罩区间，不证明零像素读取，精确取消隐私闸门仍待验。

## 合成配置与恢复

第一阶段外观预览/保存/重置期间AI计数为0。用户外观检查后，提交AI前检查发现三路地址/模型已空、磁盘外观为en/light/acrylic/100；变化来源未确定，不归因应用自发清空或用户操作。此时未提交AI。正常退出后独立核对五原文件存在性、哈希及时间戳全部恢复，见 `restoration-phase1.json`。

第二阶段使用同一EXE重新初始化，仅连接127.0.0.1:6194固定合成模型；三路前置检查通过后再显式提交AI。最终统计6请求/6完成，stage5-text=1、stage5-input=2、stage5-vision=3，无拒绝/活动请求。用户说明提取/翻译两项正常不等于后台仅两次视觉请求，第三次操作来源未确定。见 `phase2/mock-service-final-status.json`，无真实外部AI服务。

第二阶段用户正常退出后，settings.json、三个DPAPI文件和native-window-state.json五原文件存在性、哈希、LastWriteTimeUtc均匹配，见 `phase2/restoration-validation.json`。两阶段私有备份精确路径清理完成；候选、可见/隐藏合成夹具均已退出，6194监听0，见 `cleanup-validation.json`。未覆盖正式安装，结束时不重新启动应用。

## 最终产物

EXE 14,152,704字节，SHA-256 `6F47A64AEE99771834063F91ECF86C700DAC739C71DAC5D3949763AFE95CD2F7`，与两阶段GUI实测相同。

产物位于 `target/gpui-candidate/artifacts/`：

| 产物 | 字节 | SHA-256 |
| --- | ---: | --- |
| QuickPick_0.1.4_gpui_candidate_x64.zip | 7,650,785 | CCECBBC8EA7305B39FDE7085B0A30764AB89A4CCA93061B4B965565605F90283 |
| QuickPick_0.1.4_gpui_candidate_x64_setup.exe | 4,286,287 | 412E45466D9B544FA7AAC92EEE4B97A9314E324CC162381A8C63CC72C77EE761 |

`package-validation.json`：937文件，解压18,965,634字节，516个第三方包；实际ZIP/EXE一致、x64 GUI头及许可文件存在性检查通过。NSIS仅哈希校验，未执行安装/升级/回滚/卸载；许可存在性不替代许可证审查。

## 仍待验与下次建议

先处理重置按钮等高与重置路径英语下拉值残留，再对新候选复测。继续保留拖动期间模糊消失、松手恢复的既有体验反馈，本轮未改材质生命周期。

完整八语言/英语全页、保存期间禁用/tooltip/Tab、125/150/200%单屏DPI、更多窗口/几何/主题的固定失焦矩阵、V-02旧几何结果重复和V-03多轮还原、精确取消零捕获/零写入证据仍待补齐。多屏负坐标/混合DPI/竖屏及隔离安装升级卸载/无WebView2环境受条件限制，不能用本轮静态包检查替代。整体不标完整通过。
