// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  hide: vi.fn(),
  minimize: vi.fn(),
  toggleMaximize: vi.fn(),
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => mocks,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
}));

import { defaultAppSettings } from "../../lib/settingsTypes";
import { SettingsPage } from "./SettingsPage";

describe("SettingsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.hide.mockResolvedValue(undefined);
    mocks.minimize.mockResolvedValue(undefined);
    mocks.toggleMaximize.mockResolvedValue(undefined);
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_app_settings") {
        return Promise.resolve(defaultAppSettings);
      }
      if (command === "get_api_key_status") {
        return Promise.resolve({ configured: false });
      }
      if (command === "set_hotkey_capture_mode") {
        return Promise.resolve({ message: "ok" });
      }
      if (command === "save_app_settings") {
        return Promise.resolve({ message: "设置已保存" });
      }
      return Promise.resolve({ message: "ok" });
    });
  });

  afterEach(() => {
    cleanup();
  });

  it("renders the ShadCN settings navigation and save action", async () => {
    render(<SettingsPage coreStatus="已连接" />);

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "划词模型" })).toBeTruthy(),
    );
    expect(screen.getByRole("button", { name: "截图模型" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "输入模型" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "保存设置" })).toBeTruthy();
  });

  it("navigates to the input model card", async () => {
    render(<SettingsPage coreStatus="已连接" />);

    fireEvent.click(await screen.findByRole("button", { name: "输入模型" }));
    expect(await screen.findByText("输入翻译行为")).toBeTruthy();
  });

  it("saves settings through the service layer", async () => {
    render(<SettingsPage coreStatus="已连接" />);

    fireEvent.click(await screen.findByRole("button", { name: "保存设置" }));
    await waitFor(() =>
      expect(mocks.invoke).toHaveBeenCalledWith(
        "save_app_settings",
        expect.objectContaining({ settings: expect.any(Object) }),
      ),
    );
  });
});
