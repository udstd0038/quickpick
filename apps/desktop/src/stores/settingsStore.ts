import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import {
  defaultAppSettings,
  normalizeAppSettings,
  type AppSettings,
  type SettingsStatus,
} from "../windows/settings/SettingsWindow";

interface SettingsState {
  settings: AppSettings;
  status: SettingsStatus;
  updateSetting: <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K],
  ) => void;
  markDirty: (message?: string) => void;
  saveSettings: () => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: defaultAppSettings,
  status: { kind: "idle", message: "设置已载入" },
  updateSetting: (key, value) => {
    set((state) => ({
      settings: { ...state.settings, [key]: value },
      status: { kind: "idle", message: "有未保存的修改" },
    }));
  },
  markDirty: (message = "有未保存的修改") => {
    set({ status: { kind: "idle", message } });
  },
  saveSettings: async () => {
    const settings = normalizeAppSettings(get().settings);
    set({ status: { kind: "saving", message: "正在保存" } });
    try {
      await invoke("save_app_settings", { settings });
      set({
        settings,
        status: { kind: "success", message: "设置已保存" },
      });
    } catch (error) {
      set({
        status: {
          kind: "error",
          message: typeof error === "string" ? error : "保存设置失败",
        },
      });
    }
  },
}));
