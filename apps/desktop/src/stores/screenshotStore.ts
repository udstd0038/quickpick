import { create } from "zustand";

interface ScreenshotState {
  selected: boolean;
  status: "idle" | "capturing" | "ready" | "error";
}

export const useScreenshotStore = create<ScreenshotState>(() => ({
  selected: false,
  status: "idle",
}));
