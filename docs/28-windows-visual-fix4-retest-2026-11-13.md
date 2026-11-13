# Windows visual-fix4 本机复测（2026-11-13）

对象：`quickpick-gpui-visual-fix4-2026-11-13.zip`，GPUI 0.1.4，独立目录 `gpui-visual-fix4/snapshot`。第27号说明的两项针对性修复在本轮已测路径通过：重置按钮等高、中文系统上英语重置/首次回滚不再混入中文。已完成本轮检查、回归、正常退出恢复及重新打包；完整验收仍未通过，未测矩阵继续保留。

## 针对性结果

- 系统Culture/UICulture均zh-CN。通过UI保存en/dark/acrylic/70后点击重置，首次显示General当前值System Language/Simplified Chinese/Auto Detect仍为英语。浏览七个设置页，所见标签/当前值均为英语；展开语言9项、供应商7项、主题3项，文案正常。Appearance首次为Follow System/100。
- 未保存重置期间设置文件哈希与已保存基线完全一致，本地AI请求0。**不切换语言、不重选主题、不重启**，关闭后Alt+0首次重开，Theme为Dark、透明度70，未复现fix3的“深色”残留。限定路径通过，不外推所有隐藏/重建场景。
- 明确保存默认草稿后，磁盘uiLanguage=system、themeMode=system、opacity100，页面和选项一起切为中文。中文页选择英语但未保存时显示语言仍中文；保存后页面和下拉一起英语，ID=en。再英语页选择中文，保存后一同中文，ID=zh-Hans。默认保存清空模型地址属于原默认范围，AI回归另开第二会话，不误判配置丢失。
- 用户实屏确认：中英文×浅深主题下保存/重置**等高且完整**，Tab可达两个按钮，重置tooltip使用对应语言。右侧图标顺序保留，本轮按钮高度反馈闭环。
- 受控保存失败时显示Failed to write settings file，30%预览保留，磁盘仍70且哈希不变。仅对已备份临时设置持有短时阻止替换的读共享句柄，不读内容、不改ACL，已释放。保存期间禁用/加载瞬态未捕获，不能将错误后恢复可用视为禁用验证。

## 来源、保留和代码范围

输入ZIP 5,020,315字节，SHA-256 `7DB37FD2F5344C87E3B56C018988B812055B70CA0F8E48408F45C5CF340B3D7E`。168项路径/链接检查通过，schema2 entries共167文件长度/SHA全部匹配。

相对本机fix3，应用差异仅app.rs/main.rs/settings.rs及新增settings_choices.rs；Cargo.lock、crates、其他应用源和脚本相同。本轮应用源码未改；结束时110项应用/脚本/Cargo清单全部匹配，见 `source-preservation.json`。私有full_acceptance.rs保留，SHA-256 `45BD3B0B5C451D1DC11F80A26CACC114B696A119C36FE331D3B3E8AF5F0B1D3D`。

第26号原报告与13张fix3证据另存 `../baseline-visual-fix3/`，不作为fix4通过证据。本轮18张纯合成画面证据在 `screenshots/visual-fix4/`；哈希见 `target/gpui-candidate/visual-fix4-reports/screenshot-index.json`。本轮维护报告、当天日志和进度入口，未覆盖正式安装或删除旧GUI。

## Windows自动检查

fmt、全目标Clippy（-D warnings）、MSVC Release通过。68项测试：49 core、原full_acceptance 4、full_acceptance_regression 5、GPUI选项2、platform8。新增两项选项本地化/ID稳定性回归已在本机执行，不是仅交叉编译；纯函数覆盖不代替八语言完整实屏。

候选自带PowerShell工具夹具20项通过。正常退出后原打包脚本再次运行，68项测试和全部检查仍通过。日志 `build-validation.txt`、`script-validation.txt`、`package-build.txt`；不沿用fix3 EXE/包哈希。本轮未重复九次底层剪贴板example，其fix2历史证据不计成本轮结果。

## 关键GUI证据

初始设置捕获422×601，英语保存基线与新等高按钮在 `settings-initial-english-dark70.png`。英语重置/展开证据：`reset-first-general-english.png`、`reset-language-list-english.png`、`reset-provider-list-english.png`、`reset-first-appearance-follow-system100.png`、`reset-theme-list-english.png`，其余页面分别保留。未保存哈希比较见 `reset-unsaved-disk.json`。

关键首次回滚：`reset-reopen-first-dark70-english.png`；受控保存失败：`save-failure-retains-preview-dark30.png`及`save-failure-validation.json`。默认保存及中英互转：`reset-default-saved-chinese-general.png`、`saved-chinese-language-list.png`、`chinese-to-english-saved-general.png`、`english-to-chinese-saved-general.png`。

