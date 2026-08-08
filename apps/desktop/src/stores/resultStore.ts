import { create } from "zustand";

interface ResultState {
  content: string;
  detail: string;
  status: "empty" | "loading" | "success" | "error";
  sourceLanguage: string;
  targetLanguage: string;
  direction: "left" | "right";
  pinned: boolean;
}

export const useResultStore = create<ResultState>(() => ({
  content: "",
  detail: "",
  status: "empty",
  sourceLanguage: "auto",
  targetLanguage: "zh-Hans",
  direction: "right",
  pinned: false,
}));
