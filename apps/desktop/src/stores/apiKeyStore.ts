import { create } from "zustand";

type ApiKeyScope = "text" | "vision" | "input";

interface ApiKeyState {
  configured: Record<ApiKeyScope, boolean>;
  status: Record<ApiKeyScope, string>;
}

export const useApiKeyStore = create<ApiKeyState>(() => ({
  configured: {
    text: false,
    vision: false,
    input: false,
  },
  status: {
    text: "未配置",
    vision: "未配置",
    input: "未配置",
  },
}));
