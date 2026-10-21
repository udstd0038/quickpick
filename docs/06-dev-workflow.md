# QuickPick 开发工作流

## 1. 每次开发开始前

必须先做：

- 阅读 `AGENTS.md`。
- 阅读与本次任务相关的 `docs/` 文件。
- 查看当天开发日志是否存在。
- 确认本次只做一个小阶段或一个明确子任务。

如果任务范围过大，应先拆分，不要一次性做太多。

## 2. 每次开发中

执行原则：

- 先了解现状，再修改。
- 优先遵循现有结构。
- 不引入与本阶段无关的大依赖。
- 不做顺手重构。
- 不实现未进入 MVP 的功能。
- 复杂系统调用要有最小可验证用例。

文件修改原则：

- 文档修改保持中文。
- 文档和代码注释只描述当前状态，按 `AGENTS.md` 的硬规则维护当前有效决策，不把临时规划或未经确认的实现偏差写成业务规则。
- 代码文件按项目技术栈规范执行。
- 不把敏感数据写入仓库。
- 不提交 API Key、截图内容、AI 响应全文。

## 3. 每次开发结束前

必须完成：

- 运行适合本阶段的检查或说明无法运行的原因。
- 若本次修改影响应用代码、配置、资源或发行产物，在可运行的 Windows 环境完成验证、覆盖默认 release 后自动启动 `apps/desktop/src-tauri/target/release/quickpick.exe`，方便用户直接验收新版；环境不支持时说明原因和待验证范围。纯文档修改无需构建或启动应用。
- 更新 `dev-logs/YYYY-MM-DD.md`。
- 写明今日完成、待办、关键决策、问题、验证结果、下次建议。
- 如果修改了需求、架构、设计、安全或测试规则，同步更新对应 `docs/` 文件。

## 4. 开发日志格式

每日日志使用以下结构：

```markdown
# YYYY-MM-DD 开发日志

## 今日完成

- ...

## 今日待办

- ...

## 关键决策

- ...

## 遇到问题

- ...

## 验证结果

- ...

## 下次建议

- ...
```

## 5. 小步推进标准

一个合格的小阶段应满足：

- 目标清楚。
- 文件范围有限。
- 能被测试或检查。
- 完成后项目状态更稳定。

不合格的阶段例子：

- “把所有功能都做完。”
- “顺便重构所有架构。”
- “先加一堆库以后可能用。”

## 6. 冲突处理

统一遵循 [`AGENTS.md` 的冲突处理规则](../AGENTS.md#冲突处理)：

- 代码、测试和必要运行验证核实实际行为；当前明确的用户要求、有效决策与 Contract 确定预期行为。
- 先识别描述过期、实现回归、测试错误或已获批准的行为变更，再分别修订文档、修复实现、修正测试或按授权同步受影响内容。
- 测试通过不能单独证明符合有效决策，也不能据此覆盖有效决策、弱化 Claim 或降低证据强度。
- 已有明确授权则执行；依据不足时说明具体冲突与影响，只对尚有分歧的部分请求判断，其余工作继续。

预期行为的优先级和文档硬规则以 `AGENTS.md` 为准，安全和隐私要求优先于开发便利。

## 7. 推荐开发节奏

建议每次只做下列之一：

- 一个窗口骨架。
- 一个 Tauri 命令。
- 一个系统集成能力。
- 一个设置项。
- 一个 AI 调用链路。
- 一个验收修复。

不要把多个核心能力混在一次开发里。

## 8. 安全审计

发布前必须执行：

```bash
pnpm security:audit
```

跨平台入口 `scripts/security-audit.mjs` 会从仓库根目录和 Rust 项目目录依次执行以下命令；任一步失败都会返回非零退出码：

- `pnpm --filter quickpick-desktop audit --prod`
- 应用 `cargo audit`
- `tools/native-helper` 独立锁文件的 `cargo audit`

如果本机未安装 `cargo-audit`，先执行：

```bash
cargo install cargo-audit --locked
```

仓库同时提供 `.github/workflows/security-audit.yml`，在 push、PR 和每周计划任务中使用 Ubuntu/Bash 调用相同审计入口。Windows、macOS 和 Linux 本地开发均可使用 `pnpm security:audit`，无需 PowerShell。高危漏洞不允许进入 release。

## 9. 收尾说明

每次回复用户时应简要说明：

- 做了什么。
- 修改了哪些关键文件。
- 如何验证。
- 下一步建议。

如果有检查没跑，必须说明原因。

## 10. 开发资源和 Windows 验收工具

活跃入口为 `pnpm tools:icons`、`pnpm tools:installer-assets`、`pnpm verify:hotkeys --fixture-exe <独立 fixture.exe>` 和 `pnpm capture:readme --fixture-exe <独立 fixture.exe>`；原四个 PowerShell 工具已移除，第一阶段的安全审计入口仍为 `pnpm security:audit`。

资源生成复用项目 Tauri CLI 与已锁定的 `image` PNG 解码器；不使用 PowerShell、系统字体或联网字体。Windows UI 驱动使用标准 Win32 `SendInput`、窗口/focus 检查及自有窗口 `PrintWindow`，无需额外 npm 自动化依赖。

默认发行不启用 `dev-tools`。需要交互验收时，先 `pnpm build`，再使用 `cargo build --locked --release --features dev-tools,tauri/custom-protocol --bin quickpick-fixture --manifest-path apps/desktop/src-tauri/Cargo.toml --target-dir apps/desktop/src-tauri/target` 构建独立工具 binary。只把该 binary 的路径传给工具；PE marker、握手 token/PID/构建标识验证失败即拒绝启动或验收，结束时只回收本次工具创建的进程及临时目录。

`.github/workflows/development-tools.yml` 的 Ubuntu/Bash 阶段检查实际资源、helper 测试及 Windows API 交叉编译；Windows/cmd 的独立 `windows-release` 阶段使用默认 feature 测试应用并构建 release 和 NSIS，上传可执行文件、安装包与包含源码提交、源码树和 SHA256 的 `build-info.json`。独立 `windows-acceptance` 阶段检查 fixture 和实际桌面验收，并上传仅含合成内容的验收程序。无桌面时记录“未执行”，保留人工验收项，不报告功能通过。Linux 开发环境不能启动 Windows release exe，必须在日志记录并通过 Windows CI/桌面补验。

本次变更位于 `codex/powershell-tools-main-20261015` 草稿开发分支，已同步主线划词与 Firefox 截图修复，尚未合并默认 `main`，因此不能把此工作区的无 PowerShell 入口状态当作默认分支已更新。macOS 功能及实体 Mac 验收在独立 macOS 分支推进；没有可用实体 Mac，权限、Retina、外屏、Spaces、安装升级与登录自启验收统一保留待办。

下载 `quickpick-windows-desktop-check-<提交 SHA>` artifact 并解压后，可在已登录且未锁屏的 Windows 11 前台终端运行：

```cmd
quickpick-native-helper.exe verify-hotkeys --fixture-exe quickpick-fixture.exe
quickpick-native-helper.exe capture-readme --fixture-exe quickpick-fixture.exe --output-dir tool-validation --themes light,dark --languages zh-Hans,en --include-screenshot
```

退出码 0 才表示该项通过；退出码 2 仍为未执行。记录 artifact 提交、Windows 版本、两项退出码和错误类型；fixture 通过也不能替代默认应用的真实跨应用取词、截图、安装、升级和自启验收。
