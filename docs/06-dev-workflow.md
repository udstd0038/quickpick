# QuickPick 开发工作流

## 1. 每次开发开始前

必须先做：

- 阅读 `AGENT.md`。
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
- 代码文件按项目技术栈规范执行。
- 不把敏感数据写入仓库。
- 不提交 API Key、截图内容、AI 响应全文。

## 3. 每次开发结束前

必须完成：

- 运行适合本阶段的检查或说明无法运行的原因。
- 若本次修改影响应用代码、配置、资源或发行产物，在可运行的 Windows 开发环境覆盖默认 release 并启动 `apps/desktop/src-tauri/target/release/quickpick.exe` 供验收；其他平台记录无法启动的原因及对应原生 CI/设备验收状态。
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

## 6. 文档优先级

发生冲突时按以下顺序判断：

1. 用户最新明确要求。
2. `AGENT.md` 工作说明。
3. `docs/00-product-requirements.md`。
4. `docs/04-security-privacy-spec.md`。
5. 其他 `docs/` 标准文件。
6. 当前代码实现。

安全和隐私规则优先级高于便利性。

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
- 应用 Rust 目录的 `cargo audit`
- `tools/native-helper/` 目录的 `cargo audit`

如果本机未安装 `cargo-audit`，先执行：

```bash
cargo install cargo-audit --locked
```

仓库同时提供 `.github/workflows/security-audit.yml`，在 push、PR 和每周计划任务中使用 Ubuntu/Bash 调用相同审计入口。Windows、macOS 和 Linux 本地开发均可使用 `pnpm security:audit`，无需 PowerShell。高危漏洞不允许进入 release。

## 9. 开发工具

先使用仓库锁定版本安装 Node.js/pnpm 依赖，并准备 Rust 工具链。资源生成使用已安装的 Tauri CLI 和仓库原生 helper，不下载独立渲染器、不依赖 PowerShell。

```bash
pnpm tools:icons
pnpm tools:installer-assets
```

默认输出分别为 `apps/desktop/src-tauri/icons/` 和 `apps/desktop/src-tauri/installer/`。开发新资源时先使用 `--output-dir <临时目录>` 验证文件结构、尺寸、透明度及实际外观，再替换已验收资源。应用 PNG/ICO/ICNS、透明托盘图标和安装器 BMP 不能仅凭文件存在就报告生成正确。任何子工具失败都必须返回非零。

图标 SVG 源稿保留在 `apps/desktop/src-tauri/icons/source/`；安装器布局源稿保留在 `tools/assets/`，文字使用轮廓以避免系统字体差异。Rust helper 有独立 `Cargo.lock`，构建使用 `--locked`。生成器会清理中间文件，仅发布桌面应用需要的资源。

资源集成检查在临时目录真实渲染 PNG/ICO/ICNS/托盘/BMP，运行：

```bash
pnpm test:tools
cargo test --locked --manifest-path tools/native-helper/Cargo.toml --target-dir tools/native-helper/target
```

快捷键与 README 截图工具必须在交互式 Windows 桌面运行，并使用 opt-in 的专用 fixture，不能将正在运行的正式应用或普通 release 程序作为输入。fixture binary 名称为 `quickpick-fixture`，没有 `dev-tools` feature 时不会构建；该 feature 不加入默认发行构建。

从仓库根目录执行以下命令。显式 `--target-dir` 固定产物位置，避免本地 Cargo 默认目录与示例路径不同；`tauri/custom-protocol` 将刚构建的前端嵌入 fixture，不依赖开发服务器。

```text
pnpm build
cargo build --locked --release --features dev-tools,tauri/custom-protocol --bin quickpick-fixture --manifest-path apps/desktop/src-tauri/Cargo.toml --target-dir apps/desktop/src-tauri/target
pnpm verify:hotkeys --fixture-exe apps/desktop/src-tauri/target/release/quickpick-fixture.exe
pnpm capture:readme --fixture-exe apps/desktop/src-tauri/target/release/quickpick-fixture.exe --output-dir tool-validation --themes light,dark --languages zh-Hans,en --include-screenshot
```

