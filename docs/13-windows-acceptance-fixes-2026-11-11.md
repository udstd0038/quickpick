# Windows 本轮验收问题修复

日期：2026-11-11（Asia/Hong_Kong）。本页对应用户提供的 [本机验收记录](12-windows-full-acceptance-2026-11-11.md)。原记录保留不改写。fix1 修复范围现已在 Windows 复测通过，见 [复测记录](14-windows-fix1-retest-2026-11-11.md)；剩余全量闸门见 [阶段 5](15-gpui-stage5-validation.md)，不需要重复修改已通过项。

## 已修复

| 问题 | 当前实现 |
| --- | --- |
| W-01 成功正文被裁剪 | 文本、输入、图片三个完成路径直接返回解码后的原始 content。trim 只用于判断是否全为空白，不修改有效正文；字面量 i18n: 不翻译 |
| PowerShell 吞掉 Clippy 的 -- | Invoke-Cargo 使用显式 string[] 参数和数组展开，完整传递 --、-D、warnings |
| 中文窗口标题误报 | Rust 子进程 stdout 显式按 UTF-8 解码 |
| ZIP 时间戳越界 | 只修正暂存副本的 LastWriteTime 到 ZIP 的 1980..2107 范围，不修改 Cargo 缓存或许可正文 |
| NSIS 多语言脚本编码 | 编译时显式传递 /INPUTCHARSET UTF8 |

用户记录中的四项脚本修复已同步到本工作区。由于用户仅上传验收报告、未上传本机脚本和测试源文件，本轮依据记录补齐实现，并独立新增 `full_acceptance_regression.rs`；不声称恢复了本机测试原文，也不覆盖本机已有的 `full_acceptance.rs`。

## 验证

- W-01 新增 HTTP 复现用例在修改前失败；修改后通过。全部测试只连接动态端口 127.0.0.1，使用合成数据，不输出请求/响应正文或测试凭据。
- Linux workspace：38 项 core 单元测试、5 项 HTTP 集成测试和 1 项平台测试通过。HTTP 用例覆盖三域正文原样、纯空白拒绝、各自端点/模型、HTTP 错误以及实际超时后独立重试。
- Windows GNU 全目标 Clippy（`--locked`、`-D warnings`）通过；Windows 特有代码和测试仅交叉检查，不是 Windows MSVC 运行。
- PowerShell 7.5.4（Linux）：三个修改脚本语法解析通过；实际外部进程夹具确认 Clippy 参数数组不丢 `--`；1970/2108 时间戳夹具规范后 ZIP 成功，文件哈希不变。这不代替 Windows PowerShell 5.1 本机复测。
- NSIS 3.08（Linux）以 UTF-8 和占位 payload 编译通过；不是可发行的 Windows 应用安装包。

## 已通过复测的命令与合并说明

可从修复包的全新 `snapshot` 构建，避免覆盖本机的验收夹具和脚本修改。若合并到已有工程，保留本机 `full_acceptance.rs`，应用 `W-01.patch` 并补入本包新增的回归文件；脚本可对照表格合并已有修复。

先运行本包新增测试：

```powershell
cargo test -p quickpick-core --test full_acceptance_regression --locked --target-dir target/gpui-candidate
```

本机已有的失败用例应继续保留并重跑：

```powershell
cargo test -p quickpick-core --test full_acceptance successful_response_preserves_leading_indentation_and_trailing_newline --locked --target-dir target/gpui-candidate
```

再执行完整构建、测试与候选启动；退出候选后重新打包：

```powershell
powershell -ExecutionPolicy Bypass -File scripts\build-gpui.ps1 -Launch
powershell -ExecutionPolicy Bypass -File scripts\build-gpui.ps1 -Package
```

fix1 随后已在用户本机完成修复范围复测与重新生成真实 MSVC release、ZIP 和 NSIS，见第 14 号报告；当前推进阶段 5 的本地执行器仍为 Linux，不将新工具的夹具结果扩展成剩余桌面/发行矩阵通过。旧安装实测为 0.1.3，后续升级/回退基线记录真实版本；无副屏的混合 DPI 场景继续列为受条件限制，不记为通过。
