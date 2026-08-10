// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import {
  applyDocumentTranslation,
  createTranslator,
  targetLanguageForUiLanguage,
  translateText,
} from "./i18n";

describe("i18n", () => {
  it("maps ui language to the translation target language", () => {
    expect(targetLanguageForUiLanguage("en")).toBe("en");
    expect(targetLanguageForUiLanguage("zh-Hant")).toBe("zh-Hant");
    expect(targetLanguageForUiLanguage("ja")).toBe("ja");
  });

  it("translates known UI text", () => {
    expect(translateText("复制", "en")).toBe("Copy");
    expect(translateText("翻译", "ja")).toBe("翻訳");
    expect(translateText("保存设置", "fr")).toBe("Enregistrer les paramètres");
  });

  it("translates key-based UI strings", () => {
    const t = createTranslator("en");
    expect(t("common.copy")).toBe("Copy");
    expect(t("result.loading")).toBe("Processing");
    expect(t("input.placeholder")).toBe("Enter text to translate");
  });

  it("applies translations to rendered DOM and attributes", () => {
    document.body.innerHTML = `
      <button>复制</button>
      <input placeholder="输入要翻译的文本" />
    `;

    applyDocumentTranslation("en");

    expect(document.body.querySelector("button")?.textContent).toBe("Copy");
    expect(
      document.body.querySelector("input")?.getAttribute("placeholder"),
    ).toBe("Enter text to translate");
  });
});
