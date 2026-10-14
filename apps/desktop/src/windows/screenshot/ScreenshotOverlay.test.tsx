// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ButtonHTMLAttributes, ReactNode } from "react";
import type { PercentCrop, PixelCrop } from "react-image-crop";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { MonitorScreenshotPayload } from "../../services/invoke";

type CropCallbacks = {
  onComplete: (pixel: PixelCrop, percent: PercentCrop) => void;
  onDragStart: () => void;
};

const mocks = vi.hoisted(() => ({
  hide: vi.fn(),
  invoke: vi.fn(),
  listen: vi.fn(),
  cropCallbacks: [] as CropCallbacks[],
  copyCallbacks: [] as Array<() => void>,
  crop: { unit: "px", x: 10.25, y: 20.75, width: 100.5, height: 40.4 } as PixelCrop,
}));

vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => mocks }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("react-image-crop", () => ({
  default: (props: CropCallbacks & { children: ReactNode; disabled?: boolean }) => {
    mocks.cropCallbacks.push(props);
    return (
      <div>
        {props.children}
        <button disabled={props.disabled} onClick={() => props.onComplete(mocks.crop, { ...mocks.crop, unit: "%" })}>
          select region
        </button>
      </div>
    );
  },
}));
vi.mock("../../components/ActionButton", () => ({
  ActionButton: (props: ButtonHTMLAttributes<HTMLButtonElement>) => {
    if (props.children === "复制") mocks.copyCallbacks.push(() => props.onClick?.({} as never));
    return <button {...props} />;
  },
}));

