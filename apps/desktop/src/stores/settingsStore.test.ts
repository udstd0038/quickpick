import { beforeEach, describe, expect, it } from "vitest";
import { defaultAppSettings } from "../lib/settingsTypes";
import { useSettingsStore } from "./settingsStore";

describe("settingsStore", () => {
  beforeEach(() => {
    useSettingsStore.setState({
      settings: defaultAppSettings,
      status: { kind: "idle", message: "设置已载入" },
    });
  });

  it("updates a setting and marks it dirty", () => {
    useSettingsStore.getState().updateSetting("aiTimeoutSeconds", 45);
    expect(useSettingsStore.getState().settings.aiTimeoutSeconds).toBe(45);
    expect(useSettingsStore.getState().status.message).toBe("有未保存的修改");
  });

  it("supports functional setter updates", () => {
    useSettingsStore.getState().setSettings((current) => ({
      ...current,
      themeMode: "light",
    }));
    expect(useSettingsStore.getState().settings.themeMode).toBe("light");
  });

  it("tracks saving status", () => {
    useSettingsStore.getState().setStatus({
      kind: "saving",
      message: "正在保存",
    });
    expect(useSettingsStore.getState().status.kind).toBe("saving");
  });
});
