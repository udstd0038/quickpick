# Windows GPUI 视觉候选验收

本报告仅覆盖 `quickpick-gpui-visual-candidate-2026-11-12.zip` 的独立 Windows/MSVC 快照。本轮结论为视觉验收不通过：存在实屏确认的设置窗口最大化还原裁切及英语文案不完整，其他矩阵亦有待验项。不沿用 fix2 的 GUI 或打包结论。正式安装未被覆盖，无副屏及隔离安装环境。

## 来源与构建

- 输入 ZIP：5,051,654 字节，SHA-256 `C8D252A847F81B42B9580502A1BE5C7E9F0455B2006D89663F8BDC652685C32A`。
- 归档 157 项；清单 156 个文件的长度与 SHA 校验通过。复用本机 `full_acceptance.rs`；Cargo.lock 未变更，应用源码未修改。
- `verify-gpui-validation.ps1`：20 项通过。`build-gpui.ps1`：格式、all-targets Clippy（-D warnings）、Windows MSVC release 均通过；56 项测试通过（42 core、4 HTTP、5 本机回归、5 platform）。
- 显式运行 `verify_clipboard_image --replace-clipboard-with-synthetic-image`：9 个 CF_DIB/CF_BITMAP 合成尺寸、行序与 RGB 回读用例通过。尚不能据此关闭本候选的实际 GUI 复制验收。
- 日志：`target/gpui-candidate/gpui-visual-reports/build-validation.txt`、`clipboard-roundtrip.txt`。
- 最终 EXE：14,150,144 字节，SHA-256 `2598A98E5547EE1722A7C1E9AC8E7FEA6F66074CF76AAB6F8DCEEF2849D86F12`；退出后重新打包的 EXE 与本轮 GUI 测试文件哈希一致。

## 已完成的实机检查

环境：Windows 11 25H2，build 26200.9711；显示缩放 100%（96 DPI）。已保存 30 张部分覆盖的截图，位于 `screenshots/gpui-visual/`，文件哈希索引见 `target/gpui-candidate/gpui-visual-reports/screenshot-index.json`。测试内容仅使用固定合成文本、合成四色画面与本地回环假 AI 服务。

- 设置：中文浅色七分类均可达，快捷键内容可滚动到底，底部保存区固定；英文长导航可横向滚动到外观/隐私。
- 外观预览：浅深主题立即变化；未保存预览不写磁盘；最小化后重开保留预览，关闭后重开回滚；Mica/70% 预览关闭后恢复 Acrylic/100%。
- 使用无效业务 URL 草稿后仍可预览；保存失败显示 Base URL 错误并保留预览，磁盘仍为原合法回环 URL，未触发 AI。修正 URL 后可保存。
- 输入：中英文深色空态、加载、成功、503 错误与重试已观察。固定按钮点击未拖动窗口；固定后失焦保留，取消固定后失焦隐藏，再打开清空内容。
- 真实中文 IME：物理按键 n/i 出现候选，空格提交“你”；普通 Enter 换行未增加请求，Ctrl+Enter 明确提交并成功。尚未覆盖组合输入未提交时的 Ctrl+Enter。
- 划词：从合成 WinForms 文本框 Alt+2 显示单行中文深色工具条，四个动作与固定/关闭同排；明确点击翻译返回合成结果。
- 结果：交互重绘后实际复制成功；固定合成响应的首尾空白回读保持不变，检查结果见 `text-copy-result.json`。
- 已有固定结果窗：保存 UI 英语及浅色主题后，下拉文案与外观更新，仍保留 Auto Detect → Simplified Chinese。服务请求数保存前后均为 5，没有自动提交 AI。
- Chrome 实际截图：用户确认实时计数/橙条继续运动、选区移动及八方向缩放正常，Enter 保留遮罩，明确点击 Copy 成功。回读 612×130、79,560 像素：红 14,430、绿 26,000、蓝 26,000、白 13,130；四色顺序正确，其他颜色及边框蓝均为 0。`gui-copy-colors.json` 为本候选实测证据；此限定场景 S5-01 GUI 回归通过，不能外推全部 DPI/坐标。服务请求数仍为 5，截图交互/复制未新增 AI 请求。

## 发现与证据边界

