# GPUI 阶段 0 验收

用户已确认 Windows 实机验证通过，并授权继续全量重构。阶段 0 结论以用户反馈为依据；本工作区没有新增实机日志或测量报告，不能由此推出后续业务和发行验收通过。

保留 `--feasibility` 独立样机，用于回归输入、材质、原生事件与实时透明遮罩：

```powershell
powershell -ExecutionPolicy Bypass -File scripts\verify-gpui-stage0.ps1 -Launch
```

样机不读取业务设置/密钥，不上传 AI。只有显式“复制选区”才捕获；进入、拖动和取消不捕获。原生测试热键为 `Alt+Shift+F12`，需显式启用。

当前已进入全量候选阶段，工程边界、构建、打包命令与新增验收见 [全量候选验收](11-gpui-full-validation.md)。迁移完成判据仍以 [重构方案](09-gpui-refactor-plan.md) 为准。
