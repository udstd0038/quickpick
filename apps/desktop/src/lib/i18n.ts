import { useSyncExternalStore } from "react";

export type UiLanguage =
  | "system"
  | "zh-Hans"
  | "zh-Hant"
  | "en"
  | "ko"
  | "ja"
  | "fr"
  | "de"
  | "es";

export type TranslationLanguage = Exclude<UiLanguage, "system">;

export const uiLanguageOptions: Array<{ id: UiLanguage; label: string }> = [
  { id: "system", label: "settings.languageSystem" },
  { id: "zh-Hans", label: "language.zhHans" },
  { id: "zh-Hant", label: "language.zhHant" },
  { id: "en", label: "language.en" },
  { id: "ko", label: "language.ko" },
  { id: "ja", label: "language.ja" },
  { id: "fr", label: "language.fr" },
  { id: "de", label: "language.de" },
  { id: "es", label: "language.es" },
];

export const translationLanguageOptions: Array<{
  id: TranslationLanguage;
  label: string;
}> = [
  { id: "zh-Hans", label: "language.zhHans" },
  { id: "zh-Hant", label: "language.zhHant" },
  { id: "en", label: "language.en" },
  { id: "ko", label: "language.ko" },
  { id: "ja", label: "language.ja" },
  { id: "fr", label: "language.fr" },
  { id: "de", label: "language.de" },
  { id: "es", label: "language.es" },
];

export function systemLanguage(): TranslationLanguage {
  const language = (navigator.language || "").toLowerCase();
  if (language.startsWith("zh")) {
    return /zh-(tw|hk|mo)/i.test(navigator.language)
      ? "zh-Hant"
      : "zh-Hans";
  }
  if (language.startsWith("en")) {
    return "en";
  }
  if (language.startsWith("ko")) {
    return "ko";
  }
  if (language.startsWith("ja")) {
    return "ja";
  }
  if (language.startsWith("fr")) {
    return "fr";
  }
  if (language.startsWith("de")) {
    return "de";
  }
  if (language.startsWith("es")) {
    return "es";
  }
  return "zh-Hans";
}

export function resolveUiLanguage(value: UiLanguage): TranslationLanguage {
  return value === "system" ? systemLanguage() : value;
}

export function targetLanguageForUiLanguage(value: UiLanguage): TranslationLanguage {
  return resolveUiLanguage(value);
}

type LocaleStrings = Record<TranslationLanguage, string>;

const entry = (
  zhHant: string,
  en: string,
  ko: string,
  ja: string,
  fr: string,
  de: string,
  es: string,
): LocaleStrings => ({
  "zh-Hans": "",
  "zh-Hant": zhHant,
  en,
  ko,
  ja,
  fr,
  de,
  es,
});

