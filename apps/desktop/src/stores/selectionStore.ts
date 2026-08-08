import { create } from "zustand";

interface SelectionState {
  text: string;
  preview: string;
  charCount: number;
  status: "empty" | "captured" | "unsupported" | "error";
}

export const useSelectionStore = create<SelectionState>(() => ({
  text: "",
  preview: "",
  charCount: 0,
  status: "empty",
}));
