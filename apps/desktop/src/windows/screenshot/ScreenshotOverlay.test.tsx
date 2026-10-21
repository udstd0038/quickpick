// @vitest-environment jsdom
import type { ReactNode } from "react";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { MonitorScreenshotPayload } from "../../services/invoke";
import { setI18nLanguage } from "../../lib/i18n";

const mocks = vi.hoisted(() => ({
  hide: vi.fn(),
  invoke: vi.fn(),
  listen: vi.fn(),
  handlers: new Map<string, (event: { payload: unknown }) => void>(),
}));

vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => mocks }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("react-image-crop", () => ({
  default: ({ children, onComplete }: {
    children: ReactNode;
    onComplete: (pixel: unknown, percent: unknown) => void;
  }) => (
    <div>
      {children}
      <button type="button" onClick={() => onComplete(
        { unit: "px", x: 10, y: 20, width: 100, height: 50 },
        { unit: "%", x: 1, y: 2, width: 10, height: 5 },
      )}>
        Finish crop
      </button>
    </div>
  ),
}));

import { ScreenshotOverlay } from "./ScreenshotOverlay";

const screenshot: MonitorScreenshotPayload = {
  width: 2048,
  height: 1536,
  monitorX: 100,
  monitorY: 200,
  monitorName: "Test screen",
  pngDataUrl: "data:image/png;base64,c2NyZWVu",
};
const nextScreenshot: MonitorScreenshotPayload = {
  ...screenshot,
  pngDataUrl: "data:image/png;base64,bmV3",
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((done, fail) => {
    resolve = done;
    reject = fail;
  });
  return { promise, resolve, reject };
}

function emit(name: string, payload: unknown) {
  act(() => mocks.handlers.get(name)!({ payload }));
}

