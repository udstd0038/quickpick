import { create } from "zustand";

interface UiState {
  settingsNav: string;
  pinnedWindows: Record<string, boolean>;
}

export const useUiStore = create<UiState>(() => ({
  settingsNav: "general",
  pinnedWindows: {},
}));
