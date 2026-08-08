import { describe, expect, it } from "vitest";
import { useInputStore } from "./inputStore";

describe("inputStore", () => {
  it("updates text and languages", () => {
    useInputStore.getState().setText("你好");
    useInputStore
      .getState()
      .setLanguages("zh-Hans", "en", "left");

    const state = useInputStore.getState();
    expect(state.text).toBe("你好");
    expect(state.sourceLanguage).toBe("zh-Hans");
    expect(state.targetLanguage).toBe("en");
    expect(state.direction).toBe("left");
  });
});
