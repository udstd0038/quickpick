#!/usr/bin/env node
import { runNativeHelper } from './run-native-helper.mjs';

try {
  const result = await runNativeHelper(['capture-readme', ...process.argv.slice(2)]);
  process.exitCode = result.code;
} catch (error) {
  console.error(`README 截图入口无法启动：${error.message}`);
  process.exitCode = 1;
}
