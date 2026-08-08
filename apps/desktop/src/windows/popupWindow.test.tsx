// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  hide: vi.fn(),
  setFocus: vi.fn(),
  startDragging: vi.fn(),
  setAlwaysOnTop: vi.fn(),
  invoke: vi.fn(),
  listen: vi.fn(),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => mocks,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: mocks.listen,
}));

import { InputWindow } from "./input/InputWindow";
import { ResultWindow } from "./result/ResultWindow";
import { SelectionWindow } from "./selection/SelectionWindow";
import { useInputStore } from "../stores/inputStore";
import { useResultStore } from "../stores/resultStore";

describe("WebView popup behavior", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.hide.mockResolvedValue(undefined);
    mocks.setFocus.mockResolvedValue(undefined);
    mocks.startDragging.mockResolvedValue(undefined);
    mocks.setAlwaysOnTop.mockResolvedValue(undefined);
    mocks.listen.mockResolvedValue(() => undefined);
    useInputStore.setState({
      text: "",
      result: "",
      status: "waiting",
      sourceLanguage: "auto",
      targetLanguage: "zh-Hans",
      direction: "right",
    });
    useResultStore.setState({
      content: "",
      detail: "",
      status: "empty",
      sourceLanguage: "auto",
      targetLanguage: "zh-Hans",
      direction: "right",
      pinned: false,
    });
    Object.defineProperty(window, "focus", {
      value: vi.fn(),
      writable: true,
    });
  });

  afterEach(() => {
    cleanup();
  });

  it("closes the selection window on Escape", async () => {
    mocks.invoke.mockResolvedValue({
      status: "captured",
      text: "hello",
      preview: "hello",
      charCount: 5,
      message: "已读取",
      source: "uia",
    });

    render(<SelectionWindow />);
    await waitFor(() =>
      expect(mocks.invoke).toHaveBeenCalledWith("get_selection_snapshot"),
    );

    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(mocks.hide).toHaveBeenCalled());
  });

  it("hides the selection window after an action succeeds", async () => {
    mocks.invoke.mockResolvedValue({ message: "ok" });
    render(<SelectionWindow />);

    fireEvent.click(screen.getByRole("button", { name: "复制" }));
    await waitFor(() => expect(mocks.hide).toHaveBeenCalled());
  });

  it("closes the result window on Escape and toggles pinned state", async () => {
    render(<ResultWindow />);

    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(mocks.hide).toHaveBeenCalled());

    fireEvent.click(screen.getByRole("button", { name: "固定" }));
    await waitFor(() => expect(mocks.setAlwaysOnTop).toHaveBeenCalledWith(true));
  });

  it("closes the input window on Escape and sends Ctrl+Enter translation", async () => {
    mocks.invoke.mockResolvedValue({ message: "ok" });
    render(<InputWindow />);

    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(mocks.hide).toHaveBeenCalled());

    const textarea = screen.getByPlaceholderText("输入要翻译的文本");
    fireEvent.change(textarea, { target: { value: "hello" } });
    fireEvent.keyDown(textarea, { key: "Enter", ctrlKey: true });

    await waitFor(() =>
      expect(mocks.invoke).toHaveBeenCalledWith(
        "request_input_translation",
        expect.objectContaining({
          inputText: "hello",
          sourceLanguage: "auto",
          targetLanguage: "zh-Hans",
          direction: "right",
        }),
      ),
    );
  });
});
