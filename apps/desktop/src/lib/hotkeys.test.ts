import { describe, expect, it } from "vitest";
import { defaultAppSettings } from "./settingsTypes";
import { findHotkeyConflict, sameShortcut } from "./hotkeys";

describe("hotkeys", () => {
  it("normalizes shortcut comparison", () => {
    expect(sameShortcut("Alt+2", " alt + 2 ")).toBe(true);
    expect(sameShortcut("Alt+2", "Alt+3")).toBe(false);
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