const translations: Record<string, LocaleStrings> = {
  系统语言: entry("系統語言", "System Language", "시스템 언어", "システム言語", "Langue du système", "Systemsprache", "Idioma del sistema"),
  简体中文: entry("簡體中文", "Simplified Chinese", "중국어 간체", "簡体中文", "Chinois simplifié", "Vereinfachtes Chinesisch", "Chino simplificado"),
  繁体中文: entry("繁體中文", "Traditional Chinese", "중국어 번체", "繁体中文", "Chinois traditionnel", "Traditionelles Chinesisch", "Chino tradicional"),
  英文: entry("英文", "English", "영어", "英語", "Anglais", "Englisch", "Inglés"),
  英语: entry("英語", "English", "영어", "英語", "Anglais", "Englisch", "Inglés"),
  韩文: entry("韓文", "Korean", "한국어", "韓国語", "Coréen", "Koreanisch", "Coreano"),
  韩语: entry("韓語", "Korean", "한국어", "韓国語", "Coréen", "Koreanisch", "Coreano"),
  日文: entry("日文", "Japanese", "일본어", "日本語", "Japonais", "Japanisch", "Japonés"),
  日语: entry("日語", "Japanese", "일본어", "日本語", "Japonais", "Japanisch", "Japonés"),
  法文: entry("法文", "French", "프랑스어", "フランス語", "Français", "Französisch", "Francés"),
  法语: entry("法語", "French", "프랑스어", "フランス語", "Français", "Französisch", "Francés"),
  德文: entry("德文", "German", "독일어", "ドイツ語", "Allemand", "Deutsch", "Alemán"),
  德语: entry("德語", "German", "독일어", "ドイツ語", "Allemand", "Deutsch", "Alemán"),
  西班牙文: entry("西班牙文", "Spanish", "스페인어", "スペイン語", "Espagnol", "Spanisch", "Español"),
  西班牙语: entry("西班牙語", "Spanish", "스페인어", "スペイン語", "Espagnol", "Spanisch", "Español"),
  自动检测: entry("自動偵測", "Auto Detect", "자동 감지", "自動検出", "Détection automatique", "Automatisch erkennen", "Detección automática"),
  语言: entry("語言", "Language", "언어", "言語", "Langue", "Sprache", "Idioma"),
  通用: entry("通用", "General", "일반", "一般", "Général", "Allgemein", "General"),
  快捷键: entry("快捷鍵", "Hotkeys", "단축키", "ショートカット", "Raccourcis", "Tastenkürzel", "Atajos"),
  划词模型: entry("劃詞模型", "Selection Model", "선택 모델", "選択モデル", "Modèle de sélection", "Auswahlmodell", "Modelo de selección"),
  截图模型: entry("截圖模型", "Screenshot Model", "스크린샷 모델", "スクリーンショットモデル", "Modèle de capture", "Screenshot-Modell", "Modelo de captura"),
  输入模型: entry("輸入模型", "Input Model", "입력 모델", "入力モデル", "Modèle de saisie", "Eingabemodell", "Modelo de entrada"),
  外观: entry("外觀", "Appearance", "모양", "外観", "Apparence", "Darstellung", "Apariencia"),
  隐私: entry("隱私", "Privacy", "개인정보", "プライバシー", "Confidentialité", "Datenschutz", "Privacidad"),
  开机自启: entry("開機自啟", "Start on Boot", "부팅 시 시작", "起動時に開始", "Démarrage automatique", "Beim Start ausführen", "Iniciar con el sistema"),
  托盘常驻: entry("托盤常駐", "Tray Resident", "트레이 상주", "トレイ常駐", "Résident dans la barre", "Im Tray bleiben", "Residente en bandeja"),
  "AI 请求超时": entry("AI 請求逾時", "AI Request Timeout", "AI 요청 제한 시간", "AIリクエストのタイムアウト", "Délai de requête IA", "KI-Anfrage-Timeout", "Tiempo de espera de IA"),
  已启用: entry("已啟用", "Enabled", "사용 중", "有効", "Activé", "Aktiviert", "Activado"),
  已关闭: entry("已關閉", "Disabled", "사용 안 함", "無効", "Désactivé", "Deaktiviert", "Desactivado"),
  划词菜单: entry("劃詞選單", "Selection Menu", "선택 메뉴", "選択メニュー", "Menu de sélection", "Auswahlmenü", "Menú de selección"),
  区域截图: entry("區域截圖", "Region Screenshot", "영역 캡처", "領域スクリーンショット", "Capture de région", "Bereichs-Screenshot", "Captura de región"),
  输入翻译: entry("輸入翻譯", "Input Translation", "입력 번역", "入力翻訳", "Traduction de saisie", "Eingabeübersetzung", "Traducción de entrada"),
  设置: entry("設定", "Settings", "설정", "設定", "Paramètres", "Einstellungen", "Configuración"),
  设置分类: entry("設定分類", "Settings Categories", "설정 분류", "設定カテゴリ", "Catégories de paramètres", "Einstellungskategorien", "Categorías de configuración"),
  保存设置: entry("儲存設定", "Save Settings", "설정 저장", "設定を保存", "Enregistrer les paramètres", "Einstellungen speichern", "Guardar configuración"),
  设置已保存: entry("設定已儲存", "Settings Saved", "설정이 저장되었습니다", "設定を保存しました", "Paramètres enregistrés", "Einstellungen gespeichert", "Configuración guardada"),
  设置已载入: entry("設定已載入", "Settings Loaded", "설정을 불러왔습니다", "設定を読み込みました", "Paramètres chargés", "Einstellungen geladen", "Configuración cargada"),
  "读取设置失败，已使用默认值": entry("讀取設定失敗，已使用預設值", "Failed to load settings, defaults used", "설정을 불러오지 못해 기본값을 사용합니다", "設定の読み込みに失敗、既定値を使用", "Échec du chargement, valeurs par défaut", "Einstellungen konnten nicht geladen werden", "No se pudieron cargar los ajustes"),
  有未保存的修改: entry("有未儲存的修改", "Unsaved Changes", "저장되지 않은 변경 사항", "未保存の変更", "Modifications non enregistrées", "Ungespeicherte Änderungen", "Cambios sin guardar"),
  正在保存: entry("正在儲存", "Saving", "저장 중", "保存中", "Enregistrement", "Wird gespeichert", "Guardando"),
  保存设置失败: entry("儲存設定失敗", "Failed to Save Settings", "설정 저장 실패", "設定の保存に失敗", "Échec de l'enregistrement", "Speichern fehlgeschlagen", "No se pudo guardar"),
  主题: entry("主題", "Theme", "테마", "テーマ", "Thème", "Design", "Tema"),
  跟随系统: entry("跟隨系統", "Follow System", "시스템 따르기", "システムに従う", "Suivre le système", "System folgen", "Seguir sistema"),
  浅色: entry("淺色", "Light", "라이트", "ライト", "Clair", "Hell", "Claro"),
  深色: entry("深色", "Dark", "다크", "ダーク", "Sombre", "Dunkel", "Oscuro"),
  窗口效果: entry("視窗效果", "Window Effect", "창 효과", "ウィンドウ効果", "Effet de fenêtre", "Fenstereffekt", "Efecto de ventana"),
  不透明度: entry("不透明度", "Opacity", "불투명도", "不透明度", "Opacité", "Deckkraft", "Opacidad"),
  文本供应商: entry("文字供應商", "Text Provider", "텍스트 공급자", "テキストプロバイダー", "Fournisseur de texte", "Textanbieter", "Proveedor de texto"),
  "文本 Base URL": entry("文字 Base URL", "Text Base URL", "텍스트 Base URL", "テキストBase URL", "URL de base du texte", "Text-Basis-URL", "URL base de texto"),
  文本模型: entry("文字模型", "Text Model", "텍스트 모델", "テキストモデル", "Modèle de texte", "Textmodell", "Modelo de texto"),
  "文本 API Key": entry("文字 API Key", "Text API Key", "텍스트 API 키", "テキストAPIキー", "Clé API de texte", "Text-API-Schlüssel", "Clave API de texto"),
  "文本 Key 状态": entry("文字 Key 狀態", "Text Key Status", "텍스트 키 상태", "テキストキー状態", "État de la clé de texte", "Text-Key-Status", "Estado de clave de texto"),
  已加密保存: entry("已加密儲存", "Encrypted", "암호화 저장됨", "暗号化して保存", "Chiffré", "Verschlüsselt", "Cifrado"),
  未配置: entry("未配置", "Not Configured", "설정되지 않음", "未設定", "Non configuré", "Nicht konfiguriert", "No configurado"),
  "保存文本 Key": entry("儲存文字 Key", "Save Text Key", "텍스트 키 저장", "テキストキーを保存", "Enregistrer la clé de texte", "Text-Key speichern", "Guardar clave de texto"),
  视觉供应商: entry("視覺供應商", "Vision Provider", "비전 공급자", "ビジョンプロバイダー", "Fournisseur visuel", "Bildanbieter", "Proveedor visual"),
  "视觉 Base URL": entry("視覺 Base URL", "Vision Base URL", "비전 Base URL", "ビジョンBase URL", "URL de base visuelle", "Bild-Basis-URL", "URL base visual"),
  视觉模型: entry("視覺模型", "Vision Model", "비전 모델", "ビジョンモデル", "Modèle visuel", "Bildmodell", "Modelo visual"),
  "视觉 API Key": entry("視覺 API Key", "Vision API Key", "비전 API 키", "ビジョンAPIキー", "Clé API visuelle", "Bild-API-Schlüssel", "Clave API visual"),
  "视觉 Key 状态": entry("視覺 Key 狀態", "Vision Key Status", "비전 키 상태", "ビジョンキー状態", "État de la clé visuelle", "Bild-Key-Status", "Estado de clave visual"),
  "保存视觉 Key": entry("儲存視覺 Key", "Save Vision Key", "비전 키 저장", "ビジョンキーを保存", "Enregistrer la clé visuelle", "Bild-Key speichern", "Guardar clave visual"),
  输入供应商: entry("輸入供應商", "Input Provider", "입력 공급자", "入力プロバイダー", "Fournisseur de saisie", "Eingabeanbieter", "Proveedor de entrada"),
  "输入 Base URL": entry("輸入 Base URL", "Input Base URL", "입력 Base URL", "入力Base URL", "URL de base de saisie", "Eingabe-Basis-URL", "URL base de entrada"),
  "输入 API Key": entry("輸入 API Key", "Input API Key", "입력 API 키", "入力APIキー", "Clé API de saisie", "Eingabe-API-Schlüssel", "Clave API de entrada"),
  "输入 Key 状态": entry("輸入 Key 狀態", "Input Key Status", "입력 키 상태", "入力キー状態", "État de la clé de saisie", "Eingabe-Key-Status", "Estado de clave de entrada"),
  "保存输入 Key": entry("儲存輸入 Key", "Save Input Key", "입력 키 저장", "入力キーを保存", "Enregistrer la clé de saisie", "Eingabe-Key speichern", "Guardar clave de entrada"),
  清除: entry("清除", "Clear", "지우기", "クリア", "Effacer", "Löschen", "Borrar"),
  复制: entry("複製", "Copy", "복사", "コピー", "Copier", "Kopieren", "Copiar"),
  翻译: entry("翻譯", "Translate", "번역", "翻訳", "Traduire", "Übersetzen", "Traducir"),
  总结: entry("總結", "Summarize", "요약", "要約", "Résumer", "Zusammenfassen", "Resumir"),
  搜索: entry("搜尋", "Search", "검색", "検索", "Rechercher", "Suchen", "Buscar"),
  提取: entry("提取", "Extract", "추출", "抽出", "Extraire", "Extrahieren", "Extraer"),
  固定: entry("固定", "Pin", "고정", "固定", "Épingler", "Anpinnen", "Fijar"),
  取消固定: entry("取消固定", "Unpin", "고정 해제", "固定解除", "Détacher", "Lösen", "Desfijar"),
  关闭: entry("關閉", "Close", "닫기", "閉じる", "Fermer", "Schließen", "Cerrar"),
  源语言: entry("來源語言", "Source Language", "원본 언어", "ソース言語", "Langue source", "Quellsprache", "Idioma de origen"),
  目标语言: entry("目標語言", "Target Language", "대상 언어", "ターゲット言語", "Langue cible", "Zielsprache", "Idioma de destino"),
  切换翻译方向: entry("切換翻譯方向", "Switch Translation Direction", "번역 방향 전환", "翻訳方向を切り替え", "Inverser la direction", "Übersetzungsrichtung wechseln", "Cambiar dirección"),
  输入要翻译的文本: entry("輸入要翻譯的文字", "Enter text to translate", "번역할 텍스트 입력", "翻訳するテキストを入力", "Saisissez le texte à traduire", "Zu übersetzenden Text eingeben", "Introduce texto para traducir"),
  翻译中: entry("翻譯中", "Translating", "번역 중", "翻訳中", "Traduction en cours", "Übersetze", "Traduciendo"),
  "正在处理...": entry("正在處理...", "Processing...", "처리 중...", "処理中...", "Traitement...", "Verarbeitung...", "Procesando..."),
  翻译请求失败: entry("翻譯請求失敗", "Translation Request Failed", "번역 요청 실패", "翻訳リクエストに失敗", "Échec de la demande de traduction", "Übersetzungsanfrage fehlgeschlagen", "Error en la solicitud de traducción"),
  切换语言失败: entry("切換語言失敗", "Language Switch Failed", "언어 전환 실패", "言語切替に失敗", "Échec du changement de langue", "Sprachwechsel fehlgeschlagen", "Error al cambiar idioma"),
  暂无结果: entry("暫無結果", "No Results", "결과 없음", "結果はありません", "Aucun résultat", "Keine Ergebnisse", "Sin resultados"),
  待处理: entry("待處理", "Pending", "대기 중", "待機中", "En attente", "Ausstehend", "Pendiente"),
  正在处理: entry("正在處理", "Processing", "처리 중", "処理中", "Traitement", "Verarbeitung", "Procesando"),
  翻译结果: entry("翻譯結果", "Translation Result", "번역 결과", "翻訳結果", "Résultat de traduction", "Übersetzungsergebnis", "Resultado de traducción"),
  处理失败: entry("處理失敗", "Failed", "처리 실패", "処理に失敗", "Échec", "Fehlgeschlagen", "Error"),
  复制结果: entry("複製結果", "Copy Result", "결과 복사", "結果をコピー", "Copier le résultat", "Ergebnis kopieren", "Copiar resultado"),
  重试: entry("重試", "Retry", "다시 시도", "再試行", "Réessayer", "Erneut versuchen", "Reintentar"),
  "正在读取屏幕截图...": entry("正在讀取螢幕截圖...", "Reading screenshot...", "스크린샷 읽는 중...", "スクリーンショットを読み込み中...", "Lecture de la capture...", "Screenshot wird geladen...", "Leyendo captura..."),
  当前屏幕: entry("目前螢幕", "Current Screen", "현재 화면", "現在の画面", "Écran actuel", "Aktueller Bildschirm", "Pantalla actual"),
  等待截图区域: entry("等待截圖區域", "Waiting for Screenshot Region", "캡처 영역 대기 중", "キャプチャ領域を待機中", "En attente de la région", "Warte auf Screenshot-Bereich", "Esperando región de captura"),
  正在检查核心连接: entry("正在檢查核心連線", "Checking Core Connection", "코어 연결 확인 중", "コア接続を確認中", "Vérification de la connexion", "Prüfe Kernverbindung", "Comprobando conexión"),
  前端预览模式: entry("前端預覽模式", "Frontend Preview Mode", "프런트엔드 미리보기 모드", "フロントエンドプレビューモード", "Mode aperçu frontend", "Frontend-Vorschau", "Modo de vista previa"),
  "QuickPick 设置": entry("QuickPick 設定", "QuickPick Settings", "QuickPick 설정", "QuickPick 設定", "Paramètres QuickPick", "QuickPick-Einstellungen", "Configuración de QuickPick"),
  "QuickPick 结果": entry("QuickPick 結果", "QuickPick Result", "QuickPick 결과", "QuickPick 結果", "Résultat QuickPick", "QuickPick-Ergebnis", "Resultado de QuickPick"),
  "QuickPick 划词": entry("QuickPick 劃詞", "QuickPick Selection", "QuickPick 선택", "QuickPick 選択", "Sélection QuickPick", "QuickPick-Auswahl", "Selección de QuickPick"),
  最小化: entry("最小化", "Minimize", "최소화", "最小化", "Réduire", "Minimieren", "Minimizar"),
  最大化或还原: entry("最大化或還原", "Maximize or Restore", "최대화 또는 복원", "最大化または元に戻す", "Agrandir ou restaurer", "Maximieren oder Wiederherstellen", "Maximizar o restaurar"),
  关闭设置: entry("關閉設定", "Close Settings", "설정 닫기", "設定を閉じる", "Fermer les paramètres", "Einstellungen schließen", "Cerrar configuración"),
  内容历史: entry("內容歷史", "Content History", "콘텐츠 기록", "コンテンツ履歴", "Historique du contenu", "Inhaltsverlauf", "Historial de contenido"),
  "默认不保存文本、截图和 AI 结果": entry("預設不儲存文字、截圖和 AI 結果", "Text, screenshots, and AI results are not saved by default", "기본적으로 텍스트, 스크린샷, AI 결과를 저장하지 않습니다", "既定ではテキスト、スクリーンショット、AI結果を保存しません", "Le texte, les captures et les résultats IA ne sont pas enregistrés", "Text, Screenshots und KI-Ergebnisse werden standardmäßig nicht gespeichert", "El texto, las capturas y los resultados de IA no se guardan de forma predeterminada"),
  "API Key": entry("API Key", "API Key", "API 키", "APIキー", "Clé API", "API-Schlüssel", "Clave API"),
  "使用 Windows DPAPI 本机加密保存": entry("使用 Windows DPAPI 本機加密儲存", "Stored encrypted locally with Windows DPAPI", "Windows DPAPI로 로컬 암호화 저장", "Windows DPAPIでローカル暗号化保存", "Chiffré localement avec Windows DPAPI", "Lokal mit Windows DPAPI verschlüsselt", "Cifrado localmente con Windows DPAPI"),
  供应商: entry("供應商", "Provider", "공급자", "プロバイダー", "Fournisseur", "Anbieter", "Proveedor"),
  "AI 配置": entry("AI 配置", "AI Configuration", "AI 설정", "AI構成", "Configuration IA", "KI-Konfiguration", "Configuración de IA"),
  "AI 配置已保存": entry("AI 配置已儲存", "AI Configuration Saved", "AI 설정 저장됨", "AI構成を保存しました", "Configuration IA enregistrée", "KI-Konfiguration gespeichert", "Configuración de IA guardada"),
  "AI 配置已载入": entry("AI 配置已載入", "AI Configuration Loaded", "AI 설정 불러옴", "AI構成を読み込みました", "Configuration IA chargée", "KI-Konfiguration geladen", "Configuración de IA cargada"),
  "API Key 不能为空": entry("API Key 不能為空", "API Key cannot be empty", "API 키는 비어 있을 수 없습니다", "APIキーは空にできません", "La clé API ne peut pas être vide", "API-Schlüssel darf nicht leer sein", "La clave API no puede estar vacía"),
  "Key 已保存": entry("Key 已儲存", "Key Saved", "키 저장됨", "キーを保存しました", "Clé enregistrée", "Schlüssel gespeichert", "Clave guardada"),
  "Key 未配置": entry("Key 未配置", "Key Not Configured", "키 미설정", "キー未設定", "Clé non configurée", "Schlüssel nicht konfiguriert", "Clave no configurada"),
  "正在检查文本模型 Key": entry("正在檢查文字模型 Key", "Checking Text Model Key", "텍스트 모델 키 확인 중", "テキストモデルキーを確認中", "Vérification de la clé de texte", "Prüfe Textmodell-Schlüssel", "Comprobando clave de texto"),
  "正在检查视觉模型 Key": entry("正在檢查視覺模型 Key", "Checking Vision Model Key", "비전 모델 키 확인 중", "ビジョンモデルキーを確認中", "Vérification de la clé visuelle", "Prüfe Bildmodell-Schlüssel", "Comprobando clave visual"),
  "正在检查输入模型 Key": entry("正在檢查輸入模型 Key", "Checking Input Model Key", "입력 모델 키 확인 중", "入力モデルキーを確認中", "Vérification de la clé de saisie", "Prüfe Eingabemodell-Schlüssel", "Comprobando clave de entrada"),
  "文本模型 Key 已加密保存": entry("文字模型 Key 已加密儲存", "Text Model Key Encrypted", "텍스트 모델 키 암호화 저장", "テキストモデルキーを暗号化保存", "Clé de texte chiffrée", "Textmodell-Schlüssel verschlüsselt", "Clave de texto cifrada"),
  "文本模型 Key 未配置": entry("文字模型 Key 未配置", "Text Model Key Not Configured", "텍스트 모델 키 미설정", "テキストモデルキー未設定", "Clé de texte non configurée", "Textmodell-Schlüssel nicht konfiguriert", "Clave de texto no configurada"),
  "视觉模型 Key 已加密保存": entry("視覺模型 Key 已加密儲存", "Vision Model Key Encrypted", "비전 모델 키 암호화 저장", "ビジョンモデルキーを暗号化保存", "Clé visuelle chiffrée", "Bildmodell-Schlüssel verschlüsselt", "Clave visual cifrada"),
  "视觉模型 Key 未配置": entry("視覺模型 Key 未配置", "Vision Model Key Not Configured", "비전 모델 키 미설정", "ビジョンモデルキー未設定", "Clé visuelle non configurée", "Bildmodell-Schlüssel nicht konfiguriert", "Clave visual no configurada"),
  "输入模型 Key 已加密保存": entry("輸入模型 Key 已加密儲存", "Input Model Key Encrypted", "입력 모델 키 암호화 저장", "入力モデルキーを暗号化保存", "Clé de saisie chiffrée", "Eingabemodell-Schlüssel verschlüsselt", "Clave de entrada cifrada"),
  "输入模型 Key 未配置": entry("輸入模型 Key 未配置", "Input Model Key Not Configured", "입력 모델 키 미설정", "入力モデルキー未設定", "Clé de saisie non configurée", "Eingabemodell-Schlüssel nicht konfiguriert", "Clave de entrada no configurada"),
  选择外观主题: entry("選擇外觀主題", "Select Appearance Theme", "테마 선택", "テーマを選択", "Choisir le thème", "Design auswählen", "Seleccionar tema"),
  选择窗口效果: entry("選擇視窗效果", "Select Window Effect", "창 효과 선택", "ウィンドウ効果を選択", "Choisir l'effet", "Fenstereffekt auswählen", "Seleccionar efecto"),
  "选择 AI 供应商": entry("選擇 AI 供應商", "Select AI Provider", "AI 공급자 선택", "AIプロバイダーを選択", "Choisir le fournisseur IA", "KI-Anbieter auswählen", "Seleccionar proveedor de IA"),
  选择文本模型供应商: entry("選擇文字模型供應商", "Select Text Model Provider", "텍스트 모델 공급자 선택", "テキストモデルプロバイダーを選択", "Choisir le fournisseur de texte", "Textanbieter auswählen", "Seleccionar proveedor de texto"),
  选择视觉模型供应商: entry("選擇視覺模型供應商", "Select Vision Model Provider", "비전 모델 공급자 선택", "ビジョンモデルプロバイダーを選択", "Choisir le fournisseur visuel", "Bildanbieter auswählen", "Seleccionar proveedor visual"),
  选择输入模型供应商: entry("選擇輸入模型供應商", "Select Input Model Provider", "입력 모델 공급자 선택", "入力モデルプロバイダーを選択", "Choisir le fournisseur de saisie", "Eingabeanbieter auswählen", "Seleccionar proveedor de entrada"),
  按下快捷键组合: entry("按下快捷鍵組合", "Press a Hotkey", "단축키를 누르세요", "ショートカットキーを押してください", "Appuyez sur un raccourci", "Tastenkürzel drücken", "Pulsa un atajo"),
  正在读取选中文本: entry("正在讀取選中文字", "Reading Selected Text", "선택한 텍스트 읽는 중", "選択テキストを読み込み中", "Lecture du texte sélectionné", "Lese ausgewählten Text", "Leyendo texto seleccionado"),
  未选中: entry("未選中", "Not Selected", "선택 안 됨", "未選択", "Non sélectionné", "Nicht ausgewählt", "No seleccionado"),
  不支持: entry("不支援", "Not Supported", "지원 안 됨", "非対応", "Non pris en charge", "Nicht unterstützt", "No compatible"),
  失败: entry("失敗", "Failed", "실패", "失敗", "Échec", "Fehlgeschlagen", "Error"),
  已读取: entry("已讀取", "Read", "읽음", "読み取り済み", "Lu", "Gelesen", "Leído"),
  不能使用同一个快捷键: entry("不能使用同一個快捷鍵", "cannot use the same hotkey", "같은 단축키를 사용할 수 없습니다", "同じショートカットは使用できません", "ne peuvent pas utiliser le même raccourci", "können nicht dasselbe Tastenkürzel verwenden", "no pueden usar el mismo atajo"),
  划词和截图: entry("劃詞和截圖", "Selection and screenshot", "선택 및 스크린샷", "選択とスクリーンショット", "Sélection et capture", "Auswahl und Screenshot", "Selección y captura"),
  截图和输入翻译: entry("截圖和輸入翻譯", "Screenshot and input translation", "스크린샷 및 입력 번역", "スクリーンショットと入力翻訳", "Capture et traduction de saisie", "Screenshot und Eingabeübersetzung", "Captura y traducción de entrada"),
  输入翻译和划词: entry("輸入翻譯和劃詞", "Input translation and selection", "입력 번역 및 선택", "入力翻訳と選択", "Traduction de saisie et sélection", "Eingabeübersetzung und Auswahl", "Traducción de entrada y selección"),
  重置默认设置: entry("重設預設設定", "Reset Default Settings", "기본 설정 초기화", "既定設定にリセット", "Réinitialiser les paramètres", "Standardwerte zurücksetzen", "Restablecer configuración"),
  正在重置默认设置: entry("正在重設預設設定", "Resetting Default Settings", "기본 설정을 초기화하는 중", "既定設定をリセット中", "Réinitialisation des paramètres", "Setze Standardwerte zurück", "Restableciendo configuración"),
  重置默认设置失败: entry("重設預設設定失敗", "Failed to Reset Default Settings", "기본 설정 초기화 실패", "既定設定のリセットに失敗", "Échec de la réinitialisation", "Zurücksetzen fehlgeschlagen", "No se pudo restablecer"),
  "文本模型 Key 已加密保存，输入新 Key 可替换": entry("文字模型 Key 已加密儲存，輸入新 Key 可取代", "Text model key encrypted; entering a new key will replace it", "텍스트 모델 키가 암호화되어 저장됨, 새 키를 입력하면 교체됩니다", "テキストモデルキーは暗号化済み、新しいキーで置き換えられます", "Clé de texte chiffrée, une nouvelle clé la remplacera", "Textmodell-Schlüssel verschlüsselt, neue Eingabe ersetzt ihn", "Clave de texto cifrada, una nueva clave la reemplazará"),
  "输入文本模型 API Key": entry("輸入文字模型 API Key", "Enter text model API Key", "텍스트 모델 API 키 입력", "テキストモデルAPIキーを入力", "Saisir la clé API du modèle de texte", "Textmodell-API-Schlüssel eingeben", "Introduce la clave API del modelo de texto"),
  "视觉模型 Key 已加密保存，输入新 Key 可替换": entry("視覺模型 Key 已加密儲存，輸入新 Key 可取代", "Vision model key encrypted; entering a new key will replace it", "비전 모델 키가 암호화되어 저장됨, 새 키를 입력하면 교체됩니다", "ビジョンモデルキーは暗号化済み、新しいキーで置き換えられます", "Clé visuelle chiffrée, une nouvelle clé la remplacera", "Bildmodell-Schlüssel verschlüsselt, neue Eingabe ersetzt ihn", "Clave visual cifrada, una nueva clave la reemplazará"),
  "输入视觉模型 API Key": entry("輸入視覺模型 API Key", "Enter vision model API Key", "비전 모델 API 키 입력", "ビジョンモデルAPIキーを入力", "Saisir la clé API du modèle visuel", "Bildmodell-API-Schlüssel eingeben", "Introduce la clave API del modelo visual"),
  "输入模型 Key 已加密保存，输入新 Key 可替换": entry("輸入模型 Key 已加密儲存，輸入新 Key 可取代", "Input model key encrypted; entering a new key will replace it", "입력 모델 키가 암호화되어 저장됨, 새 키를 입력하면 교체됩니다", "入力モデルキーは暗号化済み、新しいキーで置き換えられます", "Clé de saisie chiffrée, une nouvelle clé la remplacera", "Eingabemodell-Schlüssel verschlüsselt, neue Eingabe ersetzt ihn", "Clave de entrada cifrada, una nueva clave la reemplazará"),
  "输入输入模型 API Key": entry("輸入輸入模型 API Key", "Enter input model API Key", "입력 모델 API 키 입력", "入力モデルAPIキーを入力", "Saisir la clé API du modèle de saisie", "Eingabemodell-API-Schlüssel eingeben", "Introduce la clave API del modelo de entrada"),
  "检查文本模型 Key 状态失败": entry("檢查文字模型 Key 狀態失敗", "Failed to check text model key status", "텍스트 모델 키 상태 확인 실패", "テキストモデルキー状態の確認に失敗", "Échec de la vérification de la clé de texte", "Textmodell-Schlüsselstatus konnte nicht geprüft werden", "No se pudo comprobar la clave de texto"),
  "检查视觉模型 Key 状态失败": entry("檢查視覺模型 Key 狀態失敗", "Failed to check vision model key status", "비전 모델 키 상태 확인 실패", "ビジョンモデルキー状態の確認に失敗", "Échec de la vérification de la clé visuelle", "Bildmodell-Schlüsselstatus konnte nicht geprüft werden", "No se pudo comprobar la clave visual"),
  "检查输入模型 Key 状态失败": entry("檢查輸入模型 Key 狀態失敗", "Failed to check input model key status", "입력 모델 키 상태 확인 실패", "入力モデルキー状態の確認に失敗", "Échec de la vérification de la clé de saisie", "Eingabemodell-Schlüsselstatus konnte nicht geprüft werden", "No se pudo comprobar la clave de entrada"),
  "文本模型供应商默认值已填入，请保存设置": entry("文字模型供應商預設值已填入，請儲存設定", "Text model provider defaults filled, please save", "텍스트 모델 공급자 기본값이 입력됨, 저장하세요", "テキストモデルプロバイダーの既定値を入力しました。保存してください", "Valeurs du fournisseur de texte remplies, enregistrez", "Textanbieter-Standardwerte ausgefüllt, bitte speichern", "Valores del proveedor de texto rellenados, guarde"),
  "视觉模型供应商默认值已填入，请保存设置": entry("視覺模型供應商預設值已填入，請儲存設定", "Vision model provider defaults filled, please save", "비전 모델 공급자 기본값이 입력됨, 저장하세요", "ビジョンモデルプロバイダーの既定値を入力しました。保存してください", "Valeurs du fournisseur visuel remplies, enregistrez", "Bildanbieter-Standardwerte ausgefüllt, bitte speichern", "Valores del proveedor visual rellenados, guarde"),
  "输入模型供应商默认值已填入，请保存设置": entry("輸入模型供應商預設值已填入，請儲存設定", "Input model provider defaults filled, please save", "입력 모델 공급자 기본값이 입력됨, 저장하세요", "入力モデルプロバイダーの既定値を入力しました。保存してください", "Valeurs du fournisseur de saisie remplies, enregistrez", "Eingabeanbieter-Standardwerte ausgefüllt, bitte speichern", "Valores del proveedor de entrada rellenados, guarde"),
  "输入新 Key 后保存会替换文本模型 Key": entry("輸入新 Key 後儲存會取代文字模型 Key", "Saving a new key will replace the text model key", "새 키를 저장하면 텍스트 모델 키가 교체됩니다", "新しいキーを保存するとテキストモデルキーが置き換わります", "Enregistrer une nouvelle clé remplacera la clé de texte", "Speichern eines neuen Schlüssels ersetzt den Textmodell-Schlüssel", "Guardar una nueva clave reemplazará la clave de texto"),
  "输入新 Key 后保存会替换视觉模型 Key": entry("輸入新 Key 後儲存會取代視覺模型 Key", "Saving a new key will replace the vision model key", "새 키를 저장하면 비전 모델 키가 교체됩니다", "新しいキーを保存するとビジョンモデルキーが置き換わります", "Enregistrer une nouvelle clé remplacera la clé visuelle", "Speichern eines neuen Schlüssels ersetzt den Bildmodell-Schlüssel", "Guardar una nueva clave reemplazará la clave visual"),
  "输入新 Key 后保存会替换输入模型 Key": entry("輸入新 Key 後儲存會取代輸入模型 Key", "Saving a new key will replace the input model key", "새 키를 저장하면 입력 모델 키가 교체됩니다", "新しいキーを保存すると入力モデルキーが置き換わります", "Enregistrer une nouvelle clé remplacera la clé de saisie", "Speichern eines neuen Schlüssels ersetzt den Eingabemodell-Schlüssel", "Guardar una nueva clave reemplazará la clave de entrada"),
  "请直接按下新的快捷键组合，Esc 取消": entry("請直接按下新的快捷鍵組合，Esc 取消", "Press a new hotkey, Esc to cancel", "새 단축키를 누르세요. Esc로 취소", "新しいショートカットキーを押してください。Escでキャンセル", "Appuyez sur un nouveau raccourci, Échap pour annuler", "Neue Tastenkombination drücken, Esc zum Abbrechen", "Pulsa un nuevo atajo, Esc para cancelar"),
  "已取消快捷键录制": entry("已取消快捷鍵錄製", "Hotkey capture canceled", "단축키 녹화 취소됨", "ショートカットの記録をキャンセル", "Enregistrement du raccourci annulé", "Tastenkürzel-Aufnahme abgebrochen", "Captura de atajo cancelada"),
  "快捷键必须包含 Alt，请重新按下组合键": entry("快捷鍵必須包含 Alt，請重新按下組合鍵", "Hotkey must include Alt, please press again", "단축키는 Alt를 포함해야 합니다. 다시 눌러주세요", "ショートカットにはAltが必要です。押し直してください", "Le raccourci doit inclure Alt, réessayez", "Tastenkürzel muss Alt enthalten, bitte erneut drücken", "El atajo debe incluir Alt, vuelve a pulsarlo"),
  "这个按键暂不支持，请换一个组合键": entry("這個按鍵暫不支援，請換一個組合鍵", "This key is not supported, try another", "이 키는 지원되지 않습니다. 다른 조합을 사용하세요", "このキーは対応していません。別の組み合わせを試してください", "Cette touche n'est pas prise en charge", "Diese Taste wird nicht unterstützt", "Esta tecla no es compatible"),
  "快捷键已记录，请保存设置": entry("快捷鍵已記錄，請儲存設定", "Hotkey recorded, please save", "단축키가 기록됨, 저장하세요", "ショートカットを記録しました。保存してください", "Raccourci enregistré, enregistrez", "Tastenkürzel aufgenommen, bitte speichern", "Atajo registrado, guarde"),
  "正在同步 AI 设置": entry("正在同步 AI 設定", "Syncing AI settings", "AI 설정 동기화 중", "AI設定を同期中", "Synchronisation des paramètres IA", "KI-Einstellungen werden synchronisiert", "Sincronizando configuración de IA"),
  "正在加密保存": entry("正在加密儲存", "Encrypting", "암호화 저장 중", "暗号化して保存中", "Chiffrement en cours", "Wird verschlüsselt", "Cifrando"),
  "同步 AI 设置失败": entry("同步 AI 設定失敗", "Failed to sync AI settings", "AI 설정 동기화 실패", "AI設定の同期に失敗", "Échec de la synchronisation IA", "KI-Synchronisierung fehlgeschlagen", "No se pudo sincronizar la configuración de IA"),
  "正在清除": entry("正在清除", "Clearing", "지우는 중", "クリア中", "Effacement", "Löschen", "Borrando"),
  "文本模型 Key": entry("文字模型 Key", "Text Model Key", "텍스트 모델 키", "テキストモデルキー", "Clé de texte", "Textmodell-Schlüssel", "Clave de texto"),
  "视觉模型 Key": entry("視覺模型 Key", "Vision Model Key", "비전 모델 키", "ビジョンモデルキー", "Clé visuelle", "Bildmodell-Schlüssel", "Clave visual"),
  "输入模型 Key": entry("輸入模型 Key", "Input Model Key", "입력 모델 키", "入力モデルキー", "Clé de saisie", "Eingabemodell-Schlüssel", "Clave de entrada"),
  "不能为空": entry("不能為空", "cannot be empty", "비어 있을 수 없습니다", "空にできません", "ne peut pas être vide", "darf nicht leer sein", "no puede estar vacía"),
  "保存失败": entry("儲存失敗", "Failed to save", "저장 실패", "保存に失敗", "Échec de l'enregistrement", "Speichern fehlgeschlagen", "No se pudo guardar"),
  "清除失败": entry("清除失敗", "Failed to clear", "지우기 실패", "クリアに失敗", "Échec de l'effacement", "Löschen fehlgeschlagen", "No se pudo borrar"),
  "同步设置失败": entry("同步設定失敗", "Failed to sync settings", "설정 동기화 실패", "設定の同期に失敗", "Échec de la synchronisation", "Synchronisierung fehlgeschlagen", "No se pudo sincronizar"),
  "保存前同步设置失败": entry("儲存前同步設定失敗", "Failed to sync before saving", "저장 전 동기화 실패", "保存前の同期に失敗", "Échec de la synchronisation avant enregistrement", "Synchronisierung vor dem Speichern fehlgeschlagen", "No se pudo sincronizar antes de guardar"),
  "通用 OpenAI 兼容": entry("通用 OpenAI 相容", "Generic OpenAI Compatible", "일반 OpenAI 호환", "汎用OpenAI互換", "OpenAI compatible générique", "Generisch OpenAI-kompatibel", "Compatible OpenAI genérico"),
  "小米 MiMo": entry("小米 MiMo", "Xiaomi MiMo", "샤오미 MiMo", "Xiaomi MiMo", "Xiaomi MiMo", "Xiaomi MiMo", "Xiaomi MiMo"),
  "读取设置文件失败": entry("讀取設定檔案失敗", "Failed to read settings file", "설정 파일을 읽지 못했습니다", "設定ファイルを読み込めませんでした", "Impossible de lire le fichier de paramètres", "Einstellungsdatei konnte nicht gelesen werden", "No se pudo leer el archivo de configuración"),
  "设置文件格式无效": entry("設定檔案格式無效", "Invalid settings file format", "설정 파일 형식이 잘못되었습니다", "設定ファイルの形式が無効です", "Format de fichier de paramètres invalide", "Ungültiges Format der Einstellungsdatei", "Formato de archivo de configuración no válido"),
  "创建设置目录失败": entry("建立設定目錄失敗", "Failed to create settings directory", "설정 디렉터리를 만들지 못했습니다", "設定ディレクトリの作成に失敗", "Échec de création du dossier de paramètres", "Einstellungsverzeichnis konnte nicht erstellt werden", "No se pudo crear el directorio de configuración"),
  "序列化设置失败": entry("序列化設定失敗", "Failed to serialize settings", "설정 직렬화 실패", "設定のシリアル化に失敗", "Échec de sérialisation des paramètres", "Einstellungen konnten nicht serialisiert werden", "No se pudieron serializar los ajustes"),
  "保存设置文件失败": entry("儲存設定檔案失敗", "Failed to write settings file", "설정 파일 저장 실패", "設定ファイルの保存に失敗", "Échec d'écriture du fichier de paramètres", "Einstellungsdatei konnte nicht geschrieben werden", "No se pudo escribir el archivo de configuración"),
  "定位应用设置目录失败": entry("定位應用設定目錄失敗", "Failed to locate app settings directory", "앱 설정 디렉터리를 찾지 못했습니다", "アプリ設定ディレクトリを特定できませんでした", "Échec de localisation du dossier de paramètres", "App-Einstellungsverzeichnis konnte nicht gefunden werden", "No se pudo ubicar el directorio de configuración"),
  "快捷键不能为空": entry("快捷鍵不能為空", "Hotkey cannot be empty", "단축키는 비어 있을 수 없습니다", "ショートカットは空にできません", "Le raccourci ne peut pas être vide", "Tastenkürzel darf nicht leer sein", "El atajo no puede estar vacío"),
  "快捷键首版需包含 Alt，例如 Alt+2": entry("快捷鍵首版需包含 Alt，例如 Alt+2", "Hotkey must include Alt, e.g. Alt+2", "단축키는 Alt를 포함해야 합니다. 예: Alt+2", "ショートカットにはAltを含める必要があります（例：Alt+2）", "Le raccourci doit inclure Alt, ex. Alt+2", "Tastenkürzel muss Alt enthalten, z. B. Alt+2", "El atajo debe incluir Alt, p. ej. Alt+2"),
  "快捷键格式无效，请使用 Alt+2 或 Ctrl+Alt+Q 这类格式": entry("快捷鍵格式無效，請使用 Alt+2 或 Ctrl+Alt+Q 這類格式", "Invalid hotkey format, use Alt+2 or Ctrl+Alt+Q", "단축키 형식이 잘못되었습니다. Alt+2 또는 Ctrl+Alt+Q 형식을 사용하세요", "ショートカット形式が無効です。Alt+2やCtrl+Alt+Q形式を使用してください", "Format de raccourci invalide, utilisez Alt+2 ou Ctrl+Alt+Q", "Ungültiges Tastenkürzel, verwenden Sie Alt+2 oder Ctrl+Alt+Q", "Formato de atajo no válido, usa Alt+2 o Ctrl+Alt+Q"),
  "设置快捷键不能和划词菜单或区域截图使用同一个快捷键": entry("設定快捷鍵不能和劃詞選單或區域截圖使用同一個快捷鍵", "Settings hotkey cannot share a hotkey with selection or screenshot", "설정 단축키는 선택 또는 스크린샷과 같은 단축키를 사용할 수 없습니다", "設定ショートカットは選択またはスクリーンショットと同一にできません", "Le raccourci des paramètres ne peut pas être partagé", "Einstellungen-Tastenkürzel darf nicht mit Auswahl oder Screenshot geteilt werden", "El atajo de configuración no puede compartirse con selección o captura"),
  "划词菜单和区域截图不能使用同一个快捷键": entry("劃詞選單和區域截圖不能使用同一個快捷鍵", "Selection and screenshot cannot share a hotkey", "선택 및 스크린샷은 같은 단축키를 사용할 수 없습니다", "選択とスクリーンショットは同じショートカットにできません", "La sélection et la capture ne peuvent pas partager un raccourci", "Auswahl und Screenshot können kein Tastenkürzel teilen", "La selección y la captura no pueden compartir un atajo"),
  "输入翻译不能和其他功能使用同一个快捷键": entry("輸入翻譯不能和其他功能使用同一個快捷鍵", "Input translation cannot share a hotkey with other actions", "입력 번역은 다른 기능과 같은 단축키를 사용할 수 없습니다", "入力翻訳は他の機能と同一のショートカットにできません", "La traduction de saisie ne peut pas partager un raccourci", "Eingabeübersetzung kann kein Tastenkürzel mit anderen Aktionen teilen", "La traducción de entrada no puede compartir un atajo con otras acciones"),
  "主题模式无效": entry("主題模式無效", "Invalid theme mode", "테마 모드가 잘못되었습니다", "テーマモードが無効です", "Mode de thème invalide", "Ungültiger Designmodus", "Modo de tema no válido"),
  "界面语言无效": entry("介面語言無效", "Invalid interface language", "인터페이스 언어가 잘못되었습니다", "インターフェース言語が無効です", "Langue d'interface invalide", "Ungültige Oberflächensprache", "Idioma de interfaz no válido"),
  "窗口效果无效": entry("視窗效果無效", "Invalid window effect", "창 효과가 잘못되었습니다", "ウィンドウ効果が無効です", "Effet de fenêtre invalide", "Ungültiger Fenstereffekt", "Efecto de ventana no válido"),
  "不透明度需在 30% 到 100% 之间": entry("不透明度需在 30% 到 100% 之間", "Opacity must be between 30% and 100%", "불투명도는 30%에서 100% 사이여야 합니다", "不透明度は30%から100%の間である必要があります", "L'opacité doit être entre 30% et 100%", "Deckkraft muss zwischen 30% und 100% liegen", "La opacidad debe estar entre 30% y 100%"),
  "请求超时需在 5 到 120 秒之间": entry("請求逾時需在 5 到 120 秒之間", "Request timeout must be between 5 and 120 seconds", "요청 제한 시간은 5초에서 120초 사이여야 합니다", "リクエストタイムアウトは5秒から120秒の間である必要があります", "Le délai doit être entre 5 et 120 secondes", "Timeout muss zwischen 5 und 120 Sekunden liegen", "El tiempo de espera debe estar entre 5 y 120 segundos"),
  "模型名称过长，请缩短后保存": entry("模型名稱過長，請縮短後儲存", "Model name is too long, shorten it and save", "모델 이름이 너무 깁니다. 줄인 후 저장하세요", "モデル名が長すぎます。短くして保存してください", "Nom du modèle trop long, raccourcissez-le", "Modellname zu lang, kürzen Sie ihn", "El nombre del modelo es demasiado largo, acórtalo"),
  "Base URL 必须以 http:// 或 https:// 开头": entry("Base URL 必須以 http:// 或 https:// 開頭", "Base URL must start with http:// or https://", "Base URL은 http:// 또는 https://로 시작해야 합니다", "Base URLはhttp://またはhttps://で始まる必要があります", "L'URL de base doit commencer par http:// ou https://", "Basis-URL muss mit http:// oder https:// beginnen", "La URL base debe comenzar con http:// o https://"),
  "选中文本过长，请缩短到 12000 字以内后再试": entry("選中文字過長，請縮短到 12000 字以內後再試", "Selected text is too long, shorten it to 12000 characters", "선택한 텍스트가 너무 깁니다. 12000자 이내로 줄이세요", "選択テキストが長すぎます。12000文字以内にしてください", "Texte sélectionné trop long, réduisez-le à 12000 caractères", "Ausgewählter Text zu lang, kürzen Sie ihn auf 12000 Zeichen", "El texto seleccionado es demasiado largo, redúcelo a 12000 caracteres"),
  "初始化 AI 请求客户端失败": entry("初始化 AI 請求用戶端失敗", "Failed to initialize AI client", "AI 클라이언트 초기화 실패", "AIクライアントの初期化に失敗", "Échec d'initialisation du client IA", "KI-Client konnte nicht initialisiert werden", "No se pudo inicializar el cliente de IA"),
  "AI 返回为空，请稍后重试或检查模型配置": entry("AI 回傳為空，請稍後重試或檢查模型配置", "AI returned empty, retry or check model config", "AI 응답이 비어 있습니다. 다시 시도하거나 모델 설정을 확인하세요", "AIが空を返しました。再試行するかモデル設定を確認してください", "L'IA a renvoyé un résultat vide, réessayez", "KI-Antwort ist leer, versuchen Sie es erneut", "La IA devolvió vacío, reintenta o revisa la configuración"),
  "输入文本过长，请缩短到 12000 字以内后再试": entry("輸入文字過長，請縮短到 12000 字以內後再試", "Input text is too long, shorten it to 12000 characters", "입력 텍스트가 너무 깁니다. 12000자 이내로 줄이세요", "入力テキストが長すぎます。12000文字以内にしてください", "Texte de saisie trop long, réduisez-le à 12000 caractères", "Eingabetext zu lang, kürzen Sie ihn auf 12000 Zeichen", "El texto de entrada es demasiado largo, redúcelo a 12000 caracteres"),
  "截图图片为空，请重新框选后再试": entry("截圖圖片為空，請重新框選後再試", "Screenshot image is empty, select again", "스크린샷 이미지가 비어 있습니다. 다시 선택하세요", "スクリーンショット画像が空です。再選択してください", "Image de capture vide, recommencez", "Screenshot-Bild ist leer, wählen Sie erneut", "La imagen de captura está vacía, selecciona de nuevo"),
  "截图图片过大，请缩小框选区域后再试": entry("截圖圖片過大，請縮小框選區域後再試", "Screenshot image is too large, select a smaller region", "스크린샷 이미지가 너무 큽니다. 더 작은 영역을 선택하세요", "スクリーンショット画像が大きすぎます。領域を縮小してください", "Image de capture trop grande, réduisez la région", "Screenshot-Bild zu groß, wählen Sie einen kleineren Bereich", "La imagen de captura es demasiado grande, selecciona una región menor"),
  "未知的 AI 操作类型": entry("未知的 AI 操作類型", "Unknown AI action", "알 수 없는 AI 작업", "不明なAI操作", "Action IA inconnue", "Unbekannte KI-Aktion", "Acción de IA desconocida"),
  "未知的图片 AI 操作类型": entry("未知的圖片 AI 操作類型", "Unknown image AI action", "알 수 없는 이미지 AI 작업", "不明な画像AI操作", "Action IA d'image inconnue", "Unbekannte Bild-KI-Aktion", "Acción de IA de imagen desconocida"),
  "Base URL 未配置": entry("Base URL 未配置", "Base URL is not configured", "Base URL이 설정되지 않았습니다", "Base URLが設定されていません", "URL de base non configurée", "Basis-URL nicht konfiguriert", "URL base no configurada"),
  "文本模型未配置": entry("文字模型未配置", "Text model is not configured", "텍스트 모델이 설정되지 않았습니다", "テキストモデルが設定されていません", "Modèle de texte non configuré", "Textmodell nicht konfiguriert", "Modelo de texto no configurado"),
  "视觉模型未配置；DeepSeek 当前只用于文本模型，请选择其他多模态视觉供应商": entry("視覺模型未配置；DeepSeek 目前只用於文字模型，請選擇其他多模態視覺供應商", "Vision model is not configured; DeepSeek is text-only, choose another vision provider", "비전 모델이 설정되지 않았습니다. DeepSeek는 텍스트 전용이므로 다른 비전 공급자를 선택하세요", "ビジョンモデルが設定されていません。DeepSeekはテキスト専用です。別のビジョンプロバイダーを選択してください", "Modèle visuel non configuré ; DeepSeek est textuel, choisissez un autre fournisseur", "Bildmodell nicht konfiguriert; DeepSeek ist textbasiert, wählen Sie einen anderen Anbieter", "Modelo visual no configurado; DeepSeek es solo texto, elige otro proveedor"),
  "输入模型未配置": entry("輸入模型未配置", "Input model is not configured", "입력 모델이 설정되지 않았습니다", "入力モデルが設定されていません", "Modèle de saisie non configuré", "Eingabemodell nicht konfiguriert", "Modelo de entrada no configurado"),
  "AI 请求超时，请稍后重试或调大请求超时": entry("AI 請求逾時，請稍後重試或調大請求逾時", "AI request timed out, retry or increase timeout", "AI 요청 시간이 초과되었습니다. 다시 시도하거나 시간을 늘리세요", "AIリクエストがタイムアウトしました。再試行またはタイムアウトを延長してください", "Délai IA dépassé, réessayez ou augmentez le délai", "KI-Anfrage-Timeout, versuchen Sie es erneut", "Se agotó el tiempo de IA, reintenta o aumenta el tiempo"),
  "无法连接 AI 服务，请检查 Base URL 和网络连接": entry("無法連線 AI 服務，請檢查 Base URL 和網路連線", "Cannot connect to AI service, check Base URL and network", "AI 서비스에 연결할 수 없습니다. Base URL과 네트워크를 확인하세요", "AIサービスに接続できません。Base URLとネットワークを確認してください", "Connexion IA impossible, vérifiez l'URL et le réseau", "KI-Dienst nicht erreichbar, prüfen Sie URL und Netzwerk", "No se puede conectar con la IA, revisa la URL base y la red"),
  "AI 请求失败，请检查网络、Base URL 和模型配置": entry("AI 請求失敗，請檢查網路、Base URL 和模型配置", "AI request failed, check network, Base URL and model config", "AI 요청 실패. 네트워크, Base URL, 모델 설정을 확인하세요", "AIリクエストに失敗。ネットワーク、Base URL、モデル設定を確認してください", "Échec de la requête IA, vérifiez le réseau et la configuration", "KI-Anfrage fehlgeschlagen, prüfen Sie Netzwerk und Konfiguration", "Error de solicitud de IA, revisa red y configuración"),
  "AI 请求被服务拒绝，请检查模型名称和服务兼容性": entry("AI 請求被服務拒絕，請檢查模型名稱和服務相容性", "AI request rejected, check model name and compatibility", "AI 요청이 거부되었습니다. 모델 이름과 호환성을 확인하세요", "AIリクエストが拒否されました。モデル名と互換性を確認してください", "Requête IA rejetée, vérifiez le modèle", "KI-Anfrage abgelehnt, prüfen Sie Modellname und Kompatibilität", "Solicitud de IA rechazada, revisa modelo y compatibilidad"),
  "AI 鉴权失败，请检查 API Key 是否有效": entry("AI 鑑權失敗，請檢查 API Key 是否有效", "AI authentication failed, check API Key", "AI 인증 실패. API 키를 확인하세요", "AI認証に失敗。APIキーを確認してください", "Échec d'authentification IA, vérifiez la clé API", "KI-Authentifizierung fehlgeschlagen, prüfen Sie den API-Schlüssel", "Error de autenticación de IA, revisa la clave API"),
  "AI 接口不存在，请确认 Base URL 是否包含正确的 /v1 路径": entry("AI 介面不存在，請確認 Base URL 是否包含正確的 /v1 路徑", "AI endpoint not found, check Base URL includes /v1", "AI 엔드포인트를 찾을 수 없습니다. Base URL에 /v1이 있는지 확인하세요", "AIエンドポイントが見つかりません。Base URLに/v1が含まれるか確認してください", "Point de terminaison IA introuvable, vérifiez /v1", "KI-Endpunkt nicht gefunden, prüfen Sie /v1 in der URL", "No se encuentra el endpoint de IA, revisa /v1 en la URL"),
  "AI 服务响应超时，请稍后重试": entry("AI 服務回應逾時，請稍後重試", "AI service timed out, retry later", "AI 서비스 응답 시간 초과. 나중에 다시 시도하세요", "AIサービスの応答がタイムアウトしました。後で再試行してください", "Service IA en timeout, réessayez", "KI-Dienst-Timeout, versuchen Sie es später", "Tiempo de respuesta de IA agotado, reintenta"),
  "AI 服务限流或余额不足，请稍后重试或检查账户状态": entry("AI 服務限流或餘額不足，請稍後重試或檢查帳戶狀態", "AI rate limited or out of balance, retry or check account", "AI 요청이 제한되었거나 잔액이 부족합니다. 나중에 시도하거나 계정을 확인하세요", "AIのレート制限または残高不足。後で再試行するかアカウントを確認してください", "IA limitée ou solde insuffisant, réessayez", "KI-Dienst begrenzt oder Guthaben fehlt", "IA limitada o saldo insuficiente, reintenta"),
  "AI 服务暂时不可用，请稍后重试": entry("AI 服務暫時不可用，請稍後重試", "AI service temporarily unavailable, retry later", "AI 서비스를 일시적으로 사용할 수 없습니다. 나중에 다시 시도하세요", "AIサービスが一時的に利用できません。後で再試行してください", "Service IA temporairement indisponible", "KI-Dienst vorübergehend nicht verfügbar", "Servicio de IA temporalmente no disponible"),
  "AI 请求失败，HTTP 状态码": entry("AI 請求失敗，HTTP 狀態碼", "AI request failed, HTTP status code", "AI 요청 실패, HTTP 상태 코드", "AIリクエスト失敗、HTTPステータスコード", "Échec de la requête IA, code HTTP", "KI-Anfrage fehlgeschlagen, HTTP-Statuscode", "Error de solicitud de IA, código HTTP"),
  "视觉模型拒绝了截图请求，请检查视觉模型供应商、Base URL、模型名称和 API Key 是否匹配。": entry("視覺模型拒絕了截圖請求，請檢查視覺模型供應商、Base URL、模型名稱和 API Key 是否匹配。", "Vision model rejected the screenshot request, check provider, Base URL, model and API Key", "비전 모델이 스크린샷 요청을 거부했습니다. 공급자, Base URL, 모델, API 키를 확인하세요", "ビジョンモデルがスクリーンショット要求を拒否しました。プロバイダー、Base URL、モデル、APIキーを確認してください", "Le modèle visuel a refusé la capture, vérifiez la configuration", "Bildmodell hat Screenshot-Anfrage abgelehnt, prüfen Sie die Konfiguration", "El modelo visual rechazó la captura, revisa la configuración"),
  "快捷键状态": entry("快捷鍵狀態", "Hotkey Status", "단축키 상태", "ショートカット状態", "État des raccourcis", "Tastenkürzel-Status", "Estado de atajos"),
  "已注册": entry("已註冊", "Registered", "등록됨", "登録済み", "Enregistré", "Registriert", "Registrado"),
  "已被占用": entry("已被佔用", "Occupied", "사용 중", "使用中", "Occupé", "Belegt", "Ocupado"),
  "注册失败": entry("註冊失敗", "Registration Failed", "등록 실패", "登録に失敗", "Échec de l'enregistrement", "Registrierung fehlgeschlagen", "Error de registro"),
};

