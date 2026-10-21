import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile, readdir, stat, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { join, relative, resolve } from "node:path";

const root = fileURLToPath(new URL("../", import.meta.url));
const target = resolve(root, process.env.CARGO_TARGET_DIR || "apps/desktop/src-tauri/target");
const release = join(target, "release");
const installers = join(release, "bundle", "nsis");
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const commit = git("rev-parse", "HEAD");
const tree = git("rev-parse", "HEAD^{tree}");
if (process.env.GITHUB_SHA && process.env.GITHUB_SHA !== commit) {
  throw new Error("The checked-out source does not match the workflow commit");
}
const names = (await readdir(installers)).filter((name) => name.endsWith(".exe")).sort();
if (!names.length) throw new Error("The default NSIS installer is missing");
const files = [join(release, "quickpick.exe"), ...names.map((name) => join(installers, name))];
const artifacts = [];
for (const path of files) {
  const bytes = await readFile(path);
  if (bytes.length < 64 || bytes.toString("ascii", 0, 2) !== "MZ" || !(await stat(path)).isFile()) {
    throw new Error(`Expected a Windows executable: ${relative(release, path)}`);
  }
  artifacts.push({
    path: relative(release, path).replaceAll("\\", "/"),
    bytes: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  });
}
await writeFile(join(release, "build-info.json"), JSON.stringify({
  commit,
  tree,
  version: JSON.parse(await readFile(join(root, "package.json"), "utf8")).version,
  platform: "windows-x86_64",
  features: "default; dev-tools disabled",
  signed: false,
  artifacts,
}, null, 2) + "\n");
console.log(`Recorded ${artifacts.length} default Windows build artifacts for ${commit}`);
