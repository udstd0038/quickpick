// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  hide: vi.fn(),
  setFocus: vi.fn(),
  startDragging: vi.fn(),
  setAlwaysOnTop: vi.fn(),
  setSize: vi.fn(),
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
import { ScreenshotOverlay } from "./screenshot/ScreenshotOverlay";
import { SelectionWindow } from "./selection/SelectionWindow";
import { useInputStore } from "../stores/inputStore";
import { useResultStore } from "../stores/resultStore";
import { setI18nLanguage } from "../lib/i18n";
import type { ResultSnapshotPayload, SelectionSnapshot } from "../services/invoke";

const capturedSelection: SelectionSnapshot = {
  status: "captured",
  text: "hello",
  preview: "hello",
  charCount: 5,
  message: "已读取",
  source: "uia",
};

const emptySelection: SelectionSnapshot = {
  status: "empty",
  text: "",
  preview: "",
  charCount: 0,
  message: "i18n:selection.noText",
  source: "uia",
};

const screenshotResult: ResultSnapshotPayload = {
  status: "success",
  title: "图片文字提取",
  content: "Screenshot text",
  detail: "",
  sourceLanguage: "auto",
  targetLanguage: "zh-Hans",
  translationDirection: "right",
  canSwitchLanguage: true,
};

describe("WebView popup behavior", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setI18nLanguage("zh-Hans");
    mocks.hide.mockResolvedValue(undefined);
    mocks.setFocus.mockResolvedValue(undefined);
    mocks.startDragging.mockResolvedValue(undefined);
    mocks.setAlwaysOnTop.mockResolvedValue(undefined);
    mocks.setSize.mockResolvedValue(undefined);
    mocks.listen.mockResolvedValue(() => undefined);
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_selection_snapshot") {
        return Promise.resolve(capturedSelection);
      }
      if (command === "get_result_snapshot") {
        return Promise.resolve({
          status: "success",
          title: "文本翻译",
          content: "",
          detail: "",
          sourceLanguage: "auto",
          targetLanguage: "zh-Hans",
          translationDirection: "right",
          canSwitchLanguage: true,
        });
      }
      if (command === "get_input_snapshot" || command === "get_screenshot_snapshot") {
        return Promise.resolve(null);
      }
      return Promise.resolve({ message: "ok" });
    });
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
    setI18nLanguage("zh-Hans");
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
    render(<SelectionWindow />);

    await waitFor(() =>
      expect((screen.getByRole("button", { name: "复制" }) as HTMLButtonElement).disabled).toBe(false),
    );
    fireEvent.click(screen.getByRole("button", { name: "复制" }));
    await waitFor(() => expect(mocks.hide).toHaveBeenCalled());
  });

  it("shows a localized selection failure and prevents empty actions", async () => {
    mocks.invoke.mockResolvedValue(emptySelection);
    render(<SelectionWindow />);

    expect(await screen.findByText("未读取到可操作的选中文本，请重新选择后再试")).toBeTruthy();
    for (const button of screen.getAllByRole("button")) {
      expect((button as HTMLButtonElement).disabled).toBe(true);
      fireEvent.click(button);
    }
    expect(mocks.invoke).not.toHaveBeenCalledWith("copy_selection_text");
    expect(mocks.invoke).not.toHaveBeenCalledWith("run_text_ai_action", expect.anything());
    expect(mocks.invoke).not.toHaveBeenCalledWith("search_selection_text");
    expect(screen.queryByText("i18n:selection.noText")).toBeNull();
  });

  it("refreshes a reused selection window on each selection-ready event", async () => {
    mocks.invoke.mockResolvedValue(emptySelection);
    render(<SelectionWindow />);
    await screen.findByText("未读取到可操作的选中文本，请重新选择后再试");
    const ready = mocks.listen.mock.calls.find(([name]) => name === "selection-ready")![1];

    act(() => ready({ payload: capturedSelection }));
    expect(screen.getByRole("main").title).toBe("hello");
    expect((screen.getByRole("button", { name: "复制" }) as HTMLButtonElement).disabled).toBe(false);

    fireEvent.blur(window);
    expect((screen.getByRole("button", { name: "复制" }) as HTMLButtonElement).disabled).toBe(true);

    act(() => ready({ payload: { ...capturedSelection, text: "world", preview: "world" } }));
    expect(screen.getByRole("main").title).toBe("world");
    fireEvent.click(screen.getByRole("button", { name: "复制" }));
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("copy_selection_text"));
  });

  it.each([
    ["selection.hotkeyHeld", "Release the hotkey, then select the text again and retry"],
    ["selection.sourceUnfocused", "The source app lost focus. Return to it and select the text again"],
    ["selection.controlUnavailable", "Unable to read the source app's text control"],
  ])("preserves the specific selection failure reason for %s", async (key, message) => {
    setI18nLanguage("en");
    mocks.invoke.mockResolvedValue({ ...emptySelection, status: "error", message: `i18n:${key}` });
    render(<SelectionWindow />);

    expect(await screen.findByText(message)).toBeTruthy();
    expect(screen.getByRole("main").title).toBe(message);
    expect((screen.getByRole("button", { name: "Copy" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("refreshes the selection snapshot when the reused window regains focus", async () => {
    render(<SelectionWindow />);
    await screen.findByTitle("hello");

    fireEvent.blur(window);
    mocks.invoke.mockImplementation((command: string) =>
      Promise.resolve(command === "get_selection_snapshot"
        ? { ...capturedSelection, text: "world", preview: "world" }
        : { message: "ok" }),
    );
    fireEvent.focus(window);

    await screen.findByTitle("world");
    expect(mocks.invoke.mock.calls.filter(([name]) => name === "get_selection_snapshot")).toHaveLength(2);
    expect((screen.getByRole("button", { name: "复制" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("does not replace a new selection event with an older snapshot response", async () => {
    let resolveSnapshot!: (snapshot: SelectionSnapshot) => void;
    mocks.invoke.mockImplementation(() => new Promise<SelectionSnapshot>((resolve) => {
      resolveSnapshot = resolve;
    }));
    render(<SelectionWindow />);
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("get_selection_snapshot"));
    const ready = mocks.listen.mock.calls.find(([name]) => name === "selection-ready")![1];

    act(() => ready({ payload: capturedSelection }));
    await act(async () => resolveSnapshot(emptySelection));

    expect(screen.getByRole("main").title).toBe("hello");
    expect((screen.getByRole("button", { name: "复制" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("localizes a selection error delivered in the result snapshot", async () => {
    setI18nLanguage("en");
    mocks.invoke.mockResolvedValue({
      status: "error",
      content: "",
      detail: "i18n:selection.noText",
      sourceLanguage: "auto",
      targetLanguage: "zh-Hans",
      translationDirection: "right",
    });
    render(<ResultWindow />);

    expect(await screen.findByText("No operable selected text, select again")).toBeTruthy();
    expect(screen.queryByText("i18n:selection.noText")).toBeNull();
  });

  it("localizes result errors from events while preserving successful content", async () => {
    render(<ResultWindow />);
    await waitFor(() => expect(mocks.listen).toHaveBeenCalled());
    const ready = mocks.listen.mock.calls.find(([name]) => name === "result-ready")![1];
    const payload = {
      status: "error",
      content: "i18n:selection.noText",
      detail: "",
      sourceLanguage: "auto",
      targetLanguage: "zh-Hans",
      translationDirection: "right",
    };

    act(() => ready({ payload }));
    expect(screen.getByText("未读取到可操作的选中文本，请重新选择后再试")).toBeTruthy();

    act(() => ready({ payload: { ...payload, status: "success" } }));
    expect(screen.getByText("i18n:selection.noText")).toBeTruthy();
  });

  it("does not let an old selection error snapshot replace a new screenshot result", async () => {
    let resolveSnapshot!: (snapshot: ResultSnapshotPayload) => void;
    mocks.invoke.mockImplementation(() => new Promise<ResultSnapshotPayload>((resolve) => {
      resolveSnapshot = resolve;
    }));
    render(<ResultWindow />);
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("get_result_snapshot"));
    const ready = mocks.listen.mock.calls.find(([name]) => name === "result-ready")![1];

    act(() => ready({ payload: screenshotResult }));
    await act(async () => resolveSnapshot({
      ...screenshotResult,
      status: "error",
      content: "i18n:selection.noText",
    }));

    expect(screen.getByText("Screenshot text")).toBeTruthy();
    expect(useResultStore.getState().status).toBe("success");
    expect(screen.queryByText("未读取到可操作的选中文本，请重新选择后再试")).toBeNull();
  });

  it("keeps a result event received while the listener is registering", async () => {
    let finishRegistration!: (cleanup: () => void) => void;
    mocks.listen.mockImplementation(() => new Promise<() => void>((resolve) => {
      finishRegistration = resolve;
    }));
    render(<ResultWindow />);
    expect(mocks.invoke).not.toHaveBeenCalledWith("get_result_snapshot");
    const ready = mocks.listen.mock.calls.find(([name]) => name === "result-ready")![1];

    act(() => ready({ payload: screenshotResult }));
    await act(async () => finishRegistration(() => undefined));

    expect(screen.getByText("Screenshot text")).toBeTruthy();
    expect(mocks.invoke).not.toHaveBeenCalledWith("get_result_snapshot");
  });

  it("does not restore an old result snapshot after closing the window", async () => {
    let resolveSnapshot!: (snapshot: ResultSnapshotPayload) => void;
    mocks.invoke.mockImplementation((command: string) => command === "get_result_snapshot"
      ? new Promise<ResultSnapshotPayload>((resolve) => { resolveSnapshot = resolve; })
      : Promise.resolve({ message: "ok" }),
    );
    render(<ResultWindow />);
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("get_result_snapshot"));

    fireEvent.keyDown(window, { key: "Escape" });
    await act(async () => resolveSnapshot(screenshotResult));

    expect(useResultStore.getState().status).toBe("empty");
    expect(useResultStore.getState().content).toBe("");
  });

  it("cleans up a result listener that finishes registering after unmount", async () => {
    let finishRegistration!: (cleanup: () => void) => void;
    mocks.listen.mockImplementation(() => new Promise<() => void>((resolve) => {
      finishRegistration = resolve;
    }));
    const { unmount } = render(<ResultWindow />);
    const unsubscribe = vi.fn();

    unmount();
    await act(async () => finishRegistration(unsubscribe));

    expect(unsubscribe).toHaveBeenCalledOnce();
    expect(mocks.invoke).not.toHaveBeenCalledWith("get_result_snapshot");
  });

  it("closes the result window on Escape and toggles pinned state", async () => {
    render(<ResultWindow />);

    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(mocks.hide).toHaveBeenCalled());

    fireEvent.click(screen.getByRole("button", { name: "固定" }));
    await waitFor(() => expect(mocks.setAlwaysOnTop).toHaveBeenCalledWith(true));
  });

  it("closes the input window on Escape and sends Ctrl+Enter translation", async () => {
    mocks.invoke.mockImplementation((command: string) =>
      command === "get_input_snapshot"
        ? Promise.resolve(null)
        : Promise.resolve({ message: "ok" }),
    );
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

  it("copies the latest input translation result", async () => {
    mocks.invoke.mockResolvedValue({ message: "ok" });
    useInputStore.setState({
      result: "translated result",
      status: "success",
    });
    render(<InputWindow />);

    fireEvent.click(screen.getByRole("button", { name: "复制" }));
    await waitFor(() =>
      expect(mocks.invoke).toHaveBeenCalledWith("copy_input_result"),
    );
  });

  it("renders a skeleton while the result is loading", () => {
    mocks.invoke.mockImplementation((command: string) =>
      command === "get_result_snapshot"
        ? new Promise<never>(() => {})
        : Promise.resolve({ message: "ok" }),
    );
    useResultStore.setState({ status: "loading" });

    render(<ResultWindow />);

    expect(screen.getByTestId("result-loading-skeleton")).toBeTruthy();
  });

  it("renders a skeleton while the input result is loading", () => {
    mocks.invoke.mockImplementation((command: string) =>
      command === "get_input_snapshot"
        ? new Promise<never>(() => {})
        : Promise.resolve({ message: "ok" }),
    );
    useInputStore.setState({ status: "loading" });

    render(<InputWindow />);

    expect(screen.getByTestId("input-loading-skeleton")).toBeTruthy();
  });

  it("renders a skeleton while the screenshot is being captured", () => {
    mocks.invoke.mockImplementation((command: string) =>
      command === "get_screenshot_snapshot"
        ? new Promise<never>(() => {})
        : Promise.resolve({ message: "ok" }),
    );

    render(<ScreenshotOverlay />);

    expect(screen.getByTestId("screenshot-loading-skeleton")).toBeTruthy();
  });

  it("closes the screenshot overlay on Escape", () => {
    render(<ScreenshotOverlay />);

    fireEvent.keyDown(window, { key: "Escape" });
    expect(mocks.hide).toHaveBeenCalled();
  });

  it("cancels the screenshot overlay on right-click", () => {
    const { container } = render(<ScreenshotOverlay />);
    const overlay = container.firstChild as HTMLElement;

    fireEvent.contextMenu(overlay);
    expect(mocks.hide).toHaveBeenCalled();
  });
});
