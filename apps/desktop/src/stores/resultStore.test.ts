import { describe, expect, it } from "vitest";
import { useResultStore } from "./resultStore";

describe("resultStore", () => {
  it("keeps language and direction state in sync", () => {
    useResultStore.getState().setLanguages("auto", "en", "left");

    const state = useResultStore.getState();
    expect(state.sourceLanguage).toBe("auto");
    expect(state.targetLanguage).toBe("en");
    expect(state.direction).toBe("left");
  });
});