第二阶段Alt+2唤出英语划词菜单，显式点击Translate产生1次stage5-text并完成。结果400×392首次显示完整，语言控件、正文和Copy Result可见；固定后激活合成夹具仍显示，取消固定再激活夹具隐藏。见 `phase2-result-first-display.png`。仅一次创建及限定失焦路径，不关闭V-02旧几何重复闸门。

Alt+4输入首次为空、Copy Result禁用。用户明确确认中文IME候选Enter只上屏不提交、Ctrl+Enter本地合成返回、固定后失焦保留/取消固定失焦隐藏、Alt+4重开输入和结果清空全部正常。第一阶段Alt+0与本阶段Alt+2/4均实际运行；初始自动Alt+0需重试，不认定产品热键故障。

用户明确复制Chrome合成矩形后立即回读：600×153，91,800像素；红14,076、绿30,600、蓝30,600、白16,524，other=0、overlay_border_blue=0，中行E02020→208020→2040E0→FFFFFF。见 `phase2/gui-clipboard-validation.json`。不保存剪贴板图片或读文字，只证明该矩形复制，不推广所有DPI/坐标。

截图动画、移动/八向缩放及Enter保持遮罩的本轮询问，用户仅回复“已复制”和“已退出”，未明确回答各项。因此**不借用fix3交互结果将本轮这些项目标通过**；Alt+3进入并完成Copy的路径有明确证据。本轮截图AI提取/翻译未执行。

最后请求框选→Enter→Esc并正常退出，用户回复已退出。约388秒人工区间剪贴板序号281→281，未读内容，见 `phase2/cancel-sequence-validation.json`。不证明精确遮罩区间零写入或零像素读取，取消隐私闸门继续待验。

## 两阶段隔离与退出恢复

第一阶段默认保存会清空测试模型，故结束后先正常托盘退出。夹具恢复settings.json、三个DPAPI文件和native-window-state.json，独立核对存在性/哈希/LastWriteTimeUtc五项均匹配，见 `restoration-phase1.json`。

第二阶段同一EXE未重编译，重新备份启动，并在AI前核对三路地址/模型仅指向127.0.0.1:14182对应固定合成服务。最终2请求/2完成，stage5-text=1、stage5-input=1，无拒绝/活动请求，见 `phase2/mock-service-final-status.json`。无真实外部AI服务，不保存请求/响应内容。

第二阶段用户正常退出后，同样五原文件独立核对全部匹配，见 `phase2/restoration-validation.json`。两阶段私有备份精确路径已清理；候选、合成桌面夹具及14182监听均为0，见 `cleanup-validation.json`。受控失败读句柄已释放，未改系统权限。结束不重新启动应用，原安装和旧GUI保留。

## 最终产物

EXE14,152,704字节，SHA-256 `4C6B7D51EF7E59EBED3EF949C76C779882EB0109D68EDA0CE51CC5AF1C321B2C`，与两阶段GUI实测一致。

目录 `target/gpui-candidate/artifacts/`：

| 产物 | 字节 | SHA-256 |
| --- | ---: | --- |
| QuickPick_0.1.4_gpui_candidate_x64.zip | 7,650,683 | 8813C47F8417CBDA7210EBE6353B4EB684AC58FC7FDF977A90C8AE0F68099007 |
| QuickPick_0.1.4_gpui_candidate_x64_setup.exe | 4,286,177 | 0D71841231E9013693CC7969938F7421FC0FBACE012A2522F08027574A51E679 |

`package-validation.json`通过：ZIP937文件，解压18,965,634字节，516第三方包，实际ZIP/EXE一致、x64 GUI头及许可文件存在性正常。NSIS仅哈希校验，未运行安装/升级/回滚/卸载；许可文件存在性不替代许可证审查。

## 仍待补齐

两项针对性失败在已测路径闭环，完整验收仍包含：八语言全页与下拉完整实屏、保存期间禁用/加载态、125/150/200%DPI、更多主题/几何/窗口生命周期、划词固定/失焦本轮完整矩阵、V-02旧几何结果重复与V-03多轮还原、本轮截图交互明确实屏确认/截图AI动作、精确取消隐私证据。

多屏负坐标/混合DPI/竖屏及隔离安装升级卸载/无WebView2受条件限制；不以包静态校验替代。拖动期间模糊消失/松手恢复继续作为既有体验反馈，不改写为产品要求。本轮修复通过不能关闭这些剩余闸门。
