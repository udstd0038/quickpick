import type { AppSettings } from "./settingsTypes";

export type HotkeySettingKey =
  | "settingsHotkey"
  | "selectionHotkey"
  | "screenshotHotkey"
  | "inputTranslateHotkey";

export function sameShortcut(left: string, right: string): boolean {
  return (
    left.replace(/\s+/g, "").toLowerCase() ===
    right.replace(/\s+/g, "").toLowerCase()
  );
}

export function findHotkeyConflict(
  shortcut: string,
  settings: AppSettings,
  currentKey: HotkeySettingKey,
): HotkeySettingKey | null {
  const candidates: Array<[HotkeySettingKey, string]> = [
    ["settingsHotkey", settings.settingsHotkey],
    ["selectionHotkey", settings.selectionHotkey],
    ["screenshotHotkey", settings.screenshotHotkey],
    ["inputTranslateHotkey", settings.inputTranslateHotkey],
  ];

  for (const [key, value] of candidates) {
    if (key !== currentKey && sameShortcut(shortcut, value)) {
      return key;
    }
  }
  return null;
}
