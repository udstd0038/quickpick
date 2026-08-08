import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const desktopRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
const srcDir = path.join(desktopRoot, "src");
const capabilityPath = path.join(
  desktopRoot,
  "src-tauri",
  "capabilities",
  "default.json",
);

describe("security baseline", () => {
  it("does not grant the broad core:default permission set", async () => {
    const capability = JSON.parse(
      await readFile(capabilityPath, "utf8"),
    ) as {
      permissions: string[];
    };

    expect(capability.permissions).not.toContain("core:default");
    expect(capability.permissions).toContain("core:event:allow-listen");
    expect(capability.permissions).toContain("core:event:allow-unlisten");
    expect(capability.permissions).toContain("core:window:allow-start-dragging");
  });

  it("does not store settings or API keys in browser storage", async () => {
    const files = (await readdir(srcDir, { recursive: true })).filter(
      (file) =>
        /\.(ts|tsx)$/.test(file) &&
        file !== "security.test.ts" &&
        !file.endsWith(".test.ts"),
    );

    for (const file of files) {
      const content = await readFile(path.join(srcDir, file), "utf8");
      expect(content).not.toMatch(/localStorage/);
      expect(content).not.toMatch(/sessionStorage/);
    }
  });
});
