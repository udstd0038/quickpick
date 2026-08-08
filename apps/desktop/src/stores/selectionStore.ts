import { create } from "zustand";

interface SelectionState {
  text: string;
  preview: string;
  charCount: number;
  status: "empty" | "captured" | "unsupported" | "error";
  setText: (text: string, preview: string, charCount: number) => void;
  setStatus: (status: SelectionState["status"]) => void;
}

export const useSelectionStore = create<SelectionState>((set) => ({
  text: "",
  preview: "",
  charCount: 0,
  status: "empty",
  setText: (text, preview, charCount) => set({ text, preview, charCount }),
  setStatus: (status) => set({ status }),
}));