describe("screenshot activation and actions", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.handlers.clear();
    setI18nLanguage("zh-Hans");
    mocks.hide.mockResolvedValue(undefined);
    mocks.listen.mockImplementation((name: string, handler: (event: { payload: unknown }) => void) => {
      mocks.handlers.set(name, handler);
      return Promise.resolve(vi.fn());
    });
    mocks.invoke.mockImplementation((command: string) => Promise.resolve(
      command === "get_screenshot_snapshot" ? screenshot : { message: "ok" },
    ));
    Object.defineProperty(window, "innerWidth", { value: 1024, configurable: true });
    Object.defineProperty(window, "innerHeight", { value: 768, configurable: true });
  });

  afterEach(() => {
    cleanup();
    setI18nLanguage("zh-Hans");
  });

  it.each([
    ["复制", "copy"],
    ["提取", "extract"],
    ["翻译", "translate"],
  ])("routes screenshot %s only to capture_region_rect", async (label, action) => {
    render(<ScreenshotOverlay />);
    await screen.findByRole("img");
    fireEvent.click(screen.getByRole("button", { name: "Finish crop" }));
    fireEvent.click(screen.getByRole("button", { name: label }));

    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("capture_region_rect", {
      screenX: 120,
      screenY: 240,
      width: 200,
      height: 100,
      action,
    }));
    const commands = mocks.invoke.mock.calls.map(([command]) => command);
    expect(commands).not.toContain("run_text_ai_action");
    expect(commands).not.toContain("copy_selection_text");
    expect(commands).not.toContain("search_selection_text");
    expect(commands).not.toContain("get_selection_snapshot");
    const captureIndex = commands.indexOf("capture_region_rect");
    expect(mocks.hide.mock.invocationCallOrder[0]).toBeLessThan(mocks.invoke.mock.invocationCallOrder[captureIndex]);
  });

  it("waits for both screenshot subscriptions before reading the initial snapshot", async () => {
    const ready = deferred<() => void>();
    const failed = deferred<() => void>();
    mocks.listen.mockImplementation((name: string) => name === "screenshot-ready" ? ready.promise : failed.promise);
    render(<ScreenshotOverlay />);
    expect(mocks.invoke).not.toHaveBeenCalled();

    await act(async () => ready.resolve(() => undefined));
    expect(mocks.invoke).not.toHaveBeenCalled();
    await act(async () => failed.resolve(() => undefined));

    expect(mocks.invoke).toHaveBeenCalledWith("get_screenshot_snapshot");
    expect(screen.getByRole("img").getAttribute("src")).toBe(screenshot.pngDataUrl);
  });

  it("keeps a screenshot event delivered during listener registration", async () => {
    const registration = deferred<() => void>();
    mocks.listen.mockImplementation((name: string, handler: (event: { payload: unknown }) => void) => {
      mocks.handlers.set(name, handler);
      return registration.promise;
    });
    render(<ScreenshotOverlay />);

    emit("screenshot-ready", nextScreenshot);
    await act(async () => registration.resolve(() => undefined));

    expect(mocks.invoke).not.toHaveBeenCalledWith("get_screenshot_snapshot");
    expect(screen.getByRole("img").getAttribute("src")).toBe(nextScreenshot.pngDataUrl);
  });

  it.each(["resolve", "reject"])("ignores an old screenshot snapshot that later %ss", async (outcome) => {
    const initial = deferred<MonitorScreenshotPayload>();
    mocks.invoke.mockReturnValue(initial.promise);
    render(<ScreenshotOverlay />);
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("get_screenshot_snapshot"));

    emit("screenshot-ready", nextScreenshot);
    await act(async () => outcome === "resolve"
      ? initial.resolve(screenshot)
      : initial.reject("i18n:screenshot.focusFailed"));

    expect(screen.getByRole("img").getAttribute("src")).toBe(nextScreenshot.pngDataUrl);
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("keeps the latest localized screenshot error over an older successful snapshot", async () => {
    const initial = deferred<MonitorScreenshotPayload>();
    mocks.invoke.mockImplementation((command: string) => command === "get_screenshot_snapshot"
      ? initial.promise
      : Promise.resolve({ message: "ok" }));
    render(<ScreenshotOverlay />);
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("get_screenshot_snapshot"));

    emit("screenshot-error", "i18n:screenshot.hotkeyHeld");
    await act(async () => initial.resolve(screenshot));

    expect(screen.getByRole("alert").textContent).toContain("请松开快捷键后重新截图");
    expect(screen.queryByRole("img")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "关闭" }));
    expect(mocks.hide).toHaveBeenCalled();
  });

  it("resets the crop and clears errors when the hidden overlay is reused", async () => {
    render(<ScreenshotOverlay />);
    await screen.findByRole("img");
    fireEvent.click(screen.getByRole("button", { name: "Finish crop" }));
    expect(screen.getByRole("button", { name: "提取" })).toBeTruthy();

    emit("screenshot-ready", nextScreenshot);
    expect(screen.queryByRole("button", { name: "提取" })).toBeNull();
    emit("screenshot-error", "i18n:screenshot.sourceUnfocused");
    expect(screen.getByRole("alert").textContent).toContain("原应用已失去焦点");
    emit("screenshot-ready", nextScreenshot);

    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByRole("img").getAttribute("src")).toBe(nextScreenshot.pngDataUrl);
    expect(screen.queryByRole("button", { name: "提取" })).toBeNull();
  });

  it("does not repopulate a closed overlay from an old snapshot", async () => {
    const initial = deferred<MonitorScreenshotPayload>();
    mocks.invoke.mockImplementation((command: string) => command === "get_screenshot_snapshot"
      ? initial.promise
      : Promise.resolve({ message: "ok" }));
    render(<ScreenshotOverlay />);
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("get_screenshot_snapshot"));

    fireEvent.keyDown(window, { key: "Escape" });
    await act(async () => initial.resolve(screenshot));

    expect(screen.queryByRole("img")).toBeNull();
    expect(mocks.hide).toHaveBeenCalled();
  });

  it("handles a capture rejection without hiding the next activation's screenshot", async () => {
    const capture = deferred<never>();
    mocks.invoke.mockImplementation((command: string) => command === "get_screenshot_snapshot"
      ? Promise.resolve(screenshot)
      : command === "capture_region_rect" ? capture.promise : Promise.resolve({ message: "ok" }));
    render(<ScreenshotOverlay />);
    await screen.findByRole("img");
    fireEvent.click(screen.getByRole("button", { name: "Finish crop" }));
    fireEvent.click(screen.getByRole("button", { name: "提取" }));
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("capture_region_rect", expect.anything()));

    emit("screenshot-ready", nextScreenshot);
    await act(async () => capture.reject("i18n:screenshot.focusFailed"));

    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByRole("img").getAttribute("src")).toBe(nextScreenshot.pngDataUrl);
    fireEvent.click(screen.getByRole("button", { name: "Finish crop" }));
    expect((screen.getByRole("button", { name: "提取" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("handles a capture error and localizes the error state without reopening the overlay", async () => {
    mocks.invoke.mockImplementation((command: string) => command === "get_screenshot_snapshot"
      ? Promise.resolve(screenshot)
      : command === "capture_region_rect"
        ? Promise.reject("i18n:screenshot.focusFailed")
        : Promise.resolve({ message: "ok" }));
    render(<ScreenshotOverlay />);
    await screen.findByRole("img");
    fireEvent.click(screen.getByRole("button", { name: "Finish crop" }));
    fireEvent.click(screen.getByRole("button", { name: "翻译" }));

    await screen.findByText("截图窗口聚焦失败，请重试");
    expect(mocks.hide).toHaveBeenCalledOnce();
    expect(screen.queryByRole("img")).toBeNull();
  });

  it("shows a local error if hiding the overlay fails before invoking capture", async () => {
    mocks.hide.mockRejectedValue("i18n:screenshot.focusFailed");
    render(<ScreenshotOverlay />);
    await screen.findByRole("img");
    fireEvent.click(screen.getByRole("button", { name: "Finish crop" }));
    fireEvent.click(screen.getByRole("button", { name: "复制" }));

    await screen.findByText("截图窗口聚焦失败，请重试");
    expect(mocks.invoke).not.toHaveBeenCalledWith("capture_region_rect", expect.anything());
  });

  it("cleans up screenshot subscriptions that finish after unmount", async () => {
    const registration = deferred<() => void>();
    mocks.listen.mockReturnValue(registration.promise);
    const { unmount } = render(<ScreenshotOverlay />);
    const unsubscribe = vi.fn();

    unmount();
    await act(async () => registration.resolve(unsubscribe));

    expect(unsubscribe).toHaveBeenCalledTimes(2);
    expect(mocks.invoke).not.toHaveBeenCalled();
  });
});
