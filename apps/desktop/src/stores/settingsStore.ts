import { create } from "zustand";
import {
  defaultAppSettings,
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
}

export const useSettingsStore = create<SettingsState>((set) => ({
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
}));
