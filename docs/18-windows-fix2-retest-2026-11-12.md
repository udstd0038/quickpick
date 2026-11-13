# fix2 Windows 复测（S5-01 修复范围通过）

对象为独立解压的 `quickpick-gpui-full-candidate-fix2-2026-11-12.zip`，SHA256 `11B1EBAD95582EC8A7728CF2E40E2E43111C0F18DE3AA348F1A097D1E9C04044`。保留 fix1 失败工程及第 16 号报告，未覆盖正式安装。

## 完整性与自动化证据

- 同步清单的 136 个文件长度与 SHA256 全部匹配。未修改上传的应用源码。使用新 snapshot，不向 fix1 强行应用补丁。
- 根 Cargo.lock 与 fix1 哈希均为 `18754557839ABB06B91272CCEEDCEB8F90D83333BFF4471B4217E60D36D166F9`。
- 原本机 `full_acceptance.rs` 原样复制，SHA256 `45BD3B0B5C451D1DC11F80A26CACC114B696A119C36FE331D3B3E8AF5F0B1D3D`；未重写或忽略原断言。
- Windows MSVC `cargo test --workspace --locked`：38 core 单元 + 4 原本机验收 + 5 HTTP 回归 + 5 platform/DPAPI/编码测试，共 52 项通过，无失败或忽略。不与历史测试数量相加。
- 显式运行 `verify_clipboard_image --replace-clipboard-with-synthetic-image`：3 轮 × 3 尺寸共 9 次真实 `probe::copy_image` 复制通过；分别读取 CF_DIB/CF_BITMAP，逐像素比较 RGB、尺寸和行方向。未捕获屏幕或保存图片。缺少替换标记时退出码 2，拒绝操作。
- Windows PowerShell 5.1 的 20 项验收工具夹具通过，包括新增多许可声明场景；未对工具作二次兼容性修改。
- `scripts/build-gpui.ps1` 返回 0：fmt、全目标 Clippy 拒绝警告、完整测试、MSVC Release 及依赖闭包检查通过。
- `scripts/verify-hotkeys.ps1` 在运行中的 fix2 上返回 0：settings/selection/input/screenshot 四路显示通过，非设置工具 Esc 隐藏通过；只用合成文本，无 AI 提交。
- 新 Release EXE 为 13,969,920 字节，SHA256 `BEC41716B966B9EBA6166FA941FC3E66E9128E0DC3334001A004793C1E6C83EF`。旧 fix1 的 EXE/ZIP/NSIS 哈希不适用于 fix2。

## GUI 截图复测

fix2 以 --candidate 运行，Chrome 打开本地合成动态页面。用户反馈“八个方向、Enter、取消 都正常”，八向缩放、Enter 保持遮罩和 Esc 取消的人工交互结果通过。测试结束后用户正常退出，已确认无 QuickPick 进程。

取消前后只读剪贴板序号为 52→68，证据为 `../target/gpui-candidate/gpui-fix2-reports/clipboard-cancel-result.json`。两次读数之间包含打开 Chrome、人工操作及回复的长间隔，无法确定变化来源。“取消不改变剪贴板”未获证实，也不直接归因为取消回归。随后用进程标题观察缩短窗口，用户再次框选/Enter/Esc 后退出，但观察器未捕获窗口打开/关闭，结果仍无法判定（`clipboard-cancel-controlled.json`），不能将其 unchanged=false 当成应用失败。未读取用户剪贴板正文。

用户确认 GUI 明确复制成功，不再报位图写入失败。首次截图为 846×435，修正像素检查工具的 PowerShell byte 位移截断后，得到红/绿/蓝各 44,000 个精确原色像素，白色 231,410，其他 4,600，选区边框蓝色 0；该截图包含矩形外白边与深色内容，不能据此宣称整幅图片只有源色、没有工具栏。首次未转换 Int32 的错误检查结果已用 `gui-copy-pixels-invalidated.json` 标记无效，不能作为应用缺陷证据；有效初次结果为 `gui-copy-pixels-checked.json`。

用户再次通过 GUI 明确复制纯四色窄带，检查结果为 717×177，共 126,909 像素：红 26,727、绿 35,400、蓝 35,400、白 29,382、其他 0、选区边框蓝色 0；中间行颜色顺序为红→绿→蓝→白，与原合成页面一致。整幅图片均为预期源色，本次结果未混入暗化、边框或工具栏像素。只在内存中读取，没有保存图片。证据为 `gui-copy-band-pixels.json`。未额外记录框选显示尺寸，因此不外推所有坐标/DPI 场景均准确。

S5-01 原失败在真实函数及本次 GUI 复制中均不再复现，修复范围复测通过；不代表阶段 5 全量验收通过。

## 最终产物

候选正常退出后重新执行 build-gpui.ps1 -Package。首次因当前 PATH 缺 NSIS 中止，保留日志；仅在重跑进程中加入上一轮已验证的工作区便携 NSIS 3.13 路径，未更改系统 PATH、应用代码或构建脚本。重跑返回 0，52 项测试及全部构建检查再次通过，ZIP/NSIS/SHA256.json 生成成功。

| 产物 | 字节 | SHA256 |
| --- | ---: | --- |
| QuickPick_0.1.4_gpui_candidate_x64.zip | 7,531,006 | 96299A2A606B52FE845E944E9869D579561E0EC1720EB53F7DA7A7DDC82E6024 |
| QuickPick_0.1.4_gpui_candidate_x64_setup.exe | 4,176,744 | 46F2E77DA5D2F5B2A1C72DB573E90570C5B672EFCD0B2CE6EFE8A056AA17D567 |

实际 `verify-gpui-package.ps1` 校验通过：937 个 ZIP 文件、解压载荷 18,782,850 字节、516 个第三方声明包；路径、清单、许可材料存在性、x64 PE32+ GUI 头和 EXE 一致性通过。打包前后 Release EXE 哈希均为上述 BEC41716…，与热键/GUI 复测对象一致。证据为 `../target/gpui-candidate/artifacts/package-validation.json`。NSIS 仅生成及哈希校验，未执行安装；许可材料存在不代替法律审查，哈希一致不等于发布者签名。

## 范围

本轮未提交 AI，不临时替换用户配置或 DPAPI 密钥。fix1 的资源观察、基本 IME、提取/翻译等历史证据仍以第 16 号报告的版本与范围为准，不自动扩展为 fix2 全量产品通过。

安装升级/回退/卸载/无 WebView2 仍缺可还原隔离环境；多屏混合 DPI 仍无设备。严格晚到请求、固定/失焦、跨应用取词与多格式恢复、单屏 DPI/系统事件等剩余矩阵未因本修复完成。

本轮结论：S5-01 修复范围通过；阶段 5 全量验收仍未完成，取消不改剪贴板的自动证据仍待复核。保留旧 GUI 和正式安装；修复版本保持退出，后续使用本目录新产物，不使用 fix1 哈希。