function languageLabelTranslation(value: string, locale: TranslationLanguage): string {
  return translations[value]?.[locale] || value;
}

function localizePatterns(text: string, locale: TranslationLanguage): string {
  let result = text;
  const sourceTarget = text.match(/^源语言：(.+?)\s*→\s*目标语言：(.+)$/);
  if (sourceTarget) {
    return `${translations["源语言"][locale]}${languageLabelTranslation(sourceTarget[1], locale)} → ${translations["目标语言"][locale]}${languageLabelTranslation(sourceTarget[2], locale)}`;
  }

  result = result.replace(
    /(\d+(?:\.\d+)?)\s*字符/g,
    (_match, count) =>
      `${count} ${
        {
          "zh-Hans": "字符",
          "zh-Hant": "字元",
          en: "characters",
          ko: "자",
          ja: "文字",
          fr: "caractères",
          de: "Zeichen",
          es: "caracteres",
        }[locale]
      }`,
  );

  const conflict = text.match(/^(.+?)不能使用同一个快捷键$/);
  if (conflict) {
    return `${languageLabelTranslation(conflict[1], locale)} ${translations["不能使用同一个快捷键"]?.[locale] || "cannot use the same hotkey"}`;
  }

  return result;
}

export function translateText(source: string, language: UiLanguage): string {
  const locale = resolveUiLanguage(language);
  if (locale === "zh-Hans") {
    return source;
  }
  const trimmed = source.trim();
  const exact = translations[trimmed]?.[locale];
  if (exact !== undefined) {
    const leading = source.slice(0, source.indexOf(trimmed));
    const trailing = source.slice(source.indexOf(trimmed) + trimmed.length);
    return `${leading}${exact}${trailing}`;
  }
  return localizePatterns(source, locale);
}

