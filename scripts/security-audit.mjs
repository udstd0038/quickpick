import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repoRoot = fileURLToPath(new URL("../", import.meta.url));
const rustRoot = fileURLToPath(
  new URL("../apps/desktop/src-tauri/", import.meta.url),
);
const nativeToolsRoot = fileURLToPath(new URL("../tools/native-helper/", import.meta.url));

function runCommand(label, command, args, cwd, options = {}) {
  const result = spawnSync(command, args, {
    cwd,
    stdio: "inherit",
    shell: false,
    ...options,
  });

  if (result.error) {
    throw new Error(`${label} could not start: ${result.error.message}`);
  }
  if (result.signal) {
    throw new Error(`${label} was terminated by signal ${result.signal}`);
  }
  if (result.status !== 0) {
    throw new Error(`${label} failed with exit code ${result.status}`);
  }
}

try {
  console.log("Running pnpm dependency audit...");
  if (process.platform === "win32") {
    // .cmd launchers require cmd.exe. Keep this command literal: never add
    // paths, environment values, or user input to the command string.
    runCommand(
      "pnpm audit",
      process.env.ComSpec || "cmd.exe",
      ["/d", "/s", "/c", "pnpm.cmd --filter quickpick-desktop audit --prod"],
      repoRoot,
    );
  } else {
    runCommand(
      "pnpm audit",
      "pnpm",
      ["--filter", "quickpick-desktop", "audit", "--prod"],
      repoRoot,
    );
  }

  try {
    runCommand(
      "cargo-audit availability check",
      "cargo-audit",
      ["--version"],
      rustRoot,
      { stdio: "ignore" },
    );
  } catch (error) {
    throw new Error(
      `${error.message}\nInstall cargo-audit with: cargo install cargo-audit --locked`,
    );
  }

  console.log("Running cargo dependency audit...");
  runCommand("cargo audit", "cargo", ["audit"], rustRoot);
  console.log("Auditing isolated development helper dependencies...");
  runCommand("native helper cargo audit", "cargo", ["audit"], nativeToolsRoot);
  console.log("Security audit passed.");
} catch (error) {
  console.error(`Security audit failed: ${error.message}`);
  process.exitCode = 1;
}
