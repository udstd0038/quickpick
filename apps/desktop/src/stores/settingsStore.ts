import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import {
  defaultAppSettings,
  normalizeAppSettings,
  type AppSettings,
  type SettingsStatus,
} from "../lib/settingsTypes";

interface SettingsState {
  settings: AppSettings;
  status: SettingsStatus;
  setSettings: (
    settings: AppSettings | ((previous: AppSettings) => AppSettings),
  ) => void;
  setStatus: (
    status: SettingsStatus | ((previous: SettingsStatus) => SettingsStatus),
  ) => void;
  updateSetting: <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K],
  ) => void;
  markDirty: (message?: string) => void;
  saveSettings: () => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: defaultAppSettings,
  status: { kind: "idle", message: "settings.loaded" },
  setSettings: (settings) => {
    set((state) => ({
      settings:
        typeof settings === "function"
          ? settings(state.settings)
          : settings,
    }));
  },
  setStatus: (status) => {
    set((state) => ({
      status:
        typeof status === "function" ? status(state.status) : status,
    }));
  },
  updateSetting: (key, value) => {
    set((state) => ({
      settings: { ...state.settings, [key]: value },
      status: { kind: "idle", message: "settings.unsaved" },
    }));
  },
  markDirty: (message = "settings.unsaved") => {
    set({ status: { kind: "idle", message } });
  },
  saveSettings: async () => {
    const settings = normalizeAppSettings(get().settings);
    set({ status: { kind: "saving", message: "settings.saving" } });
    try {
      await invoke("save_app_settings", { settings });
      set({
        settings,
        status: { kind: "success", message: "settings.saved" },
      });
    } catch (error) {
      set({
        status: {
          kind: "error",
          message: typeof error === "string" ? error : "settings.saveFailed",
        },
      });
    }
  },
}));