Node.js 入口分别为 `scripts/verify-hotkeys.mjs` 和 `tools/capture-readme-screenshots.mjs`。它们按调用者工作目录解析路径，自动以独立锁文件构建原生 helper，并保留 helper 的退出码。必须显式提供 `--fixture-exe`；启动前检查编译标记，启动后核实身份、随机 token、隔离目录和 loopback 控制接口，禁止附着其他进程。

fixture 使用独立应用标识、设置、窗口状态和 WebView profile；取词、截图 RGBA、Key 配置状态与 AI 结果都是固定样例，不读取真实用户配置、Key、选区、剪贴板或屏幕，不修改真实自启、不向 AI 服务发请求。导出只用 `PrintWindow` 读取 owned 窗口客户区，关闭透明玻璃以避免背景穿透，禁止桌面 `BitBlt` 兜底；只结束工具拥有的 PID 并清理本次临时目录。

`capture:readme` 默认导出 zh-Hans/en、light/dark 的设置、划词与输入窗口；`--themes` 和 `--languages` 允许选择其子集，`--include-screenshot` 额外导出合成截图界面。`--output-dir` 默认是调用者当前目录，内部仍使用 `screenshots/zh-cn/<窗口>-<主题>.png` 与 `screenshots/en/<窗口>-<主题>.png`，最大宽度 1280 px。建议先输出到 `tool-validation`，检查实际界面后再替换 README 图片。

退出码 0 表示请求的动作或图片全部验证成功；1 表示失败；2 表示“未执行”，包括非 Windows、无可用交互桌面、无法激活 owned 窗口或无法获取非空白 WebView 图片。`.github/workflows/development-tools.yml` 检查资源、编译并测试 helper/fixture，同时尝试交互检查；遇到 2 时在 Summary 显示 `NOT RUN` 并保留人工验收待办，不能因其他步骤通过就将交互检查标为通过。

该工作流另外提供独立的 `windows-release` job：不启用 `dev-tools` 运行默认应用 Rust 测试，再使用 `tauri build --bundles nsis --ci --no-sign -- --locked` 构建默认 Windows release 和 NSIS 安装包。`scripts/record-windows-build.mjs` 核对实际 checkout 与 workflow 提交一致，并将提交、源码树、版本、默认 feature 状态、可执行文件及安装包的 SHA256 写入 `build-info.json`。产物 `quickpick-windows-default-<完整提交 SHA>` 保留 14 天，仅供验收，不发布 Release。

`windows-acceptance` job 还上传平铺的 `quickpick-windows-desktop-check-<完整提交 SHA>`，包含 `quickpick-fixture.exe`、`quickpick-native-helper.exe` 和 `source-commit.txt`。在已登录、未锁屏的 Windows 11 桌面解压后，可以从该目录的前台终端直接运行以下命令，不需要安装 Node.js、pnpm 或 Rust：

```text
.\quickpick-native-helper.exe verify-hotkeys --fixture-exe .\quickpick-fixture.exe
.\quickpick-native-helper.exe capture-readme --fixture-exe .\quickpick-fixture.exe --output-dir tool-validation --themes light,dark --languages zh-Hans,en --include-screenshot
```

每条命令分别记录退出码及完整结果，并用 `source-commit.txt` 关联本次候选；fixture 不得当作日常应用使用。默认应用与安装包构建通过不代表上述交互命令通过，fixture 交互通过也不替代真实跨应用取词、剪贴板、多屏截图或安装/升级/登录自启验收。

macOS 工具 PR #5 保持草稿，等待其本次提交的完整 CI 及 Windows 11 交互结果；实体 Mac 验收继续延期，签名、公证与正式分发尚未完成。各阶段 CI 与手动验收必须按精确源码及产物提交分别记录，不能把编译成功计作真实桌面动作通过。当前已验收的二进制图标与安装器资源未替换，生成工具只在临时目录验证新结果。

## 10. 收尾说明

每次回复用户时应简要说明：

- 做了什么。
- 修改了哪些关键文件。
- 如何验证。
- 下一步建议。

如果有检查没跑，必须说明原因。
