import type { AppSettings } from "./settingsTypes";

export type HotkeySettingKey =
  | "settingsHotkey"
  | "selectionHotkey"
  | "screenshotHotkey"
  | "inputTranslateHotkey";

export function sameShortcut(left: string, right: string): boolean {
  const normalize = (value: string) =>
    value
      .replace(/\s+/g, "")
      .replace(/\b(?:Super|Cmd|Meta)\b/gi, "Command")
      .replace(/\bOption\b/gi, "Alt")
      .toLowerCase();

  return (
    normalize(left) === normalize(right)
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
