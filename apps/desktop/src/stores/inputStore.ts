import { create } from "zustand";

interface InputState {
  text: string;
  result: string;
  status: "waiting" | "loading" | "success" | "error";
  sourceLanguage: string;
  targetLanguage: string;
  direction: "left" | "right";
  setText: (text: string) => void;
  setResult: (result: string) => void;
  setStatus: (status: InputState["status"]) => void;
  setLanguages: (
    sourceLanguage: string,
    targetLanguage: string,
    direction: "left" | "right",
  ) => void;
  reset: () => void;
}

export const useInputStore = create<InputState>((set) => ({
  text: "",
  result: "",
  status: "waiting",
  sourceLanguage: "auto",
  targetLanguage: "zh-Hans",
  direction: "right",
  setText: (text) => set({ text }),
  setResult: (result) => set({ result }),
  setStatus: (status) => set({ status }),
  setLanguages: (sourceLanguage, targetLanguage, direction) =>
    set({ sourceLanguage, targetLanguage, direction }),
  reset: () =>
    set({
      text: "",
      result: "",
      status: "waiting",
      sourceLanguage: "auto",
      targetLanguage: "zh-Hans",
      direction: "right",
    }),
}));
