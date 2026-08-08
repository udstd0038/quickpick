import { describe, expect, it } from "vitest";
import { useApiKeyStore } from "./apiKeyStore";

describe("apiKeyStore", () => {
  it("tracks configured status and input per scope", () => {
    useApiKeyStore.getState().setConfigured("text", true);
    useApiKeyStore.getState().setInput("text", "sk-test");
    useApiKeyStore.getState().setStatus("text", "已加密保存");

    const state = useApiKeyStore.getState();
    expect(state.configured.text).toBe(true);
    expect(state.input.text).toBe("sk-test");
    expect(state.status.text).toBe("已加密保存");
  });
});
