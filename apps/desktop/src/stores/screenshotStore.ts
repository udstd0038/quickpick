import { create } from "zustand";

interface ScreenshotState {
  selected: boolean;
  status: "idle" | "capturing" | "ready" | "error";
  setSelected: (selected: boolean) => void;
  setStatus: (status: ScreenshotState["status"]) => void;
}

export const useScreenshotStore = create<ScreenshotState>((set) => ({
  selected: false,
  status: "idle",
  setSelected: (selected) => set({ selected }),
  setStatus: (status) => set({ status }),
}));
