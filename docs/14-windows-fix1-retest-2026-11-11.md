# Windows fix1 本机复测

日期：2026-11-11（Asia/Hong_Kong）。本记录针对 `quickpick-gpui-full-candidate-fix1-2026-11-11.zip`，不改写上一轮[全量验收记录](12-windows-full-acceptance-2026-11-11.md)。

结论：**W-01 与四项脚本修复的本机复测通过**。这一结论不等于全量产品、安装升级和性能验收通过。

## 输入与复现用例保留

- 修复包 SHA-256：`C28D258DDEED879B702196BBECE49E5D51EEE51FECE4593D46F9813E1EC5D06C`。
- 采用全新独立目录 `gpui-full-candidate-fix1/snapshot`，121 个清单文件及完整补丁哈希校验通过。没有向上一轮候选目录覆盖应用或脚本。
- 原样复制上一轮本机 `crates/quickpick-core/tests/full_acceptance.rs` 到新快照。复制前后 SHA-256 均为 `45BD3B0B5C451D1DC11F80A26CACC114B696A119C36FE331D3B3E8AF5F0B1D3D`，没有弱化、忽略或重写原始失败断言。
- Windows 11 x64，Rust/Cargo `1.96.1`，MSVC 工具链；实际使用 Windows PowerShell `5.1.26100.9711` 和本机现有 Windows SDK/Build Tools。
- 测试请求仅连接动态端口 `127.0.0.1`，使用固定合成输入和测试凭据；日志不输出请求、响应正文或凭据，没有调用用户真实 AI 服务。

## 复测结果

| 范围 | 本机结果 |
| --- | --- |
| 新增 `full_acceptance_regression` | 5/5 通过；覆盖文本/输入/图片三域的缩进、制表符、CR/LF 与末尾换行保留，`i18n:` 字面量、纯空白拒绝、域端点/模型、HTTP 错误、真实超时及随后独立重试 |
| 原本机失败用例 | `full_acceptance::successful_response_preserves_leading_indentation_and_trailing_newline` 原样重跑通过，退出码 0 |
| 原本机完整 HTTP 验收 | `full_acceptance` 全部 4/4 通过 |
| 原有 core 单元测试 | 38/38 通过 |
| Windows platform/DPAPI | 2/2 通过，合成文件的加解密与域隔离，不代表用户真实旧密钥文件已完成回读矩阵 |
| 完整 workspace | **49 项通过，0 失败，0 忽略**；不重复累计单独运行的同一用例 |
| 格式与 Clippy | `cargo fmt --all --check`、全 workspace/all-targets Clippy `-D warnings` 通过 |
| MSVC Release | `scripts/build-gpui.ps1` 在 Windows PowerShell 5.1 成功，候选 `quickpick.exe` 已生成 |
| Clippy 参数修复 | 未修改 fix1 提供的脚本，实际完整脚本成功运行，`--` / `-D warnings` 未丢失 |
| 中文窗口标题解码 | 启动 fix1 后，自带热键脚本的设置/划词/输入/截图路由和预期窗口显示检查全部通过，适用工具窗口的 Esc 检查通过 |
| ZIP 时间戳 | 复用打包脚本原始规范化代码测试 1970/2108 年文件，分别规范为 1980/2107；内容哈希不变，Windows PowerShell 5.1 `Compress-Archive` 成功。实际候选 ZIP 同样生成成功 |
| NSIS UTF-8 | 本机便携 NSIS 3.13 以 `/INPUTCHARSET UTF8` 成功处理八语言脚本并生成真实安装包，没有使用占位 EXE |
| 依赖审计 | 锁文件 868 个包，已知漏洞 0，停止维护提示 8，无忽略列表；RustSec 数据库 commit `31cf5a8673e10d3a4d7d829ad5095f47ed419049` |

三个 AI 完成路径的代码差异仅将“先裁剪正文再判空”改为“只以 trim 判断是否为空白，返回原 content”。本轮没有再修改 fix1 的应用代码或三个构建/验证脚本；新增的本机测试复制件与本报告单独保留。

## 启动、退出与打包边界

- 用户确认正常退出上一轮候选后，本机进程检查确认无其他 QuickPick，随后启动 fix1 `--candidate`，实测进程路径位于新快照目录。
- fix1 的四条热键均通过，本轮没有启动旧版与新候选并行运行。
- 初次打包在等待上一轮候选退出期间进行；当时 fix1 尚未运行且使用独立输出目录，不会覆盖运行中的旧候选。
- 用户随后确认正常退出 fix1；进程检查确认没有 QuickPick 后，重新完整运行 `build-gpui.ps1 -Package`，格式、Clippy、49 项测试、Release 和 NSIS 均成功。候选在本轮结束时已退出。

## 最终产物

| 文件（相对新快照根目录） | 字节数 | SHA-256 |
| --- | ---: | --- |
| `target/gpui-candidate/release/quickpick.exe` | 13,969,408 | `2D139FEBAF97A06BD5B604945FA1E578602DD256A21ADCE2B98C065B84AB3CF5` |
| `target/gpui-candidate/artifacts/QuickPick_0.1.4_gpui_candidate_x64.zip` | 7,530,790 | `05BAE3DA9ADD6E9239D2D3DCF072A7B7202B6D5DC267D388C0A394F1FE9ED69E` |
| `target/gpui-candidate/artifacts/QuickPick_0.1.4_gpui_candidate_x64_setup.exe` | 4,175,928 | `54A31F12EB36254D965B431C7CF205234CAA241BD743B3FA3440AD383A32B93D` |

最终 SHA-256 清单位于同目录 `SHA256.json`。ZIP 含 937 个条目，包括 933 个第三方许可目录文件、EXE、项目许可证及声明/来源清单。生成安装包不代表已经执行新装、升级或卸载。打包器包含时间戳等构建信息，因此本轮最终产物哈希替代首次打包的哈希；没有将这种差异当作可复现构建结论。

## 证据

`target/gpui-candidate/gpui-fix1-reports/` 保存原失败用例复测、完整构建、热键、打包、ZIP 时间戳夹具、审计和测试来源哈希；自动生成的依赖树和许可字段清单位于 `target/gpui-candidate/gpui-stage0-reports/`。

原始包文件、上一轮本机 `full_acceptance.rs` 和上一轮全量报告均保留。新增报告与开发日志的更新不会被当作 fix1 原始快照哈希一致的一部分。

## 保留待验项目

[第 12 号报告](12-windows-full-acceptance-2026-11-11.md) 中尚未完成的真实 IME、完整截图像素/交互、浏览器/Office、实际旧配置/密钥回读、锁屏/挂起、八语言完整视觉、安装升级/回退/卸载、无 WebView2 干净机和受控旧新性能对比仍未完成。旧安装基线仍为实测 `0.1.3`；本机无副屏，负原点和混合 DPI 实机矩阵不能标记通过。本次未覆盖正式安装或修改正式自启动。