| 编号 | 实际行为 | 证据与状态 |
| --- | --- | --- |
| V-01 | 英语通用页剪贴板兜底标签、隐私页部分正文仍是中文 | `settings-general-en-dark-settled-100.png`、`settings-privacy-en-dark-100.png`；locales.json 英语对应值亦为中文。属于本候选验收问题，尚未判定是否较 fix2 新增。 |
| V-02 | 结果窗恢复既有约 400×392 截图尺寸时，首次显示裁切，目标语言/固定/关闭/复制不可见；复制 UIA 位置越界。后续鼠标交互触发重绘后，同尺寸内恢复全部控件并可复制 | `result-success-restored-clipped-zh-cn-dark-100.png` 与 `result-success-redrawn-zh-cn-dark-100.png`。只确认观察到的首次恢复布局异常，根因与稳定复现次数待核实。 |
| V-03 | 设置窗口最大化后还原，右侧标题按钮和底部保存区裁切，切换通用后仍裁切 | `settings-restored-clipped-en-light-100.png`；用户最新明确确认“实际显示有裁切”，因此确认实屏缺陷。Alt+F4 隐藏后 Alt+0 重开恢复默认尺寸及完整布局，只是规避路径，不是验收通过。 |

原生下拉弹出层在捕获图中存在叠加观感差异，暂不作为确认缺陷，需区分窗口捕获合成与真实屏幕显示。

设置最大化后导航和固定保存区可见；还原后连续两次捕获均裁切右侧标题按钮及底部保存区，点击通用后仍裁切。用户已确认实际屏幕也有裁切。V-02 的结果窗场景尚未获得独立实屏确认，不能仅因设置窗存在相似问题就认定二者根因相同。

Windows 显示设置的自动化应用授权超时，未更改显示缩放，125/150/200% 尚未执行。

## 取消与环境恢复

- 用户完成 Alt+3、框选、Enter、Esc 后从托盘正常退出。剪贴板序号 145 → 145，观察区间未发现剪贴板写入；两次记录相隔约 29,022 秒，包含用户等待，不能当作精确包围遮罩打开/关闭的受控时序证据，也不能证明应用未读取像素。`cancel-sequence-result.json` 保存序号及限制，未读取剪贴板内容。
- 候选进程已退出。原 settings.json、三个 DPAPI 密钥文件均由启动器恢复，再次独立核对哈希一致；native-window-state.json 另行恢复且哈希一致。`restoration-validation.json` 保存五项布尔结果。
- 全部恢复成功后已清理本轮私有临时备份；本地假 AI 服务已停止，6740 端口无监听。未覆盖正式安装，未修改系统缩放。

## 待完成

- 中英文浅深全窗口截图矩阵、八语言长标签、125/150/200% DPI；裁切修复后重新验证最大化/还原及各类重开尺寸。
- 长文本滚动、更多键盘/悬停状态、截图工具栏主题矩阵、取消受控证据。
- 隔离安装升级、多屏保持待验；不把缺少设备的项目记为通过。

## 最终打包与包校验

正常退出后执行 `build-gpui.ps1 -Package`，格式、Clippy、56 项测试及 MSVC release 再次通过，生成本轮独立 ZIP/NSIS。`verify-gpui-package.ps1 -Executable ...` 通过：ZIP 内 EXE 与 release 哈希一致，x64 PE32+ GUI 头正确，937 个文件、516 个依赖包的许可材料存在。NSIS 仅哈希检查，未执行安装、升级、卸载；许可文件存在不等于许可人工审核完成。

| 文件 | 字节 | SHA-256 |
| --- | ---: | --- |
| `QuickPick_0.1.4_gpui_candidate_x64.zip` | 7,648,224 | `B4E08DF2F1F7F98480C19FA0D4B30A89322982E3966C3ACB8539ACD9532413B9` |
| `QuickPick_0.1.4_gpui_candidate_x64_setup.exe` | 4,283,488 | `E0760BE3672FB5CDC3B13FE98B421FF25B5E420869086A075A3385E782C12014` |

产物：`target/gpui-candidate/artifacts/`；日志及校验：`target/gpui-candidate/gpui-visual-reports/package-build.txt`、`package-validation.json`。这些产物保留已发现的视觉缺陷，仅供复现与后续修复对照，未切换正式发行。
