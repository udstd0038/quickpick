// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  mac: true,
  invoke: vi.fn(),
  check: vi.fn(),
  request: vi.fn(),
  open: vi.fn(),
  onFocusChanged: vi.fn(),
  unlisten: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => mocks }));
vi.mock("../../lib/platform", () => ({ get runningOnMac() { return mocks.mac; } }));

import { setI18nLanguage } from "../../lib/i18n";
import type { MacPermissionStatus } from "../../services/macosPermissions";
import { MacPermissionsPanel } from "./MacPermissionsPanel";

const denied: MacPermissionStatus = {
  accessibility: false,
  screenRecording: false,
  inputMonitoring: false,
};
const granted: MacPermissionStatus = {
  accessibility: true,
  screenRecording: true,
  inputMonitoring: true,
};
let focusChanged: (event: { payload: boolean }) => void;

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((accept) => { resolve = accept; });
  return { promise, resolve };
}

function permissionRow(label: string) {
  return screen.getByText(label).closest<HTMLDivElement>(".setting-field")!;
}

async function renderDenied() {
  const view = render(<MacPermissionsPanel clipboardFallbackEnabled />);
  await waitFor(() => expect(screen.getAllByText("未授权")).toHaveLength(3));
  return view;
}

describe("macOS permission controls", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.mac = true;
    setI18nLanguage("zh-Hans");
    mocks.check.mockResolvedValue(denied);
    mocks.request.mockResolvedValue(granted);
    mocks.open.mockResolvedValue(undefined);
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_macos_permission_status") return mocks.check();
      if (command === "request_macos_permission") return mocks.request();
      if (command === "open_macos_permission_settings") return mocks.open();
      return Promise.reject("unexpected command");
    });
    mocks.onFocusChanged.mockImplementation((handler) => {
      focusChanged = handler;
      return Promise.resolve(mocks.unlisten);
    });
  });

  afterEach(cleanup);

  it("checks on mount without requesting permission", async () => {
    await renderDenied();
    expect(mocks.invoke).toHaveBeenCalledWith("get_macos_permission_status");
    expect(mocks.check).toHaveBeenCalledTimes(1);
    expect(mocks.request).not.toHaveBeenCalled();
    expect(mocks.open).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "请求授权 辅助功能" })).toBeTruthy();
  });

  it("shows unknown on query errors and allows a manual retry", async () => {
    mocks.check.mockRejectedValueOnce("backend details");
    render(<MacPermissionsPanel clipboardFallbackEnabled />);
    expect(await screen.findByRole("alert")).toHaveProperty("textContent", "权限状态读取失败，请重试。");
    expect(screen.getAllByText("状态未知")).toHaveLength(3);
    expect(screen.queryByText("已授权")).toBeNull();
    expect(screen.queryByText("未授权")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "重新检查" }));
    await waitFor(() => expect(screen.getAllByText("未授权")).toHaveLength(3));
    expect(screen.queryByRole("alert")).toBeNull();
    expect(mocks.request).not.toHaveBeenCalled();
  });

  it("rejects an incomplete query response instead of showing false permissions", async () => {
    mocks.check.mockResolvedValueOnce({ accessibility: true });
    render(<MacPermissionsPanel clipboardFallbackEnabled />);
    await screen.findByRole("alert");
    expect(screen.getAllByText("状态未知")).toHaveLength(3);
    expect(screen.queryByText("已授权")).toBeNull();
  });

  it("replaces previously granted status with unknown when a refresh fails", async () => {
    mocks.check.mockResolvedValueOnce(granted);
    render(<MacPermissionsPanel clipboardFallbackEnabled />);
    await waitFor(() => expect(screen.getAllByText("已授权")).toHaveLength(3));
    mocks.check.mockRejectedValueOnce("backend details");
    fireEvent.click(screen.getByRole("button", { name: "重新检查" }));
    await screen.findByRole("alert");
    expect(screen.getAllByText("状态未知")).toHaveLength(3);
    expect(screen.queryByText("已授权")).toBeNull();
    expect(screen.queryByText("未授权")).toBeNull();
  });

  it("uses a fresh check after request completion and keeps denied access denied", async () => {
    await renderDenied();
    fireEvent.click(screen.getByRole("button", { name: "请求授权 屏幕录制" }));
    await waitFor(() => expect(mocks.check).toHaveBeenCalledTimes(2));
    const row = permissionRow("屏幕录制");
    await waitFor(() => expect(within(row).getByText("未授权")).toBeTruthy());
    expect(within(row).queryByText("已授权")).toBeNull();
    expect(within(row).getByText("授权尚未生效，请在系统设置中确认；如系统要求，请重启 QuickPick。")).toBeTruthy();
    expect(mocks.invoke).toHaveBeenCalledWith("request_macos_permission", { kind: "screen_recording" });
  });

  it("reports failed requests and checks the actual status again", async () => {
    mocks.request.mockRejectedValueOnce("backend details");
    await renderDenied();
    fireEvent.click(screen.getByRole("button", { name: "请求授权 辅助功能" }));
    await waitFor(() => expect(mocks.check).toHaveBeenCalledTimes(2));
    expect(await screen.findByText("请求授权失败，请使用系统设置后重查。")).toBeTruthy();
    expect(screen.queryByText("backend details")).toBeNull();
  });

  it("refreshes when focus returns and removes a stale denied hint", async () => {
    await renderDenied();
    fireEvent.click(screen.getByRole("button", { name: "请求授权 屏幕录制" }));
    await screen.findByText("授权尚未生效，请在系统设置中确认；如系统要求，请重启 QuickPick。");
    mocks.check.mockResolvedValue(granted);
    await act(async () => focusChanged({ payload: false }));
    expect(mocks.check).toHaveBeenCalledTimes(2);
    await act(async () => focusChanged({ payload: true }));
    await waitFor(() => expect(screen.getAllByText("已授权")).toHaveLength(3));
    expect(screen.queryByText("授权尚未生效，请在系统设置中确认；如系统要求，请重启 QuickPick。")).toBeNull();
  });

  it("ignores an older query arriving after a newer focus check", async () => {
    const oldCheck = deferred<MacPermissionStatus>();
    mocks.check.mockReturnValueOnce(oldCheck.promise).mockResolvedValue(granted);
    render(<MacPermissionsPanel clipboardFallbackEnabled />);
    await act(async () => focusChanged({ payload: true }));
    await waitFor(() => expect(screen.getAllByText("已授权")).toHaveLength(3));
    await act(async () => oldCheck.resolve(denied));
    expect(screen.getAllByText("已授权")).toHaveLength(3);
    expect(screen.queryByText("未授权")).toBeNull();
  });

  it.each([
    ["辅助功能", "accessibility"],
    ["屏幕录制", "screen_recording"],
    ["输入监控", "input_monitoring"],
  ])("opens system settings for %s without requesting access", async (label, kind) => {
    await renderDenied();
    fireEvent.click(screen.getByRole("button", { name: `系统设置 ${label}` }));
    await waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("open_macos_permission_settings", { kind }));
    expect(mocks.request).not.toHaveBeenCalled();
  });

  it("offers a useful fallback when system settings cannot open", async () => {
    mocks.open.mockRejectedValueOnce("backend details");
    await renderDenied();
    fireEvent.click(screen.getByRole("button", { name: "系统设置 辅助功能" }));
    expect(await screen.findByText("无法打开系统设置，请手动打开隐私与安全性。")).toBeTruthy();
    expect(mocks.request).not.toHaveBeenCalled();
  });

  it("does not offer input monitoring when clipboard fallback is disabled", async () => {
    render(<MacPermissionsPanel clipboardFallbackEnabled={false} />);
    await waitFor(() => expect(screen.getAllByText("未授权")).toHaveLength(2));
    expect(screen.queryByText("输入监控")).toBeNull();
    expect(screen.getByRole("button", { name: "请求授权 屏幕录制" })).toBeTruthy();
  });

  it("does not show or query permissions on Windows", () => {
    mocks.mac = false;
    const { container } = render(<MacPermissionsPanel clipboardFallbackEnabled />);
    expect(container.childElementCount).toBe(0);
    expect(mocks.invoke).not.toHaveBeenCalled();
    expect(mocks.onFocusChanged).not.toHaveBeenCalled();
  });

  it("cleans up a focus listener that resolves after unmount", async () => {
    const subscription = deferred<() => void>();
    mocks.onFocusChanged.mockReturnValueOnce(subscription.promise);
    const { unmount } = await renderDenied();
    unmount();
    await act(async () => subscription.resolve(mocks.unlisten));
    expect(mocks.unlisten).toHaveBeenCalledTimes(1);
  });

  it("does not refresh after a request finishes on an unmounted panel", async () => {
    const requesting = deferred<MacPermissionStatus>();
    mocks.request.mockReturnValueOnce(requesting.promise);
    const { unmount } = await renderDenied();
    fireEvent.click(screen.getByRole("button", { name: "请求授权 辅助功能" }));
    unmount();
    await act(async () => requesting.resolve(granted));
    expect(mocks.check).toHaveBeenCalledTimes(1);
  });
});
