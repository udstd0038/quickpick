import { describe, expect, it } from "vitest";
import { defaultAppSettings } from "./settingsTypes";
import { findHotkeyConflict, sameShortcut } from "./hotkeys";

describe("hotkeys", () => {
  it("normalizes shortcut comparison", () => {
    expect(sameShortcut("Alt+2", " alt + 2 ")).toBe(true);
    expect(sameShortcut("Alt+2", "Alt+3")).toBe(false);
  });

  it("compares macOS modifier aliases and order as the same shortcut", () => {
    expect(sameShortcut("Command+Option+2", "Option+Command+2")).toBe(true);
    expect(
      sameShortcut("Command+Ctrl+Shift+Option+Q", "Alt+Shift+Control+Meta+KeyQ"),
    ).toBe(true);
    expect(sameShortcut("Command+Option+2", "alt + cmd + Digit2")).toBe(true);
    expect(sameShortcut("Command+Option+2", "Super+Alt+2")).toBe(true);
    expect(sameShortcut("Command+Option+2", "Command+Option+3")).toBe(false);
    expect(sameShortcut("Command+Option+2", "Ctrl+Option+2")).toBe(false);
    expect(
      sameShortcut("Command+Option+2", "Command+Shift+Option+2"),
    ).toBe(false);
  });

  it("keeps Windows shortcut meanings and the final key distinct", () => {
    expect(
      sameShortcut("Ctrl+Shift+Alt+Win+Q", "Super+Alt+Shift+Control+KeyQ"),
    ).toBe(true);
    expect(sameShortcut("Ctrl+Alt+2", "Alt+Control+Digit2")).toBe(true);
    expect(sameShortcut("Alt+2", "Ctrl+Alt+2")).toBe(false);
    expect(sameShortcut("Ctrl+Alt+Q", "Alt+Ctrl+W")).toBe(false);
    expect(sameShortcut("Command+A", "A+Command")).toBe(false);
  });

  it("detects conflicts against reordered shortcuts loaded from settings", () => {
    const settings = {
      ...defaultAppSettings,
      selectionHotkey: "Option+Command+2",
      screenshotHotkey: "Command+Option+3",
      inputTranslateHotkey: "Command+Option+4",
    };

    expect(
      findHotkeyConflict("Command+Option+2", settings, "screenshotHotkey"),
    ).toBe("selectionHotkey");
    expect(
      findHotkeyConflict("Command+Option+2", settings, "selectionHotkey"),
    ).toBeNull();
  });

  it("detects conflicts for every hotkey field", () => {
    const settings = {
      ...defaultAppSettings,
      selectionHotkey: "Alt+2",
      screenshotHotkey: "Alt+3",
      inputTranslateHotkey: "Alt+4",
    };

    expect(
      findHotkeyConflict("Alt+2", settings, "settingsHotkey"),
    ).toBe("selectionHotkey");
    expect(
      findHotkeyConflict("Alt+4", settings, "screenshotHotkey"),
    ).toBe("inputTranslateHotkey");
    expect(
      findHotkeyConflict("Alt+9", settings, "inputTranslateHotkey"),
    ).toBeNull();
  });
});