const keyToSource: Record<string, string> = {
  "app.checkingCore": "正在检查核心连接",
  "app.previewMode": "前端预览模式",
  "language.auto": "自动检测",
  "language.zhHans": "简体中文",
  "language.zhHant": "繁体中文",
  "language.en": "英语",
  "language.ja": "日语",
  "language.ko": "韩语",
  "language.fr": "法语",
  "language.de": "德语",
  "language.ru": "俄语",
  "language.es": "西班牙语",
  "providers.openaiCompatible": "通用 OpenAI 兼容",
  "providers.xiaomiMimo": "小米 MiMo",
  "common.copy": "复制",
  "common.close": "关闭",
  "common.pin": "固定",
  "common.unpin": "取消固定",
  "common.search": "搜索",
  "common.translate": "翻译",
  "common.summarize": "总结",
  "common.extract": "提取",
  "common.retry": "重试",
  "selection.title": "QuickPick 划词",
  "result.empty": "暂无结果",
  "result.pending": "待处理",
  "result.loading": "正在处理",
  "result.success": "翻译结果",
  "result.error": "处理失败",
  "result.copy": "复制结果",
  "result.processing": "正在处理...",
  "result.noResult": "暂无结果",
  "result.switchFailed": "切换语言失败",
  "result.sourceLanguage": "源语言",
  "result.targetLanguage": "目标语言",
  "result.switchDirection": "切换翻译方向",
  "result.sourceTarget": "源语言：{source} → 目标语言：{target}",
  "input.title": "输入翻译",
  "input.placeholder": "输入要翻译的文本",
  "input.translating": "翻译中",
  "input.requestFailed": "翻译请求失败",
  "input.sourceLanguage": "源语言",
  "input.targetLanguage": "目标语言",
  "input.switchDirection": "切换翻译方向",
  "input.processing": "正在处理...",
  "input.copy": "复制",
  "input.sourceTarget": "源语言：{source} → 目标语言：{target}",
  "screenshot.reading": "正在读取屏幕截图...",
  "screenshot.currentScreen": "当前屏幕",
  "screenshot.waiting": "等待截图区域",
  "screenshot.copy": "复制",
  "screenshot.extract": "提取",
  "screenshot.translate": "翻译",
  "settings.general": "通用",
  "settings.hotkeys": "快捷键",
  "settings.hotkeyStatus": "快捷键状态",
  "hotkey.status.registered": "已注册",
  "hotkey.status.occupied": "已被占用",
  "hotkey.status.failed": "注册失败",
  "settings.aiSelection": "划词模型",
  "settings.aiScreenshot": "截图模型",
  "settings.aiInput": "输入模型",
  "settings.appearance": "外观",
  "settings.privacy": "隐私",
  "settings.title": "QuickPick 设置",
  "settings.minimize": "最小化",
  "settings.maximize": "最大化或还原",
  "settings.close": "关闭设置",
  "settings.categories": "设置分类",
  "settings.save": "保存设置",
  "settings.saved": "设置已保存",
  "settings.loaded": "设置已载入",
  "settings.loadFailed": "读取设置失败，已使用默认值",
  "settings.unsaved": "有未保存的修改",
  "settings.saving": "正在保存",
  "settings.saveFailed": "保存设置失败",
  "settings.reset": "重置默认设置",
  "settings.resetting": "正在重置默认设置",
  "settings.resetFailed": "重置默认设置失败",
  "settings.autostart": "开机自启",
  "settings.enabled": "已启用",
  "settings.disabled": "已关闭",
  "settings.language": "语言",
  "settings.languageSystem": "系统语言",
  "settings.themeSystem": "跟随系统",
  "settings.themeLight": "浅色",
  "settings.themeDark": "深色",
  "settings.aiTimeout": "AI 请求超时",
  "settings.hotkeySettings": "设置",
  "settings.hotkeySelection": "划词菜单",
  "settings.hotkeyScreenshot": "区域截图",
  "settings.hotkeyInput": "输入翻译",
  "settings.textProvider": "文本供应商",
  "settings.textBaseUrl": "文本 Base URL",
  "settings.textModel": "文本模型",
  "settings.textApiKey": "文本 API Key",
  "settings.textKeyStatus": "文本 Key 状态",
  "settings.encrypted": "已加密保存",
  "settings.notConfigured": "未配置",
  "settings.saveTextKey": "保存文本 Key",
  "settings.clear": "清除",
  "settings.visionProvider": "视觉供应商",
  "settings.visionBaseUrl": "视觉 Base URL",
  "settings.visionModel": "视觉模型",
  "settings.visionApiKey": "视觉 API Key",
  "settings.visionKeyStatus": "视觉 Key 状态",
  "settings.saveVisionKey": "保存视觉 Key",
  "settings.inputProvider": "输入供应商",
  "settings.inputBaseUrl": "输入 Base URL",
  "settings.inputModel": "输入模型",
  "settings.inputApiKey": "输入 API Key",
  "settings.inputKeyStatus": "输入 Key 状态",
  "settings.saveInputKey": "保存输入 Key",
  "settings.textKeyEncryptedReplace": "文本模型 Key 已加密保存，输入新 Key 可替换",
  "settings.enterTextApiKey": "输入文本模型 API Key",
  "settings.visionKeyEncryptedReplace": "视觉模型 Key 已加密保存，输入新 Key 可替换",
  "settings.enterVisionApiKey": "输入视觉模型 API Key",
  "settings.inputKeyEncryptedReplace": "输入模型 Key 已加密保存，输入新 Key 可替换",
  "settings.enterInputApiKey": "输入输入模型 API Key",
  "settings.theme": "主题",
  "settings.windowEffect": "窗口效果",
  "settings.opacity": "不透明度",
  "settings.contentHistory": "内容历史",
  "settings.contentHistoryValue": "默认不保存文本、截图和 AI 结果",
  "settings.apiKey": "API Key",
  "settings.apiKeyValue": "使用 Windows DPAPI 本机加密保存",
  "settings.selectTheme": "选择外观主题",
  "settings.selectWindowEffect": "选择窗口效果",
  "settings.selectTextProvider": "选择文本模型供应商",
  "settings.selectVisionProvider": "选择视觉模型供应商",
  "settings.selectInputProvider": "选择输入模型供应商",
  "settings.hotkeyCapture": "按下快捷键组合",
  "settings.recordHotkey": "录制{label}快捷键",
  "settings.checkingTextKey": "正在检查文本模型 Key",
  "settings.checkingVisionKey": "正在检查视觉模型 Key",
  "settings.checkingInputKey": "正在检查输入模型 Key",
  "settings.textKeyEncrypted": "文本模型 Key 已加密保存",
  "settings.textKeyNotConfigured": "文本模型 Key 未配置",
  "settings.visionKeyEncrypted": "视觉模型 Key 已加密保存",
  "settings.visionKeyNotConfigured": "视觉模型 Key 未配置",
  "settings.inputKeyEncrypted": "输入模型 Key 已加密保存",
  "settings.inputKeyNotConfigured": "输入模型 Key 未配置",
  "settings.checkTextKeyFailed": "检查文本模型 Key 状态失败",
  "settings.checkVisionKeyFailed": "检查视觉模型 Key 状态失败",
  "settings.checkInputKeyFailed": "检查输入模型 Key 状态失败",
  "settings.textProviderFilled": "文本模型供应商默认值已填入，请保存设置",
  "settings.visionProviderFilled": "视觉模型供应商默认值已填入，请保存设置",
  "settings.inputProviderFilled": "输入模型供应商默认值已填入，请保存设置",
  "settings.replaceTextKeyHint": "输入新 Key 后保存会替换文本模型 Key",
  "settings.replaceVisionKeyHint": "输入新 Key 后保存会替换视觉模型 Key",
  "settings.replaceInputKeyHint": "输入新 Key 后保存会替换输入模型 Key",
  "settings.hotkeyPrompt": "请直接按下新的快捷键组合，Esc 取消",
  "settings.hotkeyCancelled": "已取消快捷键录制",
  "settings.hotkeyNeedsAlt": "快捷键必须包含 Alt，请重新按下组合键",
  "settings.hotkeyUnsupported": "这个按键暂不支持，请换一个组合键",
  "settings.hotkeyConflict": "{conflict}不能使用同一个快捷键",
  "settings.hotkeyRecorded": "快捷键已记录，请保存设置",
  "settings.conflictSelectionScreenshot": "划词和截图",
  "settings.conflictScreenshotInput": "截图和输入翻译",
  "settings.conflictInputSelection": "输入翻译和划词",
  "settings.syncingAi": "正在同步 AI 设置",
  "settings.encrypting": "正在加密保存",
  "settings.syncAiFailed": "同步 AI 设置失败",
  "settings.clearing": "正在清除",
  "settings.textKeyLabel": "文本模型 Key",
  "settings.visionKeyLabel": "视觉模型 Key",
  "settings.inputKeyLabel": "输入模型 Key",
  "settings.cannotBeEmpty": "不能为空",
  "settings.saveKeyFailed": "保存失败",
  "settings.clearFailed": "清除失败",
  "settings.syncFailed": "同步设置失败",
  "settings.syncBeforeSaveFailed": "保存前同步设置失败",
  "settings.readFailed": "读取设置文件失败",
  "settings.invalidFormat": "设置文件格式无效",
  "settings.createDirFailed": "创建设置目录失败",
  "settings.serializeFailed": "序列化设置失败",
  "settings.writeFailed": "保存设置文件失败",
  "settings.locateFailed": "定位应用设置目录失败",
  "hotkey.empty": "快捷键不能为空",
  "hotkey.altRequired": "快捷键首版需包含 Alt，例如 Alt+2",
  "hotkey.invalid": "快捷键格式无效，请使用 Alt+2 或 Ctrl+Alt+Q 这类格式",
  "hotkey.settingsDuplicate": "设置快捷键不能和划词菜单或区域截图使用同一个快捷键",
  "hotkey.selectionScreenshotDuplicate": "划词菜单和区域截图不能使用同一个快捷键",
  "hotkey.inputDuplicate": "输入翻译不能和其他功能使用同一个快捷键",
  "settings.themeInvalid": "主题模式无效",
  "settings.uiLanguageInvalid": "界面语言无效",
  "settings.windowEffectInvalid": "窗口效果无效",
  "settings.opacityInvalid": "不透明度需在 30% 到 100% 之间",
  "settings.timeoutInvalid": "请求超时需在 5 到 120 秒之间",
  "settings.modelTooLong": "模型名称过长，请缩短后保存",
  "settings.baseUrlInvalid": "Base URL 必须以 http:// 或 https:// 开头",
  "ai.textTooLong": "选中文本过长，请缩短到 12000 字以内后再试",
  "ai.clientInitFailed": "初始化 AI 请求客户端失败",
  "ai.emptyResponse": "AI 返回为空，请稍后重试或检查模型配置",
  "ai.inputTooLong": "输入文本过长，请缩短到 12000 字以内后再试",
  "ai.imageEmpty": "截图图片为空，请重新框选后再试",
  "ai.imageTooLarge": "截图图片过大，请缩小框选区域后再试",
  "ai.unknownAction": "未知的 AI 操作类型",
  "ai.unknownImageAction": "未知的图片 AI 操作类型",
  "ai.baseUrlMissing": "Base URL 未配置",
  "ai.baseUrlInvalid": "Base URL 必须以 http:// 或 https:// 开头",
  "ai.textModelMissing": "文本模型未配置",
  "ai.visionModelMissing": "视觉模型未配置；DeepSeek 当前只用于文本模型，请选择其他多模态视觉供应商",
  "ai.inputModelMissing": "输入模型未配置",
  "ai.requestTimeout": "AI 请求超时，请稍后重试或调大请求超时",
  "ai.connectFailed": "无法连接 AI 服务，请检查 Base URL 和网络连接",
  "ai.requestFailed": "AI 请求失败，请检查网络、Base URL 和模型配置",
  "ai.httpBadRequest": "AI 请求被服务拒绝，请检查模型名称和服务兼容性",
  "ai.httpUnauthorized": "AI 鉴权失败，请检查 API Key 是否有效",
  "ai.httpNotFound": "AI 接口不存在，请确认 Base URL 是否包含正确的 /v1 路径",
  "ai.httpTimeout": "AI 服务响应超时，请稍后重试",
  "ai.httpRateLimited": "AI 服务限流或余额不足，请稍后重试或检查账户状态",
  "ai.httpServerError": "AI 服务暂时不可用，请稍后重试",
  "ai.httpStatus": "AI 请求失败，HTTP 状态码",
  "ai.visionRejected": "视觉模型拒绝了截图请求，请检查视觉模型供应商、Base URL、模型名称和 API Key 是否匹配。",
};

