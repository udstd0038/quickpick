import { create } from "zustand";

type ApiKeyScope = "text" | "vision" | "input";

interface ApiKeyState {
  configured: Record<ApiKeyScope, boolean>;
  input: Record<ApiKeyScope, string>;
  status: Record<ApiKeyScope, string>;
  setConfigured: (scope: ApiKeyScope, configured: boolean) => void;
  setInput: (scope: ApiKeyScope, input: string) => void;
  setStatus: (scope: ApiKeyScope, status: string) => void;
}

export const useApiKeyStore = create<ApiKeyState>((set) => ({
  configured: {
    text: false,
    vision: false,
    input: false,
  },
  input: {
    text: "",
    vision: "",
    input: "",
  },
  status: {
    text: "未配置",
    vision: "未配置",
    input: "未配置",
  },
  setConfigured: (scope, configured) =>
    set((state) => ({
      configured: { ...state.configured, [scope]: configured },
    })),
  setInput: (scope, input) =>
    set((state) => ({
      input: { ...state.input, [scope]: input },
    })),
  setStatus: (scope, status) =>
    set((state) => ({
      status: { ...state.status, [scope]: status },
    })),
}));
