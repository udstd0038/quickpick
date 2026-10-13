// @vitest-environment jsdom
import {
  act,
  cleanup,
  fireEvent,
  render,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
  setDecorations: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => mocks }));
vi.mock("../../lib/platform", () => ({
  runningOnMac: true,
  setPlatformDataset: vi.fn(),
}));

import { setI18nLanguage } from "../../lib/i18n";
import { defaultAppSettings } from "../../lib/settingsTypes";
import { useSettingsStore } from "../../stores/settingsStore";
import { SettingsWindow } from "./SettingsWindow";

const scopes = [
  { scope: "text", section: "ai-selection", label: "文本 Key 状态" },
  { scope: "vision", section: "ai-screenshot", label: "视觉 Key 状态" },
  { scope: "input", section: "ai-input", label: "输入 Key 状态" },
] as const;

type Scope = (typeof scopes)[number]["scope"];

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: string) => void;
  const promise = new Promise<T>((accept, decline) => {
    resolve = accept;
    reject = decline;
  });
  return { promise, resolve, reject };
}

function backend(
  scope: Scope,
  status: () => Promise<{ configured: boolean }>,
  clear: () => Promise<{ message: string }>,
) {
  mocks.invoke.mockImplementation((command: string, args?: { scope?: Scope }) => {
    if (command === "get_app_settings") {
      return Promise.resolve({ ...defaultAppSettings, uiLanguage: "zh-Hans" });
    }
    if (command === "get_api_key_status") {
      return args?.scope === scope
        ? status()
        : Promise.resolve({ configured: false });
    }
    if (command === "clear_api_key" && args?.scope === scope) {
      return clear();
    }
    return Promise.resolve({ message: "ok" });
  });
}

async function sectionControls(config: (typeof scopes)[number]) {
  render(<SettingsWindow />);
  await waitFor(() =>
    expect(document.getElementById(config.section)).toBeTruthy(),
  );
  const section = document.getElementById(config.section)!;
  const clear = within(section).getByRole("button", {
    name: "清除",
  }) as HTMLButtonElement;
  const statusRow = within(section)
    .getByText(config.label)
    .closest<HTMLDivElement>(".setting-row")!;
  return { section, clear, statusRow };
}

describe("API key status and clearing", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.listen.mockResolvedValue(() => undefined);
    mocks.setDecorations.mockResolvedValue(undefined);
    Object.defineProperty(window, "matchMedia", {
      configurable: true,
      value: vi.fn(() => ({
        matches: false,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      })),
    });
    setI18nLanguage("zh-Hans");
    useSettingsStore.setState({
      settings: { ...defaultAppSettings, uiLanguage: "zh-Hans" },
      status: { kind: "idle", message: "settings.loaded" },
    });
  });

  afterEach(cleanup);

  it.each(scopes)("allows deleting $scope after status access is denied", async (config) => {
    const reading = deferred<{ configured: boolean }>();
    const deleting = deferred<{ message: string }>();
    backend(config.scope, () => reading.promise, () => deleting.promise);
    const { section, clear, statusRow } = await sectionControls(config);
    expect(clear.disabled).toBe(true);

    await act(async () =>
      reading.reject("i18n:security.readFailed:keychain access denied or locked"),
    );
    await waitFor(() => {
      expect(within(statusRow).getByText("读取 API Key 失败")).toBeTruthy();
      expect(clear.disabled).toBe(false);
    });
    const input = section.querySelector('input[type="password"]')!;
    fireEvent.change(input, { target: { value: "replacement-test-key" } });
    expect(
      within(section).getByText("读取 API Key 失败 keychain access denied or locked"),
    ).toBeTruthy();

    fireEvent.click(clear);
    expect(mocks.invoke).toHaveBeenCalledWith("clear_api_key", {
      scope: config.scope,
    });
    expect(clear.disabled).toBe(true);
    await act(async () => deleting.resolve({ message: "测试 Key 已清除" }));
    await waitFor(() => {
      expect(within(statusRow).getByText("未配置")).toBeTruthy();
      expect(clear.disabled).toBe(true);
    });
  });

  it.each(scopes)("disables clearing $scope when the backend confirms no entry", async (config) => {
    backend(
      config.scope,
      () => Promise.resolve({ configured: false }),
      () => Promise.resolve({ message: "ok" }),
    );
    const { clear, statusRow } = await sectionControls(config);
    await waitFor(() =>
      expect(within(statusRow).getByText("未配置")).toBeTruthy(),
    );
    expect(clear.disabled).toBe(true);
    fireEvent.click(clear);
    expect(mocks.invoke).not.toHaveBeenCalledWith("clear_api_key", {
      scope: config.scope,
    });
  });

  for (const configured of [null, true] as const) {
    it.each(scopes)(`preserves ${configured === null ? "unknown" : "configured"} $scope status when deletion is denied`, async (config) => {
      backend(
        config.scope,
        () =>
          configured === null
            ? Promise.reject("i18n:security.readFailed:keychain access denied or locked")
            : Promise.resolve({ configured }),
        () =>
          Promise.reject("i18n:security.clearFailed:keychain access denied or locked"),
      );
      const { section, clear, statusRow } = await sectionControls(config);
      const label = configured === null ? "读取 API Key 失败" : "已加密保存";
      await waitFor(() => {
        expect(within(statusRow).getByText(label)).toBeTruthy();
        expect(clear.disabled).toBe(false);
      });
      fireEvent.click(clear);
      await waitFor(() => {
        expect(
          within(section).getByText("清除 API Key 失败 keychain access denied or locked"),
        ).toBeTruthy();
        expect(within(statusRow).getByText(label)).toBeTruthy();
        expect(clear.disabled).toBe(false);
      });
      expect(mocks.invoke).toHaveBeenCalledWith("clear_api_key", {
        scope: config.scope,
      });
    });
  }
});