export type Translator = (
  key: string,
  params?: Record<string, string | number>,
) => string;

let currentUiLanguage: UiLanguage = "zh-Hans";
const i18nListeners = new Set<() => void>();

export function setI18nLanguage(language: UiLanguage) {
  currentUiLanguage = language;
  for (const listener of i18nListeners) {
    listener();
  }
}

function subscribeI18n(listener: () => void) {
  i18nListeners.add(listener);
  return () => {
    i18nListeners.delete(listener);
  };
}

function getI18nSnapshot() {
  return currentUiLanguage;
}

export function createTranslator(language: UiLanguage): Translator {
  return (key, params) => {
    let detail = "";
    let resolvedKey = key;
    if (key.startsWith("i18n:")) {
      const rest = key.slice(5);
      const separator = rest.indexOf(":");
      if (separator >= 0) {
        resolvedKey = rest.slice(0, separator);
        detail = rest.slice(separator + 1);
      } else {
        resolvedKey = rest;
      }
    }
    const source = keyToSource[resolvedKey] ?? key;
    let value = translateText(source, language);
    if (detail) {
      value = `${value} ${detail}`;
    }
    if (params) {
      for (const [name, param] of Object.entries(params)) {
        value = value.replaceAll(`{${name}}`, String(param));
      }
    }
    return value;
  };
}

export function useI18n(): Translator {
  const language = useSyncExternalStore(subscribeI18n, getI18nSnapshot);
  return createTranslator(language);
}