import { ScreenshotOverlay } from "./ScreenshotOverlay";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (cause: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

function snapshot(sessionId = 1): MonitorScreenshotPayload {
  return {
    sessionId,
    monitorId: 23,
    monitorWidth: 1000,
    monitorHeight: 500,
    width: 2000,
    height: 1000,
    monitorX: -1600,
    monitorY: -200,
    monitorName: "Public test monitor",
    pngDataUrl: `data:image/png;base64,public-fixture-${sessionId}`,
  };
}

const listeners = new Map<string, (event: { payload: unknown }) => void>();
const unlisten = new Map<string, ReturnType<typeof vi.fn>>();

function emit(name: string, payload: unknown) {
  act(() => listeners.get(name)?.({ payload }));
}

async function ready() {
  await screen.findByRole("img", { name: "当前屏幕" });
}

function select() {
  fireEvent.click(screen.getByRole("button", { name: "select region" }));
}

function copy() {
  fireEvent.click(screen.getByRole("button", { name: "复制" }));
}

function expectImage(sessionId: number) {
  expect(screen.getByRole("img", { name: "当前屏幕" }).getAttribute("src")).toBe(snapshot(sessionId).pngDataUrl);
}

describe("ScreenshotOverlay sessions", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    listeners.clear();
    unlisten.clear();
    mocks.cropCallbacks.length = 0;
    mocks.copyCallbacks.length = 0;
    mocks.crop = { unit: "px", x: 10.25, y: 20.75, width: 100.5, height: 40.4 };
    Object.defineProperty(window, "innerWidth", { value: 1000, writable: true });
    Object.defineProperty(window, "innerHeight", { value: 500, writable: true });
    mocks.hide.mockResolvedValue(undefined);
    mocks.listen.mockImplementation((name: string, callback: (event: { payload: unknown }) => void) => {
      listeners.set(name, callback);
      const dispose = vi.fn();
      unlisten.set(name, dispose);
      return Promise.resolve(dispose);
    });
    mocks.invoke.mockImplementation((command: string) => Promise.resolve(
      command === "get_screenshot_snapshot" ? snapshot() : { message: "ok" },
    ));
  });

  afterEach(cleanup);

  it.each(["copy", "extract", "translate"] as const)("sends %s in Retina snapshot pixels without monitor origin or pre-hiding", async (action) => {
    const operation = deferred<{ message: string }>();
    mocks.invoke.mockImplementation((command: string) => command === "get_screenshot_snapshot"
      ? Promise.resolve(snapshot()) : operation.promise);
    render(<ScreenshotOverlay />);
    await ready();
    select();
    fireEvent.click(screen.getByRole("button", { name: { copy: "复制", extract: "提取", translate: "翻译" }[action] }));
    expect(mocks.invoke).toHaveBeenCalledWith("capture_region_rect", {
      sessionId: 1, x: 20, y: 41, width: 202, height: 82, action,
    });
    expect(mocks.hide).not.toHaveBeenCalled();
    expect(mocks.invoke).not.toHaveBeenCalledWith("clear_popup_state", expect.anything());
    expectImage(1);
    emit("screenshot-closed", { sessionId: 1 });
    expect(screen.queryByRole("img")).toBeNull();
    await act(async () => operation.resolve({ message: "ok" }));
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("uses the displayed image bounds instead of assuming the monitor scale", async () => {
    render(<ScreenshotOverlay />);
    await ready();
    const image = screen.getByRole("img");
    vi.spyOn(image, "getBoundingClientRect").mockReturnValue({ width: 800, height: 400 } as DOMRect);
    select();
    copy();
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("capture_region_rect", {
      sessionId: 1, x: 25, y: 51, width: 252, height: 102, action: "copy",
    }));
  });

  it("clamps edge crops and rejects nonfinite or undersized regions", async () => {
    render(<ScreenshotOverlay />);
    await ready();
    for (const crop of [
      { x: Number.NaN, y: 0, width: 100, height: 100 },
      { x: 20, y: 20, width: 3, height: 3 },
      { x: 1000, y: 20, width: 100, height: 100 },
    ]) {
      mocks.crop = { unit: "px", ...crop };
      select();
      expect(screen.queryByRole("button", { name: "复制" })).toBeNull();
    }
    mocks.crop = { unit: "px", x: 990.2, y: 490.2, width: 20, height: 20 };
    select();
    copy();
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("capture_region_rect", {
      sessionId: 1, x: 1980, y: 980, width: 20, height: 20, action: "copy",
    }));
  });

  it("blocks duplicate capture callbacks even before a busy rerender", async () => {
    const operation = deferred<{ message: string }>();
    mocks.invoke.mockImplementation((command: string) => command === "get_screenshot_snapshot"
      ? Promise.resolve(snapshot()) : operation.promise);
    render(<ScreenshotOverlay />);
    await ready();
    select();
    const callback = mocks.copyCallbacks.at(-1)!;
    act(() => { callback(); callback(); });
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "capture_region_rect")).toHaveLength(1);
    await act(async () => operation.resolve({ message: "ok" }));
  });

  it("cleans up a failed capture and allows a fresh session", async () => {
    mocks.invoke.mockImplementation((command: string) => command === "get_screenshot_snapshot"
      ? Promise.resolve(snapshot()) : Promise.reject("public capture failure"));
    render(<ScreenshotOverlay />);
    await ready();
    select();
    copy();
    expect((await screen.findByRole("alert")).textContent).toBe("public capture failure");
    expect(screen.queryByRole("img")).toBeNull();
    mocks.invoke.mockResolvedValue({ message: "ok" });
    emit("screenshot-ready", snapshot(2));
    expectImage(2);
    select();
    copy();
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("capture_region_rect", expect.objectContaining({ sessionId: 2 })));
  });

  it("cancels only the known session and waits for backend closure", async () => {
    const operation = deferred<{ message: string }>();
    mocks.invoke.mockImplementation((command: string) => command === "get_screenshot_snapshot"
      ? Promise.resolve(snapshot()) : operation.promise);
    const { container } = render(<ScreenshotOverlay />);
    await ready();
    fireEvent.contextMenu(container.firstChild as HTMLElement);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "cancel_screenshot")).toEqual([
      ["cancel_screenshot", { sessionId: 1 }],
    ]);
    expect(mocks.hide).not.toHaveBeenCalled();
    expectImage(1);
    emit("screenshot-closed", { sessionId: 1 });
    expect(screen.queryByRole("img")).toBeNull();
    await act(async () => operation.resolve({ message: "ok" }));
  });

  it("reports failed cancellation and releases the old frame", async () => {
    mocks.invoke.mockImplementation((command: string) => command === "get_screenshot_snapshot"
      ? Promise.resolve(snapshot()) : Promise.reject("public cancellation failure"));
    render(<ScreenshotOverlay />);
    await ready();
    fireEvent.keyDown(window, { key: "Escape" });
    expect((await screen.findByRole("alert")).textContent).toBe("public cancellation failure");
    expect(screen.queryByRole("img")).toBeNull();
    emit("screenshot-ready", snapshot(2));
    expectImage(2);
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("queries an unknown session and leaves the window alone when no snapshot exists", async () => {
    mocks.invoke.mockResolvedValue(null);
    render(<ScreenshotOverlay />);
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("get_screenshot_snapshot"));
    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledTimes(2));
    expect(mocks.hide).not.toHaveBeenCalled();
    expect(mocks.invoke.mock.calls.map(([command]) => command)).toEqual([
      "get_screenshot_snapshot", "get_screenshot_snapshot",
    ]);
    emit("screenshot-ready", snapshot(2));
    expectImage(2);
  });

  it("cancels an unknown session only after finding its matching snapshot ID", async () => {
    mocks.invoke.mockResolvedValueOnce(null).mockResolvedValueOnce(snapshot()).mockResolvedValue({ message: "ok" });
    render(<ScreenshotOverlay />);
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledTimes(1));
    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("cancel_screenshot", { sessionId: 1 }));
    expect(mocks.hide).not.toHaveBeenCalled();
  });

  it("reports an unknown-session query failure and allows a fresh ready event", async () => {
    mocks.invoke.mockResolvedValueOnce(null).mockRejectedValueOnce("public cancellation query failure");
    render(<ScreenshotOverlay />);
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledTimes(1));
    fireEvent.keyDown(window, { key: "Escape" });
    expect((await screen.findByRole("alert")).textContent).toBe("public cancellation query failure");
    emit("screenshot-ready", snapshot(2));
    expectImage(2);
    select();
    expect((screen.getByRole("button", { name: "复制" }) as HTMLButtonElement).disabled).toBe(false);
    expect(mocks.hide).not.toHaveBeenCalled();
  });

  it.each(["old", "fresh", "reject"] as const)("does not cancel a new ready frame when an unknown-session query returns %s", async (result) => {
    const query = deferred<MonitorScreenshotPayload | null>();
    mocks.invoke.mockResolvedValueOnce(null).mockReturnValueOnce(query.promise);
    render(<ScreenshotOverlay />);
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledTimes(1));
    fireEvent.keyDown(window, { key: "Escape" });
    expect(mocks.invoke).toHaveBeenCalledTimes(2);
    emit("screenshot-ready", snapshot(2));
    await act(async () => result === "reject"
      ? query.reject("public stale cancellation query failure")
      : query.resolve(snapshot(result === "old" ? 1 : 2)));
    expectImage(2);
    expect(screen.queryByRole("alert")).toBeNull();
    expect(mocks.invoke).not.toHaveBeenCalledWith("cancel_screenshot", expect.anything());
    expect(mocks.hide).not.toHaveBeenCalled();
  });

  it.each(["resolve", "reject"] as const)("ignores an initial getter that %ss after a ready event", async (result) => {
    const initial = deferred<MonitorScreenshotPayload | null>();
    mocks.invoke.mockReturnValue(initial.promise);
    render(<ScreenshotOverlay />);
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("get_screenshot_snapshot"));
    emit("screenshot-ready", snapshot(2));
    await act(async () => result === "resolve" ? initial.resolve(snapshot()) : initial.reject("public old getter failure"));
    expectImage(2);
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it.each([
    ["capture", "resolve"], ["capture", "reject"],
    ["cancel", "resolve"], ["cancel", "reject"],
  ] as const)("ignores a pending %s that %ss after a newer ready event", async (kind, result) => {
    const operation = deferred<{ message: string }>();
    mocks.invoke.mockImplementation((command: string) => command === "get_screenshot_snapshot"
      ? Promise.resolve(snapshot()) : operation.promise);
    render(<ScreenshotOverlay />);
    await ready();
    if (kind === "capture") { select(); copy(); }
    else fireEvent.keyDown(window, { key: "Escape" });
    emit("screenshot-ready", snapshot(2));
    select();
    await act(async () => result === "resolve"
      ? operation.resolve({ message: "ok" })
      : operation.reject("public stale operation failure"));
    expectImage(2);
    expect(screen.queryByRole("alert")).toBeNull();
    expect((screen.getByRole("button", { name: "复制" }) as HTMLButtonElement).disabled).toBe(false);
    mocks.invoke.mockResolvedValue({ message: "ok" });
    copy();
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("capture_region_rect", expect.objectContaining({ sessionId: 2 })));
    expect(mocks.hide).not.toHaveBeenCalled();
  });

  it("ignores old crop and toolbar callbacks after a newer ready event", async () => {
    render(<ScreenshotOverlay />);
    await ready();
    select();
    const oldCopy = mocks.copyCallbacks.at(-1)!;
    const oldCrop = mocks.cropCallbacks.at(-1)!;
    emit("screenshot-ready", snapshot(2));
    act(() => {
      oldCrop.onComplete({ unit: "px", x: 100, y: 100, width: 100, height: 100 }, { unit: "%", x: 10, y: 20, width: 10, height: 20 });
      oldCopy();
    });
    expectImage(2);
    expect(screen.queryByRole("button", { name: "复制" })).toBeNull();
    expect(mocks.invoke.mock.calls.map(([command]) => command)).toEqual(["get_screenshot_snapshot"]);
    select();
    act(() => oldCrop.onDragStart());
    expect(screen.getByRole("button", { name: "复制" })).toBeTruthy();
  });

  it("cancel can supersede capture without the old capture failure overriding cancellation", async () => {
    const capture = deferred<{ message: string }>();
    const cancel = deferred<{ message: string }>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_screenshot_snapshot") return Promise.resolve(snapshot());
      return command === "capture_region_rect" ? capture.promise : cancel.promise;
    });
    render(<ScreenshotOverlay />);
    await ready();
    select();
    copy();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(mocks.invoke).toHaveBeenCalledWith("cancel_screenshot", { sessionId: 1 });
    emit("screenshot-closed", { sessionId: 1 });
    await act(async () => capture.reject("public superseded capture failure"));
    expect(screen.queryByRole("alert")).toBeNull();
    await act(async () => cancel.reject("public cancellation failure"));
    expect(screen.getByRole("alert").textContent).toBe("public cancellation failure");
  });

  it("an event received during listener registration takes priority over the initial getter", async () => {
    const registrations = [deferred<() => void>(), deferred<() => void>(), deferred<() => void>()];
    mocks.listen.mockImplementation((name: string, callback: (event: { payload: unknown }) => void) => {
      listeners.set(name, callback);
      return registrations[["screenshot-ready", "screenshot-error", "screenshot-closed"].indexOf(name)].promise;
    });
    render(<ScreenshotOverlay />);
    emit("screenshot-ready", snapshot(2));
    await act(async () => registrations.forEach((registration) => registration.resolve(vi.fn())));
    expectImage(2);
    expect(mocks.invoke).not.toHaveBeenCalled();
  });

  it("ignores old errors and closures without replacing the active image", async () => {
    render(<ScreenshotOverlay />);
    await ready();
    emit("screenshot-ready", snapshot(2));
    select();
    emit("screenshot-error", { sessionId: 1, message: "public stale error" });
    emit("screenshot-closed", { sessionId: 1 });
    expectImage(2);
    expect(screen.getByRole("button", { name: "复制" })).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("retires a closed session and prevents duplicate ready events resurrecting its PNG", async () => {
    render(<ScreenshotOverlay />);
    await ready();
    emit("screenshot-closed", { sessionId: 1 });
    emit("screenshot-ready", snapshot());
    expect(screen.queryByRole("img")).toBeNull();
    emit("screenshot-ready", snapshot(2));
    expectImage(2);
  });

  it("a fresh session error replaces the old image and is cleared by the next ready event", async () => {
    render(<ScreenshotOverlay />);
    await ready();
    emit("screenshot-error", { sessionId: 2, message: "public new capture failure" });
    expect(screen.queryByRole("img")).toBeNull();
    expect(screen.getByRole("alert").textContent).toBe("public new capture failure");
    emit("screenshot-ready", snapshot(2));
    expect(screen.queryByRole("img")).toBeNull();
    emit("screenshot-ready", snapshot(3));
    expectImage(3);
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("clears a failed image decode but still cancels the known session", async () => {
    render(<ScreenshotOverlay />);
    await ready();
    fireEvent.error(screen.getByRole("img"));
    expect(screen.getByRole("alert")).toBeTruthy();
    expect(screen.queryByRole("img")).toBeNull();
    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("cancel_screenshot", { sessionId: 1 }));
    expect(mocks.hide).not.toHaveBeenCalled();
  });

  it("disposes listeners that finish registering after unmount", async () => {
    const registrations = [deferred<() => void>(), deferred<() => void>(), deferred<() => void>()];
    const disposers = registrations.map(() => vi.fn());
    mocks.listen.mockImplementation((name: string, callback: (event: { payload: unknown }) => void) => {
      listeners.set(name, callback);
      return registrations[["screenshot-ready", "screenshot-error", "screenshot-closed"].indexOf(name)].promise;
    });
    const { unmount } = render(<ScreenshotOverlay />);
    unmount();
    await act(async () => registrations.forEach((registration, index) => registration.resolve(disposers[index])));
    disposers.forEach((dispose) => expect(dispose).toHaveBeenCalledOnce());
    emit("screenshot-ready", snapshot(2));
    expect(mocks.invoke).not.toHaveBeenCalled();
  });

  it("disposes installed listeners once and handles a subscription failure", async () => {
    mocks.listen.mockImplementation((name: string) => {
      if (name === "screenshot-error") return Promise.reject("public listener failure");
      const dispose = vi.fn();
      unlisten.set(name, dispose);
      return Promise.resolve(dispose);
    });
    const { unmount } = render(<ScreenshotOverlay />);
    expect((await screen.findByRole("alert")).textContent).toBe("public listener failure");
    unmount();
    unlisten.forEach((dispose) => expect(dispose).toHaveBeenCalledOnce());
    expect(mocks.invoke).not.toHaveBeenCalled();
  });
});
