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
const tauriConfigPath = path.join(desktopRoot, "src-tauri", "tauri.conf.json");
const libSourcePath = path.join(desktopRoot, "src-tauri", "src", "lib.rs");

describe("security baseline", () => {
  it("does not grant the broad core:default permission set", async () => {
    const capability = JSON.parse(
      await readFile(capabilityPath, "utf8"),
    ) as {
      permissions: string[];
    };

    expect(capability.permissions).not.toContain("core:default");
    expect(capability.permissions).not.toContain("core:window:default");
    expect(capability.permissions).toContain("core:event:allow-listen");
    expect(capability.permissions).toContain("core:event:allow-unlisten");
    expect(capability.permissions).toContain("core:window:allow-start-dragging");
    expect(capability.permissions).toContain("core:window:allow-set-decorations");
  });

  it("enables a non-null WebView CSP", async () => {
    const config = JSON.parse(await readFile(tauriConfigPath, "utf8")) as {
      app: { security: { csp: string | null } };
    };

    expect(config.app.security.csp).toBeTruthy();
    expect(config.app.security.csp).toContain("default-src 'self'");
  });

  it("does not open URLs through a cmd shell fallback", async () => {
    const source = await readFile(libSourcePath, "utf8");

    expect(source).not.toContain('Command::new("cmd")');
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
