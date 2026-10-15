import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const nativeRoot = fileURLToPath(new URL("./native-helper/", import.meta.url));
const manifest = join(nativeRoot, "Cargo.toml");
const targetRoot = join(nativeRoot, "target");
let build;

function run(command, args, cwd = nativeRoot) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd, stdio: "inherit", shell: false });
    child.once("error", reject);
    child.once("exit", (code, signal) => {
      if (signal) reject(new Error(`Native tool terminated by ${signal}`));
      else resolve({ code: code ?? 1 });
    });
  });
}

export async function runNativeHelper(args) {
  build ??= run("cargo", ["build", "--release", "--locked", "--manifest-path", manifest, "--target-dir", targetRoot]);
  const result = await build;
  if (result.code !== 0) return result;
  const executable = join(targetRoot, "release", process.platform === "win32" ? "quickpick-native-helper.exe" : "quickpick-native-helper");
  // CLI paths and the default README output are relative to the caller, while
  // compilation always uses the helper manifest's fixed directory.
  return run(executable, args, process.cwd());
}
