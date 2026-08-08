import { create } from "zustand";

interface ResultState {
  content: string;
  detail: string;
  status: "empty" | "placeholder" | "loading" | "success" | "error";
  sourceLanguage: string;
  targetLanguage: string;
  direction: "left" | "right";
  pinned: boolean;
  setContent: (content: string, detail?: string) => void;
  setStatus: (status: ResultState["status"]) => void;
  setLanguages: (
    sourceLanguage: string,
    targetLanguage: string,
    direction: "left" | "right",
  ) => void;
  setPinned: (pinned: boolean) => void;
}

export const useResultStore = create<ResultState>((set) => ({
  content: "",
  detail: "",
  status: "empty",
  sourceLanguage: "auto",
  targetLanguage: "zh-Hans",
  direction: "right",
  pinned: false,
  setContent: (content, detail = "") => set({ content, detail }),
  setStatus: (status) => set({ status }),
  setLanguages: (sourceLanguage, targetLanguage, direction) =>
    set({ sourceLanguage, targetLanguage, direction }),
  setPinned: (pinned) => set({ pinned }),
}));
