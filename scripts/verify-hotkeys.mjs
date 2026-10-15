#!/usr/bin/env node
import { runNativeHelper } from '../tools/run-native-helper.mjs';

try {
  const result = await runNativeHelper(['verify-hotkeys', ...process.argv.slice(2)]);
  process.exitCode = result.code;
} catch (error) {
  console.error(`快捷键验收入口无法启动：${error.message}`);
  process.exitCode = 1;
}
