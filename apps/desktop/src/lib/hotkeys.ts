import type { AppSettings } from "./settingsTypes";

export type HotkeySettingKey =
  | "settingsHotkey"
  | "selectionHotkey"
  | "screenshotHotkey"
  | "inputTranslateHotkey";

export function sameShortcut(left: string, right: string): boolean {
  const modifierAliases: Record<string, string> = {
    alt: "alt",
    option: "alt",
    ctrl: "ctrl",
    control: "ctrl",
    shift: "shift",
    command: "super",
    cmd: "super",
    meta: "super",
    super: "super",
    win: "super",
  };
  const normalize = (value: string) => {
    const parts = value.replace(/\s+/g, "").toLowerCase().split("+");
    const key = parts.pop() || "";
    const modifiers = parts.map((part) => modifierAliases[part]);
    if (!key || modifiers.some((modifier) => !modifier)) {
      return value.replace(/\s+/g, "").toLowerCase();
    }

    const normalizedKey = key
      .replace(/^key([a-z])$/, "$1")
      .replace(/^digit([0-9])$/, "$1");
    return [...new Set(modifiers)].sort().join("+") + "+" + normalizedKey;
  };

  return normalize(left) === normalize(right);
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
