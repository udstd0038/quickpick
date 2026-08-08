import { create } from "zustand";

interface InputState {
  text: string;
  result: string;
  status: "waiting" | "loading" | "success" | "error";
  sourceLanguage: string;
  targetLanguage: string;
  direction: "left" | "right";
}

export const useInputStore = create<InputState>(() => ({
  text: "",
  result: "",
  status: "waiting",
  sourceLanguage: "auto",
  targetLanguage: "zh-Hans",
  direction: "right",
}));
